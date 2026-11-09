import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

// The profiles as tiles, as Chrome's profile menu shows them: a coloured circle with the initial, the name below,
// the current profile ringed, and a last tile to add one. Clicking another profile opens it in its own window
// (each profile is a separate process with its own cookies and storage). It hangs under the header bar.
Popup {
    id: popup
    required property var bridge
    required property var profiles
    /// The tiles as the profile bridge lists them.
    property var tiles: []
    /// While true the "add" tile shows a name field.
    property bool naming: false

    /// A profile (not the default, not the one open here) is to be deleted: ask first.
    signal deleteRequested(string id, string name)
    /// Something worth telling the person happened (a toast).
    signal notice(string text)

    /// When the popup last closed: the toolbar button ignores the click that closed it by pressing outside.
    property double closedAt: 0

    // libadwaita's palette; the colour of a profile is chosen by the core from its identifier.
    readonly property var colours: ["#1c71d8", "#26a269", "#e66100", "#c01c28", "#813d9c", "#865e3c", "#1a7f8e", "#5e5c64"]

    parent: Overlay.overlay
    x: parent.width - width - 8
    y: 54
    width: 360
    padding: 16
    modal: false
    focus: true
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside
    onAboutToShow: { naming = false; reload() }
    onClosed: closedAt = Date.now()

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

    function reload() { tiles = JSON.parse(profiles.tiles_json()) }

    function create(name) {
        const result = JSON.parse(profiles.create(name))
        if (result.ok) { naming = false; reload() }
    }

    contentItem: ColumnLayout {
        spacing: 12

        Label {
            Layout.fillWidth: true
            text: popup.bridge.message("Profiles")
            font.bold: true
            font.pixelSize: 16
        }
        Label {
            Layout.fillWidth: true
            wrapMode: Text.Wrap
            text: popup.bridge.message("Each profile keeps its own cookies, logins, history and cache, and runs as a separate process.")
            color: palette.placeholderText
            font.pointSize: Qt.application.font.pointSize * 0.85
        }

        GridLayout {
            Layout.fillWidth: true
            columns: 3
            columnSpacing: 8
            rowSpacing: 8

            Repeater {
                model: popup.tiles
                delegate: Item {
                    id: tile
                    required property var modelData
                    readonly property bool removable: !modelData.isDefault && !modelData.isCurrent
                    Layout.fillWidth: true
                    Layout.preferredHeight: 104

                    HoverHandler { id: tileHover }
                    TapHandler {
                        onTapped: {
                            if (!tile.modelData.isCurrent) popup.profiles.open(tile.modelData.id)
                            popup.close()
                        }
                    }
                    Rectangle {
                        anchors.fill: parent
                        radius: 10
                        color: tileHover.hovered ? Qt.alpha(palette.windowText, 0.07) : "transparent"
                    }
                    ColumnLayout {
                        anchors.fill: parent
                        anchors.margins: 6
                        spacing: 6
                        Rectangle {
                            Layout.alignment: Qt.AlignHCenter
                            Layout.preferredWidth: 56
                            Layout.preferredHeight: 56
                            radius: 28
                            color: popup.colours[tile.modelData.colourIndex % popup.colours.length]
                            border.width: tile.modelData.isCurrent ? 3 : 0
                            border.color: palette.highlight
                            Label {
                                anchors.centerIn: parent
                                text: tile.modelData.initial
                                textFormat: Text.PlainText
                                color: "#ffffff"
                                font.bold: true
                                font.pixelSize: 24
                            }
                        }
                        Label {
                            Layout.fillWidth: true
                            horizontalAlignment: Text.AlignHCenter
                            text: tile.modelData.name
                            textFormat: Text.PlainText
                            elide: Text.ElideRight
                            font.weight: tile.modelData.isCurrent ? Font.DemiBold : Font.Normal
                        }
                    }
                    IconButton {
                        visible: tile.removable && tileHover.hovered
                        anchors.top: parent.top
                        anchors.right: parent.right
                        anchors.margins: 2
                        iconName: "close"
                        implicitWidth: 24
                        implicitHeight: 24
                        icon.width: 12
                        icon.height: 12
                        tip: popup.bridge.message("Delete")
                        onClicked: popup.deleteRequested(tile.modelData.id, tile.modelData.name)
                    }
                }
            }

            // The tile that adds a profile: a plus in a dashed circle, or the name field once it is pressed.
            Item {
                Layout.fillWidth: true
                Layout.preferredHeight: 104

                HoverHandler { id: addHover }
                TapHandler {
                    enabled: !popup.naming
                    onTapped: { popup.naming = true; nameField.forceActiveFocus() }
                }
                Rectangle {
                    anchors.fill: parent
                    radius: 10
                    color: addHover.hovered && !popup.naming ? Qt.alpha(palette.windowText, 0.07) : "transparent"
                }
                ColumnLayout {
                    anchors.fill: parent
                    anchors.margins: 6
                    spacing: 6
                    Rectangle {
                        Layout.alignment: Qt.AlignHCenter
                        Layout.preferredWidth: 56
                        Layout.preferredHeight: 56
                        radius: 28
                        color: Qt.alpha(palette.windowText, 0.08)
                        TintedIcon { anchors.centerIn: parent; name: "new-tab"; size: 24 }
                    }
                    Label {
                        Layout.fillWidth: true
                        horizontalAlignment: Text.AlignHCenter
                        text: popup.bridge.message("Add Profile")
                        elide: Text.ElideRight
                    }
                }
            }
        }

        TextField {
            id: nameField
            Layout.fillWidth: true
            visible: popup.naming
            placeholderText: popup.bridge.message("New profile name")
            maximumLength: 40
            onAccepted: popup.create(text)
            Keys.onEscapePressed: function(event) { popup.naming = false; text = ""; event.accepted = true }
            onVisibleChanged: if (!visible) text = ""
        }
    }
}
