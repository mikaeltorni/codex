//! Native goal-prefix edits retain the complete composer draft and Vim history.

use super::*;

impl ChatComposer {
    /// Prepend the goal command without replacing text, elements or attachments.
    /// Parameters: self - the editable composer. Returns: None.
    pub(super) fn prepend_goal(&mut self) {
        if let Some(pasted) = self.draft.paste_burst.flush_before_modified_input() {
            self.apply_paste(pasted);
        }
        self.draft.paste_burst.clear_after_explicit_paste();
        let suffix = self.draft.textarea.text().strip_prefix("/goal");
        if !self.draft.is_bash_mode
            && suffix.is_some_and(|suffix| suffix.starts_with(char::is_whitespace))
        {
            return;
        }
        let bare_goal = !self.draft.is_bash_mode && suffix == Some("");
        let started_vim_edit = self.begin_direct_vim_edit();
        if bare_goal {
            self.draft.textarea.insert_str_at(/*pos*/ 5, " ");
        } else {
            if self.draft.is_bash_mode {
                self.draft.textarea.insert_str_at(/*pos*/ 0, "!");
                self.draft.is_bash_mode = false;
            }
            self.draft.textarea.insert_str_at(/*pos*/ 0, "/goal ");
        }
        self.dismiss_sparkle();
        self.history.reset_navigation();
        self.sync_popups();
        if started_vim_edit {
            self.finish_vim_edit();
        }
    }
}
