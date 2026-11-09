import QtQuick
import QtQuick.Controls

// A text button as libadwaita draws one in a banner or a preference row; the suggested one in the accent colour.
AbstractButton {
    id: button
    property string label
    property bool suggested: false

    implicitHeight: 32
    implicitWidth: buttonLabel.implicitWidth + 28
    Accessible.name: label
    background: Rectangle {
        radius: 9
        color: button.suggested
               ? (button.down ? Qt.darker(palette.highlight, 1.2) : button.hovered ? Qt.lighter(palette.highlight, 1.08) : palette.highlight)
               : Qt.alpha(palette.windowText, button.down ? 0.19 : button.hovered ? 0.12 : 0.08)
    }
    contentItem: Label {
        id: buttonLabel
        text: button.label
        font.bold: true
        color: button.suggested ? palette.highlightedText : palette.windowText
        horizontalAlignment: Text.AlignHCenter
        verticalAlignment: Text.AlignVCenter
    }
}
