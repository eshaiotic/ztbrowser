import QtQuick
import QtQuick.Controls

// Minimise, maximise or restore, and close: the window buttons of a window that draws its own title bar. The
// default style is libadwaita's (round, 24 px, a little apart, grey, icons inside); the `macos` style is three
// coloured dots with the glyph showing while the pointer is over them. They carry no tooltips, like GNOME's own.
Row {
    id: controls
    required property var window
    /// `default` or `macos`.
    property string style: "default"
    readonly property bool isMaximized: window.visibility === Window.Maximized
    readonly property bool isMacOs: style === "macos"

    spacing: isMacOs ? 8 : 13

    component Control: AbstractButton {
        id: button
        property string iconName
        implicitWidth: 24
        implicitHeight: 24
        background: Rectangle {
            radius: 12
            color: button.down ? Qt.alpha(palette.windowText, 0.19)
                 : button.hovered ? Qt.alpha(palette.windowText, 0.14) : Qt.alpha(palette.windowText, 0.08)
        }
        contentItem: Item {
            TintedIcon { anchors.centerIn: parent; size: 16; name: button.iconName }
        }
    }

    // macOS draws red to close, yellow to minimise and green to maximise, close first; the glyph of each shows
    // on all three as soon as the pointer is over any of them.
    component Dot: AbstractButton {
        id: dot
        property string iconName
        property color dotColor
        property bool glyphShown: false
        implicitWidth: 14
        implicitHeight: 14
        background: Rectangle {
            radius: 7
            color: dot.down ? Qt.darker(dot.dotColor, 1.2) : dot.dotColor
            border.width: 1
            border.color: Qt.darker(dot.dotColor, 1.25)
        }
        contentItem: Item {
            TintedIcon {
                anchors.centerIn: parent
                size: 10
                color: "#000000"
                name: dot.iconName
                opacity: dot.glyphShown ? 0.7 : 0
            }
        }
    }

    HoverHandler { id: groupHover }

    Control { visible: !controls.isMacOs; iconName: "minimize"; onClicked: controls.window.showMinimized() }
    Control {
        visible: !controls.isMacOs
        iconName: controls.isMaximized ? "restore" : "maximize"
        onClicked: controls.isMaximized ? controls.window.showNormal() : controls.window.showMaximized()
    }
    Control { visible: !controls.isMacOs; iconName: "close"; onClicked: controls.window.close() }

    Dot {
        visible: controls.isMacOs
        iconName: "close"
        dotColor: "#ff5f57"
        glyphShown: groupHover.hovered
        onClicked: controls.window.close()
    }
    Dot {
        visible: controls.isMacOs
        iconName: "minimize"
        dotColor: "#febc2e"
        glyphShown: groupHover.hovered
        onClicked: controls.window.showMinimized()
    }
    Dot {
        visible: controls.isMacOs
        iconName: controls.isMaximized ? "restore" : "maximize"
        dotColor: "#28c840"
        glyphShown: groupHover.hovered
        onClicked: controls.isMaximized ? controls.window.showNormal() : controls.window.showMaximized()
    }
}
