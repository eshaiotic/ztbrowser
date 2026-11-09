import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

// One download in the list, as the GTK edition shows it: the file's name over the folder it went to (or how far
// it has got, or what became of it), and a button for its folder at the row's end, centred on the two lines.
ItemDelegate {
    id: row
    required property var bridge
    required property int id
    required property string name
    required property string path
    required property string state
    required property real received
    required property real total
    required property bool missing

    property bool removable: false

    signal showInFolder(string path)
    signal removeRequested(int id)

    readonly property string folder: path.substring(0, path.lastIndexOf("/"))
    readonly property real fraction: total > 0 ? received / total : 0

    width: ListView.view ? ListView.view.width : implicitWidth
    implicitHeight: state === "progress" ? 66 : 54
    leftPadding: 10
    rightPadding: 6
    hoverEnabled: true
    onClicked: if (state === "finished" && !missing) row.showInFolder(path)

    background: Rectangle {
        radius: 8
        color: row.down ? Qt.alpha(palette.windowText, 0.14)
             : row.hovered ? Qt.alpha(palette.windowText, 0.07) : "transparent"
    }

    contentItem: RowLayout {
        spacing: 6
        ColumnLayout {
            Layout.fillWidth: true
            Layout.alignment: Qt.AlignVCenter
            spacing: 2
            // A file that was moved or deleted is greyed out: there is nothing to open.
            opacity: row.state === "finished" && row.missing ? 0.45 : 1
            Text {
                Layout.fillWidth: true
                text: row.name
                textFormat: Text.PlainText
                elide: Text.ElideMiddle
                color: row.palette.windowText
            }
            Text {
                Layout.fillWidth: true
                visible: row.state !== "progress"
                text: row.state === "failed" ? row.bridge.message("Download failed")
                    : row.missing ? row.bridge.message("File Moved or Deleted") : row.folder
                textFormat: Text.PlainText
                elide: Text.ElideMiddle
                color: Qt.alpha(row.palette.windowText, 0.6)
                font.pixelSize: 12
            }
            ProgressBar {
                id: bar
                Layout.fillWidth: true
                Layout.topMargin: 2
                visible: row.state === "progress"
                indeterminate: row.total <= 0
                value: row.fraction
                implicitHeight: 4
                background: Rectangle { radius: 2; color: Qt.alpha(row.palette.windowText, 0.12) }
                contentItem: Item {
                    Rectangle {
                        width: bar.indeterminate ? parent.width * 0.3 : parent.width * bar.visualPosition
                        height: parent.height
                        radius: 2
                        color: row.palette.highlight
                        SequentialAnimation on x {
                            running: bar.indeterminate && bar.visible
                            loops: Animation.Infinite
                            NumberAnimation { from: 0; to: bar.width * 0.7; duration: 900; easing.type: Easing.InOutQuad }
                            NumberAnimation { from: bar.width * 0.7; to: 0; duration: 900; easing.type: Easing.InOutQuad }
                        }
                    }
                }
            }
            Text {
                visible: row.state === "progress"
                text: row.total > 0
                    ? row.bridge.message("Downloading… {percent}%").replace("{percent}", Math.floor(100 * row.fraction))
                    : row.bridge.message("Downloading…")
                color: Qt.alpha(row.palette.windowText, 0.6)
                font.pixelSize: 12
            }
        }
        IconButton {
            Layout.alignment: Qt.AlignVCenter
            iconName: "folder"
            tip: row.bridge.message("Show in Folder")
            visible: row.state === "finished"
            onClicked: row.showInFolder(row.path)
        }
        IconButton {
            Layout.alignment: Qt.AlignVCenter
            iconName: "close"
            tip: row.bridge.message("Remove from List")
            visible: row.removable
            onClicked: row.removeRequested(row.id)
        }
    }
}
