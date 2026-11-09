import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

// A titled group of preference rows in one rounded card, like the preference groups of GNOME's own apps.
ColumnLayout {
    id: group
    property string title
    property string description
    default property alias rows: card.data

    Layout.fillWidth: true
    spacing: 0

    Label {
        visible: text !== ""
        text: group.title
        font.bold: true
        font.pointSize: Qt.application.font.pointSize * 1.0
        Layout.bottomMargin: group.description !== "" ? 4 : 10
    }
    Label {
        Layout.fillWidth: true
        visible: text !== ""
        text: group.description
        wrapMode: Text.Wrap
        color: palette.placeholderText
        Layout.bottomMargin: 10
    }
    Rectangle {
        Layout.fillWidth: true
        Layout.preferredHeight: card.implicitHeight
        radius: 12
        color: palette.base
        border.width: 1
        border.color: Qt.alpha(palette.windowText, 0.08)

        ColumnLayout {
            id: card
            anchors.fill: parent
            spacing: 0
        }
    }
}
