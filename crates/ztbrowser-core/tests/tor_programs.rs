//! Finding the `tor` and transport programs the Tor window runs, bundled ones first, never through `PATH`.

use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

use ztbrowser_core::tor::pluggable_transport::PluggableTransport;
use ztbrowser_core::tor::tor_programs::TorPrograms;

struct InstalledProgramsScenario {
    executable: PathBuf,
    installed: BTreeSet<PathBuf>,
}

impl InstalledProgramsScenario {
    fn given_ztbrowser_at(executable: &str) -> Self {
        Self { executable: PathBuf::from(executable), installed: BTreeSet::new() }
    }

    fn with_program(mut self, path: &str) -> Self {
        self.installed.insert(PathBuf::from(path));
        self
    }

    fn when_located(&self) -> ProgramsOutcome {
        ProgramsOutcome { programs: TorPrograms::locate(&self.executable, |path| self.installed.contains(path)) }
    }
}

struct ProgramsOutcome {
    programs: Option<TorPrograms>,
}

impl ProgramsOutcome {
    fn programs(&self) -> &TorPrograms {
        self.programs.as_ref().expect("tor is installed")
    }

    fn then_runs_tor(&self, tor: &str, library_directory: Option<&str>) {
        let programs = self.programs();
        assert_eq!((programs.tor(), programs.library_directory()), (Path::new(tor), library_directory.map(Path::new)));
    }

    fn then_runs_transport_with(&self, transport: PluggableTransport, program: Option<&str>) {
        assert_eq!(self.programs().program_for(transport), program.map(Path::new));
    }

    fn then_finds_no_tor(&self) {
        assert!(self.programs.is_none());
    }
}

#[test]
fn should_prefer_tor_bundled_with_ztbrowser_and_its_libraries() {
    InstalledProgramsScenario::given_ztbrowser_at("/tmp/.mount_InDuck/usr/bin/ztbrowser")
        .with_program("/tmp/.mount_InDuck/usr/lib/ztbrowser/tor/tor")
        .with_program("/usr/bin/tor")
        .when_located()
        .then_runs_tor(
            "/tmp/.mount_InDuck/usr/lib/ztbrowser/tor/tor",
            Some("/tmp/.mount_InDuck/usr/lib/ztbrowser/tor/lib"),
        );
}

#[test]
fn should_run_the_systems_tor_with_the_systems_libraries() {
    InstalledProgramsScenario::given_ztbrowser_at("/usr/bin/ztbrowser")
        .with_program("/usr/sbin/tor")
        .when_located()
        .then_runs_tor("/usr/sbin/tor", None);
}

#[test]
fn should_prefer_lyrebird_over_its_predecessor_for_obfs4() {
    InstalledProgramsScenario::given_ztbrowser_at("/usr/bin/ztbrowser")
        .with_program("/usr/bin/tor")
        .with_program("/usr/bin/obfs4proxy")
        .with_program("/usr/local/bin/lyrebird")
        .when_located()
        .then_runs_transport_with(PluggableTransport::Obfs4, Some("/usr/local/bin/lyrebird"));
}

#[test]
fn should_know_webtunnel_cannot_run_without_lyrebird() {
    InstalledProgramsScenario::given_ztbrowser_at("/usr/bin/ztbrowser")
        .with_program("/usr/bin/tor")
        .with_program("/usr/bin/obfs4proxy")
        .when_located()
        .then_runs_transport_with(PluggableTransport::WebTunnel, None);
}

#[test]
fn should_find_no_tor_in_directories_outside_its_search_list() {
    InstalledProgramsScenario::given_ztbrowser_at("/usr/bin/ztbrowser")
        .with_program("/home/user/.local/bin/tor")
        .when_located()
        .then_finds_no_tor();
}
