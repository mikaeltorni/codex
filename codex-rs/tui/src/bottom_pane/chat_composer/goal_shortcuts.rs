//! Native goal-prefix edits retain the complete composer draft and Vim history.

use super::*;

/// Recognize a leading goal token and at most one separating whitespace character.
/// Parameters: text - the raw draft text. Returns: removable prefix bytes, if present.
fn goal_prefix_len(text: &str) -> Option<usize> {
    #[cfg(test)]
    println!("text={text:?}");
    let prefix_len = text
        .strip_prefix("/goal")
        .and_then(|suffix| match suffix.chars().next() {
            None => Some(5),
            Some(separator) if separator.is_whitespace() => Some(5 + separator.len_utf8()),
            Some(_) => None,
        });
    #[cfg(test)]
    println!("{prefix_len:?}");
    prefix_len
}

impl ChatComposer {
    /// Toggle the goal prefix while retaining the draft, attachments and Vim history.
    /// Parameters: self - the editable composer. Returns: None.
    pub(super) fn prepend_goal(&mut self) {
        #[cfg(test)]
        println!("self={self:p}");
        if let Some(pasted) = self.draft.paste_burst.flush_before_modified_input() {
            self.apply_paste(pasted);
        }
        self.draft.paste_burst.clear_after_explicit_paste();
        let prefix_len = if self.draft.is_bash_mode {
            None
        } else {
            goal_prefix_len(self.draft.textarea.text())
        };
        let started_vim_edit = self.begin_direct_vim_edit();
        if let Some(prefix_len) = prefix_len {
            self.draft.textarea.replace_range(0..prefix_len, "");
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
        #[cfg(test)]
        println!("None");
    }
}
