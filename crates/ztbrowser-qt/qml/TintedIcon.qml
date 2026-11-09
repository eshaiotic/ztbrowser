import QtQuick
import QtQuick.Effects

// One of the application's icons drawn in a single colour (the theme's text colour unless told otherwise).
Item {
    id: icon
    property string name
    property color color: palette.windowText
    property int size: 16

    implicitWidth: size
    implicitHeight: size

    Image {
        id: picture
        anchors.fill: parent
        sourceSize: Qt.size(icon.size * 2, icon.size * 2)
        source: icon.name === "" ? "" : "qrc:/icons/" + icon.name + ".svg"
        visible: false
    }
    MultiEffect {
        anchors.fill: picture
        source: picture
        brightness: 1.0
        colorization: 1.0
        colorizationColor: icon.color
    }
}
