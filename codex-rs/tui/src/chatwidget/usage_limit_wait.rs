//! Renders and refreshes the visible auto-resume countdown while a turn waits for quota reset.

use super::*;
use crate::bottom_pane::ActionableBanner;
use crate::bottom_pane::BannerDismissal;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

const USAGE_LIMIT_WAIT_VIEW_ID: &str = "usage-limit-auto-resume";

impl ChatWidget {
    pub(super) fn update_usage_limit_wait(&mut self, retry_at_ms: Option<i64>) {
        let was_waiting = self.usage_limit_wait_retry_at_ms.is_some();
        self.usage_limit_wait_retry_at_ms = retry_at_ms;
        self.usage_limit_wait_next_tick =
            retry_at_ms.map(|_| Instant::now() + Duration::from_secs(1));
        if retry_at_ms.is_none() && !was_waiting {
            return;
        }
        self.refresh_usage_limit_wait_banner();
    }

    pub(crate) fn refresh_usage_limit_wait_for_time_tick(&mut self) {
        if self.usage_limit_wait_retry_at_ms.is_none() {
            self.usage_limit_wait_next_tick = None;
            return;
        }
        self.usage_limit_wait_next_tick = Some(Instant::now() + Duration::from_secs(1));
        self.refresh_usage_limit_wait_banner();
    }

    fn refresh_usage_limit_wait_banner(&mut self) {
        let Some(retry_at_ms) = self.usage_limit_wait_retry_at_ms else {
            self.bottom_pane.set_status_resume_countdown(None);
            self.bottom_pane.set_inline_banner(None);
            self.restore_backend_banner_after_usage_wait();
            self.request_redraw();
            return;
        };

        self.bottom_pane
            .set_status_resume_countdown(Some(format_remaining_time(retry_at_ms)));
        self.bottom_pane.set_inline_banner(Some(ActionableBanner {
            title: "Usage limit reached".to_string(),
            description: format!(
                "Auto-continue is enabled. Resuming in {}. Press Ctrl-C to stop.",
                format_remaining_time(retry_at_ms),
            ),
            dismissal: BannerDismissal::Persistent,
            view_id: Some(USAGE_LIMIT_WAIT_VIEW_ID),
            visible_while_task_running: true,
            ..Default::default()
        }));
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
    use super::*;
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
