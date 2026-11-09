//! A connection to a filtered host is refused unless a request to it was just allowed.
//!
//! Resource hints (`<link rel="preconnect">`) make the engine connect to a host before any request exists, so
//! no request filter sees them. The gate answers the question the engine's connection layer can ask instead:
//! "may I open a connection to this host?".

use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use ztbrowser_core::network::connection_clock::ConnectionClock;
use ztbrowser_core::network::connection_gate::ConnectionGate;
use ztbrowser_core::network::network_request::NetworkRequest;
use ztbrowser_core::network::request_decision::RequestDecision;
use ztbrowser_core::network::request_policy::RequestPolicy;

struct ManualClock {
    elapsed: Mutex<Duration>,
}

impl ManualClock {
    fn advance(&self, by: Duration) {
        *self.elapsed.lock().unwrap() += by;
    }
}

impl ConnectionClock for ManualClock {
    fn elapsed(&self) -> Duration {
        *self.elapsed.lock().unwrap()
    }
}

/// Blocks every request to the hosts it was given, as the tracker and Guard policies do.
struct BlocksHosts {
    hosts: Vec<&'static str>,
}

impl RequestPolicy for BlocksHosts {
    fn decide(&self, request: &NetworkRequest) -> RequestDecision {
        if request.host().is_some_and(|host| self.hosts.contains(&host)) {
            RequestDecision::Block
        } else {
            RequestDecision::Pass
        }
    }
}

struct GateScenario {
    clock: Arc<ManualClock>,
    gate: ConnectionGate,
}

impl GateScenario {
    fn given_a_gate_without_policies() -> Self {
        let clock = Arc::new(ManualClock { elapsed: Mutex::new(Duration::ZERO) });
        let gate = ConnectionGate::new(Arc::clone(&clock) as Arc<dyn ConnectionClock>);
        Self { clock, gate }
    }

    fn given_a_gate_that_filters(hosts: Vec<&'static str>) -> Self {
        let scenario = Self::given_a_gate_without_policies();
        scenario.gate.install(Arc::new(BlocksHosts { hosts }));
        scenario
    }
}

#[test]
fn should_refuse_every_host_until_policies_are_installed() {
    let scenario = GateScenario::given_a_gate_without_policies();
    assert!(!scenario.gate.admits("news.example"));
}

#[test]
fn should_admit_a_host_no_policy_blocks() {
    let scenario = GateScenario::given_a_gate_that_filters(vec!["ads.example"]);
    assert!(scenario.gate.admits("news.example"));
}

#[test]
fn should_refuse_a_host_a_policy_blocks() {
    let scenario = GateScenario::given_a_gate_that_filters(vec!["ads.example"]);
    assert!(!scenario.gate.admits("ads.example"));
}

#[test]
fn should_admit_a_filtered_host_after_a_request_to_it_was_allowed() {
    let scenario = GateScenario::given_a_gate_that_filters(vec!["ads.example"]);
    scenario.gate.approve("ads.example");
    assert!(scenario.gate.admits("ads.example"));
}

#[test]
fn should_stop_admitting_a_filtered_host_when_the_approval_has_expired() {
    let scenario = GateScenario::given_a_gate_that_filters(vec!["ads.example"]);
    scenario.gate.approve("ads.example");
    scenario.clock.advance(ConnectionGate::APPROVAL_LIFETIME + Duration::from_secs(1));
    assert!(!scenario.gate.admits("ads.example"));
}

#[test]
fn should_keep_admitting_a_filtered_host_while_requests_to_it_keep_being_allowed() {
    let scenario = GateScenario::given_a_gate_that_filters(vec!["ads.example"]);
    scenario.gate.approve("ads.example");
    scenario.clock.advance(ConnectionGate::APPROVAL_LIFETIME.checked_sub(Duration::from_secs(1)).unwrap());
    scenario.gate.approve("ads.example");
    scenario.clock.advance(ConnectionGate::APPROVAL_LIFETIME.checked_sub(Duration::from_secs(1)).unwrap());
    assert!(scenario.gate.admits("ads.example"));
}

#[test]
fn should_not_let_the_approval_of_one_host_open_its_neighbours() {
    let scenario = GateScenario::given_a_gate_that_filters(vec!["ads.example", "tags.example"]);
    scenario.gate.approve("ads.example");
    assert!(!scenario.gate.admits("tags.example"));
}

#[test]
fn should_compare_host_names_without_regard_to_case() {
    let scenario = GateScenario::given_a_gate_that_filters(vec!["ads.example"]);
    scenario.gate.approve("Ads.Example");
    assert!(scenario.gate.admits("ADS.example"));
}

#[test]
fn should_follow_the_policies_installed_last() {
    let scenario = GateScenario::given_a_gate_that_filters(vec!["ads.example"]);
    scenario.gate.install(Arc::new(BlocksHosts { hosts: vec!["tags.example"] }));
    assert!(scenario.gate.admits("ads.example"));
    assert!(!scenario.gate.admits("tags.example"));
}

#[test]
fn should_forget_expired_approvals_so_the_memory_stays_small() {
    let scenario = GateScenario::given_a_gate_that_filters(vec![]);
    for number in 0..ConnectionGate::PRUNE_THRESHOLD + 10 {
        scenario.gate.approve(&format!("host-{number}.example"));
    }
    scenario.clock.advance(ConnectionGate::APPROVAL_LIFETIME + Duration::from_secs(1));
    scenario.gate.approve("fresh.example");
    assert!(scenario.gate.approval_count() <= 1);
}
