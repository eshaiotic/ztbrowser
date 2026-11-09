//! The request and cookie policies of one profile, put together from the core's building blocks.

use std::sync::Arc;
use std::sync::RwLock;

use ztbrowser_core::guard::shared_guard_engine::SharedGuardEngine;
use ztbrowser_core::navigation::https_upgrade_policy::HttpsUpgradePolicy;
use ztbrowser_core::network::advertising_tracker_request_policy::AdvertisingTrackerRequestPolicy;
use ztbrowser_core::network::chained_request_policy::ChainedRequestPolicy;
use ztbrowser_core::network::content_blocker_request_policy::ContentBlockerRequestPolicy;
use ztbrowser_core::network::guard_request_policy::GuardRequestPolicy;
use ztbrowser_core::network::https_upgrade_request_policy::HttpsUpgradeRequestPolicy;
use ztbrowser_core::network::link_cleaning_request_policy::LinkCleaningRequestPolicy;
use ztbrowser_core::network::public_suffix_lookup::PublicSuffixLookup;
use ztbrowser_core::network::replaceable_request_policy::ReplaceableRequestPolicy;
use ztbrowser_core::network::request_policy::RequestPolicy;
use ztbrowser_core::network::site_allow_list::SiteAllowList;
use ztbrowser_core::network::third_party_request_policy::ThirdPartyRequestPolicy;
use ztbrowser_core::network::unprotected_site_request_policy::UnprotectedSiteRequestPolicy;
use ztbrowser_core::profile::unprotected_sites::UnprotectedSites;
use ztbrowser_core::protection::privacy_configuration::PrivacyConfiguration;
use ztbrowser_core::protection::tracker_data_set::TrackerDataSet;
use ztbrowser_core::storage::cookie_policy::CookiePolicy;
use ztbrowser_core::storage::replaceable_cookie_policy::ReplaceableCookiePolicy;
use ztbrowser_core::storage::strict_cookie_policy::StrictCookiePolicy;
use ztbrowser_core::storage::tracker_cookie_policy::TrackerCookiePolicy;

use crate::protection_documents::ProtectionDocuments;

/// How strictly a profile treats third parties.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileStrictness {
    Standard,
    /// The *Strict* profile: third-party requests and cookies are refused unless the person allowed them.
    Strict,
}

/// The request and cookie policies of a profile. Both are the ones the engine holds for its whole life; what
/// they decide with changes when protection data arrives ([`Self::use_documents`]).
pub struct ProfilePolicies {
    pub request: Arc<dyn RequestPolicy>,
    pub cookies: Arc<dyn CookiePolicy>,
    request_slot: Arc<ReplaceableRequestPolicy>,
    cookie_slot: Arc<ReplaceableCookiePolicy>,
    recipe: PolicyRecipe,
}

/// What the policies are built from besides the protection data, kept to build them again.
struct PolicyRecipe {
    unprotected_sites: Arc<RwLock<UnprotectedSites>>,
    guard: Arc<SharedGuardEngine>,
    allow_list: Arc<RwLock<SiteAllowList>>,
    strictness: ProfileStrictness,
}

impl ProfilePolicies {
    /// Without downloaded protection data the policies still upgrade to HTTPS and, in the strict profile,
    /// refuse third parties; tracker blocking and link cleaning start when the data exists.
    #[must_use]
    pub fn assemble(
        documents: Option<&ProtectionDocuments>,
        unprotected_sites: Arc<RwLock<UnprotectedSites>>,
        guard: Arc<SharedGuardEngine>,
        allow_list: SiteAllowList,
        strictness: ProfileStrictness,
    ) -> Self {
        let recipe =
            PolicyRecipe { unprotected_sites, guard, allow_list: Arc::new(RwLock::new(allow_list)), strictness };
        let (request, cookies) = recipe.build(documents);
        let request_slot = Arc::new(ReplaceableRequestPolicy::new(request));
        let cookie_slot = Arc::new(ReplaceableCookiePolicy::new(cookies));
        Self {
            request: Arc::clone(&request_slot) as Arc<dyn RequestPolicy>,
            cookies: Arc::clone(&cookie_slot) as Arc<dyn CookiePolicy>,
            request_slot,
            cookie_slot,
            recipe,
        }
    }

    /// From now on the policies decide with `documents`: freshly downloaded protection data takes effect
    /// without a restart.
    pub fn use_documents(&self, documents: &ProtectionDocuments) {
        let (request, cookies) = self.recipe.build(Some(documents));
        self.request_slot.replace(request);
        self.cookie_slot.replace(cookies);
    }
}

impl PolicyRecipe {
    const HTTPS_FEATURE: &'static str = "https";

    fn build(&self, documents: Option<&ProtectionDocuments>) -> (Arc<dyn RequestPolicy>, Arc<dyn CookiePolicy>) {
        let sites = Arc::new(PublicSuffixLookup);
        let is_strict = self.strictness == ProfileStrictness::Strict;

        let mut policies: Vec<Box<dyn RequestPolicy>> =
            vec![Box::new(ThirdPartyRequestPolicy::new(is_strict, Arc::clone(&self.allow_list), sites.clone()))];
        let mut cookies: Arc<dyn CookiePolicy> = Arc::new(StrictCookiePolicy::new(Arc::clone(&self.allow_list)));
        let mut excluded_from_https = vec![];
        if let Some(documents) = documents {
            let configuration =
                PrivacyConfiguration::new(documents.configuration.clone(), PrivacyConfiguration::COMPATIBILITY_VERSION);
            let data_set = Arc::new(TrackerDataSet::from_json(&documents.tracker_data));
            excluded_from_https = configuration.exception_domains(Self::HTTPS_FEATURE);
            // DuckDuckGo's rules first, with their exceptions; then the stricter policy that has none.
            policies.push(Box::new(ContentBlockerRequestPolicy::from_documents(
                &data_set,
                &configuration,
                sites.clone(),
            )));
            policies.push(Box::new(AdvertisingTrackerRequestPolicy::new(&data_set, sites)));
            policies.push(Box::new(LinkCleaningRequestPolicy::new(configuration, Arc::clone(&self.unprotected_sites))));
            if !is_strict {
                cookies = Arc::new(TrackerCookiePolicy::new(data_set));
            }
        }
        policies.push(Box::new(GuardRequestPolicy::new(Arc::clone(&self.guard))));
        let upgrades = Arc::new(RwLock::new(HttpsUpgradePolicy::new(&excluded_from_https)));
        policies.push(Box::new(HttpsUpgradeRequestPolicy::new(upgrades)));

        let chain: Arc<dyn RequestPolicy> = Arc::new(ChainedRequestPolicy::new(policies));
        (Arc::new(UnprotectedSiteRequestPolicy::new(chain, Arc::clone(&self.unprotected_sites))), cookies)
    }
}
