# ZTBrowser 1.2.0 on QtWebEngine — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the WebKitGTK front end with a QtWebEngine (Chromium) front end that matches Brave's scrolling speed and keeps and extends ZTBrowser's protections.

**Architecture:** `ztbrowser-core` gains engine-neutral *ports* (request, cookie, script, storage, burn decisions). A new adapter crate `ztbrowser-engine-qt` (Rust + a small C++ shim) implements them on QtWebEngine; a new crate `ztbrowser-qt` is the QML application bound by `cxx-qt`. The GTK crate keeps building until the benchmark gate passes, then is removed.

**Tech Stack:** Rust 2024 (workspace rules: pedantic clippy, `unsafe_code` forbidden in *our* crates), Qt 6.12 / QtWebEngine (Chromium 140), QML, `cxx-qt` 0.10, `adblock` crate (already used), cargo-nextest.

**Spec:** `docs/en/specs/2026-11-04-qtwebengine-migration-design.md` (Polish mirror in `docs/pl/specs/`). Read §5–§8, §11a and §12 before any task.

## Global Constraints

- Branch `feature/1.2.0`; `main` and v1.1.2 untouched until the §8 gate passes.
- Rust, plus the smallest C++ shim (≈ 300–500 lines) for Qt classes without Rust bindings.
- `docs/en/engineering-rules.md`: object-oriented Rust (every function belongs to a type), scenario-struct Given/When/Then tests, one symbol per import line, imports from the defining module, no `mod.rs` re-exports, files ≤ 1000 lines, no `Helper/Util/Manager/…` names.
- `ztbrowser-core` has no Qt, GTK or engine types. New ports live there.
- QtWebEngine sandbox, site isolation and GPU process stay ON; `--no-sandbox` only in tests.
- DuckDuckGo data (Tracker Radar, lists, privacy configuration, reference tests) is CC BY-NC-SA 4.0: fetched at run time/CI time, never committed or packaged.
- Interface: same features, words and layout as 1.1.2; may follow the system (KDE/Qt) theme. All UI text comes from the existing 66 `tr!` catalogues through a `cxx-qt` object, never Qt `.ts/.qm`.
- Every §6 extra is required in 1.2.0: Strict profile, Tor per-site isolation, per-profile ECH/DoH, uniform fingerprint profile, WebRTC limits, leak tests, reference tests.
- Performance gate (§8): scripted scroll on bpxglobal.com at 2560×1440 and 3840×2160 ≥ 58 fps, p95 frame gap ≤ 17 ms, no worse than Brave − 2 fps; memory within 20 % of Brave.
- Tor window: own process, `--proxy-server=socks5://…`, `--host-resolver-rules="MAP * ~NOTFOUND, EXCLUDE <proxy host>"`, WebRTC off; never loads before Tor is connected, never falls back to direct.
- Minimum engine version: constant in core; warn in normal windows, refuse to start the Tor window.
- Zero known vulnerabilities (`cargo audit`); commit messages and docs in English; Polish mirrors for every doc.

## Review Focus

- A request whose URL has no host, uses `ws://`/`wss://`, `data:`, `blob:` or an IDN host: policy must decide without panicking (pass for non-network schemes).
- A redirect loop (HTTPS upgrade answered by an `http://` redirect): no per-request budget exists (Qt gives no request id, and downgrading a refused redirect to a pass would weaken protection); policies are idempotent, Chromium's own 20-redirect limit ends a loop, and `HttpsUpgradePolicy::record_failed_upgrade` stops retrying that site.
- Cookie filter or interceptor called with an empty or host-less first-party URL (workers, `data:` pages, sandboxed frames): sub-resources and third-party cookies are treated as third-party (fail closed); `blob:` pages count as their creating site; only main-frame navigations are first-party by definition.
- The interceptor is called on the IO thread under load: the decision path allocates nothing per request after warm-up and never blocks on a lock held by the UI thread.
- A profile directory removed while the browser runs (burn data): the next page load recreates storage instead of crashing.
- Tor window started while Tor is not connected: shows the connection page, loads nothing.

## File Structure

- `crates/ztbrowser-core/src/network/` (new): `request_policy.rs`, `request_decision.rs`, `network_request.rs`, `third_party_policy.rs`.
- `crates/ztbrowser-core/src/storage/` (new): `cookie_policy.rs`, `cookie_decision.rs`, `data_burner.rs`.
- `crates/ztbrowser-core/src/engine/` (new): `engine_version.rs` (minimum-version rule), `page_script.rs`, `page_script_catalog.rs`.
- `crates/ztbrowser-core/src/privacy/` (new): `fingerprint_profile.rs`, `strict_profile.rs`, `resolver_settings.rs`, `tor_circuit_isolation.rs`.
- `crates/ztbrowser-core/tests/*.rs`: one scenario file per new type.
- `crates/ztbrowser-engine-qt/` (new): `build.rs`, `src/*.rs` (adapters), `cpp/*.cpp|*.h` (the shim: interceptor, cookie filter, scheme handlers).
- `crates/ztbrowser-qt/` (new): `build.rs`, `src/*.rs` (cxx-qt objects), `qml/*.qml` (UI).
- `crates/ztbrowser-bench/` (new): benchmark harness, replaces `/tmp/spike`.
- `packaging/` (modified): Qt dependencies, AppImage bundling.

---

### Task 1: Benchmark harness in the repository

**Files:**
- Create: `crates/ztbrowser-bench/{Cargo.toml,src/main.rs,src/scroll_scenario.rs,src/frame_statistics.rs,src/benchmark_server.rs}`, `crates/ztbrowser-bench/pages/scroll.html`
- Test: `crates/ztbrowser-bench/src/frame_statistics.rs` (unit tests in-file are allowed for a dev tool; core rules apply to core only)
- Modify: `Cargo.toml` (workspace members)

**Interfaces:**
- Produces: `FrameStatistics::from_gaps_ms(&[f64]) -> FrameStatistics` with `frames_per_second() -> f64`, `percentile_gap_ms(percent: u32) -> f64`, `frames_longer_than_ms(f64) -> usize`. `ScrollScenario::new(url, viewport: (u32,u32), duration_s)`; CLI `ztbrowser-bench --engine <chromium|qt|webkit> --url … --size 2560x1440`.

- [x] **Step 1: Write failing tests** `should_report_60_fps_for_16_7_ms_gaps`, `should_count_frames_longer_than_33_ms`, `should_return_p95_of_sorted_gaps` using the Chromium numbers of spec §1 as fixtures.
- [x] **Step 2: Run** `cargo test -p ztbrowser-bench` — Expected: FAIL (types missing).
- [x] **Step 3: Implement** `FrameStatistics` (sorted gaps, nearest-rank percentile).
- [x] **Step 4: Run** `cargo test -p ztbrowser-bench` — Expected: PASS.
- [x] **Step 5: Implement** the page `scroll.html` (rAF loop recording frame gaps while `scrollBy` steps run for `duration_s`, posts the array to the local server) and `BenchmarkServer` (std `TcpListener`, serves the page, stores the POSTed array). Port the measurement method of the throwaway `/tmp/spike` harness (8 s scripted scroll).
- [x] **Step 6: Commit** `git commit -m "Add the in-repository scroll benchmark harness"`.

### Task 2: Spike — QML WebEngineView with hooks from Rust (throwaway, performance gate)

**Files:** Create `spike/qt-webengine/` (excluded from the workspace, deleted at the end of Task 3). 

**Interfaces:** Produces the *decision* recorded in `docs/en/specs/…` §5.3/§11a: QML+`cxx-qt`, Widgets, or CEF.

- [x] **Step 1:** Minimal `cxx-qt` project: a Rust `QObject` exposed to QML; C++ `QWebEngineUrlRequestInterceptor` subclass calling a Rust function over `cxx`; `QWebEngineCookieStore::setCookieFilter`; one `QWebEngineScript` injected from Rust; `WebEngineView` in QML.
- [x] **Step 2: Verify the hooks** — a local page loads three subresources (own, tracker-listed, third-party); assert via logs that the interceptor saw all three, blocked the tracker, the cookie filter saw the third-party cookie, and the script ran in a subframe.
- [x] **Step 3: Performance gate** — run Task 1's benchmark on the spike at 2560×1440. Expected: ≥ 58 fps, p95 ≤ 17 ms.
- [x] **Step 4: Record the decision** in spec §5.3/§11a (both languages). If the gate fails, stop and escalate to the owner (CEF fallback per §11a); if only `cxx-qt`→QML hooks fail, switch to Widgets for Task 6.
- [x] **Step 5: Commit** the decision note only (the spike directory is not committed).

**Done 2026-11-04:** QML + `cxx-qt` confirmed, gate passed (58.0–58.6 fps vs Brave 59.9 at 2560×1440); see spec §5.3.

### Task 3: Core ports — network request policy

**Files:** Create `network/{network_request,request_decision,request_policy,redirect_budget,third_party_policy}.rs`; tests `network_request.rs`, `request_policy.rs`, `third_party_policy.rs`; modify `lib.rs` (`pub mod network;`).

**Interfaces:**
- Consumes: `TrackerDataSet`, `PrivacyConfiguration`, `TrackingParameterRemover`, `UnprotectedSites`, `RegistrableDomainLookup`, Guard rule lists via the `adblock` engine.
- Produces:
  - `NetworkRequest::new(url: &str, first_party: Option<&str>, resource_type: ResourceType, request_id: u64) -> NetworkRequest`
  - `enum RequestDecision { Pass, Block, Redirect(String) }`
  - `trait RequestPolicy: Send + Sync { fn decide(&self, request: &NetworkRequest) -> RequestDecision; }`
  - `DuckDuckGoRequestPolicy` implementing it (tracker blocking, Guard network rules, HTTPS upgrade, link-parameter cleaning, GPC header names via `fn extra_headers(&self, &NetworkRequest) -> &[(&'static str, &'static str)]`)
  - `ThirdPartyPolicy::new(strict: bool, allow_list: &SiteAllowList)`, `decide(&NetworkRequest) -> RequestDecision` (default-deny when strict).

- [x] **Step 1: Failing tests** (scenario structs): `should_block_known_tracker_request`, `should_redirect_http_to_https_when_upgrade_list_matches`, `should_pass_when_site_is_unprotected`, `should_block_cname_cloaked_tracker_on_the_same_site`, `should_block_sub_resource_of_unknown_page_in_strict_mode`, `should_protect_navigation_from_unprotected_page_to_other_site`, `should_pass_non_network_schemes`, `should_block_third_party_in_strict_mode`, `should_pass_third_party_on_site_allow_list`, `should_treat_missing_first_party_as_first_party`.
- [x] **Step 2: Run** `cargo test -p ztbrowser-core --test request_policy` — Expected: FAIL.
- [x] **Step 3: Implement** the types; decision order: unprotected site → pass; non-network scheme → pass; strict third-party; tracker/Guard block; HTTPS upgrade; parameter cleaning. Allocation-free fast path (borrowed `&str`, pre-built lookup tables).
- [x] **Step 4: Run** same command — Expected: PASS; also `cargo clippy -p ztbrowser-core --all-targets`.
- [x] **Step 5: Commit.**

### Task 4: Core ports — cookies, burn, page scripts, engine version

**Files:** Create `storage/{cookie_policy,cookie_decision,data_burner}.rs`, `engine/{engine_version,page_script,page_script_catalog}.rs`; tests of the same names.

**Interfaces:**
- Produces: `CookieAccess::new(cookie_domain, first_party: Option<&str>, is_third_party: bool)`; `trait CookiePolicy { fn allows(&self, &CookieAccess) -> bool }` (`StrictCookiePolicy`, `DuckDuckGoCookiePolicy`); `trait DataBurner { fn burn(&self, scope: BurnScope) -> BurnPlan }` where `BurnPlan` lists cookie domains, cache and storage directories to delete; `PageScript { name, source, injection_point, all_frames, world }` and `PageScriptCatalog::for_profile(&ProfileSettings) -> Vec<PageScript>` (Content Scope Scripts, Guard hiding, smooth wheel, DDG overlays, fingerprint profile); `EngineVersion::parse(&str)`, `MinimumEngineVersion::verdict(&EngineVersion, WindowKind) -> EngineVerdict { Run, Warn, Refuse }`.

- [x] **Step 1: Failing tests:** `should_deny_third_party_cookie_in_strict_mode`, `should_allow_first_party_cookie`, `should_plan_deletion_of_storage_directories_on_burn`, `should_warn_below_minimum_engine_version_in_normal_window`, `should_refuse_below_minimum_engine_version_in_tor_window`, `should_list_smooth_wheel_script_only_when_enabled`, `should_inject_fingerprint_script_into_all_frames`.
- [x] **Step 2–4:** run (FAIL) → implement → run (PASS).
- [x] **Step 5: Commit.**

### Task 5: `ztbrowser-engine-qt` — adapter and C++ shim

**Files:** Create `crates/ztbrowser-engine-qt/{Cargo.toml,build.rs,src/lib.rs,src/request_interceptor.rs,src/cookie_filter.rs,src/script_installer.rs,src/profile_factory.rs,src/engine_version_probe.rs,cpp/shim.h,cpp/shim.cpp}`; tests in `tests/`.

**Interfaces:**
- Consumes: Task 3/4 ports.
- Produces: `ProfileFactory::persistent(&ProfileDirectories) -> QtProfile`, `QtProfile::install(policy: Arc<dyn RequestPolicy>, cookies: Arc<dyn CookiePolicy>, scripts: &[PageScript])`; `EngineVersionProbe::current() -> EngineVersion`. Shim exposes a C-compatible `extern "C"` callback table (`decide_request`, `allow_cookie`) — the only boundary; Rust side contains no `unsafe` in our crates (the `cxx` bridge generates it; the crate opts out of the `forbid` with a documented crate-level `deny` for the generated code only if `cxx-qt` requires it — record in the spec).

- [x] **Step 1: Failing integration test** `should_block_tracker_and_report_decision_counts` using the Task 2 page: engine loads it headless (`QT_QPA_PLATFORM=offscreen`), interceptor blocks the tracker URL and the test reads counters.
- [x] **Step 2:** `cargo test -p ztbrowser-engine-qt` — Expected FAIL.
- [x] **Step 3: Implement** the shim (interceptor and cookie filter subclasses) and Rust adapters. Decision path allocation-free after warm-up.
- [x] **Step 4:** run — Expected PASS; run Task 1 benchmark again with the interceptor active (must still pass the gate).
- [x] **Step 5: Commit.**

### Task 6: `ztbrowser-qt` — application shell

**Files:** Create `crates/ztbrowser-qt/{Cargo.toml,build.rs,src/main.rs,src/browser_model.rs,src/tab_model.rs,src/address_bar_model.rs,src/interface_messages.rs}`, `qml/{BrowserWindow,TabStrip,Toolbar,AddressBar,MainMenu}.qml`.

**Interfaces:** Consumes: existing `ztbrowser_core::navigation`, `profile`, `language` types and the `tr!` catalogues. Produces: QML-visible `BrowserModel` (open/close tab, navigate, back/forward, reload), `InterfaceMessages.text(key: QString) -> QString`.

- [x] **Step 1: Failing test** `should_resolve_polish_message_through_the_bridge` (model-level, no window) and `should_open_and_close_tabs` (`TabModel`).
- [x] **Step 2–4:** run (FAIL) → implement → run (PASS).
- [x] **Step 5: Manual check:** `QT_QPA_PLATFORM=offscreen` smoke test that the window loads `https://example.com` and renders (screenshot via `grabToImage` saved by the test).
- [x] **Step 6: Commit.**

*Deviation (recorded at delivery):* tab state lives in QML (`TabBar` + one `WebEngineView` per tab) because the engine owns every tab's history and title; a Rust `TabModel` would only mirror it (YAGNI). Rust exposes `BrowserBridge` (address resolution, interface text, page scripts). The end-to-end test is `crates/ztbrowser-qt/tests/tracker_blocking.rs`, which starts the real binary headless.

### Task 7: Everyday features

Downloads (list, manager, clear — reuse `download_history`/`download_settings`), permissions prompt, zoom (reuse `page_zoom`, `site_zoom`), find in page, preferences (interface language page, smooth wheel, protection switches), per-site "protection off", error page. One sub-commit per feature, each with a failing core/model test first (`should_clear_finished_downloads`, `should_remember_site_zoom`, `should_ask_permission_once_per_origin`). Interfaces: engine-neutral models in `ztbrowser-qt/src/*_model.rs` over existing core types.

### Task 8: Protections end-to-end

Wire Task 3/4 ports into profiles (default and *Strict*), install Content Scope Scripts, Guard hiding, smooth wheel and DDG overlays (Duck Player, Click to Load, autoconsent) as `QWebEngineScript`s in their own world; Strict profile UI with *Allow here* toolbar button and `SiteAllowList` persisted via `StoredFile`-equivalent in core. Tests: `should_keep_overlay_script_in_isolated_world`, reference-test runner (`crates/ztbrowser-core/tests/privacy_reference.rs`, fetches suites at CI time, skipped with a clear message offline), `should_persist_site_allow_list`.

### Task 9: Fingerprint profile, WebRTC limits, resolver settings

`FingerprintProfile` (fixed screen/window buckets, language, cores, battery, plugins) injected via Content Scope Scripts configuration; WebRTC policy `disable_non_proxied_udp` through Chromium flag/profile setting; `ResolverSettings` (DoH template, ECH on/off) per profile through `--enable-features`/DoH switches and a preferences page. Tests: `should_report_uniform_screen_size_for_every_window_size`, `should_serialize_resolver_settings_per_profile`, leak check `should_not_expose_local_ip_via_webrtc` (spawned engine, page asks RTCPeerConnection for candidates).

### Task 10: Tor window and per-site isolation

Separate process `ztbrowser-qt --tor` launching Chromium with the flags in Global Constraints; reuse `tor_connection`, `tor_control_port`, `tor_settings` types from core/GTK crate moved into core (pure move). `TorCircuitIsolation::credentials_for(site) -> (String, String)` gives each registrable domain its own SOCKS username/password (stream isolation). Tests: `should_give_each_site_its_own_socks_credentials`, `should_not_load_page_before_tor_is_connected`, leak test `should_not_resolve_names_outside_proxy` (blocked DNS on a network namespace or `--host-resolver-rules` assertion using a canary hostname).

### Task 11: Web apps, profiles, burn data

Port `web_app_*` and profile launching; `BurnPlanner` plans and the adapter executes a `BurnPlan`: cookie store delete, then removal of the profile's whole `engine/` directory (the engine is pointed at `<data>/engine` by `ProfileFactory`, deny-by-default) and its cache while the profile is closed or via a fresh profile restart. Tests: `should_recreate_storage_after_burn`, `should_open_web_app_without_toolbar`.

### Task 12: Engine version check

At start, `EngineVersionProbe::current()` is checked against `MinimumEngineVersion`; normal window shows a notice, Tor window refuses. Test: `should_refuse_tor_window_start_on_old_engine`.

### Task 13: Packaging

**Status: done 2026-11-05 (version 1.2.2), corrected the same day.** The first 1.2.2 deb and AppImage were built on Debian 13 (Qt 6.8, Chromium 122), below the engine floor 140.0.7339.0, so every start showed the out-of-date-engine notice. Deviation from the task as written: there is no `.deb` (no stable Debian or Ubuntu ships Qt WebEngine 6.10 or newer; revisit when one does), and the AppImage is built in the Fedora 44 container (Chromium 140.0.7339.225, glibc 2.43 or newer). The AppImage also writes its own `.desktop` entry and icon into `~/.local/share` at start, because a Wayland desktop shows a generic icon for a window with no entry (`AppImageIntegration`). Original record: the packages carry the Qt edition. Qt 6.8 (Debian 13) crashes when `userScripts.collection` is bound while the view is created, so `BrowserTab.qml` installs the scripts afterwards and loads the first page only then. The Arch and Fedora builds need link-time optimisation off (`!lto`, `_lto_cflags`), because Rust's linker cannot read GCC's LTO objects of the C++ bridge. The AppImage is about 230 MB. Each package was installed in a clean container and started under Xvfb; it downloaded the protection data on first start. CI (`tests.yml`) runs all 860 tests, the Qt ones under Xvfb with Mesa.

Arch PKGBUILD, rpm spec, deb: depend on distribution `qt6-webengine`; AppImage bundles Qt + QtWebEngine (record size in the release notes). Update `packaging/release.sh`, desktop entry and AppStream metadata; CI `tests.yml` installs Qt dev packages. Verify: `packaging/release.sh arch deb appimage rpm` builds; the AppImage starts on a clean container.

### Task 14: Benchmarks, sign-off, GTK removal, release

Run Task 1 harness on Brave, QtWebEngine and ZTBrowser-Qt at 2560×1440 and 3840×2160; write `docs/{en,pl}/reports/performance-analysis.md` §5; check every row of spec §8. When all pass: delete `crates/ztbrowser`, move shared pieces, update `CLAUDE.md` index and architecture docs (EN + PL, new diagrams), bump to 1.2.0, merge to `main`, tag `v1.2.0`. If any §8 row fails, stop and report to the owner — do not delete the GTK crate.

---

## Self-review

- **Spec coverage:** §5.1 ports → Tasks 3–4; adapter/shim → 5; UI → 6–7; §6 table and extras → 8–10 (Strict, Tor per-site, ECH/DoH, fingerprint, WebRTC, leak, reference tests); §7 version → 4, 12; §8 → 1, 14; §10 steps map to Tasks 2, 6, 8, 7, 6/7 (i18n), 10–11, 13, 14; §11a decisions embedded in Tasks 2, 6, 10, 12.
- **Known simplification:** Tasks 7–14 are listed at feature level with their tests and interfaces; their exact steps are expanded when the owning task starts, because the Task 2 decision (QML vs Widgets vs CEF) can reshape Tasks 5–7. This is stated, not hidden.
- **Type consistency:** `NetworkRequest`, `RequestDecision`, `RequestPolicy`, `CookiePolicy`, `PageScript`, `EngineVerdict` are named once in Tasks 3–4 and reused unchanged.
