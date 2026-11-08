import QtQuick
import QtQuick.Controls
import QtTest

// The header bar's buttons draw their icons themselves, in the theme's text colour, and hand the style no icon to
// paint: Plasma's Breeze style paints a button's icon with `Kirigami.Icon`, which applies `icon.color` to mask icons
// only, so a plain SVG kept its own dark colour and vanished on a dark bar.
Item {
    width: 120
    height: 60

    IconButton {
        id: button
        iconName: "back"
        palette.buttonText: "#ffffff"
    }

    TestCase {
        name: "IconButton"
        when: windowShown

        function test_hands_the_style_no_icon_to_paint() {
            compare(button.icon.source.toString(), "")
        }

        function test_draws_the_icon_in_the_text_colour_of_the_theme() {
            const drawn = button.contentItem.children[0]
            compare(drawn.name, "back")
            compare(drawn.color.toString(), "#ffffff")
        }

        function test_follows_the_text_colour_when_the_theme_changes() {
            button.palette.buttonText = "#2e2e32"
            compare(button.contentItem.children[0].color.toString(), "#2e2e32")
        }
    }
}
