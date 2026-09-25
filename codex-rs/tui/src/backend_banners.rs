//! Backend banner payload and CTA mapping, supplied by the existing account usage read.
//!
//! The backend owns eligibility, copy, actions, and ordered model fallback instructions.

use codex_protocol::account::PlanType;
use serde::Deserialize;
use serde::Deserializer;

mod actions;
mod render;

pub(crate) const LUNA_RESERVE_BANNER: &str = "luna_reserve";
pub(crate) const LUNA_RESERVE_RECOVERY_VIEW_ID: &str = "luna-reserve-recovery";

#[cfg(test)]
#[path = "backend_banners_tests.rs"]
mod tests;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub(crate) struct BackendBanner {
    pub(crate) banner_type: String,
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) ctas: Vec<BackendBannerCta>,
    pub(crate) reset_at: Option<i64>,
    pub(crate) model_slug: Option<String>,
    pub(crate) blocked_model_slug: Option<String>,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub(crate) fallback_model_slugs: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub(crate) presentation: BannerPresentation,
    request_url: Option<String>,
    #[serde(skip)]
    pub(crate) account_id: String,
    #[serde(skip)]
    pub(crate) plan_type: Option<PlanType>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum BannerPresentation {
    #[default]
    Inline,
    Dismissible,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub(crate) struct BackendBannerCta {
    action: String,
    label: String,
}

impl BackendBanner {
    /// Supply plan recovery choices when a usage wait has no visible account banner.
    /// Rendering this banner uses the same CTA destinations as the backend response.
    pub(crate) fn recovery_fallback_for_plan(plan_type: PlanType) -> Option<Self> {
        let (action, label) = match plan_type {
            PlanType::Free | PlanType::Go | PlanType::Plus | PlanType::ProLite => {
                ("open_pricing_dialog", "Upgrade")
            }
            plan if plan.is_workspace_account() => ("request_increase", "Request increase"),
            PlanType::Pro | PlanType::Unknown => return None,
            _ => return None,
        };
        Some(Self {
            banner_type: "usage_limit_recovery_fallback".to_string(),
            title: String::new(),
            description: String::new(),
            ctas: vec![BackendBannerCta {
                action: action.to_string(),
                label: label.to_string(),
            }],
            reset_at: None,
            model_slug: None,
            blocked_model_slug: None,
            fallback_model_slugs: Vec::new(),
            presentation: BannerPresentation::Inline,
            request_url: None,
            account_id: String::new(),
            plan_type: Some(plan_type),
        })
    }

    /// Parse supported, bounded content before constructing rendered copy or CTA closures.
    pub(crate) fn parse(raw: &serde_json::Value) -> Option<Self> {
        serde_json::from_value::<Self>(raw.clone())
            .ok()
            .filter(|banner| {
                let valid_slug = |slug: &str| {
                    !slug.trim().is_empty()
                        && slug.len() <= 256
                        && !slug.chars().any(char::is_control)
                };
                banner.title.len() <= 1024
                    && banner.description.len() <= 4096
                    && banner.ctas.len() <= 8
                    && !banner.title.trim().is_empty()
                    && banner.title.lines().count() <= 3
                    && banner.description.lines().count() <= 12
                    && banner.blocked_model_slug.as_deref().is_none_or(valid_slug)
                    && banner.fallback_model_slugs.len() <= 16
                    && banner
                        .fallback_model_slugs
                        .iter()
                        .all(|slug| valid_slug(slug))
            })
    }

    /// The normal usage-limit error converts a member's credits CTA into an increase request.
    /// Apply that same recovery choice while the core keeps the turn alive for auto-resume.
    pub(crate) fn for_usage_limit_wait(&self) -> Self {
        let mut banner = self.clone();
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
}

// The account backend sends explicit nulls for omitted presentation and fallback fields.
fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}
