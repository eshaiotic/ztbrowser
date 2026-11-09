# ZTBrowser 1.2.0 na QtWebEngine — Plan implementacji

> **Dla agentów wykonawczych:** WYMAGANY PODSKILL: superpowers:subagent-driven-development (zalecany) lub superpowers:executing-plans do realizacji planu zadanie po zadaniu. Kroki używają składni checkboxów (`- [ ]`).

**Cel:** Zastąpić front-end WebKitGTK front-endem na QtWebEngine (Chromium), który dorównuje szybkości przewijania Brave oraz zachowuje i rozszerza ochrony ZTBrowsera.

**Architektura:** `ztbrowser-core` dostaje niezależne od silnika *porty* (decyzje o żądaniach, cookies, skryptach, pamięci, wypalaniu). Nowy crate-adapter `ztbrowser-engine-qt` (Rust + mały shim C++) implementuje je na QtWebEngine; nowy crate `ztbrowser-qt` to aplikacja QML powiązana przez `cxx-qt`. Crate GTK buduje się do czasu przejścia bramki benchmarkowej, potem jest usuwany.

**Stos:** Rust 2024 (reguły workspace: clippy pedantic, `unsafe_code` zabronione w *naszych* crate’ach), Qt 6.12 / QtWebEngine (Chromium 140), QML, `cxx-qt` 0.10, crate `adblock` (już używany), cargo-nextest.

**Specyfikacja:** `docs/en/specs/2026-11-04-qtwebengine-migration-design.md` (lustro PL w `docs/pl/specs/`). Przed każdym zadaniem przeczytaj §5–§8, §11a i §12.

## Ograniczenia globalne

- Gałąź `feature/1.2.0`; `main` i v1.1.2 nietknięte do przejścia bramki z §8.
- Rust plus najmniejszy możliwy shim C++ (≈ 300–500 linii) dla klas Qt bez wiązań Rusta.
- `docs/en/engineering-rules.md`: Rust obiektowy (każda funkcja należy do typu), testy scenariuszowe Given/When/Then, jeden symbol na linię importu, importy z modułu definiującego, bez re-eksportów w `mod.rs`, pliki ≤ 1000 linii, bez nazw `Helper/Util/Manager/…`.
- `ztbrowser-core` nie zawiera typów Qt, GTK ani silnika. Nowe porty żyją w nim.
- Sandbox QtWebEngine, izolacja witryn i proces GPU zostają WŁĄCZONE; `--no-sandbox` tylko w testach.
- Dane DuckDuckGo (Tracker Radar, listy, konfiguracja prywatności, testy referencyjne) są na CC BY-NC-SA 4.0: pobierane w czasie działania/CI, nigdy nie commitowane ani pakowane.
- Interfejs: te same funkcje, słowa i układ co w 1.1.2; może przejąć motyw systemu (KDE/Qt). Cały tekst UI pochodzi z istniejących 66 katalogów `tr!` przez obiekt `cxx-qt`, nigdy z Qt `.ts/.qm`.
- Każdy dodatek z §6 jest wymagany w 1.2.0: profil Ścisły, izolacja Tor per witryna, ECH/DoH per profil, jednolity odcisk, ograniczenia WebRTC, testy wycieków, testy referencyjne.
- Bramka wydajności (§8): przewijanie skryptem na bpxglobal.com w 2560×1440 i 3840×2160 ≥ 58 fps, p95 odstępu klatek ≤ 17 ms, nie gorzej niż Brave − 2 fps; pamięć w granicach 20 % Brave.
- Okno Tor: własny proces, `--proxy-server=socks5://…`, `--host-resolver-rules="MAP * ~NOTFOUND, EXCLUDE <host proxy>"`, WebRTC wyłączone; nigdy nie ładuje przed połączeniem z Tor i nigdy nie wraca do połączenia bezpośredniego.
- Minimalna wersja silnika: stała w rdzeniu; ostrzeżenie w zwykłych oknach, odmowa startu okna Tor.
- Zero znanych podatności (`cargo audit`); commity i dokumenty po angielsku; lustra PL dla każdego dokumentu.

## Fokus przeglądu

- Żądanie bez hosta, ze schematem `ws://`/`wss://`, `data:`, `blob:` lub z hostem IDN: polityka decyduje bez paniki (przepuszcza schematy nie-sieciowe).
- Pętla przekierowań (podniesienie do HTTPS odpowiedziane przekierowaniem na `http://`): nie ma budżetu na żądanie (Qt nie daje identyfikatora żądania, a zamiana odrzuconego przekierowania na przepuszczenie osłabiałaby ochronę); polityki są idempotentne, własny limit 20 przekierowań w Chromium kończy pętlę, a `HttpsUpgradePolicy::record_failed_upgrade` przestaje ponawiać dla tej witryny.
- Filtr cookies lub interceptor wywołany z pustym lub bezhostowym adresem pierwszej strony (workery, strony `data:`, ramki w piaskownicy): podzasoby i cookies stron trzecich traktowane jako strona trzecia (fail closed); strona `blob:` liczy się jako witryna, która ją utworzyła; tylko nawigacje głównej ramki są z definicji pierwszą stroną.
- Interceptor jest wołany w wątku IO pod obciążeniem: ścieżka decyzji nie alokuje nic na żądanie po rozgrzaniu i nigdy nie czeka na blokadę trzymaną przez wątek UI.
- Katalog profilu usunięty podczas działania przeglądarki (wypalanie): następne ładowanie odtwarza pamięć zamiast się wywrócić.
- Okno Tor uruchomione, gdy Tor nie jest połączony: pokazuje stronę połączenia i niczego nie ładuje.

## Struktura plików

- `crates/ztbrowser-core/src/network/` (nowy): `request_policy.rs`, `request_decision.rs`, `network_request.rs`, `third_party_policy.rs`.
- `crates/ztbrowser-core/src/storage/` (nowy): `cookie_policy.rs`, `cookie_decision.rs`, `data_burner.rs`.
- `crates/ztbrowser-core/src/engine/` (nowy): `engine_version.rs` (reguła minimalnej wersji), `page_script.rs`, `page_script_catalog.rs`.
- `crates/ztbrowser-core/src/privacy/` (nowy): `fingerprint_profile.rs`, `strict_profile.rs`, `resolver_settings.rs`, `tor_circuit_isolation.rs`.
- `crates/ztbrowser-core/tests/*.rs`: jeden plik scenariuszy na nowy typ.
- `crates/ztbrowser-engine-qt/` (nowy): `build.rs`, `src/*.rs` (adaptery), `cpp/*.cpp|*.h` (shim: interceptor, filtr cookies, obsługa schematów).
- `crates/ztbrowser-qt/` (nowy): `build.rs`, `src/*.rs` (obiekty cxx-qt), `qml/*.qml` (UI).
- `crates/ztbrowser-bench/` (nowy): narzędzie benchmarkowe, zastępuje `/tmp/spike`.
- `packaging/` (zmieniony): zależności Qt, pakowanie AppImage.

---

### Zadanie 1: Benchmark w repozytorium

**Pliki:** utwórz `crates/ztbrowser-bench/{Cargo.toml,src/main.rs,src/scroll_scenario.rs,src/frame_statistics.rs,src/benchmark_server.rs}`, `crates/ztbrowser-bench/pages/scroll.html`; zmodyfikuj `Cargo.toml` (członkowie workspace).

**Interfejsy:** produkuje `FrameStatistics::from_gaps_ms(&[f64])` z `frames_per_second()`, `percentile_gap_ms(percent: u32)`, `frames_longer_than_ms(f64)`; `ScrollScenario::new(url, viewport, duration_s)`; CLI `ztbrowser-bench --engine <chromium|qt|webkit> --url … --size 2560x1440`.

- [x] **Krok 1:** testy, które padają: `should_report_60_fps_for_16_7_ms_gaps`, `should_count_frames_longer_than_33_ms`, `should_return_p95_of_sorted_gaps` (dane z §1 specyfikacji).
- [x] **Krok 2:** `cargo test -p ztbrowser-bench` — oczekiwane: FAIL.
- [x] **Krok 3:** zaimplementuj `FrameStatistics` (posortowane odstępy, percentyl metodą najbliższej rangi).
- [x] **Krok 4:** `cargo test -p ztbrowser-bench` — oczekiwane: PASS.
- [x] **Krok 5:** strona `scroll.html` (pętla rAF zapisująca odstępy klatek podczas przewijania `scrollBy` przez `duration_s`, wysyła tablicę do lokalnego serwera) i `BenchmarkServer` (std `TcpListener`). Przenieś metodę z jednorazowego harnessu `/tmp/spike` (8 s przewijania skryptem).
- [x] **Krok 6:** commit „Add the in-repository scroll benchmark harness”.

### Zadanie 2: Spike — QML WebEngineView z hakami z Rusta (jednorazowy, bramka wydajności)

**Pliki:** utwórz `spike/qt-webengine/` (poza workspace, usunięty na końcu Zadania 3).

**Interfejsy:** produkuje *decyzję* zapisaną w specyfikacji §5.3/§11a: QML+`cxx-qt`, Widgets albo CEF.

- [x] **Krok 1:** minimalny projekt `cxx-qt`: obiekt Rusta widoczny w QML; podklasa C++ `QWebEngineUrlRequestInterceptor` wołająca funkcję Rusta przez `cxx`; `QWebEngineCookieStore::setCookieFilter`; jeden `QWebEngineScript` z Rusta; `WebEngineView` w QML.
- [x] **Krok 2: weryfikacja haków** — lokalna strona ładuje trzy podzasoby (własny, z listy trackerów, strony trzeciej); logi potwierdzają, że interceptor widział wszystkie trzy, zablokował tracker, filtr cookies widział cookie strony trzeciej, a skrypt zadziałał w podramce.
- [x] **Krok 3: bramka wydajności** — benchmark z Zadania 1 na spike’u w 2560×1440. Oczekiwane: ≥ 58 fps, p95 ≤ 17 ms.
- [x] **Krok 4: zapisz decyzję** w specyfikacji §5.3/§11a (oba języki). Jeśli bramka nie przejdzie, stop i eskalacja do właściciela (zapas CEF wg §11a); jeśli zawiodą tylko haki `cxx-qt`→QML, przejdź na Widgets w Zadaniu 6.
- [x] **Krok 5:** commit tylko notatki z decyzją (katalog spike’u nie jest commitowany).

**Zrobione 2026-11-04:** QML + `cxx-qt` potwierdzone, bramka przeszła (58,0–58,6 fps vs Brave 59,9 w 2560×1440); zob. specyfikacja §5.3.

### Zadanie 3: Porty rdzenia — polityka żądań sieciowych

**Pliki:** utwórz `network/{network_request,request_decision,request_policy,redirect_budget,third_party_policy}.rs`; testy `network_request.rs`, `request_policy.rs`, `third_party_policy.rs`; zmodyfikuj `lib.rs` (`pub mod network;`).

**Interfejsy:**
- Konsumuje: `TrackerDataSet`, `PrivacyConfiguration`, `TrackingParameterRemover`, `UnprotectedSites`, `RegistrableDomainLookup`, listy reguł Guard przez silnik `adblock`.
- Produkuje:
  - `NetworkRequest::new(url: &str, first_party: Option<&str>, resource_type: ResourceType, request_id: u64) -> NetworkRequest`
  - `enum RequestDecision { Pass, Block, Redirect(String) }`
  - `trait RequestPolicy: Send + Sync { fn decide(&self, request: &NetworkRequest) -> RequestDecision; }`
  - `DuckDuckGoRequestPolicy` implementujący go (blokowanie trackerów, reguły sieciowe Guard, podniesienie do HTTPS, czyszczenie parametrów, nagłówki GPC przez `fn extra_headers(&self, &NetworkRequest) -> &[(&'static str, &'static str)]`)
  - `ThirdPartyPolicy::new(strict: bool, allow_list: &SiteAllowList)`, `decide(&NetworkRequest) -> RequestDecision` (domyślna odmowa w trybie ścisłym).

- [x] **Krok 1:** testy scenariuszowe: `should_block_known_tracker_request`, `should_redirect_http_to_https_when_upgrade_list_matches`, `should_pass_when_site_is_unprotected`, `should_block_cname_cloaked_tracker_on_the_same_site`, `should_block_sub_resource_of_unknown_page_in_strict_mode`, `should_protect_navigation_from_unprotected_page_to_other_site`, `should_pass_non_network_schemes`, `should_block_third_party_in_strict_mode`, `should_pass_third_party_on_site_allow_list`, `should_treat_missing_first_party_as_first_party`.
- [x] **Krok 2:** `cargo test -p ztbrowser-core --test request_policy` — oczekiwane: FAIL.
- [x] **Krok 3:** implementacja; kolejność decyzji: strona bez ochrony → pass; schemat nie-sieciowy → pass; strona trzecia w trybie ścisłym; blokada trackera/Guard; podniesienie do HTTPS; czyszczenie parametrów. Szybka ścieżka bez alokacji (pożyczone `&str`, tablice wyszukiwania zbudowane wcześniej).
- [x] **Krok 4:** to samo polecenie — oczekiwane: PASS; oraz `cargo clippy -p ztbrowser-core --all-targets`.
- [x] **Krok 5:** commit.

### Zadanie 4: Porty rdzenia — cookies, wypalanie, skrypty strony, wersja silnika

**Pliki:** utwórz `storage/{cookie_policy,cookie_decision,data_burner}.rs`, `engine/{engine_version,page_script,page_script_catalog}.rs`; testy o tych samych nazwach.

**Interfejsy:** `CookieAccess::new(cookie_domain, first_party: Option<&str>, is_third_party: bool)`; `trait CookiePolicy { fn allows(&self, &CookieAccess) -> bool }` (`StrictCookiePolicy`, `DuckDuckGoCookiePolicy`); `trait DataBurner { fn burn(&self, scope: BurnScope) -> BurnPlan }`, gdzie `BurnPlan` wylicza domeny cookies, cache i katalogi pamięci do usunięcia; `PageScript { name, source, injection_point, all_frames, world }` i `PageScriptCatalog::for_profile(&ProfileSettings) -> Vec<PageScript>` (Content Scope Scripts, ukrywanie Guard, płynne kółko, nakładki DDG, profil odcisku); `EngineVersion::parse(&str)`, `MinimumEngineVersion::verdict(&EngineVersion, WindowKind) -> EngineVerdict { Run, Warn, Refuse }`.

- [x] **Krok 1:** testy: `should_deny_third_party_cookie_in_strict_mode`, `should_allow_first_party_cookie`, `should_plan_deletion_of_storage_directories_on_burn`, `should_warn_below_minimum_engine_version_in_normal_window`, `should_refuse_below_minimum_engine_version_in_tor_window`, `should_list_smooth_wheel_script_only_when_enabled`, `should_inject_fingerprint_script_into_all_frames`.
- [x] **Kroki 2–4:** uruchom (FAIL) → zaimplementuj → uruchom (PASS).
- [x] **Krok 5:** commit.

### Zadanie 5: `ztbrowser-engine-qt` — adapter i shim C++

**Pliki:** utwórz `crates/ztbrowser-engine-qt/{Cargo.toml,build.rs,src/lib.rs,src/request_interceptor.rs,src/cookie_filter.rs,src/script_installer.rs,src/profile_factory.rs,src/engine_version_probe.rs,cpp/shim.h,cpp/shim.cpp}`; testy w `tests/`.

**Interfejsy:**
- Konsumuje: porty z Zadań 3–4.
- Produkuje: `ProfileFactory::persistent(&ProfileDirectories) -> QtProfile`, `QtProfile::install(policy: Arc<dyn RequestPolicy>, cookies: Arc<dyn CookiePolicy>, scripts: &[PageScript])`; `EngineVersionProbe::current() -> EngineVersion`. Shim udostępnia zgodną z C tablicę callbacków (`decide_request`, `allow_cookie`) — to jedyna granica; strona Rusta nie zawiera `unsafe` w naszych crate’ach (kod generuje `cxx`; jeśli `cxx-qt` wymaga wyjątku od `forbid`, jest on udokumentowany w specyfikacji i ograniczony do kodu generowanego).

- [x] **Krok 1:** test integracyjny `should_block_tracker_and_report_decision_counts` na stronie z Zadania 2: silnik ładuje ją headless (`QT_QPA_PLATFORM=offscreen`), interceptor blokuje URL trackera, test odczytuje liczniki.
- [x] **Krok 2:** `cargo test -p ztbrowser-engine-qt` — oczekiwane FAIL.
- [x] **Krok 3:** implementacja shimu (podklasy interceptora i filtra cookies) i adapterów Rusta. Ścieżka decyzji bez alokacji po rozgrzaniu.
- [x] **Krok 4:** uruchom — oczekiwane PASS; ponownie benchmark z Zadania 1 z aktywnym interceptorem (nadal musi przejść bramkę).
- [x] **Krok 5:** commit.

### Zadanie 6: `ztbrowser-qt` — powłoka aplikacji

**Pliki:** utwórz `crates/ztbrowser-qt/{Cargo.toml,build.rs,src/main.rs,src/browser_model.rs,src/tab_model.rs,src/address_bar_model.rs,src/interface_messages.rs}`, `qml/{BrowserWindow,TabStrip,Toolbar,AddressBar,MainMenu}.qml`.

**Interfejsy:** konsumuje istniejące typy `ztbrowser_core::navigation`, `profile`, `language` i katalogi `tr!`. Produkuje widoczny w QML `BrowserModel` (otwórz/zamknij kartę, nawiguj, wstecz/naprzód, odśwież), `InterfaceMessages.text(key: QString) -> QString`.

- [x] **Krok 1:** testy, które padają: `should_resolve_polish_message_through_the_bridge` (na poziomie modelu, bez okna) i `should_open_and_close_tabs` (`TabModel`).
- [x] **Kroki 2–4:** uruchom (FAIL) → zaimplementuj → uruchom (PASS).
- [x] **Krok 5: kontrola ręczna:** test dymny `QT_QPA_PLATFORM=offscreen` — okno ładuje `https://example.com` i rysuje (zrzut przez `grabToImage` zapisany przez test).
- [x] **Krok 6:** commit.

*Odstępstwo (zapisane przy odbiorze):* stan kart żyje w QML (`TabBar` + jeden `WebEngineView` na kartę), bo historię i tytuł każdej karty ma silnik; `TabModel` w Ruście tylko by go powielał (YAGNI). Rust udostępnia `BrowserBridge` (rozwiązywanie adresu, teksty interfejsu, skrypty strony). Test end-to-end to `crates/ztbrowser-qt/tests/tracker_blocking.rs`, który uruchamia prawdziwy plik binarny bez okna.

### Zadanie 7: Funkcje codzienne

Pobieranie (lista, menedżer, czyszczenie — ponowne użycie `download_history`/`download_settings`), prośba o uprawnienia, zoom (`page_zoom`, `site_zoom`), szukanie na stronie, preferencje (strona języka interfejsu, płynne kółko, przełączniki ochrony), „ochrona wyłączona” per witryna, strona błędu. Jeden podkomit na funkcję, każdy z najpierw padającym testem rdzenia/modelu (`should_clear_finished_downloads`, `should_remember_site_zoom`, `should_ask_permission_once_per_origin`). Interfejsy: niezależne od silnika modele w `ztbrowser-qt/src/*_model.rs` nad istniejącymi typami rdzenia.

### Zadanie 8: Ochrony end-to-end

Podłącz porty z Zadań 3–4 do profili (domyślny i *Ścisły*), zainstaluj Content Scope Scripts, ukrywanie Guard, płynne kółko i nakładki DDG (Duck Player, Click to Load, autoconsent) jako `QWebEngineScript` we własnym świecie; interfejs profilu Ścisłego z przyciskiem *Zezwól tutaj* i `SiteAllowList` zapisywaną trwale w rdzeniu. Testy: `should_keep_overlay_script_in_isolated_world`, runner testów referencyjnych (`crates/ztbrowser-core/tests/privacy_reference.rs`, pobiera zestawy w CI, pomijany z czytelnym komunikatem offline), `should_persist_site_allow_list`.

### Zadanie 9: Profil odcisku, ograniczenia WebRTC, ustawienia resolvera

`FingerprintProfile` (stałe przedziały ekranu/okna, język, rdzenie, bateria, wtyczki) wstrzykiwany przez konfigurację Content Scope Scripts; polityka WebRTC `disable_non_proxied_udp` przez flagę Chromium/ustawienie profilu; `ResolverSettings` (szablon DoH, ECH włączone/wyłączone) per profil przez przełączniki `--enable-features`/DoH i stronę preferencji. Testy: `should_report_uniform_screen_size_for_every_window_size`, `should_serialize_resolver_settings_per_profile`, test wycieku `should_not_expose_local_ip_via_webrtc` (uruchomiony silnik, strona prosi `RTCPeerConnection` o kandydatów).

### Zadanie 10: Okno Tor i izolacja per witryna

Osobny proces `ztbrowser-qt --tor` uruchamiający Chromium z flagami z ograniczeń globalnych; ponowne użycie typów `tor_connection`, `tor_control_port`, `tor_settings` przeniesionych do rdzenia (czyste przeniesienie). `TorCircuitIsolation::credentials_for(site) -> (String, String)` daje każdej domenie rejestrowalnej własną nazwę użytkownika/hasło SOCKS (izolacja strumieni). Testy: `should_give_each_site_its_own_socks_credentials`, `should_not_load_page_before_tor_is_connected`, test wycieku `should_not_resolve_names_outside_proxy` (zablokowany DNS w przestrzeni nazw sieci lub asercja `--host-resolver-rules` z nazwą-kanarkiem).

### Zadanie 11: Aplikacje webowe, profile, wypalanie danych

Przenieś `web_app_*` i uruchamianie profili; `BurnPlanner` planuje, a adapter wykonuje `BurnPlan`: usunięcie cookies, potem usunięcie całego katalogu `engine/` profilu (silnik jest kierowany do `<data>/engine` przez `ProfileFactory`, deny-by-default) i jego cache przy zamkniętym profilu lub przez restart na świeżym profilu. Testy: `should_recreate_storage_after_burn`, `should_open_web_app_without_toolbar`.

### Zadanie 12: Kontrola wersji silnika

Przy starcie `EngineVersionProbe::current()` jest porównywane z `MinimumEngineVersion`; zwykłe okno pokazuje komunikat, okno Tor odmawia startu. Test: `should_refuse_tor_window_start_on_old_engine`.

### Zadanie 13: Pakowanie

**Status: wykonane 2026-11-05 (wersja 1.2.2), poprawione tego samego dnia.** Pierwsze deb i AppImage 1.2.2 zbudowano na Debianie 13 (Qt 6.8, Chromium 122), poniżej progu silnika 140.0.7339.0, więc przy każdym starcie pojawiał się komunikat o przestarzałym silniku. Odstępstwo od zadania w pierwotnym brzmieniu: nie ma `.deb` (żaden stabilny Debian ani Ubuntu nie ma Qt WebEngine 6.10 lub nowszego; do ponownego rozważenia, gdy się pojawi), a AppImage powstaje w kontenerze Fedory 44 (Chromium 140.0.7339.225, glibc 2.43 lub nowsze). AppImage zapisuje też przy starcie własny wpis `.desktop` i ikonę w `~/.local/share`, bo pulpit z Waylandem pokazuje ogólną ikonę okna bez takiego wpisu (`AppImageIntegration`). Pierwotny zapis: paczki zawierają edycję Qt. Qt 6.8 (Debian 13) wywala się, gdy `userScripts.collection` jest związane przy tworzeniu widoku, więc `BrowserTab.qml` instaluje skrypty po utworzeniu i dopiero wtedy ładuje pierwszą stronę. Budowy dla Arch i Fedory wymagają wyłączenia optymalizacji w czasie łączenia (`!lto`, `_lto_cflags`), bo linker Rusta nie czyta obiektów LTO GCC z części C++. AppImage ma około 230 MB. Każdą paczkę zainstalowano w czystym kontenerze i uruchomiono pod Xvfb; przy pierwszym starcie pobrała dane ochrony. CI (`tests.yml`) uruchamia wszystkie 860 testów, te Qt pod Xvfb z Mesą.

PKGBUILD dla Arch, spec rpm, deb: zależność od `qt6-webengine` dystrybucji; AppImage dołącza Qt + QtWebEngine (rozmiar zapisany w notatkach wydania). Zaktualizuj `packaging/release.sh`, plik desktop i metadane AppStream; CI `tests.yml` instaluje pakiety deweloperskie Qt. Weryfikacja: `packaging/release.sh arch deb appimage rpm` buduje; AppImage uruchamia się w czystym kontenerze.

### Zadanie 14: Benchmarki, odbiór, usunięcie GTK, wydanie

Uruchom harness z Zadania 1 na Brave, QtWebEngine i ZTBrowser-Qt w 2560×1440 i 3840×2160; opisz w `docs/{en,pl}/reports/performance-analysis.md` §5; sprawdź każdy wiersz §8 specyfikacji. Gdy wszystko przejdzie: usuń `crates/ztbrowser`, przenieś wspólne elementy, zaktualizuj indeks `CLAUDE.md` i dokumenty architektury (EN + PL, nowe diagramy), podnieś wersję do 1.2.0, scal do `main`, otaguj `v1.2.0`. Jeśli jakikolwiek wiersz §8 nie przejdzie, stop i raport do właściciela — nie usuwaj crate GTK.

---

## Samokontrola

- **Pokrycie specyfikacji:** porty §5.1 → Zadania 3–4; adapter/shim → 5; UI → 6–7; tabela §6 i dodatki → 8–10 (Ścisły, Tor per witryna, ECH/DoH, odcisk, WebRTC, wycieki, testy referencyjne); wersja §7 → 4, 12; §8 → 1, 14; kroki §10 odpowiadają Zadaniom 2, 6, 8, 7, 6/7 (i18n), 10–11, 13, 14; decyzje §11a zawarte w Zadaniach 2, 6, 10, 12.
- **Świadome uproszczenie:** Zadania 7–14 są wypisane na poziomie funkcji z testami i interfejsami; dokładne kroki rozwijam, gdy zadanie startuje, bo decyzja Zadania 2 (QML vs Widgets vs CEF) może zmienić Zadania 5–7. Jest to powiedziane wprost.
- **Spójność typów:** `NetworkRequest`, `RequestDecision`, `RequestPolicy`, `CookiePolicy`, `PageScript`, `EngineVerdict` nazwane raz w Zadaniach 3–4 i używane bez zmian.
