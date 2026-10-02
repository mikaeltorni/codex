//! Account-banner lookups used by the usage-limit wait menu.

use super::super::ChatWidget;
use crate::backend_banners::BackendBanner;
use crate::backend_banners::LUNA_RESERVE_BANNER;
use crate::bottom_pane::ActionableBanner;
use crate::model_catalog::LUNA_RESERVE_MODEL;

impl ChatWidget {
    pub(in super::super) fn applicable_backend_banner(&self) -> Option<&BackendBanner> {
        let banner = self.backend_banner_state.banner.as_ref()?;
        if self.backend_banner_state.dismissed && self.usage_limit_wait_retry_at_ms.is_none() {
            return None;
        }
        // Keep Reserve recovery actions available while switching to Reserve.
        if banner.banner_type == LUNA_RESERVE_BANNER {
            return Some(banner);
        }
        // Explicit fallback payloads describe the selected replacement, not a pending switch.
        let matches_selected_model = match banner.blocked_model_slug.as_deref() {
            Some(blocked) if !banner.fallback_model_slugs.is_empty() => {
                blocked != self.current_model()
                    && banner
                        .fallback_model_slugs
                        .iter()
                        .any(|model| model == self.current_model())
            }
            Some(_) | None => {
                banner
                    .model_slug
                    .as_deref()
                    .is_none_or(|model| model == self.current_model())
                    || self.backend_banner_notice_model.as_deref() == Some(self.current_model())
            }
        };
        matches_selected_model.then_some(banner)
    }

    pub(in super::super) fn backend_banner_actionable_content(
        &self,
        banner: &BackendBanner,
    ) -> ActionableBanner {
        let mut content = banner.actionable_banner(self.clock_format);
        if banner.banner_type == LUNA_RESERVE_BANNER && self.current_model() != LUNA_RESERVE_MODEL {
            content.title = "Usage limit reached".to_string();
            content.description =
                "Your included usage is exhausted. Choose an option below to continue.".to_string();
        }
        content
    }

    /// Refresh the wait menu when its inputs changed, and the ordinary banner otherwise.
    pub(in super::super) fn refresh_after_backend_banner_update(
        &mut self,
        usage_wait_inputs_changed: bool,
    ) {
        if usage_wait_inputs_changed && self.usage_limit_wait_retry_at_ms.is_some() {
            self.refresh_usage_limit_wait_banner();
        } else {
            self.refresh_backend_banner_visibility();
        }
    }

    /// Reapply a normal inline account banner after the usage-wait notice released the composer.
    pub(in super::super) fn restore_backend_banner_after_usage_wait(&mut self) {
        self.observe_backend_banner_view();
        let reserve_banner_has_modal =
            self.backend_banner_state
                .banner
                .as_ref()
                .is_some_and(|banner| {
                    banner.banner_type == LUNA_RESERVE_BANNER && self.bottom_pane.has_active_modal()
                });
        if reserve_banner_has_modal {
            return;
        }
        self.backend_banner_state.presented = None;
        self.refresh_backend_banner_visibility();
    }
}
