//! Goal resume uses the same status event and app-server continuation as `/goal resume`.

use super::*;

impl ChatWidget {
    /// Consume the native resume binding without typing, submitting or replacing a draft.
    /// Parameters: self - displayed chat; key_event - terminal event. Returns: whether consumed.
    pub(super) fn handle_goal_resume_shortcut(&mut self, key_event: KeyEvent) -> bool {
        if !self.chat_keymap.resume_goal.is_pressed(key_event)
            || !self.bottom_pane.no_modal_or_popup_active()
        {
            return false;
        }
        if key_event.kind != KeyEventKind::Press
            || !self.config.features.enabled(Feature::Goals)
            || self.blocks_direct_input
            || !self.bottom_pane.composer_input_enabled()
        {
            return true;
        }
        if let Some(thread_id) = self.thread_id
            && self
                .current_goal_status
                .as_ref()
                .is_some_and(|goal| goal.can_resume(thread_id))
        {
            self.app_event_tx.send(AppEvent::SetThreadGoalStatus {
                thread_id,
                status: AppThreadGoalStatus::Active,
            });
        }
        true
    }
}
