import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

// "Install as App", as the GTK edition's alert dialog asks it: what will happen, the app's name to confirm, and
// the two answers.
Dialog {
    id: dialog
    required property var bridge
    property string host
    property string suggestedName

    /// The person chose Install, with this name.
    signal confirmed(string name)

    modal: true
    padding: 0
    // Centred in the window, though the component that owns it has no size of its own.
    parent: Overlay.overlay
    anchors.centerIn: parent
    width: Math.min(420, parent.width - 48)
    function submit() { close(); confirmed(nameField.text.trim()) }

    onAboutToShow: nameField.text = suggestedName
    onOpened: { nameField.forceActiveFocus(); nameField.selectAll() }

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
            text: dialog.bridge.message("Install as App")
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
            text: dialog.bridge.message("{host} will open in its own window, with cookies and logins kept apart from the browser and every other app.").replace("{host}", dialog.host)
        }
        Rectangle {
            Layout.fillWidth: true
            Layout.topMargin: 16
            Layout.leftMargin: 24
            Layout.rightMargin: 24
            Layout.preferredHeight: 36
            radius: 9
            color: nameField.activeFocus ? palette.base : Qt.alpha(palette.windowText, 0.08)
            border.width: nameField.activeFocus ? 2 : 0
            border.color: palette.highlight
            TextField {
                id: nameField
                anchors.fill: parent
                anchors.leftMargin: 10
                anchors.rightMargin: 10
                background: null
                padding: 0
                selectByMouse: true
                maximumLength: 80
                onAccepted: dialog.submit()
            }
        }
        RowLayout {
            Layout.fillWidth: true
            Layout.topMargin: 24
            Layout.preferredHeight: 52
            spacing: 0
            Repeater {
                model: [ { label: "_Cancel", primary: false }, { label: "_Install", primary: true } ]
                delegate: AbstractButton {
                    id: choice
                    required property var modelData
                    readonly property bool isInstall: modelData.primary
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    onClicked: isInstall ? dialog.submit() : dialog.close()
                    background: Rectangle {
                        color: choice.down ? Qt.alpha(palette.windowText, 0.12) : choice.hovered ? Qt.alpha(palette.windowText, 0.06) : "transparent"
                        bottomLeftRadius: choice.isInstall ? 0 : 16
                        bottomRightRadius: choice.isInstall ? 16 : 0
                        Rectangle { width: parent.width; height: 1; color: Qt.alpha(palette.windowText, 0.12) }
                    }
                    contentItem: Label {
                        text: dialog.bridge.message(choice.modelData.label).replace("_", "")
                        horizontalAlignment: Text.AlignHCenter
                        verticalAlignment: Text.AlignVCenter
                        font.bold: choice.isInstall
                        color: choice.isInstall ? palette.highlight : palette.windowText
                    }
                }
            }
        }
    }
}
