import QtQuick
import QtWebEngine

// "Install Site as App" for the page on screen: reads what the page says about itself in an isolated world,
// fetches the manifest and the icons without the profile's cookies, asks the person to confirm the name, and
// has Rust write the app. The icon is tried best first; the page's favicon is the last resort, and the
// browser's own icon is what a site with no usable icon gets.
Item {
    id: installation
    required property var bridge
    required property var webApps

    /// Something to tell the person; `actionText` and `action` are an optional button of the message.
    signal notice(string text, string actionText, var action)
    /// The app was installed (`id`, `name`) from `tab`; the browser hands the site over to it.
    signal installed(string id, string name, var tab)

    property var tab: null
    property var draft: null

    readonly property int iconLimit: 2 * 1024 * 1024
    readonly property int manifestLimit: 512 * 1024

    function say(text) { return bridge.message(text) }

    /// Proposes `page` (a BrowserTab) as an app.
    function offer(page) {
        const address = page ? page.address : ""
        if (!/^https?:\/\//.test(address)) {
            notice(say("Only web pages can be installed as apps"), "", null)
            return
        }
        tab = page
        page.runJavaScript(webApps.collector_script(), WebEngineScript.UserWorld, function(metadata) {
            const manifestAddress = webApps.manifest_address(metadata || "")
            if (manifestAddress === "") { propose(address, metadata || "", ""); return }
            fetchText(manifestAddress, function(text) { propose(address, metadata || "", text) })
        })
    }

    function propose(address, metadata, manifest) {
        draft = JSON.parse(webApps.draft_json(address, metadata, manifest))
        dialog.host = draft.host
        dialog.suggestedName = draft.name
        dialog.open()
    }

    function fetchText(address, whenDone) {
        const request = new XMLHttpRequest()
        request.timeout = 10000
        request.onreadystatechange = function() {
            if (request.readyState !== XMLHttpRequest.DONE) return
            const ok = request.status >= 200 && request.status < 300 && request.responseText.length <= installation.manifestLimit
            whenDone(ok ? request.responseText : "")
        }
        request.open("GET", address)
        request.send()
    }

    // The bytes of a file as base64, empty when it cannot be had or is too large.
    function fetchBase64(address, whenDone) {
        const request = new XMLHttpRequest()
        request.timeout = 10000
        request.responseType = "arraybuffer"
        request.onreadystatechange = function() {
            if (request.readyState !== XMLHttpRequest.DONE) return
            const ok = request.status >= 200 && request.status < 300 && request.response
                && request.response.byteLength > 0 && request.response.byteLength <= installation.iconLimit
            whenDone(ok ? encodeBase64(new Uint8Array(request.response)) : "")
        }
        request.open("GET", address)
        request.send()
    }

    function encodeBase64(bytes) {
        const alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
        const pieces = []
        for (let position = 0; position < bytes.length; position += 3) {
            const chunk = (bytes[position] << 16) | ((bytes[position + 1] || 0) << 8) | (bytes[position + 2] || 0)
            pieces.push(alphabet[(chunk >> 18) & 63] + alphabet[(chunk >> 12) & 63]
                + (position + 1 < bytes.length ? alphabet[(chunk >> 6) & 63] : "=")
                + (position + 2 < bytes.length ? alphabet[chunk & 63] : "="))
        }
        return pieces.join("")
    }

    // Tries the advertised icons best first, then the favicon; whatever is not usable leaves the app's icon to
    // the browser's own.
    function chooseIcon(position, whenDone) {
        if (position >= draft.icons.length) { useFavicon(whenDone); return }
        fetchBase64(draft.icons[position], function(encoded) {
            if (encoded !== "" && webApps.accept_icon(encoded)) whenDone()
            else chooseIcon(position + 1, whenDone)
        })
    }

    function useFavicon(whenDone) {
        const source = tab ? tab.icon.toString() : ""
        if (source === "") { whenDone(); return }
        favicon.source = source
        const grab = function() {
            if (favicon.status !== Image.Ready || favicon.width <= 0 ||
                !favicon.grabToImage(function(result) {
                    result.saveToFile(webApps.icon_staging_path())
                    webApps.accept_icon_file(webApps.icon_staging_path())
                    whenDone()
                })) {
                whenDone()
            }
        }
        if (favicon.status === Image.Ready) grab()
        else favicon.statusChanged.connect(function once() {
            if (favicon.status === Image.Loading) return
            favicon.statusChanged.disconnect(once)
            grab()
        })
    }

    function install(name) {
        chooseIcon(0, function() {
            const result = JSON.parse(webApps.install(name, draft.startUrl))
            if (result.ok) installed(result.id, result.name, tab)
            else notice(say("Could not install the app: {error}").replace("{error}", result.error), "", null)
        })
    }

    // The page's favicon, drawn once so it can be saved as a PNG.
    Image {
        id: favicon
        visible: true
        opacity: 0.01
        width: sourceSize.width
        height: sourceSize.height
        z: -1
    }

    InstallAppDialog {
        id: dialog
        bridge: installation.bridge
        onConfirmed: function(name) { installation.install(name) }
    }
}
