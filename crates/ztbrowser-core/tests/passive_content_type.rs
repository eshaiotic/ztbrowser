//! Which downloads open with one click: only files that merely display something.

use ztbrowser_core::download::passive_content_type::PassiveContentType;

struct OpeningScenario {
    content_type: String,
}

impl OpeningScenario {
    fn given_file_of_type(content_type: &str) -> Self {
        Self { content_type: content_type.to_owned() }
    }

    fn then_opens_with_one_click(&self) -> bool {
        PassiveContentType::new(&self.content_type).is_passive()
    }
}

#[test]
fn should_open_a_pdf_with_one_click() {
    assert!(OpeningScenario::given_file_of_type("application/pdf").then_opens_with_one_click());
}

#[test]
fn should_open_pictures_music_and_films_with_one_click() {
    for content_type in ["image/png", "audio/mpeg", "video/mp4"] {
        assert!(OpeningScenario::given_file_of_type(content_type).then_opens_with_one_click(), "{content_type}");
    }
}

#[test]
fn should_not_open_programs_scripts_and_launchers_with_one_click() {
    for content_type in [
        "application/x-executable",
        "application/x-shellscript",
        "application/x-desktop",
        "application/vnd.appimage",
        "application/x-java-archive",
        "application/zip",
        "image/svg+xml-compressed",
        "image/x-eps",
        "audio/x-mpegurl",
        "video/x-matroska",
    ] {
        assert!(!OpeningScenario::given_file_of_type(content_type).then_opens_with_one_click(), "{content_type}");
    }
}

#[test]
fn should_not_open_an_svg_picture_with_one_click_because_it_can_carry_scripts() {
    assert!(!OpeningScenario::given_file_of_type("image/svg+xml").then_opens_with_one_click());
}

#[test]
fn should_not_open_an_unknown_type_with_one_click() {
    for content_type in ["", "application/octet-stream"] {
        assert!(!OpeningScenario::given_file_of_type(content_type).then_opens_with_one_click(), "{content_type:?}");
    }
}
