import QtQuick
import QtQuick.Controls

// One choice out of a few, shown as joined buttons the way libadwaita's linked toggle buttons are.
// `options` is a list of `{ id, label }`; `current` is the id that is chosen.
Row {
    id: choice
    property var options: []
    property string current

    /// The person picked the option stored as `id`.
    signal chosen(string id)

    spacing: 0

    Repeater {
        model: choice.options
        delegate: AbstractButton {
            id: option
            required property var modelData
            required property int index
            readonly property bool isChosen: modelData.id === choice.current

            text: modelData.label
            checkable: false
            padding: 12
            implicitHeight: 34
            implicitWidth: label.implicitWidth + 24
            onClicked: choice.chosen(modelData.id)

            background: Rectangle {
                readonly property bool first: option.index === 0
                readonly property bool last: option.index === choice.options.length - 1
                topLeftRadius: first ? 9 : 0
                bottomLeftRadius: first ? 9 : 0
                topRightRadius: last ? 9 : 0
                bottomRightRadius: last ? 9 : 0
                color: option.isChosen ? palette.highlight
                     : option.down ? Qt.alpha(palette.windowText, 0.19)
                     : option.hovered ? Qt.alpha(palette.windowText, 0.14) : Qt.alpha(palette.windowText, 0.08)
                border.width: option.activeFocus ? 2 : 0
                border.color: palette.highlight
            }
            contentItem: Label {
                id: label
                text: option.text
                horizontalAlignment: Text.AlignHCenter
                verticalAlignment: Text.AlignVCenter
                color: option.isChosen ? palette.highlightedText : palette.buttonText
            }
        }
    }
}
