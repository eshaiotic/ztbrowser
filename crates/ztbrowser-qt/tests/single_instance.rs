//! One process per identity: a second launch hands its addresses to the first and goes away, so a profile's
//! files are never opened twice.

use tempfile::TempDir;
use ztbrowser_qt::single_instance::Instance;
use ztbrowser_qt::single_instance::SingleInstance;

use ztbrowser_qt as _;

const IDENTITY: &str = "io.github.ztbrowser.ZTBrowser";

fn given_a_running_instance(runtime: &TempDir) -> ztbrowser_qt::single_instance::PrimaryInstance {
    match SingleInstance::acquire(runtime.path(), IDENTITY, &[]).expect("acquire") {
        Instance::Primary(primary) => primary,
        Instance::Forwarded => panic!("nothing was running"),
    }
}

fn wait_for_forwarded(primary: &ztbrowser_qt::single_instance::PrimaryInstance) -> Vec<Vec<String>> {
    for _ in 0..200 {
        let forwarded = primary.take_forwarded();
        if !forwarded.is_empty() {
            return forwarded;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    Vec::new()
}

#[test]
fn should_become_the_primary_instance_when_nothing_runs() {
    let runtime = TempDir::new().expect("runtime");
    let _primary = given_a_running_instance(&runtime);
}

#[test]
fn should_hand_addresses_to_the_running_instance() {
    let runtime = TempDir::new().expect("runtime");
    let primary = given_a_running_instance(&runtime);
    let second =
        SingleInstance::acquire(runtime.path(), IDENTITY, &["https://example.com/".to_owned()]).expect("second");
    assert!(matches!(second, Instance::Forwarded));
    assert_eq!(wait_for_forwarded(&primary), vec![vec!["https://example.com/".to_owned()]]);
}

#[test]
fn should_ask_the_running_instance_to_come_forward_when_no_address_is_given() {
    let runtime = TempDir::new().expect("runtime");
    let primary = given_a_running_instance(&runtime);
    let _ = SingleInstance::acquire(runtime.path(), IDENTITY, &[]).expect("second");
    assert_eq!(wait_for_forwarded(&primary), vec![Vec::<String>::new()]);
}

#[test]
fn should_drop_addresses_that_are_not_web_pages() {
    let runtime = TempDir::new().expect("runtime");
    let primary = given_a_running_instance(&runtime);
    let addresses =
        ["file:///etc/passwd".to_owned(), "javascript:alert(1)".to_owned(), "https://ok.example/".to_owned()];
    let _ = SingleInstance::acquire(runtime.path(), IDENTITY, &addresses).expect("second");
    assert_eq!(wait_for_forwarded(&primary), vec![vec!["https://ok.example/".to_owned()]]);
}

#[test]
fn should_take_over_from_an_instance_that_died_without_cleaning_up() {
    let runtime = TempDir::new().expect("runtime");
    drop(given_a_running_instance(&runtime));
    assert!(matches!(SingleInstance::acquire(runtime.path(), IDENTITY, &[]).expect("again"), Instance::Primary(_)));
}

#[test]
fn should_keep_other_identities_apart() {
    let runtime = TempDir::new().expect("runtime");
    let _browser = given_a_running_instance(&runtime);
    let app = SingleInstance::acquire(runtime.path(), "io.github.ztbrowser.ZTBrowser.WebApp.mail", &[]).expect("app");
    assert!(matches!(app, Instance::Primary(_)));
}
