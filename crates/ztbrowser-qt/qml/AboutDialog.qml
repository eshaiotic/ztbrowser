import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

// "About ZTBrowser", as the GTK edition's AdwAboutDialog draws it: the application's icon, name and version over
// two rows, "Details" and "Legal", each opening a page of its own with a back arrow and a centred title.
Dialog {
    id: dialog
    required property var bridge

    // 0 is the overview, 1 the details page, 2 the legal page.
    property int page: 0
    property var about: ({ name: "", version: "", detailsTitle: "", legalTitle: "", comments: "", sections: [] })
    readonly property bool isDark: Qt.styleHints.colorScheme === Qt.Dark
    readonly property color accent: isDark ? "#78aeed" : "#1c71d8"

    modal: true
    padding: 0
    anchors.centerIn: parent
    width: Math.min(360, parent ? parent.width - 48 : 360)
    height: Math.min(432, parent ? parent.height - 48 : 432)
    onAboutToShow: { about = JSON.parse(bridge.about_json()); page = 0 }

    background: Rectangle {
        radius: 16
        color: palette.window
        border.width: 1
        border.color: Qt.alpha(palette.windowText, 0.2)
    }

    // A row of the overview: a rounded card with its title and a chevron.
    component PageRow: AbstractButton {
        id: row
        Layout.fillWidth: true
        implicitHeight: 54
        background: Rectangle {
            radius: 12
            color: row.down ? Qt.alpha(palette.windowText, 0.1) : row.hovered ? Qt.tint(palette.base, Qt.alpha(palette.windowText, 0.04)) : palette.base
            border.width: 1
            border.color: Qt.alpha(palette.windowText, 0.1)
        }
        contentItem: RowLayout {
            Label { Layout.fillWidth: true; Layout.leftMargin: 14; text: row.text; elide: Text.ElideRight }
            TintedIcon { Layout.rightMargin: 14; name: "chevron-right"; size: 16 }
        }
    }

    // The round buttons of the header: a back arrow and a close circle.
    component HeaderCircle: AbstractButton {
        id: circle
        property string iconName
        implicitWidth: 24
        implicitHeight: 24
        background: Rectangle {
            radius: 12
            color: circle.iconName === "close" ? Qt.alpha(palette.windowText, circle.hovered ? 0.14 : 0.08)
                 : circle.hovered ? Qt.alpha(palette.windowText, 0.08) : "transparent"
        }
        contentItem: Item { TintedIcon { anchors.centerIn: parent; name: circle.iconName; size: 16 } }
    }

    // A paragraph of styled text whose links open in the person's own browser, https only.
    component Paragraph: Text {
        Layout.fillWidth: true
        wrapMode: Text.Wrap
        textFormat: Text.RichText
        color: palette.windowText
        // Qt paints links in its own blue unless the text itself says otherwise.
        property string markup
        text: "<style>a { color: " + dialog.accent + " }</style>" + markup
        onLinkActivated: function(link) { if (link.startsWith("https://")) Qt.openUrlExternally(link) }
        HoverHandler { cursorShape: parent.hoveredLink !== "" ? Qt.PointingHandCursor : Qt.ArrowCursor }
    }

    contentItem: Item {
        // The header sits over every page.
        Item {
            id: header
            z: 1
            width: parent.width
            height: 46
            HeaderCircle {
                visible: dialog.page !== 0
                anchors.left: parent.left
                anchors.leftMargin: 12
                anchors.verticalCenter: parent.verticalCenter
                iconName: "back"
                onClicked: dialog.page = 0
            }
            Label {
                visible: dialog.page !== 0
                anchors.centerIn: parent
                text: dialog.page === 1 ? dialog.about.detailsTitle : dialog.about.legalTitle
                font.bold: true
            }
            HeaderCircle {
                anchors.right: parent.right
                anchors.rightMargin: 12
                anchors.verticalCenter: parent.verticalCenter
                iconName: "close"
                onClicked: dialog.close()
            }
        }

        StackLayout {
            anchors.fill: parent
            anchors.topMargin: header.height
            currentIndex: dialog.page

            ColumnLayout {
                spacing: 0
                Item { Layout.fillHeight: true }
                Image {
                    Layout.alignment: Qt.AlignHCenter
                    source: "qrc:/icons/app.svg"
                    sourceSize: Qt.size(256, 256)
                    Layout.preferredWidth: 128
                    Layout.preferredHeight: 128
                }
                Label {
                    Layout.alignment: Qt.AlignHCenter
                    Layout.topMargin: 8
                    text: dialog.about.name
                    font.bold: true
                    font.pixelSize: 24
                }
                Rectangle {
                    Layout.alignment: Qt.AlignHCenter
                    Layout.topMargin: 12
                    implicitWidth: versionLabel.implicitWidth + 24
                    implicitHeight: 28
                    radius: 14
                    color: Qt.alpha(dialog.accent, 0.15)
                    Label {
                        id: versionLabel
                        anchors.centerIn: parent
                        text: dialog.about.version
                        font.bold: true
                        color: dialog.accent
                    }
                }
                Item { Layout.fillHeight: true }
                PageRow {
                    Layout.leftMargin: 12
                    Layout.rightMargin: 12
                    text: dialog.about.detailsTitle
                    onClicked: dialog.page = 1
                }
                PageRow {
                    Layout.leftMargin: 12
                    Layout.rightMargin: 12
                    Layout.topMargin: 18
                    Layout.bottomMargin: 18
                    text: dialog.about.legalTitle
                    onClicked: dialog.page = 2
                }
            }

            ScrollView {
                id: detailsScroll
                contentWidth: availableWidth
                clip: true
                ColumnLayout {
                    width: detailsScroll.availableWidth
                    Paragraph {
                        Layout.margins: 12
                        markup: dialog.about.comments
                    }
                }
            }

            ScrollView {
                id: legalScroll
                contentWidth: availableWidth
                clip: true
                ColumnLayout {
                    width: legalScroll.availableWidth
                    spacing: 16
                    Repeater {
                        model: dialog.about.sections
                        delegate: ColumnLayout {
                            required property var modelData
                            required property int index
                            Layout.fillWidth: true
                            Layout.leftMargin: 12
                            Layout.rightMargin: 12
                            Layout.topMargin: index === 0 ? 12 : 0
                            spacing: 14
                            Paragraph {
                                visible: modelData.title !== null
                                markup: "<b>" + (modelData.title === null ? "" : modelData.title) + "</b>"
                            }
                            Paragraph { visible: modelData.copyright !== null; markup: modelData.copyright === null ? "" : modelData.copyright }
                            Paragraph { markup: modelData.body }
                        }
                    }
                    Item { Layout.preferredHeight: 8 }
                }
            }
        }
    }
}
