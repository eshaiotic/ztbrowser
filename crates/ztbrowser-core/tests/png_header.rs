//! Checking a downloaded web app icon before any image decoder sees it.

use ztbrowser_core::web_app::png_header::PngHeader;

struct IconFileScenario {
    bytes: Vec<u8>,
}

impl IconFileScenario {
    fn given_png_of_size(width: u32, height: u32) -> Self {
        let mut bytes = b"\x89PNG\r\n\x1a\n\0\0\0\x0dIHDR".to_vec();
        bytes.extend_from_slice(&width.to_be_bytes());
        bytes.extend_from_slice(&height.to_be_bytes());
        bytes.extend_from_slice(&[8, 6, 0, 0, 0]);
        Self { bytes }
    }

    fn given_file_starting_with(bytes: &[u8]) -> Self {
        Self { bytes: bytes.to_vec() }
    }

    fn when_inspected(&self) -> Option<PngHeader> {
        PngHeader::read(&self.bytes)
    }
}

#[test]
fn should_read_dimensions_of_a_png() {
    let header = IconFileScenario::given_png_of_size(192, 180).when_inspected().unwrap();

    assert_eq!((header.width, header.height), (192, 180));
}

#[test]
fn should_accept_icon_sized_png() {
    assert!(IconFileScenario::given_png_of_size(512, 512).when_inspected().unwrap().is_reasonable_icon());
}

#[test]
fn should_reject_decompression_bomb() {
    assert!(!IconFileScenario::given_png_of_size(30_000, 30_000).when_inspected().unwrap().is_reasonable_icon());
}

#[test]
fn should_reject_empty_image() {
    assert!(!IconFileScenario::given_png_of_size(0, 16).when_inspected().unwrap().is_reasonable_icon());
}

#[test]
fn should_not_treat_other_formats_as_png() {
    assert_eq!(IconFileScenario::given_file_starting_with(b"<svg xmlns=").when_inspected(), None);
}
