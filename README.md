**English** | [Polski](README.pl.md)

<p align="center">
  <img src="assets/icons/io.github.ztbrowser.ZTBrowser.svg" alt="ZTBrowser: a purple shield with a white chain" width="160">
</p>

# ZTBrowser

**A privacy-first web browser for Linux.** It rebuilds the protections of the open-source DuckDuckGo macOS
browser on **QtWebEngine** (Chromium, the engine of Chrome and Brave), with a **Qt 6 Quick** user interface and a
domain core written in **Rust**. No telemetry, no extensions to manage, every profile and installed web app kept
apart from the others.

[![tests](https://github.com/eshaiotic/ztbrowser/actions/workflows/tests.yml/badge.svg)](https://github.com/eshaiotic/ztbrowser/actions/workflows/tests.yml)
[![release](https://img.shields.io/github/v/release/eshaiotic/ztbrowser?color=6A5ACD)](https://github.com/eshaiotic/ztbrowser/releases/latest)
![cargo nextest](https://img.shields.io/badge/cargo%20nextest-899%20tests-success)
![stack](https://img.shields.io/badge/stack-Rust%20%E2%80%A2%20Qt%206%20%E2%80%A2%20QtWebEngine-8E7FE8)
![telemetry](https://img.shields.io/badge/telemetry-none-FFB300)
[![licence](https://img.shields.io/badge/licence-GPL--3.0--or--later-2E1A87)](LICENSE)

ZTBrowser is an independent project. It is not affiliated with or endorsed by Duck Duck Go, Inc.

---

## Measured against Chrome and Brave, compared with DuckDuckGo

On 2026-11-06 fresh profiles of Brave 1.98 (Chromium 156) and ZTBrowser 1.2.3 ran at the same time, each in its own
window behind a proxy that recorded only host names, for 5 minutes of doing nothing and then 20 seconds on each of 12
popular sites. Trackers were judged by a neutral list (Disconnect's), each server counted once per site. ZTBrowser ran
twice: with a fresh profile (no protection data, so it downloads it on first start) and with the data already on
disk. Google Chrome 157 was measured the same way on 2026-11-05, in two runs that show how much a single run varies.

| | Chrome 157 | Brave 1.98 | ZTBrowser, fresh profile | ZTBrowser, protection data on disk |
|---|---|---|---|---|
| Trackers let through on 12 sites | 111 / 109 | 8 | **1** | **1** |
| … of which ad servers | 90 / 88 | 3 | **0** | **0** |
| Connections in the first 5 minutes | 35 / 36 to 14 hosts | 46 to 12 hosts | 13 to 8 hosts | 13 to 8 hosts |

Two numbers in a Chrome cell are its first and its second run.

**How to read it.** ZTBrowser lets through the fewest trackers, with a fresh profile and with the protection data
already on disk alike: 1 on 12 sites, against 8 for Brave and 109 or 111 for Chrome. The one is `m.stripe.network`
(payments), which DuckDuckGo's rules allow because blocking it breaks paying; Google's sign-in prompt
(`accounts.google.com`, `www.google.com`), when a site shows it, is allowed for the same reason. The pages still load.
It gets there with DuckDuckGo's own rules, evaluated as DuckDuckGo evaluates them, a stricter layer for advertising
and analytics trackers, a local gate that refuses the connections a page opens before it sends any request
(`preconnect`, which request filters never see), and a downloader that fetches the rules on first start. The cost of
the downloader shows in idle traffic: ZTBrowser contacted the DuckDuckGo start page and its ping host plus the six hosts
that serve the protection data (13 connections to 8 hosts); Chrome contacted Google and Brave its own servers (usage
ping, P3A). Fingerprinting was not measured: ZTBrowser has no fingerprint protection yet. Chrome 157 and Brave 1.98 are
different Chromium versions, 157 and 156, and Chrome was measured on 2026-11-05. One run from one country on one network is not
a ranking; method and limits are in the [tracking comparison](docs/en/reports/tracking-comparison.md).

### Compared with DuckDuckGo's own browser

DuckDuckGo makes no browser for Linux (Windows, macOS, iOS and Android only), so its browser was **not measured**.
The table sets ZTBrowser against what DuckDuckGo says about its own browser, in the feature list of its
[comparison page](https://duckduckgo.com/compare-privacy) ("See how DuckDuckGo compares") and in its
[web tracking protections help page](https://duckduckgo.com/duckduckgo-help-pages/privacy/web-tracking-protections),
both read in November 2026. The DuckDuckGo column is DuckDuckGo's own claim, for its Windows and macOS apps
(where they differ, macOS is named); the ZTBrowser column is what ZTBrowser does, checked in the code.
For independent numbers on DuckDuckGo's browser, see [privacytests.org](https://privacytests.org/), which tests
DuckDuckGo, Brave and Chrome on desktop and does not test ZTBrowser.

| | DuckDuckGo's browser says | ZTBrowser |
|---|---|---|
| **Tracking scripts** | | |
| Blocks 3rd-party trackers from loading | yes, from its tracker list | yes, by the same list and by Guard's filter lists, on a site's own requests too; the rules are downloaded on first start and kept current (the window waits for them once) |
| Blocks 3rd-party tracker cookies | yes | tracker-list cookies are refused; the *Strict* profile (no interface yet) refuses all of them |
| Expires 1st-party cookies | macOS: 24 hours or 7 days; Windows: roadmap | no |
| Prevents fingerprint tracking | listed scripts blocked, listed APIs overridden; canvas and audio defences off on macOS | **not yet** |
| Protects against CNAME cloaking | a subset of its list on macOS | by the tracker list's cloaked host names |
| Protects against referrer tracking | referrer cut to the host name | Chromium's own default only |
| Blocks link tracking | first navigation only on macOS; Windows: roadmap | every top-level navigation |
| Blocks Google's AMP tracking | macOS: AMP links replaced | no |
| Blocks Google's sign-in pop-ups | yes | no |
| Blocks embedded Facebook content | yes | Facebook is on the tracker list; no click-to-load placeholder |
| **Ads and pop-ups** | | |
| Blocks cookie pop-ups | closes the pop-ups on its list, choosing the private option | rejects them at once with DuckDuckGo's autoconsent (Facebook's included) and hides the rest through a filter list; left alone on a site whose shield is off |
| Blocks targeted and non-tracking ads | says it attempts to block all ads, with a documented Microsoft exception: `bat.bing.com` loads on an advertiser's site after a click on a DuckDuckGo search ad | Guard's filter lists (EasyList, uBlock Origin and others), with no ad-click exception |
| Watches YouTube without ads | yes (Duck Player) | no |
| **Control** | | |
| Global Privacy Control | yes (macOS: the header only to sites known to honour it) | **not yet** |
| Deletes browsing data with one button | Fire button | Burn button |
| Upgrades to HTTPS | the sites on its upgrade list (about 90 % of search clicks) | every navigation except the sites DuckDuckGo's configuration excludes |
| Encrypts DNS queries | yes | not yet (Chromium's secure DNS is not set up) |
| Stores passwords locally, syncs privately | yes | no |
| Usage data sent to its maker | anonymous usage metrics | none |
| **Beyond the browser** | | |
| Private search by default | yes | DuckDuckGo is the default search |
| Scam and phishing protection | Scam Blocker, a Netcraft list checked on the device | no |
| Email protection | free `@duck.com` aliases, sign-up | no |
| VPN | in the paid subscription | not yet; a Tor window is planned |
| Removes personal details from data brokers, helps restore an identity | in the paid subscription (the identity service for U.S. residents) | no; planned, see *Next release* below |
| Runs on Linux | no | yes |

In short: DuckDuckGo's browser wins on finished protections, and ZTBrowser wins on what it will not do
(telemetry, ad-click exceptions, paid tiers) and on running on Linux; the finished-protection gap is the work in
front of this project, not something the numbers above hide.

---

## What it protects you from

ZTBrowser runs on **QtWebEngine** (Chromium), with the interface in Qt Quick. The last column says whether each
protection is in place.

| Protection | How it works | Status |
|---|---|---|
| Third-party trackers | DuckDuckGo's Tracker Data Set decides, request by request, which third-party requests are refused (port of `TrackerRadarKit`). | works; downloaded on first start, refreshed every 6 hours |
| Breakage | DuckDuckGo's privacy configuration supplies per-site exceptions so sites keep working; the shield button (in a web app: *Protect This Site* in its menu) switches protection off for one site. | works |
| Fingerprinting, GPC | DuckDuckGo Content Scope Scripts run in every frame; ZTBrowser Guard switches on the canvas, audio, screen and referrer defences DuckDuckGo leaves off, and sends the `Sec-GPC: 1` header. | not yet |
| Ads, trackers, cookie notices | ZTBrowser Guard downloads EasyList, EasyPrivacy, uBlock Origin's, Peter Lowe's, Polish and cookie-notice filter lists and applies them to a site's own requests too. | works; downloaded with the rest of the data |
| Cookie dialogs | DuckDuckGo's autoconsent library recognises the consent dialogs of hundreds of providers (Facebook's included) and presses *reject* at once; it never accepts. Skipped for a site whose shield is off ([details](docs/en/architecture/qt-protection.md)). | works |
| Cross-site cookies | Third-party cookies on the tracker list are refused; the *Strict* profile, which refuses all of them, exists in the code but has no interface yet. | works |
| Tracking links | A navigation carrying `utm_*`, `fbclid`, `gclid`, ... is stopped before it leaves the machine and reloaded without them. | works |
| Plain HTTP | Navigations are upgraded to HTTPS. | works |
| Leftover data | The Burn button erases the profile's cookies, storage, cache and favicons in one click. | works |
| Mixing identities | Every profile and every installed web app runs as its own process with its own cookies and storage. | works |
| Your IP address | The Tor window sends everything through ZTBrowser's own Tor; its session lives in memory only. | not yet |

There is no extension support, by design.

**Web apps.** *Install Site as App* (the button at the right end of the address bar, or the window menu)
turns any site — your bank, your mailbox — into a desktop application with the site's icon. Each app has
its own isolated session; a link you click inside it to another site opens in the browser, so the app's
login never meets foreign pages. *Installed Apps* in the menu shows them as tiles, with Open and Uninstall.

**Tor window (not yet available).** It is planned as a window whose every request goes through Tor, so sites see a
Tor exit relay instead of your address, through ZTBrowser's own `tor` rather than a system-wide service, with nothing
ever falling back to a direct connection and nothing kept on disk. It will hide your address, not that you use
ZTBrowser: where being indistinguishable among Tor users matters, use Tor Browser.

**Preferences** (`Ctrl+,`), separately in every profile: which toolbar buttons show, whether the address bar is
wide or short and centred, **where the tabs sit** (top, bottom, left or right; top by default), **how the window
buttons look** (the default round buttons on the right, or three coloured dots on the left as on macOS), smooth
mouse-wheel scrolling, the download folder, and the language of the interface (68 languages).

**Default browser.** While another browser opens the links of other programs (mail, chat, documents), the first
window asks once, in a bar under the toolbar, whether ZTBrowser should open them; nothing changes until you choose
*Make Default*, and *Don't Ask Again* silences the bar for good. *Preferences → Interface* shows which browser opens
links, offers the same change, and turns the question at start back on. The desktop's own `xdg-settings` makes the
change, so it works on KDE Plasma, GNOME, Xfce and the others; a build run from source has no menu entry and cannot
be chosen.

---

## How tracking is cut, and what proves it

A site, an advertiser or the browser's own maker can follow you in seven ways. This table names each way, the
mechanism ZTBrowser uses against it, where that mechanism lives in the code, and the check that exercises it. The
decisions live in an engine-neutral domain crate; what is still missing is listed in the
[protection table](#what-it-protects-you-from) above and in the
[migration plan](docs/en/plans/2026-11-04-qtwebengine-migration-plan.md).

| Way of following you | What ZTBrowser does | Code (`crates/ztbrowser-core/src`) | Checked by |
|---|---|---|---|
| The browser reports to its maker | There is no telemetry code at all: no usage ping, crash reporter, experiment seed or account. The only connections the program makes on its own are plain `GET`s for protection data, with no cookie jar and no identifier. | `protection/`, `guard/` (downloads only) | Idle traffic measured at host level: ZTBrowser contacts 8 hosts, the DuckDuckGo start page, its ping host and the six that serve the protection data ([report §4.5](docs/en/reports/tracking-comparison.md)) |
| Third-party trackers | DuckDuckGo's Tracker Data Set decides each request (a request interceptor on QtWebEngine, plus a local connection gate that refuses `preconnect` hints), with DuckDuckGo's per-site exceptions so pages keep working. A stricter layer covers advertising and analytics trackers. | `protection/tracker_data_set.rs`, `protection/content_blocker_rules_builder.rs`, `network/content_blocker_request_policy.rs`, `network/advertising_tracker_request_policy.rs`, `network/connection_gate.rs` | Scenario tests `tracker_data_set`, `content_blocker_rules_builder`, `content_blocker_request_policy`, `advertising_tracker_request_policy`, `connection_gate`, `rule_exceptions`, `gate_proxy`, `tracker_blocking` ([architecture](docs/en/architecture/qt-protection.md)) |
| Ads and trackers the first list misses | Guard downloads nine community lists (EasyList, EasyPrivacy, uBlock Origin, Peter Lowe's, Polish and cookie-notice lists), compiles them and applies them to a site's own requests too. Cookie notices are rejected (autoconsent) and, failing that, hidden. | `guard/guard_engine.rs`, `guard/guard_rule_list.rs`, `network/guard_request_policy.rs` | Tests `guard_engine`, `guard_rule_list`, `guard_request_policy`, `cookie_notice_script`, `tracker_blocking`; trackers on 12 sites measured above |
| Cookies and stored identifiers | Third-party cookies on the tracker list are refused, every profile and web app is its own process with its own storage, and Burn erases a profile in one click. | `storage/cookie_policy.rs`, `storage/tracker_cookie_policy.rs`, `storage/burn_planner.rs` | Tests `cookie_policies`, `burn_planner` |
| Identifiers in links | `utm_*`, `fbclid`, `gclid` and DuckDuckGo's other parameters are stripped from every top-level navigation before the request leaves the machine. | `protection/tracking_parameter_remover.rs`, `network/link_cleaning_request_policy.rs` | Tests `tracking_parameter_remover`, `link_cleaning_request_policy` |
| Fingerprinting | Not in place yet. The domain crate holds the pieces (DuckDuckGo's Content Scope Scripts configuration and Guard's canvas, audio, screen and referrer defences); the browser does not run them yet, so a site reads what Chromium gives out. | `guard/fingerprint_hardening.rs`, `protection/content_scope_script.rs` | Tests `fingerprint_hardening`, `content_scope_script` |
| What the network and the site's server see | Navigations are upgraded to HTTPS. The `Sec-GPC: 1` header and a Tor window that hides the IP address are not in place yet. | `navigation/https_upgrade_policy.rs`, `network/https_upgrade_request_policy.rs` | Tests `https_upgrade_policy`, `https_upgrade_request_policy` |

Every list and data set is downloaded, never bundled, and size-limited and validated before use. A filter list
can only block or hide: the conversion keeps no scriptlets, so a hostile list could break a site but never run
code in it. The whole suite (`./run-tests`, 899 tests including the Qt ones, clippy and `cargo audit`) runs on every push.

### The evidence

The measurement behind the [first table](#measured-against-chrome-and-brave-compared-with-duckduckgo) ran Brave 1.98
and ZTBrowser 1.2.3 side by side on 2026-11-06 (Chrome 157 on 2026-11-05), with fresh profiles, a window each, and a
proxy that logs host names only. Its numbers, repeated here so they sit next to the mechanisms:

- **Trackers reached on 12 sites:** Chrome 111 and 109 in two runs, Brave 8, ZTBrowser 1 on a fresh profile and 1 with
  the protection data on disk, judged by Disconnect's list, which none of them uses.
- **Sent to the browser's own maker in 5 idle minutes:** Chrome contacted only Google (14 hosts), Brave mostly its
  own servers (usage ping, P3A; 12 hosts), ZTBrowser the DuckDuckGo start page, its ping host and the six hosts that
  serve the protection data (8 hosts).
- **Method, limits and a script-level recipe to repeat it:** [tracking comparison](docs/en/reports/tracking-comparison.md), §3 and §8.

### What this does not prove

The measurement supports "ZTBrowser gives its own maker nothing to work with, and, once its protection data is in
place, trackers little". It does not support "ZTBrowser is more anonymous than Brave or DuckDuckGo", and the
project does not claim it.

- **A margin on one sample is not a law.** ZTBrowser blocks harder than Brave here because it
  applies a stricter layer to advertising and analytics trackers and because it refuses `preconnect`; that same
  strictness can break a page that Brave's more lenient shields leave working, and the per-site shield switch is the
  remedy. Sign-in prompts and payment trackers are deliberately left to DuckDuckGo's exceptions.
- **One run on 12 sites is not a ranking.** Chrome's two runs differ by 2; pages change between loads, and sites show a Google sign-in prompt in some runs and not in others.
- **DuckDuckGo's browser was not measured**, because it does not exist for Linux. The
  [feature table](#compared-with-duckduckgos-own-browser) compares documented behaviour, not traffic.
- **Blocking is not anonymity.** ZTBrowser is a Chromium engine like Brave and Chrome, which is good for blending in,
  but ZTBrowser does not yet change what a site can read about the browser: no fingerprint noise, no GPC.
  Installed fonts stay visible, as in Brave and Chrome.
- **No phishing or malware warnings** (no Safe Browsing), no email-alias service and no VPN yet. The planned Tor
  window will hide your address, not that you use ZTBrowser; where being indistinguishable matters, use Tor Browser.
- **Independent verification is missing:** [privacytests.org](https://privacytests.org/) does not test ZTBrowser.

---

## Status

Usable work in progress, on QtWebEngine: browsing with tabs (placed top, bottom, left or right; a tab moves to a window of its own from its right-click menu or by
dragging it off the strip), profiles of
isolated data with a Chrome-like popup of tiles, installed web apps and their tile page, downloads with a folder of
your choice, zoom per site, find-in-page, per-site protection switch, automatic rejection of cookie dialogs, Burn,
68 interface languages. Not built yet: the fingerprint and GPC protection, the Tor window, permission prompts,
session restore, bookmarks and history. The full list with reasons is in the architecture document and the migration
plan.

### Next release

Planned, not started, and to be designed before they are built: removal of your personal details from data
brokers' sites, help restoring your identity if it is stolen, and a VPN, all free. DuckDuckGo sells these three
in its paid subscription; ZTBrowser aims to include them without charge. They need servers or partners and a legal
set-up, so there is no date.

In the future ZTBrowser will ship as part of a new Linux distribution. It stays a standalone project and keeps
its packages for other distributions.

---

## Installing

Download a package from the [latest release](https://github.com/eshaiotic/ztbrowser/releases/latest).

| File | For |
|---|---|
| `ztbrowser-*.fc44.x86_64.rpm` | Fedora 44 |
| `ztbrowser-*-x86_64.pkg.tar.zst` | Arch Linux, CachyOS, EndeavourOS, Manjaro |
| `ZTBrowser-*-x86_64.AppImage` | any other distribution with glibc 2.43 or newer (about 230 MB: it contains Chromium 140) |

Every release carries `SHA256SUMS` and a build provenance attestation, which proves the file was built by
this repository's GitHub Actions:

```sh
sha256sum --check --ignore-missing SHA256SUMS
gh attestation verify ZTBrowser-*-x86_64.AppImage --repo eshaiotic/ztbrowser
```

The rpm and pacman packages use your distribution's Qt and QtWebEngine, so the browser engine keeps
receiving its security updates; they depend on the Qt Quick, QtWebChannel, QtWebEngine, SVG and xdg-utils packages, which
the package manager installs. The AppImage carries its own Qt and QtWebEngine (Chromium), takes the graphics
stack from the host, and gets engine updates only with a new ZTBrowser release. It needs no installation:
`chmod +x ZTBrowser-*-x86_64.AppImage && ./ZTBrowser-*-x86_64.AppImage`. Wayland desktops take a window's icon from an
installed `.desktop` file, which an AppImage does not have, so on start it writes one for itself
(`~/.local/share/applications/io.github.ztbrowser.ZTBrowser.desktop` and the icon beside it, pointing at the AppImage
file; start it again after moving the file). The menu entry runs that file later, so it names the real file (links
resolved) and is written only when that file and every folder above it, up to your home folder, belong to root or to
you and cannot be written to by the group or others (nothing under `/tmp` or another shared folder qualifies). The
entry is removed again when its file stops qualifying or when you install the package (your entry wins over the system
one, so without this the menu would keep starting the AppImage instead of the package; the entry goes at the AppImage's
next start, until then delete it by hand). An entry you wrote yourself is never touched (the generated one
carries `X-ZTBrowser-AppImage=<path>`), and nothing is written when the package is installed. There is no Tor
window yet, so no package brings Tor.

From a checkout of this repository:

```sh
./install                 # Arch: builds the pacman package and installs it; Fedora: installs dist/;
                          # then asks whether ZTBrowser should be your default browser (no by default)
./uninstall               # removes the package; profiles and web apps are kept
./uninstall --purge       # also deletes all ZTBrowser data and web app launchers
```

---

## Building release packages

```sh
packaging/release.sh                # all formats into dist/, with SHA256SUMS
packaging/release.sh appimage rpm   # or only some: arch, appimage, rpm
```

The AppImage and the `.rpm` are built in a Fedora 44 container (rootless podman, or `CONTAINER_ENGINE=docker`),
so they link against the libraries of the systems they target and carry or use Chromium 140. There is no `.deb`:
the Qt WebEngine of Debian 13 (Qt 6.8, Chromium 122) is older than ZTBrowser's engine floor, and no stable Debian
or Ubuntu ships a newer one. The Arch package is built on the host with `makepkg` and its
`makepkg.conf` flags (without link-time optimisation, which Rust's linker cannot read in the C++ part). The
AppImage runtime is pinned to a release and checked against its SHA-256.

Pushing a version tag publishes a release: `git tag vX.Y.Z && git push origin vX.Y.Z`. The
[release workflow](.github/workflows/release.yml) runs the same scripts (the Arch package in an Arch Linux
container), refuses dependencies with known vulnerabilities (`cargo audit`), and uploads the four packages
with `SHA256SUMS` and their attestations. The tag must match the version in `Cargo.toml`.

---

## Running from source

ZTBrowser needs Qt 6 with QtWebEngine, QtWebChannel and Quick Controls (on Arch: `qt6-base qt6-declarative
qt6-webengine qt6-webchannel qt6-svg`), a C++ compiler and Rust.

```sh
sudo pacman -S --needed rust qt6-base qt6-declarative qt6-webengine qt6-webchannel qt6-svg
cargo run --release -p ztbrowser-qt                          # browser, default profile
cargo run --release -p ztbrowser-qt -- --profile work        # browser, profile "work"
cargo run --release -p ztbrowser-qt -- https://example.com/  # open a page
```

Installed web apps start from the desktop's application menu (`ztbrowser-qt --web-app <id>`).

---

## Testing

Stack: **cargo-nextest** — 899 object-oriented Given/When/Then scenarios: 683 for the domain in
`crates/ztbrowser-core/tests/` (82 files, one per domain type), 146 for the Qt interface and its engine adapter,
61 for the report tool in `crates/ztbrowser-test-report/tests/` and 9 for the benchmark tool in
`crates/ztbrowser-bench/tests/`. CI runs all of them in an Arch container with Qt 6, the Qt ones under Xvfb with Mesa (QtWebEngine needs an OpenGL
context even offscreen); run only those with
`xvfb-run -a cargo test -p ztbrowser-qt -p ztbrowser-engine-qt` (or `QT_QPA_PLATFORM=offscreen` on a desktop).

```sh
./run-tests                     # all tests + HTML report in tests/reports/ (opens in the browser)
cargo test -p ztbrowser-core     # domain tests only (fast, no Qt needed)
cargo clippy --all-targets      # lints (pedantic, unsafe code forbidden)
cargo fmt --check               # formatting
cargo audit                     # dependency vulnerabilities (zero-vulnerability policy)
```

**Test report:** `./run-tests` always writes `tests/reports/test-report-YYYYMMDD-HHmm.html` — a run-history
dashboard with the latest run, the last 20 runs, failure rankings, features and the slowest tests, plus
`ztbrowser-core`'s coverage when `cargo-llvm-cov` is installed. See [HTML test report](docs/en/tests/test-report.md).

**Test architecture:**
- The domain (`ztbrowser-core`) has no Qt dependency, so every protection decision — rule
  building, filter list conversion, HTTPS upgrade, link cleaning, profiles, web apps, zoom — is tested
  without a display.
- Each scenario is an object: `given_…` builds it, `when_…` acts and returns an outcome, `then_…` asserts
  one thing, following [Object Oriented Programming First](docs/en/engineering-rules.md):
  `LinkCleaningScenario::given_tracking_parameters_protection().when_link_is_opened(url).then_opens_unchanged()`.
- A failing test is retried once: one that then passes is reported as FLAKY, a defect to fix, never ignored.
- The [tests workflow](.github/workflows/tests.yml) runs fmt, clippy, `cargo audit` and `./run-tests` on
  every push to `main` and on every pull request, and keeps each run's report as an artifact.

---

## Project structure

```
crates/ztbrowser-core/    domain logic without Qt: protection rules, Guard filter lists,
                         profiles, web apps, zoom, launch, tab placement and window button style
crates/ztbrowser-core/tests/
                         one OO scenario file per domain type
crates/ztbrowser-qt/      the browser: Qt Quick interface (QML) on QtWebEngine, bound to Rust with cxx-qt
crates/ztbrowser-engine-qt/
                         the QtWebEngine adapter: request and cookie policies, C++ shim
crates/ztbrowser/         GTK 4 + libadwaita application on WebKitGTK, kept until the Tor window and fingerprint protection exist in the Qt one
crates/ztbrowser-test-report/
                         developer tool behind ./run-tests: the HTML test report (never packaged)
assets/                  vendored Content Scope Scripts (Apache-2.0) and autoconsent (MPL-2.0), Guard's page script, icons,
                         error page template; never the Tracker Data Set or filter lists
packaging/               PKGBUILD, rpm spec, AppImage build and assembly, desktop entry, AppStream metadata
docs/en, docs/pl         architecture, engineering rules and reports (English canonical, Polish mirror)
```

- [Architecture](docs/en/architecture/overview.md) — processes, isolation, the protection pipeline, web apps, limitations.
- [Engineering rules](docs/en/engineering-rules.md) — Object Oriented Programming First, OO Given/When/Then tests.

---

## Licence

ZTBrowser is free software under the [GNU General Public License, version 3 or later](LICENSE)
(`GPL-3.0-or-later`). Attributions are in [NOTICE](NOTICE).

### Licences of third-party data

- Content Scope Scripts 20.0.0 — Apache-2.0, vendored in `assets/content-scope-scripts` with its licence.
- Privacy configuration — Apache-2.0, downloaded at runtime.
- Tracker Data Set — **CC BY-NC-SA 4.0** (non-commercial), downloaded at runtime, never committed.
- ZTBrowser Guard's filter lists — GPL-3.0, CC BY-SA 3.0, CC BY 3.0, CC BY-NC-SA 4.0 or their publisher's terms,
  downloaded at runtime, never committed.
- adblock-rust 0.13.5 — MPL-2.0, compiled in unmodified (source: <https://github.com/brave/adblock-rust>).
- autoconsent 18.0.0 — MPL-2.0, bundled with ZTBrowser's own entry point in `assets/autoconsent` (licence, entry source and
  rebuild recipe beside it; library source: <https://github.com/duckduckgo/autoconsent>).
