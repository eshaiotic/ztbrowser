import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtWebEngine

// Search within the page: the field, the previous/next buttons and the match count.
Rectangle {
    id: bar
    property var view: null
    property string placeholder
    property string closeTip
    property int matches: 0
    property int active: 0

    signal closed()

    visible: false
    color: palette.window
    implicitHeight: visible ? 44 : 0

    Rectangle { anchors.top: parent.top; width: parent.width; height: 1; color: Qt.alpha(palette.windowText, 0.15) }

    function open() {
        visible = true
        field.forceActiveFocus()
        field.selectAll()
    }

    function close() {
        visible = false
        if (view) view.findText("")
        bar.closed()
    }

    function find(backward) {
        if (!view) return
        view.findText(field.text, backward ? WebEngineView.FindBackward : 0)
    }

    Connections {
        target: bar.view
        function onFindTextFinished(result) {
            bar.matches = result.numberOfMatches
            bar.active = result.activeMatch
        }
    }

    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: 10
        anchors.rightMargin: 10
        spacing: 6

        TextField {
            id: field
            Layout.preferredWidth: 280
            placeholderText: bar.placeholder
            selectByMouse: true
            onTextChanged: bar.find(false)
            onAccepted: bar.find(false)
            Keys.onEscapePressed: bar.close()
        }
        IconButton { iconName: "up"; enabled: bar.matches > 0; onClicked: bar.find(true) }
        IconButton { iconName: "down"; enabled: bar.matches > 0; onClicked: bar.find(false) }
        Label {
            text: bar.matches > 0 ? bar.active + " / " + bar.matches : ""
            color: palette.placeholderText
        }
        Item { Layout.fillWidth: true }
        IconButton { iconName: "close"; tip: bar.closeTip; onClicked: bar.close() }
    }
}
