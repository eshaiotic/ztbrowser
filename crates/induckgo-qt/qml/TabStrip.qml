import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

// The open tabs: favicon, title, and a close button on each; shown only when there is more than one. They run side
// by side along the top or bottom of the page, or one above the other along its left or right edge.
Rectangle {
    id: strip
    property var tabs: []
    property int currentIndex: 0
    property string closeTip
    /// The side of the page the tabs sit on: `top`, `bottom`, `left` or `right`.
    property string placement: "top"
    readonly property bool vertical: placement === "left" || placement === "right"

    /// The words of the menu entry that moves a tab to a window of its own.
    property string moveTip

    signal selected(int index)
    signal closeRequested(int index)
    /// The tab at `index` is to become a window of its own: asked for by the tab's menu, or by dragging the tab
    /// away from the strip.
    signal moveRequested(int index)

    /// How far (in pixels) a dragged tab must leave the strip, across its length, to become a window.
    readonly property int tearOffDistance: 40

    color: palette.window
    implicitHeight: vertical ? 0 : 38
    implicitWidth: vertical ? 220 : 0

    // The line that parts the tabs from the page runs along the edge facing it.
    Rectangle {
        readonly property bool alongWidth: !strip.vertical
        anchors.top: strip.placement === "bottom" ? parent.top : undefined
        anchors.bottom: strip.placement === "top" ? parent.bottom : undefined
        anchors.left: strip.placement === "right" ? parent.left : undefined
        anchors.right: strip.placement === "left" ? parent.right : undefined
        width: alongWidth ? parent.width : 1
        height: alongWidth ? 1 : parent.height
        color: Qt.alpha(palette.windowText, 0.15)
    }

    GridLayout {
        anchors.fill: parent
        anchors.margins: strip.vertical ? 6 : 0
        anchors.leftMargin: strip.vertical ? 6 : 6
        anchors.rightMargin: 6
        anchors.topMargin: strip.vertical ? 6 : (strip.placement === "top" ? 4 : 0)
        anchors.bottomMargin: strip.vertical ? 6 : (strip.placement === "bottom" ? 4 : 0)
        rowSpacing: 2
        columnSpacing: 2
        flow: strip.vertical ? GridLayout.TopToBottom : GridLayout.LeftToRight
        rows: strip.vertical ? strip.tabs.length + 1 : 1
        columns: strip.vertical ? 1 : strip.tabs.length + 1

        Repeater {
            model: strip.tabs.length
            delegate: Rectangle {
                id: tabItem
                required property int index
                readonly property var tab: strip.tabs[index]
                readonly property bool current: index === strip.currentIndex

                Layout.fillWidth: true
                Layout.maximumWidth: strip.vertical ? -1 : 240
                Layout.minimumWidth: 72
                Layout.fillHeight: !strip.vertical
                Layout.preferredHeight: strip.vertical ? 34 : -1
                Layout.alignment: Qt.AlignTop
                radius: 8
                color: current ? palette.base : hover.hovered ? Qt.alpha(palette.windowText, 0.07) : "transparent"
                border.color: current ? Qt.alpha(palette.windowText, 0.18) : "transparent"

                HoverHandler { id: hover }
                TapHandler {
                    acceptedButtons: Qt.LeftButton
                    onTapped: strip.selected(tabItem.index)
                }
                TapHandler {
                    acceptedButtons: Qt.MiddleButton
                    onTapped: strip.closeRequested(tabItem.index)
                }
                TapHandler {
                    acceptedButtons: Qt.RightButton
                    onTapped: tabMenu.popup()
                }
                Menu {
                    id: tabMenu
                    MenuItem {
                        text: strip.moveTip
                        enabled: strip.tabs.length > 1
                        onTriggered: strip.moveRequested(tabItem.index)
                    }
                }
                // A tab dragged off the strip, along its width or height, leaves for a window of its own.
                DragHandler {
                    id: tearOff
                    target: null
                    acceptedButtons: Qt.LeftButton
                    onActiveChanged: {
                        if (active || strip.tabs.length < 2) return
                        const at = strip.mapFromItem(null, centroid.scenePosition.x, centroid.scenePosition.y)
                        const away = strip.vertical ? Math.max(-at.x, at.x - strip.width)
                                                    : Math.max(-at.y, at.y - strip.height)
                        if (away > strip.tearOffDistance) strip.moveRequested(tabItem.index)
                    }
                }

                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: 10
                    anchors.rightMargin: 4
                    spacing: 8

                    Image {
                        Layout.preferredWidth: 16
                        Layout.preferredHeight: 16
                        sourceSize: Qt.size(16, 16)
                        source: tabItem.tab ? tabItem.tab.icon : ""
                        visible: status === Image.Ready
                    }
                    // The title comes from the page: show it as plain text, never as markup.
                    Text {
                        Layout.fillWidth: true
                        text: tabItem.tab ? (tabItem.tab.title || tabItem.tab.address) : ""
                        textFormat: Text.PlainText
                        elide: Text.ElideRight
                        color: palette.windowText
                        font.weight: tabItem.current ? Font.DemiBold : Font.Normal
                    }
                    IconButton {
                        iconName: "close"
                        tip: strip.closeTip
                        implicitWidth: 24
                        implicitHeight: 24
                        icon.width: 14
                        icon.height: 14
                        opacity: tabItem.current || hover.hovered ? 1 : 0
                        onClicked: strip.closeRequested(tabItem.index)
                    }
                }
            }
        }
        Item {
            Layout.fillWidth: !strip.vertical
            Layout.fillHeight: strip.vertical
            Layout.minimumWidth: 0
            visible: strip.vertical || strip.tabs.length < 4
        }
    }
}
