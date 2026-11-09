import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

// The main menu, drawn as GNOME draws a popover menu: icon, title and shortcut per row, a zoom bar, sections.
Popup {
    id: menu
    required property var window
    required property var bridge
    required property Action newTab
    required property Action newWindow
    required property Action showDownloads
    required property Action findInPage
    required property Action preferences
    required property Action about
    required property Action installApp
    required property Action installedApps
    required property Action protectSite
    required property Action openInBrowser
    required property Action clearAppData
    /// The window is an installed web app's: it offers the app's own menu, not the browser's.
    property bool webApp: false
    /// The page on screen can be installed as an app.
    property bool installable: false
    /// Whether protections are on for the page's site, read when the menu opens.
    property bool siteProtected: true
    onAboutToShow: siteProtected = window.currentHost === "" || bridge.is_site_protected(window.currentHost)

    padding: 8
    width: 343
    /// When the menu last closed: its button ignores the click that closed it by pressing outside.
    property double closedAt: 0
    onClosed: closedAt = Date.now()
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside
    // Below the header bar, against the window's right edge, wherever the button that opened it sits.
    parent: Overlay.overlay
    x: window.width - width - 8
    y: 54

    background: Rectangle {
        radius: 12
        color: palette.base
        border.width: 1
        border.color: Qt.alpha(palette.windowText, 0.16)
    }

    // One row of the menu: it runs `action` and closes the menu.
    component Entry: AbstractButton {
        id: entry
        property Action command
        property string iconName: command ? command.icon.source.toString().replace("qrc:/icons/", "").replace(".svg", "") : ""
        /// A ticked row (a setting that is on) shows a check mark where a shortcut would be.
        property bool marked: false
        property string shortcutText: command && command.shortcut ? String(command.shortcut) : ""
        text: command ? command.text : ""
        Layout.fillWidth: true
        implicitHeight: 34
        onClicked: { menu.close(); if (command) command.trigger() }
        background: Rectangle {
            radius: 8
            color: entry.down ? Qt.alpha(palette.windowText, 0.14) : entry.hovered ? Qt.alpha(palette.windowText, 0.08) : "transparent"
        }
        contentItem: RowLayout {
            spacing: 12
            TintedIcon { Layout.leftMargin: 8; name: entry.iconName; size: 16; visible: entry.iconName !== "" }
            Label { Layout.fillWidth: true; text: entry.text; elide: Text.ElideRight }
            TintedIcon { Layout.rightMargin: 8; visible: entry.marked; name: "check"; size: 16 }
            Label {
                Layout.rightMargin: 8
                visible: text !== ""
                text: entry.shortcutText
                font.bold: true
                color: Qt.alpha(palette.windowText, 0.5)
            }
        }
    }

    component Separator: Rectangle {
        Layout.fillWidth: true
        Layout.topMargin: 4
        Layout.bottomMargin: 4
        Layout.preferredHeight: 1
        color: Qt.alpha(palette.windowText, 0.12)
    }

    contentItem: ColumnLayout {
        spacing: 0

        Entry { visible: !menu.webApp; command: menu.newTab }
        Entry { visible: !menu.webApp; command: menu.newWindow }
        Separator { visible: !menu.webApp }
        Entry { visible: !menu.webApp; command: menu.showDownloads }
        Entry { visible: !menu.webApp; command: menu.findInPage }

        // Zoom: smaller, actual size, larger, in one rounded bar.
        Rectangle {
            Layout.fillWidth: true
            Layout.topMargin: 6
            Layout.bottomMargin: 6
            Layout.preferredHeight: 34
            radius: 9
            color: Qt.alpha(palette.windowText, 0.08)
            RowLayout {
                anchors.fill: parent
                spacing: 0
                AbstractButton {
                    Layout.preferredWidth: 80
                    Layout.fillHeight: true
                    onClicked: menu.window.changeZoom(-1)
                    background: Rectangle { radius: 9; color: parent.hovered ? Qt.alpha(palette.windowText, 0.08) : "transparent" }
                    contentItem: Item { TintedIcon { anchors.centerIn: parent; name: "zoom-out"; size: 16 } }
                }
                AbstractButton {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    onClicked: menu.window.changeZoom(0)
                    background: Rectangle { radius: 9; color: parent.hovered ? Qt.alpha(palette.windowText, 0.08) : "transparent" }
                    contentItem: Item {
                        Row {
                            anchors.centerIn: parent
                            spacing: 10
                            Label { text: menu.window.currentZoom + "%"; font.bold: true }
                            Label { text: "Ctrl+0"; font.bold: true; color: Qt.alpha(palette.windowText, 0.5) }
                        }
                    }
                }
                AbstractButton {
                    Layout.preferredWidth: 80
                    Layout.fillHeight: true
                    onClicked: menu.window.changeZoom(1)
                    background: Rectangle { radius: 9; color: parent.hovered ? Qt.alpha(palette.windowText, 0.08) : "transparent" }
                    contentItem: Item { TintedIcon { anchors.centerIn: parent; name: "zoom-in"; size: 16 } }
                }
            }
        }

        Separator { }
        Entry { visible: menu.webApp; command: menu.protectSite; marked: menu.siteProtected }
        Entry { visible: menu.webApp; command: menu.openInBrowser }
        Entry { visible: menu.webApp; command: menu.clearAppData }
        Entry { visible: !menu.webApp && menu.installable; command: menu.installApp }
        Entry { visible: !menu.webApp; command: menu.installedApps }
        Separator { }
        Entry { visible: !menu.webApp; command: menu.preferences }
        Entry { command: menu.about }
    }
}
