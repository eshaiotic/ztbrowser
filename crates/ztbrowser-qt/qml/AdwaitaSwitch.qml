import QtQuick
import QtQuick.Controls

// A switch drawn as libadwaita draws it: a round-ended track, the accent colour when on, a white round knob.
Switch {
    id: control
    implicitWidth: 46
    implicitHeight: 26
    padding: 0

    indicator: Rectangle {
        implicitWidth: 46
        implicitHeight: 26
        radius: 13
        color: control.checked ? palette.highlight : Qt.alpha(palette.windowText, control.hovered ? 0.2 : 0.15)

        Rectangle {
            width: 22
            height: 22
            radius: 11
            y: 2
            x: control.checked ? parent.width - width - 2 : 2
            color: "white"
            border.width: 1
            border.color: Qt.rgba(0, 0, 0, 0.1)
            Behavior on x { NumberAnimation { duration: 120; easing.type: Easing.OutCubic } }
        }
    }
    contentItem: Item { }
}
