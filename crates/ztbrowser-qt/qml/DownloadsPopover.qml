import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

// The toolbar's downloads list, drawn as the GTK edition's popover: the five latest downloads, then the manager,
// "Clear List" and the download folder as centred rows. It hangs under the header bar against the window's right edge.
Popup {
    id: popover
    required property var hub
    required property var bridge
    signal managerRequested()

    /// When the popover last closed: the toolbar button ignores the click that closed it by pressing outside.
    property double closedAt: 0

    parent: Overlay.overlay
    x: parent.width - width - 8
    y: 54
    width: 377
    padding: 8
    modal: false
    focus: true
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside
    onAboutToShow: { hub.popoverOpen = true; hub.reload() }
    onClosed: { closedAt = Date.now(); hub.popoverOpen = false }

    enter: Transition {
        NumberAnimation { property: "opacity"; from: 0; to: 1; duration: 110; easing.type: Easing.OutCubic }
        NumberAnimation { property: "scale"; from: 0.96; to: 1; duration: 140; easing.type: Easing.OutCubic }
    }
    exit: Transition {
        NumberAnimation { property: "opacity"; to: 0; duration: 80 }
    }
    transformOrigin: Popup.TopRight

    background: Rectangle {
        radius: 12
        color: palette.base
        border.width: 1
        border.color: Qt.alpha(palette.windowText, 0.16)
    }

    // One of the closing rows: its icon and title, lit while the pointer is over it.
    component CloseRow: AbstractButton {
        id: entry
        property string iconName
        Layout.fillWidth: true
        implicitHeight: 36
        background: Rectangle {
            radius: 8
            color: !entry.enabled ? "transparent"
                 : entry.down ? Qt.alpha(palette.windowText, 0.14)
                 : entry.hovered || entry.visualFocus ? Qt.alpha(palette.windowText, 0.08) : "transparent"
        }
        contentItem: RowLayout {
            spacing: 12
            opacity: entry.enabled ? 1 : 0.4
            TintedIcon { Layout.leftMargin: 8; name: entry.iconName; size: 16 }
            Label { Layout.fillWidth: true; text: entry.text; elide: Text.ElideRight }
        }
    }

    contentItem: ColumnLayout {
        spacing: 2

        Label {
            visible: popover.hub.latest.count === 0
            Layout.fillWidth: true
            Layout.topMargin: 18
            Layout.bottomMargin: 18
            horizontalAlignment: Text.AlignHCenter
            text: popover.bridge.message("No downloads yet")
            color: Qt.alpha(palette.windowText, 0.6)
        }
        ListView {
            Layout.fillWidth: true
            Layout.preferredHeight: contentHeight
            visible: popover.hub.latest.count > 0
            interactive: false
            spacing: 2
            model: popover.hub.latest
            delegate: DownloadRow {
                bridge: popover.bridge
                onShowInFolder: function(path) { popover.hub.showInFolder(path); popover.close() }
            }
        }
        Rectangle {
            Layout.fillWidth: true
            Layout.topMargin: 4
            Layout.bottomMargin: 4
            Layout.preferredHeight: 1
            color: Qt.alpha(palette.windowText, 0.12)
        }
        CloseRow {
            iconName: "download"
            text: popover.bridge.message("Show All Downloads")
            onClicked: { popover.close(); popover.managerRequested() }
        }
        CloseRow {
            iconName: "folder"
            text: popover.bridge.message("Go to Download Folder")
            onClicked: { popover.hub.openFolder(); popover.close() }
        }
        CloseRow {
            iconName: "trash"
            text: popover.bridge.message("Clear List")
            enabled: popover.hub.latest.count > 0
            onClicked: popover.hub.clearList()
        }
    }
}
