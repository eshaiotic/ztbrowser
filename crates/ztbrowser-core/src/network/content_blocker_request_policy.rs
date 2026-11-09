//! Blocks requests the way DuckDuckGo's tracker rules say, evaluating the rule list they build.
//!
//! DuckDuckGo's `TrackerRadarKit` turns the Tracker Data Set and the privacy configuration into a list of
//! content-blocking rules (`ContentBlockerRulesBuilder`, ported in the protection module). WebKitGTK compiles
//! that list itself. Chromium has no such list, so this policy evaluates the same rules directly, in order,
//! with the same effect of `ignore-previous-rules`. Everything DuckDuckGo fixed on macOS is therefore fixed
//! here: the allowlist of trackers that sites need (a bot check, a payment frame), the sites where blocking
//! stands down, per-URL rules with their exceptions, and first-party aliases that point at trackers.

use std::sync::Arc;

use crate::network::content_blocker_rule_index::ContentBlockerRuleIndex;
use crate::network::content_blocker_rule_matcher::RequestFacts;
use crate::network::network_request::NetworkRequest;
use crate::network::request_decision::RequestDecision;
use crate::network::request_policy::RequestPolicy;
use crate::network::resource_type::ResourceType;
use crate::protection::content_blocker_rule::ContentBlockerRule;
use crate::protection::content_blocker_rule::ResourceType as BlockerResourceType;
use crate::protection::content_blocker_rules_builder::ContentBlockerRulesBuilder;
use crate::protection::content_blocker_rules_builder::RuleExceptions;
use crate::protection::privacy_configuration::PrivacyConfiguration;
use crate::protection::tracker_data_set::TrackerDataSet;
use crate::web_app::registrable_domain_lookup::RegistrableDomainLookup;

pub struct ContentBlockerRequestPolicy {
    index: ContentBlockerRuleIndex,
    sites: Arc<dyn RegistrableDomainLookup + Send + Sync>,
}

impl ContentBlockerRequestPolicy {
    /// The rules DuckDuckGo's published data build. Sites the person excluded are not part of them: the policy
    /// that wraps this one answers for those, and can change while the browser runs.
    #[must_use]
    pub fn from_documents(
        data_set: &TrackerDataSet,
        configuration: &PrivacyConfiguration,
        sites: Arc<dyn RegistrableDomainLookup + Send + Sync>,
    ) -> Self {
        let exceptions = RuleExceptions::from_configuration(configuration, vec![]);
        Self::new(ContentBlockerRulesBuilder::new(data_set).build_rules(&exceptions), sites)
    }

    #[must_use]
    pub fn new(rules: Vec<ContentBlockerRule>, sites: Arc<dyn RegistrableDomainLookup + Send + Sync>) -> Self {
        Self { index: ContentBlockerRuleIndex::new(rules), sites }
    }

    /// WebKit's request kinds: `raw` is everything that is not one of the others (XHR, fetch, beacons).
    fn blocker_kind(kind: ResourceType) -> BlockerResourceType {
        match kind {
            ResourceType::MainFrame | ResourceType::SubFrame => BlockerResourceType::Document,
            ResourceType::Script => BlockerResourceType::Script,
            ResourceType::Stylesheet => BlockerResourceType::StyleSheet,
            ResourceType::Image => BlockerResourceType::Image,
            ResourceType::Font => BlockerResourceType::Font,
            ResourceType::Media => BlockerResourceType::Media,
            ResourceType::Xhr | ResourceType::WebSocket | ResourceType::Other => BlockerResourceType::Raw,
        }
    }
}

impl RequestPolicy for ContentBlockerRequestPolicy {
    fn decide(&self, request: &NetworkRequest) -> RequestDecision {
        let Some(host) = request.host().filter(|_| request.goes_over_the_network() && !request.is_navigation()) else {
            return RequestDecision::Pass;
        };
        let facts = RequestFacts {
            address: request.address(),
            page_host: request.first_party_host(),
            kind: Self::blocker_kind(request.resource_type()),
            is_third_party: request.is_third_party(self.sites.as_ref()),
        };
        if self.index.blocks(host, &facts) { RequestDecision::Block } else { RequestDecision::Pass }
    }
}
