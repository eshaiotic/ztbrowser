import QtQuick
import QtWebEngine
import QtWebChannel
import io.induckgo.browser

// One page: a Chromium view with the scripts every page gets.
WebEngineView {
    id: view
    /// The window that holds this tab; it changes when the tab moves to another window, and the tab's requests
    /// (open a link in a new tab, answer the apps page) go to the window that holds it now.
    property var owner: null
    property string startAddress
    property string smoothWheelSource
    property string extensionPromptSource
    // Qt's web-channel client followed by Guard's page script; empty when the browser has none to give.
    property string guardSource
    property string resourceHintSource
    // Rejects cookie notices; it waits in the page's own world until Guard says the shield is on for the site.
    property string cookieNoticeSource
    // A web app keeps to its own site: a link to another site opens in the browser instead.
    property bool keepsToSite: false
    property var appBridge: null
    // The apps page is loaded from text, so the address bar shows its own address instead of the blank one.
    property string appsAddress
    // showApps asked for a load that has not been announced yet.
    property bool loadingApps: false
    readonly property string address: showsApps ? appsAddress : url.toString()
    readonly property bool isLoading: loading

    /// The apps page is on screen: only then are its own links (open or uninstall an app) believed.
    property bool showsApps: false

    /// A link wants a new tab (target="_blank", middle click).
    signal openInNewTab(string address)
    /// A link of the apps page was clicked.
    signal appsLinkClicked(string address)

    function showApps(html, address) {
        appsAddress = address
        showsApps = true
        loadingApps = true
        loadHtml(html, address)
    }

    settings.javascriptCanOpenWindows: false
    // A page loaded from file:// must not read other local files, and a local page must not reach the network.
    settings.localContentCanAccessFileUrls: false
    settings.localContentCanAccessRemoteUrls: false

    /// The page scripts the sources describe. Binding them to `userScripts.collection` while the view is being
    /// created crashes Qt 6.8 (Debian 13), so they are installed once the view exists, and the first page loads
    /// only after them: it must never start without its scripts.
    readonly property var pageScripts: {
        const scripts = []
        if (smoothWheelSource !== "") scripts.push(userScript("induckgo-smooth-wheel", smoothWheelSource))
        if (extensionPromptSource !== "") scripts.push(userScript("induckgo-duckduckgo-extension-prompt", extensionPromptSource))
        if (guardSource !== "") scripts.push(userScript("induckgo-guard-page-hiding", guardSource))
        // It changes what the page's own scripts call, so it runs in the page's world (the others in a world of their own).
        if (cookieNoticeSource !== "") scripts.push(userScript("induckgo-cookie-notices", cookieNoticeSource, WebEngineScript.MainWorld))
        if (resourceHintSource !== "") scripts.push(userScript("induckgo-no-resource-hints", resourceHintSource, WebEngineScript.MainWorld))
        return scripts
    }
    property bool scriptsInstalled: false
    /// Handed the array of `pageScripts` itself, Qt applies only the last script (measured with Qt 6.11: the
    /// resource-hint script ran, the smooth-wheel, extension-prompt and Guard scripts did not), so the page showed
    /// DuckDuckGo's offer of an extension and Guard hid nothing. A copy of every script is installed instead.
    function installScripts() {
        userScripts.collection = pageScripts.map(script => Object.assign({}, script))
    }
    onPageScriptsChanged: if (scriptsInstalled) installScripts()
    Component.onCompleted: {
        installScripts()
        scriptsInstalled = true
        url = startAddress
    }

    // The page script reaches Rust through a channel of its own isolated world, which the page cannot see.
    webChannel: WebChannel { registeredObjects: [guardBridge] }
    webChannelWorld: WebEngineScript.ApplicationWorld
    GuardBridge { id: guardBridge; WebChannel.id: "guard" }

    function userScript(scriptName, source, world) {
        return {
            name: scriptName,
            sourceCode: source,
            injectionPoint: WebEngineScript.DocumentCreation,
            worldId: world === undefined ? WebEngineScript.ApplicationWorld : world,
            runsOnSubFrames: true
        }
    }

    onNewWindowRequested: function(request) {
        const target = request.requestedUrl.toString()
        if (view.keepsToSite) {
            // A web app has one view: its own site's pages open in it, the rest in the browser.
            if (!request.userInitiated) return
            if (appBridge.leaves_app(target, true)) appBridge.open_in_browser(target)
            else view.url = request.requestedUrl
        } else if (request.userInitiated) {
            view.openInNewTab(target)
        }
    }

    onNavigationRequested: function(request) {
        const target = request.url.toString()
        // The apps page's links are not navigations: they are answered here, and only for a click inside the apps
        // page's own document.
        if (target.startsWith("induckgo-app-")) {
            request.reject()
            const clicked = request.navigationType === WebEngineNavigationRequest.LinkClickedNavigation
            if (view.showsApps && request.isMainFrame && clicked) view.appsLinkClicked(target)
            return
        }
        // Any other navigation of the tab, whatever its scheme, replaces the apps page: its own load (asked for by
        // showApps) and a reload of it are the only ones that keep it.
        if (request.isMainFrame) {
            const reloaded = request.navigationType === WebEngineNavigationRequest.ReloadNavigation
            if (view.loadingApps) view.loadingApps = false
            else if (!reloaded) view.showsApps = false
        }
        if (!view.keepsToSite || !request.isMainFrame) return
        const clicked = request.navigationType === WebEngineNavigationRequest.LinkClickedNavigation
        if (appBridge.leaves_app(target, clicked)) {
            request.reject()
            if (/^https?:/.test(target)) appBridge.open_in_browser(target)
        }
    }

    function load(destination) { url = destination }
}
