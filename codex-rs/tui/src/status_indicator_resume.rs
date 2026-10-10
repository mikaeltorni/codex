//! Quota-reset countdown drawn in place of the normal elapsed status.

use crate::key_hint::ShortcutHint;
use ratatui::style::Stylize;
use ratatui::text::Span;

pub(super) fn resume_status_header(countdown: Option<&str>) -> Option<String> {
    countdown.map(|countdown| format!("Resuming in {countdown}"))
}

pub(super) fn push_resume_interrupt(
    spans: &mut Vec<Span<'static>>,
    interrupt_binding: Option<&ShortcutHint>,
) {
    let Some(interrupt_binding) = interrupt_binding else {
        return;
    };
    spans.push("• ".dim());
    spans.extend(interrupt_binding.spans());
    spans.push(" to interrupt".dim());
}
