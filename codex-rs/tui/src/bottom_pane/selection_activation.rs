//! Shared activation state for recovery menus that can appear without a user request.

use crate::keymap::ListAction;
use crossterm::event::KeyEventKind;

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
    /// Classify activation permission from a semantic list action and terminal event kind.
    /// Parameters: self - activation state; action - parsed list operation; kind - press/repeat/release.
    /// Returns: whether this event may reach the selection view.
    pub(super) fn permits(&mut self, action: Option<ListAction>, kind: KeyEventKind) -> bool {
        #[cfg(test)]
        println!("self={self:?}, action={action:?}, kind={kind:?}");
        let permitted = if *self == Self::Immediate {
            true
        } else if kind != KeyEventKind::Press {
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
