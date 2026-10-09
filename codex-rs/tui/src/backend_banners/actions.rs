//! Map backend CTA names to existing CLI actions and desktop-equivalent browser destinations.

use super::BackendBanner;
use crate::app_event::AppEvent;
use crate::bottom_pane::SelectionAction;
use crate::bottom_pane::SelectionItem;
use codex_app_server_protocol::AddCreditsNudgeCreditType;
use codex_protocol::account::PlanType;

const USAGE_URL: &str = "https://chatgpt.com/codex/settings/usage";
const WORKSPACE_USAGE_URL: &str = "https://chatgpt.com/admin/usage-limits/workspace";

/// Open the regular reset picker; selecting a credit still requires its existing confirmation.
/// Parameters: name - the displayed reset action label.
/// Returns: the reset picker action with a stable selection identity.
pub(crate) fn rate_limit_reset_selection_item(name: String) -> SelectionItem {
    #[cfg(test)]
    println!("name={name}");
    let result = SelectionItem {
        selection_key: Some("reset_usage".to_string()),
        name,
        actions: vec![Box::new(|tx| {
            tx.send(AppEvent::OpenRateLimitResetCredits);
        })],
        ..Default::default()
    };
    #[cfg(test)]
    println!("result={:p}", &result);
    result
}

#[derive(Debug)]
pub(super) enum BannerAction {
    OpenUrl(String),
    NotifyOwner(AddCreditsNudgeCreditType),
    ResetUsage,
}

impl BannerAction {
    /// Route backend CTAs and local recovery choices through the same app events.
    /// Parameters: self - the resolved recovery action; name - its displayed label.
    /// Returns: a selectable action identified by its operation and destination.
    pub(super) fn selection_item(self, name: String) -> SelectionItem {
        #[cfg(test)]
        println!("self={self:?}, name={name}");
        let (selection_key, action): (String, SelectionAction) = match self {
            Self::OpenUrl(url) => (
                format!("open_url:{url}"),
                Box::new(move |tx| {
                    tx.send(AppEvent::OpenUrlInBrowser { url: url.clone() });
                }),
            ),
            Self::NotifyOwner(credit_type) => (
                format!("notify_owner:{credit_type:?}"),
                Box::new(move |tx| {
                    tx.send(AppEvent::SendAddCreditsNudgeEmail { credit_type });
                }),
            ),
            Self::ResetUsage => {
                let result = rate_limit_reset_selection_item(name);
                #[cfg(test)]
                println!("result={:p}", &result);
                return result;
            }
        };
        let result = SelectionItem {
            selection_key: Some(selection_key),
            name,
            actions: vec![action],
            ..Default::default()
        };
        #[cfg(test)]
        println!("result={:p}", &result);
        result
    }
}

impl BackendBanner {
    /// Supply one plan recovery action without manufacturing an account banner payload.
    pub(crate) fn recovery_fallback_for_plan(plan_type: PlanType) -> Option<SelectionItem> {
        let (action, label) = match plan_type {
            PlanType::Free | PlanType::Go | PlanType::Plus | PlanType::ProLite => (
                BannerAction::OpenUrl(pricing_url(Some(plan_type))),
                "Upgrade",
            ),
            PlanType::Team
            | PlanType::SelfServeBusinessProLite
            | PlanType::SelfServeBusinessUsageBased
            | PlanType::Business
            | PlanType::Ent26
            | PlanType::EnterpriseCbpAutomation
            | PlanType::EnterpriseCbpUsageBased
            | PlanType::Enterprise
            | PlanType::Edu
            | PlanType::EduPlus
            | PlanType::EduPro => (
                BannerAction::NotifyOwner(AddCreditsNudgeCreditType::UsageLimit),
                "Request increase",
            ),
            PlanType::Pro | PlanType::ProMax | PlanType::Unknown => return None,
        };
        Some(action.selection_item(label.to_string()))
    }

    pub(super) fn resolve_action(&self, action: &str) -> Option<BannerAction> {
        match action {
            "notify_owner" | "contact_owner" => {
                return Some(BannerAction::NotifyOwner(
                    AddCreditsNudgeCreditType::Credits,
                ));
            }
            "request_increase" => {
                let Some(destination) = self.request_url.as_deref() else {
                    return Some(BannerAction::NotifyOwner(
                        AddCreditsNudgeCreditType::UsageLimit,
                    ));
                };
                let url = url::Url::parse(destination).ok()?;
                if !matches!(url.scheme(), "https" | "http")
                    || url.host_str().is_none()
                    || !url.username().is_empty()
                    || url.password().is_some()
                {
                    return None;
                }
                return Some(BannerAction::OpenUrl(url.to_string()));
            }
            // Keep the existing picker and explicit confirmation before consuming a reset.
            "reset_usage" => return Some(BannerAction::ResetUsage),
            _ => {}
        }
        let destination = match action {
            "add_credits" | "buy_credits"
                if self.plan_type.is_some_and(PlanType::is_workspace_account) =>
            {
                "https://chatgpt.com/admin/billing?codex_credit_action=add_credits".to_string()
            }
            "add_credits" | "buy_credits" => format!("{USAGE_URL}?credits_modal=true"),
            "buy_reset" => "https://chatgpt.com/codex/purchase/reset".to_string(),
            "view_usage" | "request_increase_usage_settings" => USAGE_URL.to_string(),
            "view_workspace_usage" | "increase_spend_cap" => WORKSPACE_USAGE_URL.to_string(),
            "open_plus_pricing_web" => "https://chatgpt.com/explore/plus".to_string(),
            "open_pro_pricing_web" => "https://chatgpt.com/explore/pro".to_string(),
            "open_pricing_dialog" => pricing_url(self.plan_type),
            // Desktop-only referral and Premium dialogs need their own CLI flow.
            _ => return None,
        };
        let mut destination = url::Url::parse(&destination).ok()?;
        if destination.path().starts_with("/admin/") {
            if self.account_id.is_empty() {
                return None;
            }
            // The existing admin route selects the requested workspace before opening the modal.
            destination
                .query_pairs_mut()
                .append_pair("account_id", &self.account_id);
        }
        Some(BannerAction::OpenUrl(destination.to_string()))
    }
}

/// Keep the backend pricing CTA and the wait menu's upgrade destination aligned.
/// Parameters: plan_type - the account plan determining the advertised upgrade.
/// Returns: the plan's pricing URL.
fn pricing_url(plan_type: Option<PlanType>) -> String {
    #[cfg(test)]
    println!("plan_type={plan_type:?}");
    let mut url = url::Url::parse("https://chatgpt.com/")
        .unwrap_or_else(|error| unreachable!("invalid constant pricing URL: {error}"));
    let target = if matches!(plan_type, Some(PlanType::Plus | PlanType::ProLite)) {
        "pro"
    } else {
        "plus"
    };
    url.query_pairs_mut()
        .append_pair("cta_tab", "personal")
        .append_pair("highlight_plan", target);
    if plan_type == Some(PlanType::ProLite) {
        url.query_pairs_mut().append_pair("pro_variant", "2x");
    }
    url.set_fragment(Some("pricing"));
    let result = url.to_string();
    #[cfg(test)]
    println!("{result}");
    result
}
