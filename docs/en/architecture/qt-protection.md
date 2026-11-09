# Tracker protection in the Qt edition

How the QtWebEngine (Chromium) edition decides what a page may load, why a request filter alone is not enough, and
how the protection data gets onto the machine. Measured result (2026-11-05, 12 sites, Disconnect's list as the
judge): **5 trackers let through, against 6 for the WebKitGTK edition, 8 for Brave and 109 for Chrome**, with a fresh
profile and with the protection data already on disk.

## 1. The problem

The first Qt build let 90 trackers through on 12 sites with a fresh profile and 31 with the protection data on disk,
against 6 for the GTK edition. Three separate causes sat behind those numbers, and each needed its own fix:

1. **A naive tracker list.** The first Qt policy decided from a tracker's default action alone. DuckDuckGo's own rules
   are subtler: they carry per-tracker exceptions and a site allowlist so that pages keep working. Ignoring them
   broke allegro.pl and forbes.com, which loaded nothing.
2. **A request filter cannot see connections.** Chromium lets a page open a network connection without making a
   request: `<link rel="preconnect">` and `dns-prefetch` hints are read by the parser before any script runs, and the
   connection is made at once. The visitor's address and the host's name leave the machine in the TLS handshake
   although every request to that host is later refused. On bbc.com nine advertising hosts were reached this way.
3. **No downloader.** The Qt edition read the protection data that the GTK edition had left in the cache, so a fresh
   profile had no rules at all and blocked almost nothing.

## 2. Vocabulary

| Term | Meaning |
|---|---|
| **Tracker Data Set (TDS)** | DuckDuckGo's list of tracking companies and their domains, each with a default action (`block` or `ignore`), categories and per-URL rules. |
| **Privacy configuration** | DuckDuckGo's per-feature settings, including the *tracker allowlist* and the domains where protection is temporarily off because it broke them. |
| **Content-blocker rule** | One entry of WebKit's rule format (`trigger` + `action`). ZTBrowser builds about 7,000 from the TDS and evaluates them itself in Qt. |
| **Request interceptor** | The hook Chromium calls before it sends a request; it can pass, block or redirect. It sees requests only. |
| **Preconnect / dns-prefetch** | Page hints that make the browser resolve a name or open a connection early, before any request to that host exists. |
| **Connection gate** | ZTBrowser's decision point for such connections: *may the engine connect to this host?* |
| **CONNECT tunnel** | How an HTTP proxy carries a TLS connection: the client names `host:port`, the proxy connects and relays bytes. The proxy learns the host before the handshake starts. |
| **Fail closed** | When the answer is unknown (policies not installed yet), refuse. |
| **ETag** | A server's label for a version of a document; sending it back lets the server answer "unchanged" without the body. |

## 3. How it fits together

![A page request goes through the interceptor and the policy chain; a connection goes through the gate proxy; protection data is fetched, cached and swapped in](../../assets/diagrams/architecture-connection-gate.en.svg)

The work is split along the crate boundary of the project (the domain crate knows nothing of Qt):

| Responsibility | Where | Type |
|---|---|---|
| Judge one request | `ztbrowser-core` | `ContentBlockerRequestPolicy`, `AdvertisingTrackerRequestPolicy` |
| Remember which hosts the policies allowed | `ztbrowser-core` | `ConnectionGate`, `ConnectionClock` |
| Fetch and cache DuckDuckGo's data | `ztbrowser-core` | `ProtectionFeed`, `CachedDocument`, trait `DocumentFetcher` |
| Remove resource hints created by scripts | `ztbrowser-core` + `assets/guard/no-resource-hints.js` | `ResourceHintBlockingScript` |
| Hold swappable policies | `ztbrowser-core` | `ReplaceableRequestPolicy`, `ReplaceableCookiePolicy`, `SharedGuardEngine` |
| Chromium's interceptor and cookie hooks | `ztbrowser-engine-qt` | `EnginePolicies`, `PolicyDecisions` |
| Local proxy that enforces the gate | `ztbrowser-engine-qt` | `GateProxy`, `UpstreamProxy` |
| Download over Qt's network stack | `ztbrowser-engine-qt` | `QtDocumentFetcher`, `fetch_bytes` (C++) |
| Assemble the chain per profile, keep it current | `ztbrowser-qt` | `ProfilePolicies`, `ProtectionUpdates` |

## 4. Judging a request

`ProfilePolicies` builds one chain per profile, in this order, and wraps it so that a site whose shield is off (or
that DuckDuckGo lists as temporarily unprotected) is passed untouched:

1. third-party check — a request from the page's own site is never a tracker request;
2. **`ContentBlockerRequestPolicy`** — DuckDuckGo's rules, evaluated faithfully (§4.1);
3. **`AdvertisingTrackerRequestPolicy`** — ZTBrowser's stricter overlay (§4.2);
4. link cleaning (`utm_*`, `fbclid`, … removed from navigations);
5. Guard's filter lists (EasyList, EasyPrivacy, uBlock Origin, Peter Lowe's, Polish and cookie-notice lists);
6. HTTPS upgrade.

### 4.1 DuckDuckGo's rules, as DuckDuckGo evaluates them

The GTK edition hands the rules to WebKit, which evaluates them natively. Chromium has no such engine, so the Qt
edition evaluates the same rules itself. `ContentBlockerRulesBuilder` (a port of DuckDuckGo's `TrackerRadarKit`,
shared by both editions) produces the rules; `ContentBlockerRequestPolicy` then applies WebKit's semantics:

- the rules are read **in order**; a `block` rule marks the request blocked, an `ignore-previous-rules` rule clears
  the mark, and the last word wins — this is how a tracker's per-URL exception works;
- a trigger matches on its URL filter (a case-insensitive regular expression, compiled the first time it is needed),
  its resource types, its load type (first or third party), and its `if-domain` / `unless-domain` lists, where `*d`
  means the domain and its subdomains and a plain name means that host only;
- rules with a `load-context` never apply here, navigations always pass, and a rule with an invalid expression is
  skipped rather than allowed to break the others;
- the tracker allowlist, the temporarily unprotected domains and CNAME cloaking (a first-party name that points at
  a tracker) are part of the rules, so allegro.pl and forbes.com load as in DuckDuckGo's own browsers.

Evaluating about 7,000 rules for every request would be too slow, so `ContentBlockerRuleIndex` indexes them by the
tracker domain at the start of their URL filter. A request looks up its host's suffixes in a hash map and runs only
the few rules that can match, plus the handful with no domain prefix. Measured in a release build: **about 7.5 µs
per decision**, 55 ms to build the rules and 92 ms to build Guard's engine, both once per start (or per data update).

### 4.2 The stricter overlay

DuckDuckGo's exceptions exist because blocking some tracker URLs once broke something: a video player waiting for an
advertising script, a bot check. That is the right trade for a tool that must never break a page. ZTBrowser's purpose
is privacy, so after DuckDuckGo's rules `AdvertisingTrackerRequestPolicy` blocks every third-party request to a
tracker whose default action is `block` and whose categories include advertising or measurement (*Advertising*,
*Ad Motivated Tracking*, *Ad Fraud*, *Analytics*, *Audience Measurement*, *Third-Party Analytics Marketing*,
*Action Pixels*, *Session Replay*), whatever the page. Trackers that are also something a page needs (*Online Payment*,
*Federated Login*, *SSO*, *Support Chat Widget*, *Content Delivery*, *Embedded Content*) stay with DuckDuckGo's
rules. Requests to a company's own other domains are never blocked. A person who finds a page broken switches the
shield off for that site, and this overlay, like every other policy, respects that switch.

This is what puts the Qt edition ahead of the GTK edition (5 against 6): the remaining Qt hosts are the ones
DuckDuckGo deliberately allows because blocking them breaks sign-in and payments — Google's account prompt
(`accounts.google.com`, `www.google.com`) and `m.stripe.network`.

## 5. Judging a connection: the gate

The lower half of the diagram above shows this path. The interceptor cannot see a preconnect, so the Qt edition
starts a small **proxy inside the browser process** before Chromium starts and tells Chromium to use it
(`--proxy-server=http://127.0.0.1:<port>`, appended to `QTWEBENGINE_CHROMIUM_FLAGS` in C++ because the Rust crates
forbid `unsafe`, which `set_var` needs). Every connection Chromium opens, requested or not, must pass through
`GateProxy`:

- for `CONNECT host:port` (all HTTPS) and for a plain `http://` request, the proxy first asks
  `ConnectionGate::admits(host)`;
- **admitted** → it opens the connection (directly, or through the proxy named by `https_proxy` / `http_proxy` /
  `no_proxy`; only plain HTTP proxies without credentials are understood) and relays bytes;
- **refused** → it answers `403` and opens nothing, so not even a DNS lookup for that host happens.

`ConnectionGate` admits a host in two cases. Either the request policy chain **would not block** it (the gate asks the
chain about a synthetic third-party request `https://host/` from a placeholder page), or a request to that host was
**allowed within the last 60 seconds**: the interceptor calls `approve(host)` on every `Pass`. The second rule keeps
DuckDuckGo's per-site exceptions and the per-site shield switch working, because both act on requests and a
connection must follow the request they allowed. The common case costs one hash-map lookup; the policies run only
for a host that has no recent approval. Approvals are swept when there are 2,048 of them. Until the policies are
installed the gate **fails closed**: nothing connects before protection exists.

### 5.1 Hints that scripts add

`ResourceHintBlockingScript` (`assets/guard/no-resource-hints.js`) runs in the page's own world, in every frame, at
document creation, and removes `preconnect`, `dns-prefetch`, `prefetch` and `prerender` from `<link>` elements that
scripts create or change: through `rel`, `setAttribute`, `relList`, `innerHTML` and its relatives, and node insertion.
It leaves `preload` alone, because a preload is a request the policies can judge. The script is a courtesy that saves
a refused attempt; the gate is the enforcement, and hints written in the HTML, which the parser reads before any
script, are only ever stopped by the gate.

## 6. Getting the data: the protection feed

| Step | What happens |
|---|---|
| Documents | DuckDuckGo's TDS (v6, macOS edition), DuckDuckGo's privacy configuration (v4) and Guard's nine filter lists. None is bundled (the TDS is licensed CC BY-NC-SA 4.0). |
| Download | `QtDocumentFetcher` implements the core's `DocumentFetcher` over QtNetwork: a new manager per download, no cookies, the system proxy, redirects only to equally safe addresses, a 20-second transfer limit and a size limit. |
| Cache | `CachedDocument` sends the stored ETag, accepts `304 Not Modified`, checks the body before keeping it, and writes to a `.partial` file that replaces the old copy with a single rename — a failed download never damages a good copy. |
| Refresh | `ProtectionFeed::refresh` fetches the documents concurrently; failures are collected and reported, the others still update. |
| First start | There is nothing cached, so `ProtectionUpdates::starting_protection` downloads **before the window opens**: a page must never load unprotected because the rules were still on their way. |
| Later starts | The cached copies are read once and used at once; `ProtectionUpdates::keep_current` checks in the background (now, then every 6 hours). |
| Start-up order | `BrowserApplication` puts the rules together (and, on the first start, downloads them) on a worker thread while the main thread opens Chromium's profile (`QtWebEngine::open_profile`), then joins the worker before the window loads. Guard's engine is built from the filter lists only after that download (`ProtectionUpdates::starting_protection`), so on the first start it is built from the lists just fetched and not from an empty cache. Until `QtWebEngine::install_policies` runs, Chromium refuses every request and cookie, so the overlap cannot let a request skip protection. |
| Applying | `ProfilePolicies::use_documents` rebuilds the rules and the Guard engine and swaps them in through `ReplaceableRequestPolicy`, `ReplaceableCookiePolicy` and `SharedGuardEngine` (a pointer behind a lock held only to copy the pointer), without a restart. |

Idle traffic therefore changed: besides the DuckDuckGo start page and its ping host, the browser contacts the six hosts
that serve the data (13 connections to 8 hosts in the 5-minute idle window, with and without a warm cache). They are
plain `GET`s with no cookie jar and no identifier.

## 7. Why this design, and what was rejected

| Alternative | Why not |
|---|---|
| Block by domain only | Breaks sites DuckDuckGo's exceptions keep working (allegro.pl, forbes.com). |
| Hand the rules to Chromium | QtWebEngine gives an embedding application the request interceptor and nothing declarative, so the rules must be evaluated in ZTBrowser. |
| Patch Chromium to gate preconnects | Means maintaining a Chromium fork; the proxy needs none. |
| `--host-resolver-rules` or feature flags to turn preconnect off | Measured: no effect behind the interceptor, and no effect once a proxy is set. |
| Only the page script | Cannot see hints in the HTML that the parser reads first. |
| System-wide firewall or DNS filter | Needs privileges and covers other programs; this must work for a normal user. |

## 8. Known limits

- The proxy sees the destination host of each connection (never the content of a TLS connection). It does not log it.
- Only plain HTTP upstream proxies without credentials are honoured; other values are ignored, not half-used.
- The gate judges a host as a third party on a placeholder page, so a rule that depends on the real page cannot apply to
  a connection that has no recent approval; such a host is judged by its domain rules only.
- The remaining 5 hosts are deliberate (§4.2). Blocking Google's sign-in prompt is possible but breaks sign-in with
  Google on other sites; a smoother handling is a future decision.
- Fingerprint noise, GPC and the Tor window are still missing in the Qt edition ([migration plan](../plans/2026-11-04-qtwebengine-migration-plan.md)).

## 9. Cookie notices: rejected, not only hidden

**Problem.** A site that needs the person's consent to store non-essential cookies shows a dialog. Hiding the dialog
with a filter list (Guard does that too) leaves the choice unmade: the site may treat silence as consent, or the page
may stay unusable behind an invisible overlay. The person asked for the choice to be *refusing*, made at once and
without clicks.

**Vocabulary.** A *consent management platform* (CMP) is the third-party product that draws the dialog and stores the
answer (Didomi, Cookiebot, OneTrust, Termly and hundreds more; Facebook has its own). *Autoconsent* is DuckDuckGo's
open-source library (MPL-2.0) that recognises those dialogs by rules and presses their *reject* button; where no rule
fits, a heuristic looks for a dialog whose buttons say "reject all" and presses that.

**How it fits together.**

1. `assets/autoconsent/autoconsent.js` (library version 18.0.0 plus `ztbrowser-entry.ts`, see the README beside it) is
   injected by `CookieNoticeScript` into the page's own world of every frame at document creation. It only listens.
   It runs in the page's world because some rules call the consent manager's JavaScript interface (for example
   `Didomi.setUserDisagreeToAll()`); it keeps every name in one closure and puts nothing on `window`.
2. Guard's script (`assets/guard/page-hiding.js`, an isolated world with the web channel) asks Rust
   `rejects_cookie_notices(address)`, naming the **top page's** address (`location.ancestorOrigins`), because a
   consent manager's frame belongs to another site than the one the person is looking at.
3. Rust answers yes exactly when the shield is on for that site (`GuardPageHiding::rejects_cookie_notices`). The answer
   does not depend on whether the filter lists have been built yet.
4. On yes, Guard's script dispatches the `ztbrowser-reject-cookie-notices` event on the document. An event is the one
   thing the isolated world and the page's world share. The library then starts, once per document: rules first,
   then the heuristic, **only ever rejecting** (`autoAction: 'optOut'`, heuristic mode `reject`).

Switching the shield off for a site (the shield button reloads the tab) therefore leaves its notices alone, as every
other protection is left alone.

**Alternatives rejected.** *Running the library in the isolated world* would make its rules that need the page's
JavaScript interface impossible without a second, page-world helper. *Passing the list of unprotected sites into the
script* would copy the shield's matching rules into JavaScript and go stale when the person toggles a site. *A page
event with a payload* does not work: a `CustomEvent`'s `detail` does not cross worlds, so the event carries no data and
Rust's yes/no decides whether it is sent at all. A page can dispatch the event itself, which only starts the rejection
on a page that was going to show a consent dialog.

**Known limits.** The rules are the library's; a dialog it does not know and the heuristic cannot classify stays
visible (Guard's lists may still hide it). The bundle is about 650 KB per frame of a protected site; its rules stay an
unparsed string until the event arrives. A rejection that needs a page's own server round trip depends on the site.

## 10. Tests

| Subject | Test file (`crates/…/tests/`) |
|---|---|
| DuckDuckGo's rule semantics | `ztbrowser-core/content_blocker_request_policy.rs` (16) |
| Stricter overlay | `ztbrowser-core/advertising_tracker_request_policy.rs` (10) |
| Connection gate | `ztbrowser-core/connection_gate.rs` (10) |
| Swappable policies | `ztbrowser-core/replaceable_policies.rs` (4) |
| Resource-hint script | `ztbrowser-core/resource_hint_blocking_script.rs` (5) |
| Protection feed and cache | `ztbrowser-core/protection_feed.rs` (9) |
| The proxy (loopback servers) | `ztbrowser-engine-qt/gate_proxy.rs` (8), `upstream_proxy.rs` (7) |
| Interceptor approves allowed hosts | `ztbrowser-engine-qt/policy_decisions.rs` |
| Whole browser, headless: tracker image blocked, ordinary one loaded | `ztbrowser-qt/tracker_blocking.rs` |
| Cookie-notice script (the bundle's guarantees) | `ztbrowser-core/cookie_notice_script.rs` (5), `guard_page_script.rs` |
| Who is asked to reject | `ztbrowser-qt/guard_page_hiding.rs` |
| Whole browser: notice rejected, never accepted; left alone when the shield is off | `ztbrowser-qt/tracker_blocking.rs` |

The headless test runs the real browser with a local server as its upstream HTTP proxy that answers for every `.test`
name, so the page, an ordinary third party and a listed tracker all travel the gate.
