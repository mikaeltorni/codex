//! Usage-wait adaptations that sit beside the ordinary account-banner payload.

use super::BackendBanner;
use crate::bottom_pane::SelectionItem;
use serde::Deserialize;
use serde::Deserializer;

/// Treat explicit JSON nulls as the field's default. Account reads send null for omitted values.
pub(super) fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}

/// The normal usage-limit error converts a member's credits CTA into an increase request.
/// Apply that same recovery choice while the core keeps the turn alive for auto-resume.
pub(crate) fn banner_for_usage_limit_wait(banner: &BackendBanner) -> BackendBanner {
    let mut banner = banner.clone();
    if banner.banner_type == "workspace_member_credits_depleted" {
        let mut request_increase = false;
        for cta in &mut banner.ctas {
            if matches!(cta.action.as_str(), "notify_owner" | "contact_owner") {
                cta.action = "request_increase".to_string();
                cta.label = "Request increase".to_string();
                request_increase = true;
            }
        }
        if request_increase {
            banner.description = "Your turn will automatically continue when the usage limit resets. Request a limit increase to continue sooner.".to_string();
        }
    }
    banner
}

impl BackendBanner {
    /// Render valid CTAs together with their backend action names for semantic deduplication.
    pub(crate) fn selection_items(&self) -> impl Iterator<Item = (&str, SelectionItem)> {
        self.ctas.iter().filter_map(|cta| {
            if cta.label.trim().is_empty()
                || cta.label.len() > 256
                || cta.label.chars().any(char::is_control)
            {
                return None;
            }
            let action = self.resolve_action(&cta.action)?;
            Some((
                cta.action.as_str(),
                action.selection_item(cta.label.clone()),
            ))
        })
    }
}
