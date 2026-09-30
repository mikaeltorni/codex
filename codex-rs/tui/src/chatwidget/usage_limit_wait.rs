//! Renders and refreshes the visible auto-resume countdown while a turn waits for quota reset.

use super::ChatWidget;
use crate::app_event::AppEvent;
use crate::backend_banners::BackendBanner;
use crate::backend_banners::rate_limit_reset_selection_item;
use crate::bottom_pane::ActionableBanner;
use crate::bottom_pane::BannerDismissal;
use crate::bottom_pane::SelectionItem;
use codex_protocol::account::PlanType;
use std::time::Duration;
use std::time::Instant;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

const USAGE_LIMIT_WAIT_VIEW_ID: &str = "usage-limit-auto-resume";

impl ChatWidget {
    /// Reuse account recovery choices during an auto-resume wait, even if the earlier account
    /// notice was dismissed, and supply the plan's upgrade/request action if no notice arrived.
    pub(super) fn usage_limit_wait_backend_banner(&mut self) -> ActionableBanner {
        let mut content = self
            .applicable_backend_banner()
            .map(|banner| self.backend_banner_actionable_content(&banner.for_usage_limit_wait()))
            .unwrap_or_default();
        if self.has_chatgpt_account
            && let Some(plan_type) = self.plan_type
            && let Some(fallback) = BackendBanner::recovery_fallback_for_plan(plan_type)
            && !content
                .actions
                .iter()
                .any(|action| action.name == fallback.name)
        {
            content.actions.push(fallback);
        }
        if self.has_chatgpt_account
            && self
                .available_rate_limit_reset_credits
                .is_some_and(|count| count > 0)
            && !self.applicable_backend_banner().is_some_and(|banner| {
                banner
                    .selection_items()
                    .any(|(action, _)| action == "reset_usage")
            })
        {
            content
                .actions
                .push(rate_limit_reset_selection_item("Redeem reset".to_string()));
        }
        if let Some(switch) = self.backend_banner_fallback()
            && let Some(thread_id) = self.thread_id()
        {
            let model = switch.model.model;
            let model_name = self.model_catalog.display_name(&model).to_string();
            content.actions.push(SelectionItem {
                name: format!("Switch to {model_name}"),
                actions: vec![Box::new(move |tx| {
                    tx.send(AppEvent::ApplyBackendBannerFallback { thread_id });
                })],
                ..Default::default()
            });
        }
        content
    }

    pub(super) fn update_usage_limit_wait(&mut self, retry_at_ms: Option<i64>) {
        let was_waiting = self.usage_limit_wait_retry_at_ms.is_some();
        if self.usage_limit_wait_retry_at_ms != retry_at_ms {
            self.usage_limit_wait_banner_dismissed = false;
        }
        self.usage_limit_wait_retry_at_ms = retry_at_ms;
        self.usage_limit_wait_next_tick =
            retry_at_ms.map(|_| Instant::now() + Duration::from_secs(1));
        if was_waiting && retry_at_ms.is_none() {
            // Core emits the end of a quota wait just before the sampling retry begins.
            // Discard the elapsed wait so the visible working clock starts with that retry.
            self.bottom_pane.reset_status_timer(Duration::ZERO);
            self.bottom_pane.set_status_timer_origin(None);
        }
        if !was_waiting && retry_at_ms.is_some() && self.has_chatgpt_account {
            // Auto-resume keeps the core turn alive, so the ordinary rate-limit error path does
            // not request account recovery choices. Refresh through that same account endpoint.
            self.app_event_tx.send(AppEvent::RefreshRateLimits {
                origin: crate::app_event::RateLimitRefreshOrigin::Recovery,
            });
        }
        if retry_at_ms.is_none() && !was_waiting {
            return;
        }
        self.refresh_usage_limit_wait_banner();
    }

    /// Hide the wait actions while leaving the active reset deadline and redraw timer untouched.
    pub(crate) fn dismiss_usage_limit_wait_banner(&mut self) {
        if self.usage_limit_wait_retry_at_ms.is_none() {
            return;
        }
        self.usage_limit_wait_banner_dismissed = true;
        self.bottom_pane.set_inline_banner(None);
        self.request_redraw();
    }

    pub(crate) fn refresh_usage_limit_wait_for_time_tick(&mut self) {
        let Some(retry_at_ms) = self.usage_limit_wait_retry_at_ms else {
            self.usage_limit_wait_next_tick = None;
            return;
        };
        self.usage_limit_wait_next_tick = Some(Instant::now() + Duration::from_secs(1));
        // Rebuilding the menu here would reset its selection every second, potentially
        // activating a different recovery action when the user presses Enter.
        self.bottom_pane
            .set_status_resume_countdown(Some(format_remaining_time(retry_at_ms)));
        self.request_redraw();
    }

    pub(super) fn refresh_usage_limit_wait_banner(&mut self) {
        let Some(retry_at_ms) = self.usage_limit_wait_retry_at_ms else {
            self.bottom_pane.set_status_resume_countdown(None);
            self.bottom_pane.set_inline_banner(None);
            self.restore_backend_banner_after_usage_wait();
            self.request_redraw();
            return;
        };

        self.bottom_pane
            .set_status_resume_countdown(Some(format_remaining_time(retry_at_ms)));
        if self.usage_limit_wait_banner_dismissed {
            self.bottom_pane.set_inline_banner(None);
        } else {
            let mut banner = self.usage_limit_wait_backend_banner();
            // Account identity owns this copy; backend action labels can change independently.
            let plan = self.plan_type.filter(|_| self.has_chatgpt_account);
            let workspace = plan.is_some_and(PlanType::is_workspace_account);
            banner.title = if workspace {
                "Workspace usage limit reached (Auto-continue is enabled)"
            } else if plan.is_some_and(|plan| plan != PlanType::Unknown) {
                "Personal usage limit reached (Auto-continue is enabled)"
            } else {
                "Usage limit reached (Auto-continue is enabled)"
            }
            .to_string();
            if matches!(
                plan,
                Some(PlanType::Free | PlanType::Go | PlanType::Plus | PlanType::ProLite)
            ) {
                banner.description = "Upgrade your subscription to continue sooner, or wait for your usage limit to reset. Your turn will resume automatically.".to_string();
            } else if banner.description.is_empty() {
                banner.description = if workspace {
                    "Your turn will automatically continue when your workspace usage limit resets. Request a limit increase to continue sooner."
                } else {
                    "Your turn will automatically continue when the usage limit resets."
                }.to_string();
            }
            let thread_id = self.thread_id();
            banner.actions.push(SelectionItem {
                name: "Keep waiting".to_string(),
                actions: vec![Box::new(move |tx| {
                    tx.send(AppEvent::DismissUsageLimitWaitBanner { thread_id });
                })],
                ..Default::default()
            });
            banner.dismissal = BannerDismissal::Persistent;
            banner.view_id = Some(USAGE_LIMIT_WAIT_VIEW_ID);
            banner.visible_while_task_running = true;
            banner.interactive_while_task_running = true;
            banner.gap_below = true;
            self.bottom_pane.set_inline_banner(Some(banner));
        }
        self.request_redraw();
    }
}

fn format_remaining_time(retry_at_ms: i64) -> String {
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let now_ms = i64::try_from(now_ms).unwrap_or(i64::MAX);
    format_remaining_time_at(retry_at_ms, now_ms)
}

fn format_remaining_time_at(retry_at_ms: i64, now_ms: i64) -> String {
    let remaining_seconds = retry_at_ms.saturating_sub(now_ms).max(0) as u64 / 1000;
    format_remaining_seconds(remaining_seconds)
}

fn format_remaining_seconds(remaining_seconds: u64) -> String {
    let days = remaining_seconds / 86_400;
    let hours = (remaining_seconds % 86_400) / 3_600;
    let minutes = (remaining_seconds % 3_600) / 60;
    let seconds = remaining_seconds % 60;

    if days > 0 {
        format!("{days}d {hours}h {minutes:02}m {seconds:02}s")
    } else if hours > 0 {
        format!("{hours}h {minutes:02}m {seconds:02}s")
    } else {
        format!("{minutes}m {seconds:02}s")
    }
}

#[cfg(test)]
mod tests {
    use super::format_remaining_seconds;
    use super::format_remaining_time_at;
    use pretty_assertions::assert_eq;

    #[test]
    fn countdown_formats_minutes_hours_and_days() {
        assert_eq!(format_remaining_seconds(7), "0m 07s");
        assert_eq!(format_remaining_seconds(3_661), "1h 01m 01s");
        assert_eq!(format_remaining_seconds(90_061), "1d 1h 01m 01s");
    }

    #[test]
    fn countdown_decreases_when_refreshed_one_second_later() {
        let retry_at_ms = 4_322_000;
        assert_eq!(format_remaining_time_at(retry_at_ms, 1_000), "1h 12m 01s");
        assert_eq!(format_remaining_time_at(retry_at_ms, 2_000), "1h 12m 00s");
    }
}
