import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

// One row of a preference group: a title with an explanation on the left, the control it sets on the right.
Item {
    id: row
    property string title
    property string subtitle
    default property alias control: controlSlot.data
    /// The whole row can be clicked, as libadwaita's activatable rows can; the control at its end stays as it is.
    property bool activatable: false
    signal activated()
    readonly property bool isFirst: !parent || parent.children[0] === row

    Layout.fillWidth: true
    implicitHeight: Math.max(50, texts.implicitHeight + 24)

    Rectangle {
        visible: row.activatable && (rowArea.containsMouse || rowArea.pressed)
        anchors.fill: parent
        anchors.topMargin: row.isFirst ? 0 : 1
        color: Qt.alpha(palette.windowText, rowArea.pressed ? 0.1 : 0.05)
    }
    MouseArea {
        id: rowArea
        anchors.fill: parent
        enabled: row.activatable
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: row.activated()
    }
    Rectangle {
        visible: !row.isFirst
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        height: 1
        color: Qt.alpha(palette.windowText, 0.1)
    }
    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: 14
        anchors.rightMargin: 12
        spacing: 12

        ColumnLayout {
            id: texts
            Layout.fillWidth: true
            Layout.alignment: Qt.AlignVCenter
            spacing: 2
            Label {
                Layout.fillWidth: true
                text: row.title
                wrapMode: Text.Wrap
            }
            Label {
                Layout.fillWidth: true
                visible: text !== ""
                text: row.subtitle
                wrapMode: Text.Wrap
                elide: Text.ElideMiddle
                maximumLineCount: 3
                font.pointSize: Qt.application.font.pointSize * 0.82
                color: palette.placeholderText
            }
        }
        RowLayout {
            id: controlSlot
            Layout.alignment: Qt.AlignVCenter
        }
    }
}
