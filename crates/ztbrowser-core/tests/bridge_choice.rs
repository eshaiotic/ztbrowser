//! Choosing how the Tor window reaches Tor, and remembering the choice.

use std::collections::BTreeSet;

use serde_json::Value;
use serde_json::json;
use ztbrowser_core::tor::bridge_choice::BridgeChoice;
use ztbrowser_core::tor::bridge_line::BridgeLine;
use ztbrowser_core::tor::built_in_bridges::BuiltInBridges;
use ztbrowser_core::tor::pluggable_transport::PluggableTransport;

struct BridgeChoiceScenario {
    stored: Value,
}

impl BridgeChoiceScenario {
    fn given_stored(stored: Value) -> Self {
        Self { stored }
    }

    fn given_chosen(choice: &BridgeChoice) -> Self {
        Self { stored: choice.to_json() }
    }

    fn when_loaded(&self) -> BridgeChoiceOutcome {
        BridgeChoiceOutcome { choice: BridgeChoice::from_json(&self.stored) }
    }
}

struct BridgeChoiceOutcome {
    choice: BridgeChoice,
}

impl BridgeChoiceOutcome {
    fn then_is(&self, expected: &BridgeChoice) {
        assert_eq!(&self.choice, expected);
    }

    fn then_uses_bridges(&self, count: usize, transport: PluggableTransport) {
        let bridges = self.choice.bridges();
        assert_eq!((bridges.len(), self.choice.needed_transports()), (count, BTreeSet::from([transport])));
        assert!(bridges.iter().all(|bridge| bridge.transport() == Some(transport)));
    }
}

#[test]
fn should_connect_directly_when_nothing_was_chosen_yet() {
    BridgeChoiceScenario::given_stored(json!({})).when_loaded().then_is(&BridgeChoice::Direct);
}

#[test]
fn should_use_tor_browsers_built_in_snowflake_bridges() {
    BridgeChoiceScenario::given_chosen(&BridgeChoice::Snowflake)
        .when_loaded()
        .then_uses_bridges(2, PluggableTransport::Snowflake);
}

#[test]
fn should_use_tor_browsers_built_in_obfs4_bridges() {
    BridgeChoiceScenario::given_chosen(&BridgeChoice::Obfs4)
        .when_loaded()
        .then_uses_bridges(7, PluggableTransport::Obfs4);
}

#[test]
fn should_remember_bridges_of_the_users_own() {
    let own = BridgeLine::parse_all("obfs4 192.0.2.10:443 cert=abc iat-mode=0\n192.0.2.11:9001").expect("bridges");

    BridgeChoiceScenario::given_chosen(&BridgeChoice::Custom(own.clone()))
        .when_loaded()
        .then_is(&BridgeChoice::Custom(own));
}

#[test]
fn should_drop_stored_bridge_that_no_longer_parses() {
    BridgeChoiceScenario::given_stored(json!({
        "bridges": "custom",
        "customBridges": ["obfs4 192.0.2.10:443 cert=abc", "obfs4 192.0.2.12:443 cert=#"],
    }))
    .when_loaded()
    .then_is(&BridgeChoice::Custom(BridgeLine::parse_all("obfs4 192.0.2.10:443 cert=abc").expect("bridge")));
}

#[test]
fn should_fall_back_to_direct_connection_when_no_custom_bridge_is_left() {
    BridgeChoiceScenario::given_stored(json!({"bridges": "custom", "customBridges": ["not a bridge"]}))
        .when_loaded()
        .then_is(&BridgeChoice::Direct);
}

#[test]
fn should_ship_only_built_in_bridges_that_parse() {
    let listed = BuiltInBridges::listed_lines();

    assert!(listed.iter().all(|line| BridgeLine::parse(line).is_ok()), "a built-in bridge line does not parse");
}
