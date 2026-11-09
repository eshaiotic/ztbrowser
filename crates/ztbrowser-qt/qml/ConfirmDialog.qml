import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

// A question with two answers, drawn as the GTK edition's AdwAlertDialog draws it and as "About" and "Install as
// App" are drawn here: rounded, a centred heading and explanation, and a foot of two buttons side by side. The
// confirming answer is red when it destroys something, in libadwaita's destructive colour.
Dialog {
    id: dialog
    required property var bridge
    property string heading
    property string body
    /// Catalogue messages of the two buttons; "_" marks the keyboard letter in GTK and is dropped here.
    property string cancelLabel: "_Cancel"
    property string confirmLabel
    property bool destructive: true
    readonly property bool isDark: Qt.styleHints.colorScheme === Qt.Dark
    readonly property color destructiveColor: isDark ? "#ff7b63" : "#c01c28"

    /// The person chose the confirming answer.
    signal confirmed()

    modal: true
    padding: 0
    // Centred in the window, though the component that owns it may have no size of its own.
    parent: Overlay.overlay
    anchors.centerIn: parent
    width: Math.min(400, parent.width - 48)

    background: Rectangle {
        radius: 16
        color: palette.window
        border.width: 1
        border.color: Qt.alpha(palette.windowText, 0.2)
    }

    contentItem: ColumnLayout {
        spacing: 0

        Label {
            Layout.fillWidth: true
            Layout.topMargin: 24
            Layout.leftMargin: 24
            Layout.rightMargin: 24
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.Wrap
            textFormat: Text.PlainText
            text: dialog.heading
            font.bold: true
            font.pixelSize: 20
        }
        Label {
            Layout.fillWidth: true
            Layout.topMargin: 12
            Layout.leftMargin: 24
            Layout.rightMargin: 24
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.Wrap
            textFormat: Text.PlainText
            text: dialog.body
        }
        RowLayout {
            Layout.fillWidth: true
            Layout.topMargin: 24
            Layout.preferredHeight: 52
            spacing: 0
            Repeater {
                model: [ { label: dialog.cancelLabel, confirming: false }, { label: dialog.confirmLabel, confirming: true } ]
                delegate: AbstractButton {
                    id: choice
                    required property var modelData
                    readonly property bool confirming: modelData.confirming
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    onClicked: { dialog.close(); if (confirming) dialog.confirmed() }
                    background: Rectangle {
                        color: choice.down ? Qt.alpha(palette.windowText, 0.12) : choice.hovered ? Qt.alpha(palette.windowText, 0.06) : "transparent"
                        bottomLeftRadius: choice.confirming ? 0 : 16
                        bottomRightRadius: choice.confirming ? 16 : 0
                        Rectangle { width: parent.width; height: 1; color: Qt.alpha(palette.windowText, 0.12) }
                        Rectangle { visible: choice.confirming; width: 1; height: parent.height; color: Qt.alpha(palette.windowText, 0.12) }
                    }
                    contentItem: Label {
                        text: dialog.bridge.message(choice.modelData.label).replace("_", "")
                        horizontalAlignment: Text.AlignHCenter
                        verticalAlignment: Text.AlignVCenter
                        font.bold: choice.confirming
                        color: choice.confirming ? (dialog.destructive ? dialog.destructiveColor : palette.highlight) : palette.windowText
                    }
                }
            }
        }
    }
}
