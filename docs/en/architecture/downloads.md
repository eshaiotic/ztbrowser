# Downloads

ZTBrowser saves a download where the person wants it, never overwrites an existing file, and keeps the five
latest downloads one click away (a button in the toolbar and an entry in the main menu both open the same list) and the
whole history in a manager.

## 1. The problem

Before this work every download went silently into the system's Downloads folder and the only trace was a toast
with "Show in Folder". A person could not choose a folder per file, could not see what had been downloaded
earlier, and could not tell how far a large download had got.

## 2. Vocabulary

| Term | Meaning |
|---|---|
| **Download folder** | The folder files are saved into (or the save dialog starts in). The system's Downloads folder until the person picks another. |
| **Download history** | Up to 500 downloads of a profile, newest first. Only finished ones are written to disk. |
| **Clear List** | Empties the list of one window only: the window remembers the newest download it hid. The history is untouched. |
| **Manager** | A dialog with the whole history, a search field, a remove button per row and "Clear History". |
| **Mnemonic** | The underlined letter of a button label (`_Save`) that activates it with Alt. |

## 3. How it fits together

![A download goes from WebKit through the tracker and the center to the list](../../assets/diagrams/architecture-downloads.en.svg)

- **`ztbrowser-core`** holds the decisions, with no GTK: `DownloadFileName` cleans the name a website suggests and
  finds a free path, `DownloadSettings` is the two stored choices (ask where to save; download folder), and
  `DownloadHistory` is the history (500 entries at most) with each entry's state (in progress, finished, failed);
  `latest_after` yields the few entries a window shows after its own "Clear List" (a running download stays visible).
- **`DownloadTracker`** (app) listens to WebKit. For each download it proposes a path in the download folder.
  When the person asks to be asked it opens a save dialog (dismissing it cancels the download); otherwise it
  saves at once. If no file chooser can be shown at all, the file is kept at the proposed path and the problem is
  logged: a download the person wanted is never dropped.
- **`DownloadCenter`** (app) owns the list and the settings of a profile, writes `downloads.json` and
  `download-settings.json`, and tells its observers after every change. The Tor window keeps both in memory only.
- **`DownloadsList`** (app) is the widget in the toolbar popover and in the main menu's "Downloads": the five latest
  rows, then "Show All Downloads" (opens the manager), "Clear List" (this window only) and "Go to Download Folder".
- **`DownloadRow`** (app) is one row in both places: click opens a finished passive file, a folder button shows where
  it is (entries may be in different folders). A file that was moved or deleted shows "File Moved or Deleted" and
  can only be shown in its folder or removed.
- **`DownloadsManager`** (app) is the dialog with the whole history: search by name, "Remove from List" per row, and
  "Clear History", which forgets every entry (not the files on disk).

## 4. Preferences

*Preferences → Downloads* has "Ask where to save each file" (on by default) and "Download folder". The toolbar
button is one of the hideable buttons (*Preferences → Appearance*); the main menu entry is always there.

## 5. Why this approach

The decisions live in the domain crate so they are tested without a display (`tests/download_settings.rs`,
`tests/download_history.rs`). Progress is reported at most four times a second because a large file would otherwise
rebuild the list thousands of times. Only finished downloads are stored: a download that was running when the
browser closed has no file worth offering.

## 6. Limits

The save dialog goes through the desktop's file-chooser portal. It was not exercised in the headless test
environment (no session bus); the fallback above covers its absence.
