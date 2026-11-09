//! The interface font is read from the desktop's `family size` setting.

use ztbrowser_qt::desktop_font::DesktopFont;

use ztbrowser_qt as _;

#[test]
fn should_read_the_family_and_size_of_a_quoted_setting() {
    let font = DesktopFont::parse("'Adwaita Sans 11'\n");
    assert_eq!(font.family(), Some("Adwaita Sans"));
    assert_eq!(font.point_size(), 11);
}

#[test]
fn should_keep_a_family_of_several_words() {
    assert_eq!(DesktopFont::parse("Noto Sans Display 12").family(), Some("Noto Sans Display"));
}

#[test]
fn should_use_the_default_size_when_the_setting_has_no_size() {
    let font = DesktopFont::parse("Cantarell");
    assert_eq!((font.family(), font.point_size()), (None, DesktopFont::DEFAULT_POINT_SIZE));
}

#[test]
fn should_limit_an_absurd_size() {
    assert_eq!(DesktopFont::parse("Cantarell 900").point_size(), 48);
}

#[test]
fn should_describe_the_font_for_qt_in_the_default_family_when_none_is_named() {
    assert_eq!(DesktopFont::parse("").description("Noto Sans"), "Noto Sans,11");
}
