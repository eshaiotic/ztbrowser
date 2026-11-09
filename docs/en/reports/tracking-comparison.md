# Tracking comparison: ZTBrowser, Google Chrome and Brave

Measured on 2 November 2026 (21:53–22:10 CET) on one Linux machine in Poland. Policy facts were checked
against vendor sources the same day; each one links its source.

**Which ZTBrowser this measures.** Sections 1–8 measured the WebKitGTK edition (the versions before 1.2.0). ZTBrowser
is now built on QtWebEngine; its results from 5 November 2026 are in [§4.4](#44-the-qt-edition-5-november-2026), and
the mechanisms behind them in [Tracker protection in the Qt edition](../architecture/qt-protection.md).

## 1. What this report answers

A browser can expose its user in three ways. This report measures each of them in all three browsers.

1. **Through the browser itself.** The program can send data to its vendor (telemetry) without the user doing
   anything.
2. **Through the pages it loads.** A news site embeds dozens of third-party scripts. The browser decides
   whether they load.
3. **Through what pages can read about the device.** Fingerprinting recognises a browser without cookies,
   from values such as how it draws an image or how many processor cores it reports.

The short answer is in §6. Sections 3–5 show how each number was obtained, so it can be checked and repeated.

## 2. Vocabulary

| Term | Meaning |
|---|---|
| Tracker | A company or service that follows people across sites: advertising, analytics, social widgets, fingerprinting. Here, a host is a tracker if Disconnect's list says so (§3). |
| Third-party host | A host that belongs to a different site (registrable domain) than the page being visited, for example `doubleclick.net` on `onet.pl`. |
| Telemetry | Data a browser sends to its vendor about how it is used: usage statistics, crash reports, experiment assignments. |
| Fingerprinting | Recognising a browser from values it exposes (canvas drawing, audio processing, core count, screen) instead of from a cookie. |
| Farbling / noise | Brave's name, and Guard's practice, for adding small random changes to these values, so a fingerprint differs between sites. |
| GPC | Global Privacy Control: a `Sec-GPC: 1` request header and a `navigator.globalPrivacyControl` value that mean "do not sell or share my data". Some US state laws make sites honour it. |
| Safe Browsing | Google's service that warns about phishing and malware pages; the browser checks the addresses it visits against it. |
| P3A | Brave's Privacy-Preserving Product Analytics: answers to fixed usage questions, sent through STAR, a scheme that reveals an answer only when enough users gave the same one. |
| Start page | The page a browser shows on launch and in a new tab. |

## 3. How it was measured

**Versions.**

| Browser | Version | How it was run |
|---|---|---|
| Google Chrome | 156.0.8078.79 | Google's official `.deb`, unpacked into `/tmp` (not installed) |
| Brave | 1.98.63 (Chromium 156) | Arch package `brave-bin` |
| ZTBrowser | 1.0.0 with Guard (commit `0934be1`) | The release AppImage; WebKitGTK 2.54.2 from the system |

**Setup.**

- Every browser started with a **fresh profile**, its own `HOME` and XDG directories, and a private D-Bus
  session. Nothing was shared with the user's real profiles.
- Every browser ran **with a window** on its own virtual display (Xvfb). Headless mode was not used, because
  sites recognise it and serve fewer ads.
- Chrome and Brave were started with `--no-first-run --no-default-browser-check`; otherwise all settings were
  defaults.
- All three ran **at the same time**, so they saw the same versions of the same pages.
- All traffic went through a local proxy that records **only the host name** of each connection. It does not
  decrypt TLS, so it sees what an internet provider or Wi-Fi operator sees, not the contents.

**Procedure.**

1. Each browser idled for **5 minutes** on its own start page. Everything it contacted in that time, it
   contacted on its own.
2. It then opened each of **12 sites** in a new tab for **20 seconds**:
   - Polish: onet.pl, wp.pl, interia.pl, gazeta.pl, o2.pl, allegro.pl, olx.pl;
   - international: edition.cnn.com, theguardian.com, bbc.com, weather.com, forbes.com.
3. Every host contacted during a site's window was attributed to that site.

**Who decides what counts as a tracker.** A neutral list does: Disconnect's tracker list, the one Firefox uses
(categories Advertising, Analytics, Fingerprinting, Social, Cryptomining). It is neither ZTBrowser's lists nor
Brave's. A tracker owned by the visited site itself (for example a Google tracker on a Google site) is not
counted.

**Limits of the method.**

- One run, from one country.
- 20 seconds per page with no scrolling and no click on consent banners. Many sites load more trackers after
  "Accept", so all counts, and Chrome's most, understate real use.
- Host names only: two requests to the same host count once, and what was sent is not visible.
- Disconnect's list does not know every tracker.

## 4. Results

### 4.1 What each browser sends on its own

These are the hosts contacted in the 5 idle minutes, before any site was opened.

| | Chrome | Brave | ZTBrowser |
|---|---|---|---|
| Connections | 46 | 43 | 20 |
| Distinct hosts | 15 | 12 | 8 |
| Owner of those hosts | Google (all 15) | Brave (10), Google (2) | DuckDuckGo (3), filter-list hosts (5) |

**Chrome** contacted only Google. The hosts and what Google documents them for:

- `accounts.google.com`: account check. No account was signed in.
- `clientservices.googleapis.com`: the experiment ("variations") seed.
- `update.googleapis.com`, `redirector.gvt1.com`, `edgedl.me.gvt1.com` and a `*.gvt1.com` cache: component
  updates.
- `safebrowsing.googleapis.com`: Safe Browsing lists.
- `optimizationguide-pa.googleapis.com`: page-loading hints and models.
- `chromewebstore.googleapis.com`, `clients2.google.com`, `clients2.googleusercontent.com`: extension
  updates.
- `www.google.com`, `www.gstatic.com`, `ogads-pa.clients6.google.com`, `play.google.com`: the new-tab page,
  which is Google's search box and account bar.

Usage statistics and crash reports are **on by default** on Linux (opt-out).
([source code](https://github.com/chromium/chromium/blob/main/chrome/browser/first_run/first_run_internal_posix.cc),
[Google help](https://support.google.com/chrome/answer/14746339))

**Brave** contacted mostly its own servers:

- `usage-ping.brave.com`: the daily usage ping. It is on by default. Its fields include the install date, the
  week of install and a referral code. Since February 2026 it also includes yesterday's counts of Brave,
  Google and other searches.
  ([fields](https://github.com/brave/brave-core/blob/master/browser/brave_stats/brave_stats_updater_params.cc),
  [search counts](https://github.com/brave/brave-core/pull/33474))
- `collector.bsg.brave.com`, `star-randsrv.bsg.brave.com`: P3A, on by default (opt-out), sent through STAR
  ([P3A wiki](https://github.com/brave/brave-browser/wiki/P3A)).
- `variations.brave.com`: Brave's experiment seed.
- `go-updater.brave.com`, `componentupdater.brave.com`, `redirector.brave.com`, `brave-core-ext.s3.brave.com`:
  components and filter lists, through Brave's proxy for Google's updater.
- `safebrowsing.brave.com`: Safe Browsing through Brave's proxy, so Google does not see the IP address.
- `dict.brave.com`: spell-check dictionaries.
- `clients2.google.com`, `clients2.googleusercontent.com`: Google directly. Brave documents that extension
  updates still go to Google
  ([Deviations from Chromium](https://github.com/brave/brave-browser/wiki/Deviations-from-Chromium-(features-we-disable-or-remove))).

**ZTBrowser** sends no telemetry: there is no usage ping, no crash report and no experiment seed. Its 20
connections fall into two groups.

- **The protection data, fetched by the program (8 connections).**
  - `staticcdn.duckduckgo.com`: DuckDuckGo's Tracker Data Set and privacy configuration.
  - `easylist.to`, `ublockorigin.github.io`, `pgl.yoyo.org`, `raw.githubusercontent.com`,
    `secure.fanboy.co.nz`: the Guard filter lists.
  - These are plain `GET` requests with no cookie jar and no identifier, repeated every 12 hours. They reveal
    the IP address to those servers, as any blocker's list updates do.
  - A refresh sends back the server's `ETag` (a version tag) so that unchanged files are not downloaded again.
    For these static files that tag is the file's version.
- **The start page (12 connections).**
  - Six went to `duckduckgo.com`. ZTBrowser opens the chosen search engine's home page, and DuckDuckGo is the
    default.
  - Seven went to `improving.duckduckgo.com`: DuckDuckGo's own anonymous page-statistics pixels, sent by that
    page. They were sent in the first 3 seconds of a fresh profile, before the Guard lists had finished
    downloading.
  - This is the site's traffic, not the browser's. It is still traffic the user did not ask for. With Google as
    the profile's search engine, the start page would be google.com and would bring Google's own measurement.
    See §7.

### 4.2 Trackers on 12 sites

Each cell is tracker hosts / all third-party hosts contacted while the site was open.

| Site | Chrome | Brave | ZTBrowser |
|---|---|---|---|
| onet.pl | 9 / 30 | 0 / 12 | 0 / 12 |
| wp.pl | 2 / 8 | 0 / 5 | 0 / 2 |
| interia.pl | 8 / 20 | 0 / 2 | 0 / 3 |
| gazeta.pl | 2 / 10 | 0 / 4 | 0 / 4 |
| o2.pl | 3 / 14 | 0 / 8 | 0 / 5 |
| allegro.pl | 0 / 6 | 0 / 2 | 0 / 2 |
| olx.pl | 17 / 35 | 1 / 11 | 0 / 10 |
| edition.cnn.com | 17 / 37 | 1 / 9 | 0 / 1 |
| theguardian.com | 0 / 9 | 0 / 5 | 0 / 6 |
| bbc.com | 10 / 20 | 1 / 6 | 1 / 8 |
| weather.com | 19 / 30 | 2 / 4 | 1 / 6 |
| forbes.com | 7 / 24 | 0 / 14 | 0 / 13 |
| **Total** | **94 / 243** | **5 / 82** | **2 / 72** |

Split by Disconnect's category, the same tracker hosts are:

| | Chrome | Brave | ZTBrowser |
|---|---|---|---|
| Advertising | 75 | 4 | 1 |
| Analytics and other categories | 19 | 1 | 1 |

- **Chrome** blocks no trackers. It only filters ads on sites that fail the Better Ads Standards
  ([Google help](https://support.google.com/chrome/answer/7632919)). Since July 2025 it no longer runs the full
  uBlock Origin (Manifest V2 is gone)
  ([timeline](https://developer.chrome.com/docs/extensions/develop/migrate/mv2-deprecation-timeline)).
- **The 94 trackers in Chrome** include:
  - Google's ad and analytics network: `securepubads.g.doubleclick.net`, `pagead2.googlesyndication.com`,
    `google-analytics.com`;
  - Gemius, the Polish audience measurement (`*.hit.gemius.pl`);
  - Amazon advertising, Taboola, Permutive and Quantcast.
- **Brave's 5:** `securepubads.g.doubleclick.net` (twice), `pub.doubleverify.com`, `micro.rubiconproject.com`
  and `m.stripe.network`.
- **ZTBrowser's 2:** `securepubads.g.doubleclick.net` and `pub.doubleverify.com`.
- The earlier measurement from the same harness (overview §5.6, Brave 6 / 82 against Guard 3 / 70) gives the
  same order. Differences of one or two hosts between runs are within what pages change from one load to the
  next.

### 4.3 Fingerprinting

The same probe page read the values fingerprinting scripts use.

- **Two sites:** `127.0.0.1` and `localhost` count as different sites.
- **Two sessions:** the browser was restarted on the same profile between them.

A fingerprint that stays the same across sites lets two unrelated sites recognise the same person.

| Value | Chrome | Brave | ZTBrowser |
|---|---|---|---|
| Canvas drawing | same everywhere | differs per site, same after restart | differs per site and per session |
| Audio processing | same everywhere | differs per site, same after restart | differs per site and per session |
| Processor cores | 16 (the real count) | 4 or 5, per site | 8 (WebKit's cap) |
| Device memory | 32 GB | 4 or 8 GB, per site | not exposed |
| Languages | `pl-PL, pl, en-US, en` | `pl-PL` | `pl-PL` |
| Storage quota | 10 GB | 2 GB | not exposed |
| GPC | not sent | sent | sent |
| Installed fonts (of 20 probed) | 8 visible | 8 visible | 7 visible |

- **Chrome** changes nothing between sites. Since 16 February 2025 Google's advertising platform policy no
  longer forbids fingerprinting. It only requires disclosure
  ([Google policy](https://support.google.com/platformspolicy/answer/15738904),
  [analysis](https://blog.lukaszolejnik.com/biggest-privacy-erosion-in-10-years-on-googles-policy-change-towards-fingerprinting/)).
- **Brave** randomises per site. Its documentation describes the seed as per session and per site
  ([wiki](https://github.com/brave/brave-browser/wiki/Fingerprinting-Protections)). In this run, however, the
  same site got the **same values after a restart** on the same profile.
- **ZTBrowser**'s noise (DuckDuckGo's Content Scope Scripts, switched on by Guard) changed with both the site
  and the session.
- **No browser hid the installed fonts.**
- WebGL could not be compared: Chromium blocks it on a virtual display without a GPU.

### 4.4 The Qt edition, 5 November 2026

The same 12 sites, 20 seconds each, with the same host-name-logging proxy and the same judge (Disconnect's list, each
server counted once per site), on the QtWebEngine edition. Chrome 157 and Brave 1.98 were measured in the run of the
same day before the Qt protection work (Chrome 111 / 109 trackers, Brave 9 / 8 in two runs); the Qt edition was
measured again after it, twice: with a fresh profile (no protection data, so the browser downloads it on first start)
and with the data already on disk.

| Site | Qt, fresh profile | Qt, data on disk |
|---|---|---|
| onet.pl, wp.pl, interia.pl, gazeta.pl, o2.pl, allegro.pl, olx.pl | 0 | 0 |
| edition.cnn.com | 4 | 4 |
| theguardian.com, bbc.com, forbes.com | 0 | 0 |
| weather.com | 1 | 1 |
| **Total trackers** | **5** | **5** |
| All hosts contacted | 139 | 138 |
| Idle, 5 minutes | 13 connections to 8 hosts | 13 connections to 8 hosts |

- **The 5:** `accounts.google.com`, `google.com` and `www.google.com` (Google's sign-in prompt, on edition.cnn.com
  and weather.com) and `m.stripe.network` (payments, on edition.cnn.com). DuckDuckGo's rules allow them on purpose,
  because blocking them breaks signing in and paying.
- **Idle traffic** is the DuckDuckGo start page, its ping host and the six hosts that serve the protection data
  (`staticcdn.duckduckgo.com`, `easylist.to`, `secure.fanboy.co.nz`, `ublockorigin.github.io`, `pgl.yoyo.org`,
  `raw.githubusercontent.com`). The earlier Qt result of 2 hosts was the cache of the GTK edition, with no downloader.
- **Why the fresh profile equals the profile with data:** the first start downloads the data before the window opens
  ([§6 of the architecture document](../architecture/qt-protection.md)), so there is never an unprotected page.
- **Pages still load:** allegro.pl (7 hosts) and forbes.com (5), which an earlier Qt build left blank, load.
- **Limits.** Different days' runs of Chrome and Brave, one machine, one network; 5 against the GTK edition's 6 is
  within the variation of two runs. Fingerprinting was not measured for the Qt edition, which has no protection
  against it yet.

### 4.5 ZTBrowser 1.2.3 and Brave, 6 November 2026

The same 12 sites, 20 seconds each, with a host-name-logging proxy and the same judge (Disconnect's list, each server
counted once per site). Brave 1.98 (Chromium 156) and ZTBrowser 1.2.3 (the release AppImage) ran at the same time, each
with a fresh profile, a window of its own on a virtual display and a private home; ZTBrowser ran twice, with a fresh
profile and with the protection data already on disk. Each site was opened by launching the browser again with its
address, which opens a new tab; earlier tabs stayed open. Idle traffic was measured separately for ZTBrowser on its own
start page (5 minutes, both profiles). Chrome 157 was not run again; its column in the README is the run of 2026-11-05.

| Site | Brave | ZTBrowser (both profiles) |
|---|---|---|
| onet.pl | 1 (`onet.hit.gemius.pl`) | 0 |
| gazeta.pl | 1 (`gazeta.hit.gemius.pl`) | 0 |
| olx.pl | 1 (`securepubads.g.doubleclick.net`) | 0 |
| edition.cnn.com | 1 (`m.stripe.network`) | 1 (`m.stripe.network`) |
| bbc.com | 1 (`pub.doubleverify.com`) | 0 |
| weather.com | 2 (`micro.rubiconproject.com`, `securepubads.g.doubleclick.net`) | 0 |
| forbes.com | 1 (`dev.visualwebsiteoptimizer.com`) | 0 |
| wp.pl, interia.pl, o2.pl, allegro.pl, theguardian.com | 0 | 0 |
| **Total trackers** | **8** | **1** |
| … of which ad servers (Disconnect's *Advertising*) | 3 | 0 |
| All hosts contacted | 165 | 132 |
| Idle, 5 minutes | 46 connections to 12 hosts | 13 connections to 8 hosts |

- **The 1:** `m.stripe.network`, which DuckDuckGo's rules allow on purpose because blocking it breaks paying.
- **Google's sign-in prompt** (`accounts.google.com`, `www.google.com`), which appeared on edition.cnn.com and
  weather.com in the run of 2026-11-05, did not appear in this one. Two ZTBrowser builds run side by side on those two
  sites gave identical hosts, so the difference is in the sites, not in the browser.
- **Idle traffic of ZTBrowser** is the DuckDuckGo start page, its ping host (`improving.duckduckgo.com`) and the six hosts
  that serve the protection data; the numbers are the same with a fresh profile and with the data on disk, because the
  first start downloads the data before the window opens.
- **Limits.** One run, one machine, one network; Chrome's column is a day older than the others; the tabs of earlier
  sites stay open while the next one loads. Fingerprinting was not measured: ZTBrowser has no fingerprint protection yet.

## 5. Defaults and policies

| | Chrome | Brave | ZTBrowser |
|---|---|---|---|
| Usage statistics to the vendor | on (opt-out) | daily ping and P3A on (opt-out) | none |
| What is typed in the address bar | sent to the search engine as you type, with cookies ([help](https://support.google.com/chrome/answer/13730681)) | off by default in Poland; on at first run in 12 countries ([code](https://github.com/brave/brave-core/blob/master/browser/search_engines/search_engine_provider_util.cc)) | nothing is sent before Enter |
| Third-party cookies | **allowed** in normal windows ([Google](https://privacysandbox.google.com/blog/privacy-sandbox-next-steps)) | blocked; third-party storage is temporary and per site | blocked (`NoThirdParty`) plus WebKit's Intelligent Tracking Prevention |
| Tracker blocking | none | Shields (EasyList, EasyPrivacy, uBlock Origin and Brave lists) | DuckDuckGo's Tracker Data Set plus Guard (EasyList, EasyPrivacy, uBlock Origin, Polish lists) |
| Tracking parameters in links (`fbclid`, `gclid`…) | kept | removed | removed (DuckDuckGo's list) |
| Fingerprinting | no defence | per-site randomisation | per-site, per-session noise |
| GPC | not in stable; proposed ([status](https://chromestatus.com/feature/5137324344213504)) | on | on (header and script value) |
| Safe Browsing (phishing warnings) | Standard: 4-byte hash prefixes through an Oblivious HTTP relay | proxied through Brave | **none** (WebKitGTK has no such service) |
| Installed web apps (Gmail, a bank) | share the browser profile's cookies and storage: Chrome's web-app team called separate storage "not really viable" ([W3C thread, May 2020](https://lists.w3.org/Archives/Public/public-webapps-github/2020May/1169.html); not measured here) | Chromium's behaviour, as in Chrome (not measured here) | each app has its own process, cookies and storage (overview §6) |
| Privacy Sandbox ad APIs | Topics, Protected Audience and others announced for retirement in October 2025 ([Google](https://privacysandbox.google.com/blog/update-on-plans-for-privacy-sandbox-technologies)) | disabled | do not exist in WebKit |

**Independent check.** privacytests.org run 105 (22 October 2026, desktop, default settings):

- Brave 1.97 passed **108 of 146** tests.
- Chrome 156 passed **31 of 146**.
- The site's author worked at Brave from 2022 to 2025, as he discloses.
- ZTBrowser is not on that site.

([privacytests.org](https://privacytests.org/), [about](https://privacytests.org/about.html))

## 6. Summary

| Question | Chrome | Brave | ZTBrowser |
|---|---|---|---|
| Does the browser itself report to its vendor? | yes, to Google, on by default | yes, to Brave, on by default, privacy-preserving by design | no |
| Trackers reached on 12 sites | 94 | 5 | 2 |
| Same fingerprint on two different sites? | yes | no, but one site saw the same values after a restart | no, and it changes with every session; the rare engine still stands out (§7) |
| Does it warn about phishing and malware? | yes | yes | no |

- **ZTBrowser against Chrome.** There is no contest. Chrome reports to Google by default and lets every tracker
  through: 94 tracker hosts on 12 sites. It allows third-party cookies and does not defend against
  fingerprinting, which Google's ad policy has allowed since 2025.
- **ZTBrowser against Brave.** In this measurement ZTBrowser reached fewer trackers (2 against 5) and sent
  nothing to its own vendor. Brave sends a daily ping and P3A, with the install date, a referral code and
  search counts. ZTBrowser also changed its fingerprint between sessions, where Brave's stayed the same.
- **Where Brave is stronger** (§7): phishing warnings, a much larger crowd to hide in, and a few protections
  ZTBrowser lacks.

## 7. Where ZTBrowser is weaker, and what would close the gap

| Gap | Why | What would close it |
|---|---|---|
| No phishing or malware warnings | WebKitGTK has no Safe Browsing. This is a security gap, not a tracking one. | A list-based check of visited hosts with a privacy-preserving lookup. It needs a data source with a licence that allows it. |
| A rare browser stands out | The WebKitGTK User-Agent on Linux is used by few people, so the engine alone narrows down who is browsing. Brave looks like Chrome, among billions of browsers. Noise changes the values but not the engine. | Nothing in ZTBrowser can make WebKitGTK common. Presenting a Chrome User-Agent would break sites that detect the engine by other means. |
| Installed fonts are visible | WebKitGTK offers no font restriction. Brave and Chrome showed the same in this probe. | A WebKitGTK API, if one appears (overview §9). |
| Start page traffic | The start page is the search engine's home page: DuckDuckGo's anonymous pixels by default, Google's measurement if Google is chosen. | A local start page with a search box, which sends nothing until the user searches. |
| No runtime CNAME uncloaking, no debouncing, no AMP redirection | WebKit's content blockers see no DNS answers. Only the CNAME-cloaked trackers that DuckDuckGo lists are blocked, including on first-party requests. WebKit's ITP handles some bounce tracking. | DNS access does not exist in WebKitGTK. AMP redirection could use DuckDuckGo's `ampLinks` configuration. |
| No uBlock Origin scriptlets | WebKit content blockers cannot express them (overview §9). | — |

## 8. Reproducing

The harness was kept outside the repository (in `/tmp`) for this measurement.

- **Logging proxy.** A ~70-line asyncio proxy that records host names. It relays bytes unchanged and never
  intercepts TLS.
- **Per-browser launch scripts.** Each one starts a browser with a fresh profile under
  `xvfb-run -a dbus-run-session`, idles for 5 minutes, then opens each site for 20 seconds. Chrome and Brave
  are driven through the DevTools endpoint; ZTBrowser is started with each address.
- **Analysis.** A script that classifies hosts with Disconnect's `services.json` and the Public Suffix List.
- **Fingerprint probe.** A page that reads the values in §4.3 and posts them to a local server.

Anyone repeating the measurement needs three things: a fresh profile per browser, the same time window for
all of them, and a tracker list that none of the browsers uses.
