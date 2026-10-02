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
    pub(super) fn handle_interactive_inline_banner_key(&mut self, key: KeyEvent) -> bool {
        if !self.composer_is_empty()
            || (key.code == KeyCode::Esc && self.composer.shortcut_overlay_visible())
            || self.composer.popup_active()
            || self.composer.is_in_paste_burst()
            || self.composer_should_handle_vim_insert_escape(key)
            || key.kind != KeyEventKind::Press
        {
            return false;
        }
        let Some(banner) = self.inline_banner.as_mut() else {
            return false;
        };
        if !banner.visible.get() || banner.dismissed {
            return false;
        }
        match key.code {
            KeyCode::Esc if banner.dismissal == super::BannerDismissal::Persistent => return false,
            KeyCode::Esc if key.modifiers == KeyModifiers::NONE => {
                banner.dismissed = true;
            }
            KeyCode::Char(digit @ '1'..='9') if key.modifiers == KeyModifiers::NONE => {
                if digit as usize - '1' as usize >= banner.visible_action_count.get() {
                    return false;
                }
                let InlineBannerContent::Actions(view) = &mut banner.content else {
                    return false;
                };
                view.handle_key_event(key);
                let _ = view.take_last_selected_index();
            }
            _ if matches!(
                self.keymap.list.action_for(key),
                Some(ListAction::MoveUp | ListAction::MoveDown | ListAction::Accept)
            ) =>
            {
                let InlineBannerContent::Actions(view) = &mut banner.content else {
                    return false;
                };
                view.handle_key_event(key);
                let _ = view.take_last_selected_index();
            }
            _ => return false,
        }
        self.request_redraw();
        true
    }
}
