import QtQuick
import QtQuick.Dialogs
import QtWebEngine

// Takes every download the engine offers: picks where it goes, tracks its progress in the profile's history and
// keeps the lists the toolbar popover and the manager show.
Item {
    id: hub
    required property var bridge

    readonly property alias latest: latestRows
    readonly property alias everything: allRows
    property int clearedThrough: -1
    property bool managerOpen: false
    property bool popoverOpen: false
    property var waiting: null

    signal finished(string name, string path)
    signal failed(string message)

    ListModel { id: latestRows }
    ListModel { id: allRows }

    // The same downloads in the same order are updated in place, so a list being scrolled or hovered keeps still.
    function fill(model, json) {
        const rows = JSON.parse(json)
        const unchanged = model.count === rows.length && rows.every(function(row, index) { return model.get(index).id === row.id })
        if (!unchanged) model.clear()
        rows.forEach(function(row, index) {
            const values = { id: row.id, name: row.name, path: row.path, state: row.state,
                             received: row.received, total: row.total === null ? -1 : row.total, missing: row.missing }
            if (unchanged) model.set(index, values)
            else model.append(values)
        })
    }

    function reload() {
        fill(latestRows, bridge.downloads_json(clearedThrough, 5))
        if (managerOpen) fill(allRows, bridge.downloads_json(-1, 0))
    }

    // A file deleted or moved while a list is on screen greys out its row: the lists look again every two seconds.
    Timer { interval: 2000; repeat: true; running: hub.managerOpen || hub.popoverOpen; onTriggered: hub.reload() }

    // A large file reports progress thousands of times: the lists are rebuilt at most four times a second.
    Timer { id: coalesce; interval: 250; onTriggered: hub.reload() }
    function scheduleReload() { if (!coalesce.running) coalesce.start() }

    function clearList() {
        const first = latestRows.count > 0 ? latestRows.get(0).id : -1
        if (first >= 0) clearedThrough = first
        reload()
    }

    function clearHistory() { bridge.download_clear_history(); clearedThrough = -1; reload() }
    function remove(id) { bridge.download_remove(id); reload() }
    // A file that is gone cannot be selected: its folder is shown instead.
    function showInFolder(path) {
        if (!bridge.show_file_in_folder(path)) bridge.open_folder(path.substring(0, path.lastIndexOf("/")))
    }
    function openFolder() { bridge.open_folder(bridge.download_folder()) }

    function begin(download, path) {
        const slash = path.lastIndexOf("/")
        download.downloadDirectory = path.substring(0, slash)
        download.downloadFileName = path.substring(slash + 1)
        const id = bridge.download_begin(path)
        download.receivedBytesChanged.connect(function() {
            bridge.download_progress(id, download.receivedBytes, download.totalBytes)
            hub.scheduleReload()
        })
        download.stateChanged.connect(function() {
            switch (download.state) {
            case WebEngineDownloadRequest.DownloadCompleted:
                bridge.download_finish(id)
                hub.finished(path.substring(slash + 1), path)
                break
            case WebEngineDownloadRequest.DownloadInterrupted:
                bridge.download_fail(id)
                hub.failed(bridge.message("Download failed: {error}").replace("{error}", download.interruptReasonString))
                break
            case WebEngineDownloadRequest.DownloadCancelled:
                bridge.download_remove(id)
                break
            }
            hub.reload()
        })
        download.accept()
        reload()
    }

    Connections {
        target: WebEngine.defaultProfile
        function onDownloadRequested(download) {
            const path = bridge.download_target(download.suggestedFileName || download.downloadFileName)
            if (bridge.asks_where_to_save()) {
                hub.waiting = download
                saveDialog.currentFolder = "file://" + path.substring(0, path.lastIndexOf("/"))
                saveDialog.currentFile = "file://" + path
                saveDialog.open()
            } else {
                hub.begin(download, path)
            }
        }
    }

    FileDialog {
        id: saveDialog
        title: bridge.message("Save File")
        fileMode: FileDialog.SaveFile
        onAccepted: {
            const download = hub.waiting
            hub.waiting = null
            if (download) hub.begin(download, decodeURIComponent(selectedFile.toString().replace("file://", "")))
        }
        onRejected: {
            if (hub.waiting) hub.waiting.cancel()
            hub.waiting = null
        }
    }

    Component.onCompleted: reload()
}
