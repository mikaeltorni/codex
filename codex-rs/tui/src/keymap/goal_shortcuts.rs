//! Goal defaults yield to existing custom shortcuts and overlapping chord prefixes.

use super::*;

/// Resolve a goal shortcut with compatibility for configurations predating its default.
/// Parameters: keymap - configured actions; chords - resolved chords; action - shortcut identity;
/// defaults - native fallback; alias - canonical default key. Returns: bindings or a config error.
pub(super) fn resolve_goal_shortcut(
    keymap: &TuiKeymap,
    chords: &RuntimeChordKeymap,
    action: KeymapActionId,
    defaults: &[KeyBinding],
    alias: &str,
) -> Result<Vec<KeyBinding>, String> {
    let configured =
        bindings::configured_binding_for_action(keymap, action).and_then(Option::as_ref);
    if configured.is_none()
        && (configured_main_surface_alias_is_used(keymap, alias)
            || configured_context_alias_is_used(&keymap.vim_search, alias)
            || chords.bindings.iter().any(|binding| {
                binding.action.context.overlaps(action.context)
                    && defaults.contains(&binding.chord.prefix)
            }))
    {
        return Ok(Vec::new());
    }
    resolve_bindings(configured, defaults, &action.config_path())
}
