import QtQuick
import QtQuick.Controls

// A flat header-bar button that shows one of the application's icons in the theme's text colour. `checked`
// (a toggle such as the shield) shows the button pressed in, as libadwaita draws a toggled flat button.
//
// The icon is drawn here, by `TintedIcon`, and not by the style: Plasma's Breeze style paints a button's icon with
// `Kirigami.Icon`, which applies `icon.color` to icons that are masks only, so a plain SVG kept its own dark
// colour and vanished on a dark bar.
ToolButton {
    id: control
    property string iconName
    property string tip
    /// A button whose popover is open shows no tooltip over it.
    property bool tipHidden: false

    flat: true
    padding: 0
    icon.width: 16
    icon.height: 16
    opacity: enabled ? 1.0 : 0.35
    implicitWidth: 34
    implicitHeight: 34
    Accessible.name: tip
    ToolTip.visible: hovered && tip !== "" && !tipHidden
    ToolTip.delay: 500
    ToolTip.text: tip

    contentItem: Item {
        TintedIcon {
            anchors.centerIn: parent
            name: control.iconName
            size: control.icon.width
            color: control.palette.buttonText
        }
    }

    background: Rectangle {
        radius: 9
        color: control.down ? Qt.alpha(palette.windowText, 0.19)
             : control.checked ? Qt.alpha(palette.windowText, 0.12)
             : control.hovered ? Qt.alpha(palette.windowText, 0.08) : "transparent"
    }
}
