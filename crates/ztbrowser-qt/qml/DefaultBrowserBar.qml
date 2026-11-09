import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

// Offers to make ZTBrowser the default browser while another one opens the links of other programs, as libadwaita's
// banner offers an action: at most one window a process shows it, and never again once the person says so. The
// desktop is asked a moment after the window opened, on a thread of its own, so the question never delays the
// first frame.
Rectangle {
    id: bar
    required property var bridge
    /// Only browser windows ask; a web app's window has a single site to show.
    property bool asks: true

    /// Something worth telling the person happened (a toast).
    signal notice(string text)

    visible: false
    implicitHeight: content.implicitHeight + 12
    color: Qt.alpha(palette.highlight, 0.16)

    // The desktop answers within a tenth of a second; the window asks a second after it opened.
    Timer {
        interval: 1000
        running: bar.asks
        onTriggered: if (bar.bridge.asks_about_default_browser()) { bar.bridge.check_default_browser(); answer.start() }
    }
    Timer {
        id: answer
        // Set while the person's own "Make Default" waits for the desktop: its failure is worth a toast.
        property bool reportsFailure: false
        interval: 100
        repeat: true
        onTriggered: {
            const state = JSON.parse(bar.bridge.default_browser_json())
            if (state.state === "checking") return
            stop()
            if (reportsFailure) {
                if (state.error !== "")
                    bar.notice(bar.bridge.message("Could not make ZTBrowser the default browser: {error}").replace("{error}", state.error))
            } else {
                bar.visible = bar.bridge.offers_default_browser()
            }
        }
    }

    RowLayout {
        id: content
        anchors.fill: parent
        anchors.leftMargin: 16
        anchors.rightMargin: 8
        spacing: 8

        Label {
            Layout.fillWidth: true
            text: bar.bridge.message("ZTBrowser is not your default browser")
            textFormat: Text.PlainText
            elide: Text.ElideRight
        }
        PillButton {
            objectName: "makeDefault"
            label: bar.bridge.message("Make Default")
            suggested: true
            onClicked: {
                bar.visible = false
                bar.bridge.make_default_browser()
                answer.reportsFailure = true
                answer.start()
            }
        }
        PillButton {
            objectName: "dontAskAgain"
            label: bar.bridge.message("Don't Ask Again")
            onClicked: { bar.bridge.set_asks_about_default_browser(false); bar.visible = false }
        }
        IconButton {
            objectName: "close"
            iconName: "close"
            tip: bar.bridge.message("Close")
            onClicked: bar.visible = false
        }
    }
}
