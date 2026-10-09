//! Shared activation state for recovery menus that can appear without a user request.

use crate::keymap::ListAction;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use crossterm::event::KeyModifiers;

/// Require fresh navigation before accepting an asynchronously presented menu's default row.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum SelectionActivation {
    /// Ordinary user-opened pickers retain their immediate accept behavior.
    #[default]
    Immediate,
    /// A recovery surface is waiting for deliberate keyboard navigation.
    NavigationRequired,
    /// Navigation occurred; exactly one subsequent accept may activate a row.
    NavigationArmed,
}

impl SelectionActivation {
    /// Parse the explicit numbered shortcut shared by inline and focused recovery views.
    /// Parameters: key - incoming terminal event.
    /// Returns: a plain numbered press for the list dispatcher, or None for ordinary input.
    pub(super) fn numbered_shortcut(key: KeyEvent) -> Option<KeyEvent> {
        #[cfg(test)]
        println!("key={key:?}");
        let result = if key.kind == KeyEventKind::Press
            && key.modifiers == KeyModifiers::ALT
            && matches!(key.code, KeyCode::Char('1'..='9'))
        {
            Some(KeyEvent {
                modifiers: KeyModifiers::NONE,
                ..key
            })
        } else {
            None
        };
        #[cfg(test)]
        println!("{result:?}");
        result
    }

    /// Classify activation permission from a semantic list action and terminal event kind.
    /// Parameters: self - activation state; action - parsed list operation; key - incoming terminal event.
    /// Returns: whether this event may reach the selection view.
    pub(super) fn permits(&mut self, action: Option<ListAction>, key: KeyEvent) -> bool {
        #[cfg(test)]
        println!("self={self:?}, action={action:?}, key={key:?}");
        let permitted = if *self == Self::Immediate {
            true
        } else if key.kind != KeyEventKind::Press {
            false
        } else if matches!(key.code, KeyCode::Char(_)) && Self::numbered_shortcut(key).is_none() {
            // Character bindings include composer editing/history controls.
            // Only the explicit Alt-number shortcut can bypass composer input.
            *self = Self::NavigationRequired;
            false
        } else {
            match action {
                Some(ListAction::Accept) => {
                    let armed = *self == Self::NavigationArmed;
                    *self = Self::NavigationRequired;
                    armed
                }
                Some(
                    ListAction::MoveUp
                    | ListAction::MoveDown
                    | ListAction::MoveLeft
                    | ListAction::MoveRight
                    | ListAction::PageUp
                    | ListAction::PageDown
                    | ListAction::JumpTop
                    | ListAction::JumpBottom,
                ) => {
                    *self = Self::NavigationArmed;
                    true
                }
                Some(ListAction::Cancel) | None => {
                    *self = Self::NavigationRequired;
                    true
                }
            }
        };
        #[cfg(test)]
        println!("{permitted}");
        permitted
    }
}
