//! Choosing where a download lands: the name comes from the website, so it is treated as hostile.

use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;

use ztbrowser_core::download::download_file_name::DownloadFileName;

struct DownloadScenario {
    suggested: String,
    existing: HashSet<PathBuf>,
}

impl DownloadScenario {
    fn given_site_suggests(name: &str) -> Self {
        Self { suggested: name.to_owned(), existing: HashSet::new() }
    }

    fn with_existing_file(mut self, path: &str) -> Self {
        self.existing.insert(PathBuf::from(path));
        self
    }

    fn when_saving_to_downloads(&self) -> PathBuf {
        DownloadFileName::from_suggestion(&self.suggested)
            .first_free_path_in(Path::new("/home/me/Downloads"), |path| self.existing.contains(path))
    }
}

#[test]
fn should_keep_ordinary_file_name() {
    assert_eq!(
        DownloadScenario::given_site_suggests("statement 2026-10.pdf").when_saving_to_downloads(),
        Path::new("/home/me/Downloads/statement 2026-10.pdf")
    );
}

#[test]
fn should_never_leave_the_downloads_directory() {
    assert_eq!(
        DownloadScenario::given_site_suggests("../../.bashrc").when_saving_to_downloads(),
        Path::new("/home/me/Downloads/bashrc")
    );
}

#[test]
fn should_not_create_hidden_files() {
    assert_eq!(
        DownloadScenario::given_site_suggests(".profile").when_saving_to_downloads(),
        Path::new("/home/me/Downloads/profile")
    );
}

#[test]
fn should_drop_control_characters() {
    assert_eq!(
        DownloadScenario::given_site_suggests("in\nvoice\t.pdf").when_saving_to_downloads(),
        Path::new("/home/me/Downloads/invoice.pdf")
    );
}

#[test]
fn should_treat_windows_path_separators_as_directories() {
    assert_eq!(
        DownloadScenario::given_site_suggests("C:\\Users\\me\\..\\x.pdf").when_saving_to_downloads(),
        Path::new("/home/me/Downloads/x.pdf")
    );
}

#[test]
fn should_name_nameless_download() {
    assert_eq!(
        DownloadScenario::given_site_suggests("..").when_saving_to_downloads(),
        Path::new("/home/me/Downloads/download")
    );
}

#[test]
fn should_number_name_instead_of_overwriting_existing_file() {
    let scenario = DownloadScenario::given_site_suggests("report.tar.gz")
        .with_existing_file("/home/me/Downloads/report.tar.gz")
        .with_existing_file("/home/me/Downloads/report (2).tar.gz");

    assert_eq!(scenario.when_saving_to_downloads(), Path::new("/home/me/Downloads/report (3).tar.gz"));
}

#[test]
fn should_shorten_overlong_name_but_keep_its_extension() {
    let path = DownloadScenario::given_site_suggests(&format!("{}.pdf", "ż".repeat(300))).when_saving_to_downloads();
    let name = path.file_name().unwrap().to_str().unwrap();

    assert!(name.len() <= 255 && name.ends_with("ż.pdf"), "{name}");
}

#[test]
fn should_remove_a_right_to_left_override_that_disguises_the_extension() {
    assert_eq!(
        DownloadScenario::given_site_suggests("invoice\u{202E}fdp.exe").when_saving_to_downloads(),
        Path::new("/home/me/Downloads/invoicefdp.exe")
    );
}

#[test]
fn should_remove_zero_width_characters_from_the_name() {
    assert_eq!(
        DownloadScenario::given_site_suggests("re\u{200B}port\u{FEFF}.pdf").when_saving_to_downloads(),
        Path::new("/home/me/Downloads/report.pdf")
    );
}
