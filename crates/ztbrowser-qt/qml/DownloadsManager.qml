import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

// The whole download history, as the GTK edition shows it: a rounded dialog with its title, a destructive "Clear
// History" pill and a close circle on top, a search field, and the downloads in one rounded card.
Dialog {
    id: manager
    required property var hub
    required property var bridge
    readonly property bool isDark: Qt.styleHints.colorScheme === Qt.Dark

    modal: true
    padding: 0
    width: Math.min(560, parent ? parent.width - 48 : 560)
    height: Math.min(560, parent ? parent.height - 48 : 560)
    anchors.centerIn: parent
    onAboutToShow: { hub.managerOpen = true; hub.reload(); search.text = "" }
    onClosed: hub.managerOpen = false

    background: Rectangle {
        radius: 12
        color: palette.window
        border.width: 1
        border.color: Qt.alpha(palette.windowText, 0.2)
    }

    contentItem: ColumnLayout {
        spacing: 0

        Item {
            Layout.fillWidth: true
            Layout.preferredHeight: 48
            Label {
                anchors.centerIn: parent
                text: manager.bridge.message("Downloads")
                font.bold: true
            }
            AbstractButton {
                id: clearHistory
                anchors.right: closeCircle.left
                anchors.rightMargin: 8
                anchors.verticalCenter: parent.verticalCenter
                implicitHeight: 34
                implicitWidth: clearLabel.implicitWidth + 20
                enabled: manager.hub.everything.count > 0
                opacity: enabled ? 1 : 0.4
                onClicked: manager.hub.clearHistory()
                background: Rectangle {
                    radius: 9
                    color: Qt.alpha(manager.isDark ? "#ff7b63" : "#c01c28", clearHistory.down ? 0.28 : clearHistory.hovered ? 0.22 : 0.15)
                }
                contentItem: Label {
                    id: clearLabel
                    text: manager.bridge.message("Clear History")
                    font.bold: true
                    color: manager.isDark ? "#ff7b63" : "#c01c28"
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                }
            }
            AbstractButton {
                id: closeCircle
                anchors.right: parent.right
                anchors.rightMargin: 12
                anchors.verticalCenter: parent.verticalCenter
                implicitWidth: 24
                implicitHeight: 24
                onClicked: manager.close()
                background: Rectangle { radius: 12; color: Qt.alpha(palette.windowText, parent.hovered ? 0.14 : 0.08) }
                contentItem: Item { TintedIcon { anchors.centerIn: parent; name: "close"; size: 16 } }
            }
        }

        // The search field: a grey rounded bar with a magnifier, which lights up like a field when it has focus.
        Rectangle {
            Layout.fillWidth: true
            Layout.leftMargin: 12
            Layout.rightMargin: 12
            Layout.bottomMargin: 12
            Layout.preferredHeight: 36
            radius: 9
            color: search.activeFocus ? palette.base : Qt.alpha(palette.windowText, 0.08)
            border.width: search.activeFocus ? 2 : 0
            border.color: palette.highlight
            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: 10
                anchors.rightMargin: 10
                spacing: 8
                TintedIcon { name: "search"; size: 16; opacity: 0.7 }
                TextField {
                    id: search
                    Layout.fillWidth: true
                    background: null
                    padding: 0
                    placeholderText: manager.bridge.message("Search Downloads")
                    selectByMouse: true
                }
            }
        }

        Rectangle {
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.leftMargin: 12
            Layout.rightMargin: 12
            Layout.bottomMargin: 12
            radius: 12
            color: palette.base
            clip: true

            ListView {
                anchors.fill: parent
                anchors.margins: 0
                clip: true
                model: manager.hub.everything
                delegate: DownloadRow {
                    bridge: manager.bridge
                    removable: true
                    onRemoveRequested: function(id) { manager.hub.remove(id) }
                    visible: search.text === "" || name.toLowerCase().includes(search.text.toLowerCase())
                    height: visible ? implicitHeight : 0
                    onShowInFolder: function(path) { manager.hub.showInFolder(path) }
                }
                ScrollBar.vertical: ScrollBar { }
            }
            Label {
                anchors.centerIn: parent
                visible: manager.hub.everything.count === 0
                text: manager.bridge.message("No downloads yet")
                color: Qt.alpha(palette.windowText, 0.6)
            }
        }
    }
}
