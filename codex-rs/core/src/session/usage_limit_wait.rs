//! Cancellable quota waits for an active sampling turn. The caller retains the request history.

use super::input_queue::InputQueueActivity;
use super::session::Session;
use super::turn_context::TurnContext;
use chrono::DateTime;
use chrono::Utc;
use codex_protocol::error::CodexErr;
use codex_protocol::error::UsageLimitReachedError;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::UsageLimitWaitEvent;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;
use tracing::info;

#[derive(Default)]
pub(crate) struct UsageLimitWait {
    pub(crate) resume: Notify,
}

/// Wait out the reset when auto-resume is enabled, retaining the caller's history.
pub(super) async fn continue_after_usage_limit(
    sess: &Session,
    turn_context: &TurnContext,
    error: &UsageLimitReachedError,
    cancellation_token: &CancellationToken,
) -> Result<bool, CodexErr> {
    let Some(resets_at) = error
        .resets_at
        .filter(|_| turn_context.config.auto_resume_on_usage_limit)
    else {
        return Ok(false);
    };
    wait_for_usage_limit_reset(sess, turn_context, resets_at, cancellation_token).await?;
    Ok(true)
}

/// Wait until the advertised reset plus a margin for delayed backend quota updates.
/// Emit a matching end event before returning, including when the turn is interrupted.
#[tracing::instrument(skip_all, fields(turn_id = %turn_context.sub_id, %resets_at))]
pub(super) async fn wait_for_usage_limit_reset(
    sess: &Session,
    turn_context: &TurnContext,
    resets_at: DateTime<Utc>,
    cancellation_token: &CancellationToken,
) -> Result<(), CodexErr> {
    // A stale reset still waits five seconds, preventing a tight loop of quota errors.
    let now = Utc::now();
    let wait_duration = resets_at
        .signed_duration_since(now)
        .to_std()
        .unwrap_or_default()
        .saturating_add(Duration::from_secs(/*secs*/ 5));
    let retry_at_ms = now
        .timestamp_millis()
        .saturating_add(i64::try_from(wait_duration.as_millis()).unwrap_or(i64::MAX));
    let wait = turn_context
        .extension_data
        .get_or_init(UsageLimitWait::default);
    let turn_state = sess
        .active_turn
        .lock()
        .await
        .as_ref()
        .map(|active| Arc::clone(&active.turn_state));
    let (mut activity, pending) = sess
        .input_queue
        .subscribe_activity(turn_state.as_deref())
        .await;
    let steering = async {
        if pending == Some(InputQueueActivity::Steer) {
            return;
        }
        while activity.changed().await.is_ok() {
            if *activity.borrow_and_update() == InputQueueActivity::Steer {
                return;
            }
        }
    };
    sess.send_event(
        turn_context,
        EventMsg::UsageLimitWaitStarted(UsageLimitWaitEvent { retry_at_ms }),
    )
    .await;
    info!(
        wait_seconds = wait_duration.as_secs(),
        "Waiting for usage limit reset"
    );
    let result = tokio::select! {
        biased;
        _ = cancellation_token.cancelled() => {
            info!("Usage limit wait cancelled");
            Err(CodexErr::TurnAborted)
        }
        _ = wait_for_deadline(retry_at_ms, Utc::now) => {
            info!("Usage limit reset wait complete; retrying sampling request");
            Ok(())
        }
        _ = wait.resume.notified() => Ok(()),
        _ = steering => Ok(()),
    };
    turn_context.extension_data.remove::<UsageLimitWait>();
    // End the countdown before retrying so the UI starts a fresh working timer.
    sess.send_event(turn_context, EventMsg::UsageLimitWaitEnded)
        .await;
    result
}

async fn wait_for_deadline(retry_at_ms: i64, now: impl Fn() -> DateTime<Utc>) {
    loop {
        let remaining = retry_at_ms.saturating_sub(now().timestamp_millis());
        if remaining <= 0 {
            return;
        }
        // Recheck UTC after short sleeps so suspend does not extend the reset deadline.
        tokio::time::sleep(
            Duration::from_millis(remaining as u64).min(Duration::from_secs(/*secs*/ 1)),
        )
        .await;
    }
}

#[cfg(test)]
#[path = "usage_limit_wait_tests.rs"]
mod tests;
