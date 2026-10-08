//! Key handling for an inline banner that stays interactive while a task is running.

use super::super::BottomPane;
use super::super::BottomPaneView;
use super::InlineBannerContent;
use crate::keymap::ListAction;
use crate::keymap::ListKeymap;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use crossterm::event::KeyModifiers;
use ratatui::style::Stylize;
use ratatui::text::Line;

pub(super) fn interactive_banner_hint(list: &ListKeymap) -> Line<'static> {
    let mut spans = vec!["Press a number to choose".dim()];
    for (action, label) in [
        (ListAction::MoveUp, "to move up"),
        (ListAction::MoveDown, "to move down"),
        (ListAction::Accept, "to select"),
    ] {
        if let Some(hint) = list.primary_hint(action) {
            spans.push(" · ".dim());
            spans.extend(hint.spans());
            spans.push(format!(" {label}").dim());
        }
    }
    spans.into()
}

impl BottomPane {
    /// Activate recovery choices only after explicit navigation or a numbered shortcut.
    /// Parameters: self - the pane and current selection; key - the incoming terminal key.
    /// Returns: whether the banner consumed the key.
    pub(super) fn handle_interactive_inline_banner_key(&mut self, key: KeyEvent) -> bool {
        #[cfg(test)]
        println!("self={self:p}, key={key:?}");
        if !self.composer_is_empty()
            || (key.code == KeyCode::Esc && self.composer.shortcut_overlay_visible())
            || self.composer.popup_active()
            || self.composer.is_in_paste_burst()
            || self.composer_should_handle_vim_insert_escape(key)
            || key.kind != KeyEventKind::Press
        {
            if key.kind == KeyEventKind::Press
                && let Some(banner) = self.inline_banner.as_mut()
            {
                banner.action_selection_armed.set(false);
            }
            #[cfg(test)]
            println!("false");
            return false;
        }
        let Some(banner) = self.inline_banner.as_mut() else {
            #[cfg(test)]
            println!("false");
            return false;
        };
        if !banner.visible.get() || banner.dismissed {
            banner.action_selection_armed.set(false);
            #[cfg(test)]
            println!("false");
            return false;
        }
        let list_action = self.keymap.list.action_for(key);
        match key.code {
            KeyCode::Esc if banner.dismissal == super::BannerDismissal::Persistent => {
                banner.action_selection_armed.set(false);
                #[cfg(test)]
                println!("false");
                return false;
            }
            KeyCode::Esc if key.modifiers == KeyModifiers::NONE => {
                banner.dismissed = true;
            }
            KeyCode::Char(digit @ '1'..='9') if key.modifiers == KeyModifiers::NONE => {
                if digit as usize - '1' as usize >= banner.visible_action_count.get() {
                    banner.action_selection_armed.set(false);
                    #[cfg(test)]
                    println!("false");
                    return false;
                }
                let InlineBannerContent::Actions(view) = &mut banner.content else {
                    #[cfg(test)]
                    println!("false");
                    return false;
                };
                banner.action_selection_armed.set(false);
                view.handle_key_event(key);
                let _ = view.take_last_selected_index();
            }
            _ if matches!(
                list_action,
                Some(ListAction::MoveUp | ListAction::MoveDown | ListAction::Accept)
            ) =>
            {
                if list_action == Some(ListAction::Accept) && !banner.action_selection_armed.get() {
                    #[cfg(test)]
                    println!("false");
                    return false;
                }
                let InlineBannerContent::Actions(view) = &mut banner.content else {
                    #[cfg(test)]
                    println!("false");
                    return false;
                };
                // Consume the selection on activation so repeated Enter cannot reopen a URL.
                banner
                    .action_selection_armed
                    .set(list_action != Some(ListAction::Accept));
                view.handle_key_event(key);
                let _ = view.take_last_selected_index();
            }
            _ => {
                banner.action_selection_armed.set(false);
                #[cfg(test)]
                println!("false");
                return false;
            }
        }
        self.request_redraw();
        #[cfg(test)]
        println!("true");
        true
    }
}
