//! Writing the configuration of the Tor window's own Tor.

use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

use ztbrowser_core::tor::bridge_choice::BridgeChoice;
use ztbrowser_core::tor::bridge_line::BridgeLine;
use ztbrowser_core::tor::pluggable_transport::PluggableTransport;
use ztbrowser_core::tor::tor_configuration::TorConfiguration;
use ztbrowser_core::tor::tor_configuration::TorConfigurationProblem;
use ztbrowser_core::tor::tor_programs::TorPrograms;

struct TorConfigurationScenario {
    data_directory: PathBuf,
    installed: BTreeSet<PathBuf>,
    bridges: BridgeChoice,
}

impl TorConfigurationScenario {
    const OWNER_PROCESS: u32 = 4242;

    fn given_data_directory(data_directory: &str) -> Self {
        let installed = BTreeSet::from([PathBuf::from("/usr/bin/tor")]);
        Self { data_directory: PathBuf::from(data_directory), installed, bridges: BridgeChoice::Direct }
    }

    fn with_program(mut self, path: &str) -> Self {
        self.installed.insert(PathBuf::from(path));
        self
    }

    fn with_bridges(mut self, bridges: BridgeChoice) -> Self {
        self.bridges = bridges;
        self
    }

    fn with_own_bridges(self, text: &str) -> Self {
        self.with_bridges(BridgeChoice::Custom(BridgeLine::parse_all(text).expect("bridges")))
    }

    fn when_configured(&self) -> TorConfigurationOutcome {
        let programs = TorPrograms::locate(Path::new("/usr/bin/ztbrowser"), |path| self.installed.contains(path))
            .expect("tor is installed");
        let configuration =
            TorConfiguration::new(self.data_directory.clone(), Self::OWNER_PROCESS, &self.bridges, &programs);
        TorConfigurationOutcome { configuration }
    }
}

struct TorConfigurationOutcome {
    configuration: Result<TorConfiguration, TorConfigurationProblem>,
}

impl TorConfigurationOutcome {
    fn configuration(&self) -> &TorConfiguration {
        self.configuration.as_ref().expect("a usable configuration")
    }

    fn torrc_lines(&self) -> Vec<String> {
        self.configuration().to_torrc().expect("torrc").lines().map(str::to_owned).collect()
    }

    fn then_writes(&self, expected: &[&str]) {
        assert_eq!(self.torrc_lines(), expected);
    }

    fn then_contains(&self, expected: &str) {
        assert!(self.torrc_lines().iter().any(|line| line == expected), "no {expected:?} in {:?}", self.torrc_lines());
    }

    fn then_starts_tor_with(&self, arguments: &[&str]) {
        assert_eq!(
            self.configuration().command_line(),
            arguments.iter().map(Into::into).collect::<Vec<std::ffi::OsString>>()
        );
    }

    fn then_is_refused_because(&self, problem: &TorConfigurationProblem) {
        assert_eq!(self.configuration.as_ref().err(), Some(problem));
    }
}

#[test]
fn should_run_a_client_only_tor_that_picks_its_own_ports_and_exits_with_ztbrowser() {
    TorConfigurationScenario::given_data_directory("/home/ala/.local/share/ztbrowser/tor")
        .when_configured()
        .then_writes(&[
            "DataDirectory \"/home/ala/.local/share/ztbrowser/tor\"",
            "ControlPortWriteToFile \"/home/ala/.local/share/ztbrowser/tor/control-port\"",
            "CookieAuthFile \"/home/ala/.local/share/ztbrowser/tor/control-auth-cookie\"",
            "SocksPort auto",
            "ControlPort auto",
            "CookieAuthentication 1",
            "__OwningControllerProcess 4242",
            "ClientOnly 1",
            "AvoidDiskWrites 1",
            "Log notice stdout",
        ]);
}

#[test]
fn should_ignore_the_systems_tor_configuration_and_its_defaults() {
    TorConfigurationScenario::given_data_directory("/data/tor").when_configured().then_starts_tor_with(&[
        "-f",
        "/data/tor/torrc",
        "--defaults-torrc",
        "/dev/null",
    ]);
}

#[test]
fn should_quote_a_data_directory_with_spaces_and_quotes() {
    TorConfigurationScenario::given_data_directory("/home/ala/My \"Tor\"\\data")
        .when_configured()
        .then_contains("DataDirectory \"/home/ala/My \\\"Tor\\\"\\\\data\"");
}

#[test]
fn should_start_the_transport_program_the_chosen_bridges_need() {
    TorConfigurationScenario::given_data_directory("/data/tor")
        .with_program("/usr/bin/snowflake-client")
        .with_bridges(BridgeChoice::Snowflake)
        .when_configured()
        .then_contains("ClientTransportPlugin snowflake exec /usr/bin/snowflake-client");
}

#[test]
fn should_tell_tor_to_use_the_users_bridges() {
    let outcome = TorConfigurationScenario::given_data_directory("/data/tor")
        .with_own_bridges("192.0.2.11:9001")
        .when_configured();

    outcome.then_writes(&[
        "DataDirectory \"/data/tor\"",
        "ControlPortWriteToFile \"/data/tor/control-port\"",
        "CookieAuthFile \"/data/tor/control-auth-cookie\"",
        "SocksPort auto",
        "ControlPort auto",
        "CookieAuthentication 1",
        "__OwningControllerProcess 4242",
        "ClientOnly 1",
        "AvoidDiskWrites 1",
        "Log notice stdout",
        "UseBridges 1",
        "Bridge 192.0.2.11:9001",
    ]);
}

#[test]
fn should_refuse_bridges_whose_transport_is_not_installed() {
    TorConfigurationScenario::given_data_directory("/data/tor")
        .with_bridges(BridgeChoice::Obfs4)
        .when_configured()
        .then_is_refused_because(&TorConfigurationProblem::MissingTransport(PluggableTransport::Obfs4));
}
