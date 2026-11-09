//! The proxy the person's environment already names stays in use behind the connection gate.

use std::collections::HashMap;

use ztbrowser_engine_qt::upstream_proxy::UpstreamProxy;

struct EnvironmentScenario {
    variables: HashMap<&'static str, &'static str>,
}

impl EnvironmentScenario {
    fn given(variables: &[(&'static str, &'static str)]) -> Self {
        Self { variables: variables.iter().copied().collect() }
    }

    fn proxy(&self) -> UpstreamProxy {
        UpstreamProxy::from_variables(|name| self.variables.get(name).map(|value| (*value).to_owned()))
    }
}

#[test]
fn should_connect_directly_when_the_environment_names_no_proxy() {
    let scenario = EnvironmentScenario::given(&[]);
    assert_eq!(scenario.proxy().for_tunnel("example.com"), None);
}

#[test]
fn should_tunnel_through_the_https_proxy_of_the_environment() {
    let scenario = EnvironmentScenario::given(&[("https_proxy", "http://127.0.0.1:3128")]);
    assert_eq!(scenario.proxy().for_tunnel("example.com").as_deref(), Some("127.0.0.1:3128"));
}

#[test]
fn should_read_the_upper_case_variable_when_the_lower_case_one_is_missing() {
    let scenario = EnvironmentScenario::given(&[("HTTPS_PROXY", "proxy.local:8080")]);
    assert_eq!(scenario.proxy().for_tunnel("example.com").as_deref(), Some("proxy.local:8080"));
}

#[test]
fn should_send_plain_http_through_the_http_proxy_and_not_the_https_one() {
    let scenario =
        EnvironmentScenario::given(&[("http_proxy", "http://10.0.0.1:80"), ("https_proxy", "http://10.0.0.2:81")]);
    assert_eq!(scenario.proxy().for_plain_http("example.com").as_deref(), Some("10.0.0.1:80"));
    assert_eq!(scenario.proxy().for_tunnel("example.com").as_deref(), Some("10.0.0.2:81"));
}

#[test]
fn should_skip_the_proxy_for_hosts_the_no_proxy_variable_lists() {
    let scenario = EnvironmentScenario::given(&[
        ("https_proxy", "http://127.0.0.1:3128"),
        ("no_proxy", "localhost,.internal.example"),
    ]);
    assert_eq!(scenario.proxy().for_tunnel("localhost"), None);
    assert_eq!(scenario.proxy().for_tunnel("wiki.internal.example"), None);
    assert!(scenario.proxy().for_tunnel("example.com").is_some());
}

#[test]
fn should_skip_the_proxy_for_every_host_when_no_proxy_is_a_star() {
    let scenario = EnvironmentScenario::given(&[("https_proxy", "http://127.0.0.1:3128"), ("no_proxy", "*")]);
    assert_eq!(scenario.proxy().for_tunnel("example.com"), None);
}

#[test]
fn should_ignore_a_proxy_that_needs_credentials_or_is_not_an_http_proxy() {
    let scenario = EnvironmentScenario::given(&[
        ("https_proxy", "http://user:secret@127.0.0.1:3128"),
        ("http_proxy", "socks5://127.0.0.1:9050"),
    ]);
    assert_eq!(scenario.proxy().for_tunnel("example.com"), None);
    assert_eq!(scenario.proxy().for_plain_http("example.com"), None);
}
