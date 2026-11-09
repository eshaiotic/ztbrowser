//! Deciding what happens to each network request a page makes, independently of the web engine.
//!
//! The engine adapter turns its own request object into a [`network_request::NetworkRequest`] and carries out
//! the [`request_decision::RequestDecision`] a [`request_policy::RequestPolicy`] returns.

pub mod advertising_tracker_request_policy;
pub mod chained_request_policy;
pub mod connection_clock;
pub mod connection_gate;
pub mod content_blocker_request_policy;
pub mod content_blocker_rule_index;
pub mod content_blocker_rule_matcher;
pub mod guard_request_policy;
pub mod https_upgrade_request_policy;
pub mod link_cleaning_request_policy;
pub mod network_request;
pub mod public_suffix_lookup;
pub mod replaceable_request_policy;
pub mod request_decision;
pub mod request_policy;
pub mod resource_type;
pub mod site_allow_list;
pub mod third_party_request_policy;
pub mod unprotected_site_request_policy;
