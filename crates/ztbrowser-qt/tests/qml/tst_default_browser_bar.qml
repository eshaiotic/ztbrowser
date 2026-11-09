import QtQuick
import QtQuick.Controls
import QtTest

// The bar that offers to make ZTBrowser the default browser, driven by a stand-in for the Rust bridge: the desktop's
// answer, the person's wish to be asked and every call the bar makes are the stand-in's own state.
Item {
    width: 900
    height: 60

    QtObject {
        id: fakeBridge
        property bool asks: true
        property string answer: '{"state": "other", "browser": "Brave", "error": ""}'
        property bool offers: true
        property var calls: []

        function message(text) { return text }
        function asks_about_default_browser() { return asks }
        function set_asks_about_default_browser(value) { asks = value; calls = calls.concat(["asks " + value]) }
        function check_default_browser() { calls = calls.concat(["check"]) }
        function default_browser_json() { return answer }
        function offers_default_browser() { return offers }
        function make_default_browser() { calls = calls.concat(["make default"]) }
    }

    Component {
        id: barComponent
        DefaultBrowserBar { width: 900; bridge: fakeBridge }
    }

    SignalSpy { id: notices; signalName: "notice" }

    TestCase {
        name: "DefaultBrowserBar"
        when: windowShown

        function init() {
            fakeBridge.asks = true
            fakeBridge.answer = '{"state": "other", "browser": "Brave", "error": ""}'
            fakeBridge.offers = true
            fakeBridge.calls = []
            notices.clear()
        }

        function shownBar() {
            const bar = createTemporaryObject(barComponent, parent)
            notices.target = bar
            tryCompare(bar, "visible", true, 3000)
            return bar
        }

        function test_offers_the_change_once_the_desktop_names_another_browser() {
            shownBar()
            compare(fakeBridge.calls, ["check"])
        }

        function test_never_asks_the_desktop_when_the_person_does_not_want_to_be_asked() {
            fakeBridge.asks = false
            const bar = createTemporaryObject(barComponent, parent)
            wait(1500)
            compare([bar.visible, fakeBridge.calls.length], [false, 0])
        }

        function test_stays_hidden_when_another_window_already_offered_the_change() {
            fakeBridge.offers = false
            const bar = createTemporaryObject(barComponent, parent)
            tryCompare(fakeBridge, "calls", ["check"], 3000)
            wait(300)
            compare(bar.visible, false)
        }

        function test_stays_hidden_in_a_web_app_window() {
            const bar = createTemporaryObject(barComponent, parent, { asks: false })
            wait(1500)
            compare([bar.visible, fakeBridge.calls.length], [false, 0])
        }

        function test_remembers_not_to_ask_again() {
            const bar = shownBar()
            mouseClick(findChild(bar, "dontAskAgain"))
            compare([bar.visible, fakeBridge.calls], [false, ["check", "asks false"]])
        }

        function test_makes_ztbrowser_the_default_when_asked_to() {
            const bar = shownBar()
            mouseClick(findChild(bar, "makeDefault"))
            compare([bar.visible, fakeBridge.calls], [false, ["check", "make default"]])
        }

        function test_tells_why_the_desktop_refused_the_change() {
            const bar = shownBar()
            fakeBridge.answer = '{"state": "other", "browser": "Brave", "error": "no configuration"}'
            mouseClick(findChild(bar, "makeDefault"))
            notices.wait(2000)
            compare(notices.signalArguments[0][0], "Could not make ZTBrowser the default browser: no configuration")
        }

        function test_tells_nothing_when_the_change_succeeds() {
            const bar = shownBar()
            fakeBridge.answer = '{"state": "this", "browser": "", "error": ""}'
            mouseClick(findChild(bar, "makeDefault"))
            wait(500)
            compare(notices.count, 0)
        }

        function test_closes_for_now_without_changing_anything() {
            const bar = shownBar()
            mouseClick(findChild(bar, "close"))
            compare([bar.visible, fakeBridge.calls], [false, ["check"]])
        }
    }
}
