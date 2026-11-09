# Ochrona przed trackerami w edycji Qt

Jak edycja QtWebEngine (Chromium) decyduje, co strona może wczytać, dlaczego sam filtr żądań nie wystarcza i jak dane
ochrony trafiają na komputer. Zmierzony wynik (2026-11-05, 12 serwisów, sędzią lista Disconnect): **5 przepuszczonych
trackerów, wobec 6 w edycji WebKitGTK, 8 w Brave i 109 w Chrome**, zarówno ze świeżym profilem, jak i z danymi ochrony
już na dysku.

## 1. Problem

Pierwsza wersja Qt przepuszczała na 12 serwisach 90 trackerów ze świeżym profilem i 31 z danymi ochrony na dysku, przy
6 w edycji GTK. Za tymi liczbami stały trzy osobne przyczyny i każda wymagała własnej naprawy:

1. **Naiwna lista trackerów.** Pierwsza polityka Qt decydowała wyłącznie na podstawie domyślnej akcji trackera.
   Własne reguły DuckDuckGo są subtelniejsze: mają wyjątki dla pojedynczych trackerów i listę dozwolonych serwisów,
   żeby strony działały. Pominięcie ich zepsuło allegro.pl i forbes.com, które nie wczytywały niczego.
2. **Filtr żądań nie widzi połączeń.** Chromium pozwala stronie otworzyć połączenie sieciowe bez wysyłania żądania:
   podpowiedzi `<link rel="preconnect">` i `dns-prefetch` czyta parser, zanim ruszy jakikolwiek skrypt, a połączenie
   powstaje od razu. Adres odwiedzającego i nazwa hosta opuszczają komputer w uścisku dłoni TLS, choć każde żądanie do
   tego hosta zostanie potem odrzucone. Na bbc.com tą drogą kontaktowało się dziewięć serwerów reklamowych.
3. **Brak pobierania danych.** Edycja Qt czytała dane ochrony, które zostawiła w pamięci podręcznej edycja GTK, więc
   świeży profil nie miał żadnych reguł i blokował prawie nic.

## 2. Słownik

| Termin | Znaczenie |
|---|---|
| **Tracker Data Set (TDS)** | Lista firm śledzących i ich domen od DuckDuckGo; każda pozycja ma domyślną akcję (`block` albo `ignore`), kategorie i reguły dla poszczególnych adresów. |
| **Konfiguracja prywatności** | Ustawienia funkcji od DuckDuckGo, w tym *lista dozwolonych trackerów* i domeny, na których ochrona jest tymczasowo wyłączona, bo je psuła. |
| **Reguła blokowania treści** | Jeden wpis formatu reguł WebKit (`trigger` + `action`). ZTBrowser buduje z TDS około 7000 takich reguł i w Qt sam je ocenia. |
| **Przechwytywacz żądań** | Haczyk, który Chromium wywołuje przed wysłaniem żądania; może przepuścić, zablokować lub przekierować. Widzi wyłącznie żądania. |
| **Preconnect / dns-prefetch** | Podpowiedzi strony, które każą przeglądarce wcześnie rozwiązać nazwę lub otworzyć połączenie, zanim istnieje jakiekolwiek żądanie do tego hosta. |
| **Bramka połączeń** | Punkt decyzyjny ZTBrowsera dla takich połączeń: *czy silnik może połączyć się z tym hostem?* |
| **Tunel CONNECT** | Sposób, w jaki proxy HTTP przenosi połączenie TLS: klient podaje `host:port`, proxy łączy się i przekazuje bajty. Proxy zna host, zanim zacznie się uścisk dłoni. |
| **Zamknięte domyślnie (fail closed)** | Gdy odpowiedź jest nieznana (polityki jeszcze nie zainstalowane), odmawiamy. |
| **ETag** | Etykieta wersji dokumentu nadawana przez serwer; odesłana z powrotem pozwala serwerowi odpowiedzieć „bez zmian” bez treści. |

## 3. Jak to się składa w całość

![Żądanie strony przechodzi przez przechwytywacz i łańcuch polityk; połączenie przechodzi przez proxy bramki; dane ochrony są pobierane, zapisywane i podmieniane](../../assets/diagrams/architecture-connection-gate.pl.svg)

Praca jest podzielona wzdłuż granicy crate'ów projektu (crate domenowy nic nie wie o Qt):

| Odpowiedzialność | Gdzie | Typ |
|---|---|---|
| Ocena jednego żądania | `ztbrowser-core` | `ContentBlockerRequestPolicy`, `AdvertisingTrackerRequestPolicy` |
| Pamięć o hostach, które polityki dopuściły | `ztbrowser-core` | `ConnectionGate`, `ConnectionClock` |
| Pobieranie i buforowanie danych DuckDuckGo | `ztbrowser-core` | `ProtectionFeed`, `CachedDocument`, trait `DocumentFetcher` |
| Usuwanie podpowiedzi zasobów tworzonych przez skrypty | `ztbrowser-core` + `assets/guard/no-resource-hints.js` | `ResourceHintBlockingScript` |
| Wymienialne polityki | `ztbrowser-core` | `ReplaceableRequestPolicy`, `ReplaceableCookiePolicy`, `SharedGuardEngine` |
| Haczyki Chromium: przechwytywacz i ciasteczka | `ztbrowser-engine-qt` | `EnginePolicies`, `PolicyDecisions` |
| Lokalne proxy egzekwujące bramkę | `ztbrowser-engine-qt` | `GateProxy`, `UpstreamProxy` |
| Pobieranie przez sieć Qt | `ztbrowser-engine-qt` | `QtDocumentFetcher`, `fetch_bytes` (C++) |
| Składanie łańcucha dla profilu i utrzymanie go w aktualności | `ztbrowser-qt` | `ProfilePolicies`, `ProtectionUpdates` |

## 4. Ocena żądania

`ProfilePolicies` buduje jeden łańcuch dla profilu, w tej kolejności, i opakowuje go tak, że serwis z wyłączoną tarczą
(albo taki, który DuckDuckGo uznaje za tymczasowo niechroniony) przechodzi bez zmian:

1. sprawdzenie „osoba trzecia” — żądanie z własnego serwisu strony nigdy nie jest żądaniem trackera;
2. **`ContentBlockerRequestPolicy`** — reguły DuckDuckGo, oceniane wiernie (§4.1);
3. **`AdvertisingTrackerRequestPolicy`** — ostrzejsza nakładka ZTBrowsera (§4.2);
4. czyszczenie linków (`utm_*`, `fbclid`, … usuwane z nawigacji);
5. listy filtrów Guard (EasyList, EasyPrivacy, uBlock Origin, Peter Lowe, listy polskie i ukrywające komunikaty o ciasteczkach);
6. upgrade do HTTPS.

### 4.1 Reguły DuckDuckGo, ocenianie tak jak robi to DuckDuckGo

Edycja GTK oddaje reguły WebKitowi, który ocenia je natywnie. Chromium nie ma takiego silnika, więc edycja Qt ocenia
te same reguły sama. `ContentBlockerRulesBuilder` (port `TrackerRadarKit` od DuckDuckGo, wspólny dla obu edycji) tworzy
reguły, a `ContentBlockerRequestPolicy` stosuje semantykę WebKit:

- reguły są czytane **po kolei**; reguła `block` oznacza żądanie jako zablokowane, reguła `ignore-previous-rules`
  zdejmuje to oznaczenie, a ostatnie słowo wygrywa — w ten sposób działa wyjątek trackera dla konkretnego adresu;
- wyzwalacz dopasowuje filtr adresu (wyrażenie regularne bez rozróżniania wielkości liter, kompilowane przy pierwszym
  użyciu), typy zasobów, rodzaj ładowania (własne lub osoby trzeciej) oraz listy `if-domain` / `unless-domain`, gdzie
  `*d` oznacza domenę wraz z poddomenami, a zwykła nazwa — tylko ten host;
- reguły z `load-context` nigdy tu nie działają, nawigacje zawsze przechodzą, a reguła z błędnym wyrażeniem jest
  pomijana, zamiast psuć pozostałe;
- lista dozwolonych trackerów, domeny tymczasowo niechronione i ukrywanie CNAME (nazwa własnego serwisu wskazująca
  na tracker) są częścią reguł, więc allegro.pl i forbes.com ładują się tak jak w przeglądarkach DuckDuckGo.

Ocenianie około 7000 reguł dla każdego żądania byłoby za wolne, więc `ContentBlockerRuleIndex` indeksuje je według
domeny trackera na początku filtra adresu. Żądanie sprawdza w tablicy mieszającej przyrostki nazwy swojego hosta
i uruchamia tylko te nieliczne reguły, które mogą pasować, oraz garstkę bez prefiksu domeny. Pomiar w wersji release:
**około 7,5 µs na decyzję**, 55 ms na zbudowanie reguł i 92 ms na zbudowanie silnika Guard, jednorazowo przy starcie
(lub aktualizacji danych).

### 4.2 Ostrzejsza nakładka

Wyjątki DuckDuckGo istnieją, bo zablokowanie niektórych adresów trackerów kiedyś coś psuło: odtwarzacz wideo czekający
na skrypt reklamowy, kontrolę antybotową. To słuszny kompromis dla narzędzia, które nie może zepsuć żadnej strony.
Celem ZTBrowsera jest prywatność, więc po regułach DuckDuckGo `AdvertisingTrackerRequestPolicy` blokuje każde żądanie
osoby trzeciej do trackera, którego domyślną akcją jest `block`, a kategorie obejmują reklamę lub pomiar
(*Advertising*, *Ad Motivated Tracking*, *Ad Fraud*, *Analytics*, *Audience Measurement*, *Third-Party Analytics
Marketing*, *Action Pixels*, *Session Replay*), niezależnie od strony. Trackery, które są także czymś, czego strona
potrzebuje (*Online Payment*, *Federated Login*, *SSO*, *Support Chat Widget*, *Content Delivery*, *Embedded Content*),
zostają przy regułach DuckDuckGo. Żądania do innych domen tej samej firmy nigdy nie są blokowane. Osoba, której strona
nie działa, wyłącza tarczę dla tego serwisu, a ta nakładka, jak każda polityka, respektuje ten przełącznik.

To stawia edycję Qt przed edycją GTK (5 wobec 6): pozostałe hosty Qt to te, które DuckDuckGo celowo dopuszcza, bo ich
zablokowanie psuje logowanie i płatności — okno konta Google (`accounts.google.com`, `www.google.com`) oraz
`m.stripe.network`.

## 5. Ocena połączenia: bramka

Dolna połowa diagramu pokazuje tę ścieżkę. Przechwytywacz nie widzi preconnectu, więc edycja Qt uruchamia **małe proxy
wewnątrz procesu przeglądarki** jeszcze przed startem Chromium i każe Chromium z niego korzystać
(`--proxy-server=http://127.0.0.1:<port>`, dopisane do `QTWEBENGINE_CHROMIUM_FLAGS` w C++, bo crate'y Rust zabraniają
`unsafe`, którego wymaga `set_var`). Każde połączenie, które otwiera Chromium, żądane lub nie, musi przejść przez
`GateProxy`:

- dla `CONNECT host:port` (cały HTTPS) i dla zwykłego żądania `http://` proxy najpierw pyta
  `ConnectionGate::admits(host)`;
- **wpuszczony** → otwiera połączenie (bezpośrednio albo przez proxy wskazane w `https_proxy` / `http_proxy` /
  `no_proxy`; rozumiane są tylko zwykłe proxy HTTP bez danych logowania) i przekazuje bajty;
- **odrzucony** → odpowiada `403` i niczego nie otwiera, więc nie następuje nawet zapytanie DNS o ten host.

`ConnectionGate` wpuszcza host w dwóch przypadkach. Albo łańcuch polityk żądań **by go nie zablokował** (bramka pyta
łańcuch o sztuczne żądanie osoby trzeciej `https://host/` ze strony zastępczej), albo żądanie do tego hosta
**zostało dopuszczone w ciągu ostatnich 60 sekund**: przechwytywacz wywołuje `approve(host)` przy każdym `Pass`. Druga
zasada zachowuje działanie wyjątków DuckDuckGo dla poszczególnych serwisów i przełącznika tarczy per serwis, bo oba
działają na żądaniach, a połączenie musi podążać za żądaniem, które dopuściły. Zwykły przypadek kosztuje jedno
wyszukanie w tablicy mieszającej; polityki uruchamiają się tylko dla hosta bez świeżego zatwierdzenia. Zatwierdzenia są
czyszczone, gdy jest ich 2048. Dopóki polityki nie są zainstalowane, bramka **jest zamknięta**: nic się nie łączy,
zanim istnieje ochrona.

### 5.1 Podpowiedzi dodawane przez skrypty

`ResourceHintBlockingScript` (`assets/guard/no-resource-hints.js`) działa w świecie samej strony, we wszystkich
ramkach, przy tworzeniu dokumentu, i usuwa `preconnect`, `dns-prefetch`, `prefetch` i `prerender` z elementów `<link>`,
które skrypty tworzą lub zmieniają: przez `rel`, `setAttribute`, `relList`, `innerHTML` i pokrewne oraz wstawianie
węzłów. `preload` zostawia w spokoju, bo to żądanie, które polityki mogą ocenić. Skrypt oszczędza odrzuconą próbę;
egzekwuje bramka, a podpowiedzi zapisane w HTML, które parser czyta przed każdym skryptem, zatrzymuje wyłącznie ona.

## 6. Pobieranie danych: strumień ochrony

| Krok | Co się dzieje |
|---|---|
| Dokumenty | TDS DuckDuckGo (v6, edycja macOS), konfiguracja prywatności DuckDuckGo (v4) i dziewięć list filtrów Guard. Żaden nie jest dołączany do programu (TDS ma licencję CC BY-NC-SA 4.0). |
| Pobieranie | `QtDocumentFetcher` implementuje `DocumentFetcher` z rdzenia na QtNetwork: nowy menedżer dla każdego pobrania, bez ciasteczek, systemowe proxy, przekierowania tylko na równie bezpieczne adresy, limit transferu 20 sekund i limit rozmiaru. |
| Pamięć podręczna | `CachedDocument` wysyła zapisany ETag, przyjmuje `304 Not Modified`, sprawdza treść przed zachowaniem i zapisuje do pliku `.partial`, który zastępuje starą kopię jednym przemianowaniem — nieudane pobranie nigdy nie uszkodzi dobrej kopii. |
| Odświeżanie | `ProtectionFeed::refresh` pobiera dokumenty równolegle; błędy są zbierane i zgłaszane, pozostałe dokumenty i tak się aktualizują. |
| Pierwszy start | Nic nie jest w pamięci podręcznej, więc `ProtectionUpdates::starting_protection` pobiera dane **przed otwarciem okna**: strona nigdy nie może wczytać się bez ochrony dlatego, że reguły jeszcze nadchodziły. |
| Kolejne starty | Kopie z pamięci podręcznej są czytane raz i używane od razu; `ProtectionUpdates::keep_current` sprawdza w tle (od razu, potem co 6 godzin). |
| Kolejność startu | `BrowserApplication` składa reguły (a przy pierwszym starcie także je pobiera) w wątku roboczym, podczas gdy wątek główny otwiera profil Chromium (`QtWebEngine::open_profile`); przed wczytaniem okna wątek roboczy jest dołączany. Silnik Guard jest budowany z list filtrów dopiero po tym pobraniu (`ProtectionUpdates::starting_protection`), więc przy pierwszym starcie powstaje z właśnie pobranych list, a nie z pustej pamięci podręcznej. Dopóki nie wykona się `QtWebEngine::install_policies`, Chromium odrzuca każde żądanie i każde ciasteczko, więc nakładanie się tych kroków nie pozwoli żadnemu żądaniu ominąć ochrony. |
| Zastosowanie | `ProfilePolicies::use_documents` przebudowuje reguły i silnik Guard i podmienia je przez `ReplaceableRequestPolicy`, `ReplaceableCookiePolicy` i `SharedGuardEngine` (wskaźnik za blokadą trzymaną tylko na czas skopiowania wskaźnika), bez restartu. |

Ruch bezczynnej przeglądarki zatem się zmienił: oprócz strony startowej DuckDuckGo i jej hosta pingu przeglądarka
kontaktuje się z sześcioma hostami serwującymi dane (13 połączeń z 8 hostami w 5-minutowym oknie bezczynności, z ciepłą
pamięcią podręczną i bez niej). To zwykłe `GET` bez ciasteczek i bez identyfikatora.

## 7. Dlaczego tak i co odrzucono

| Alternatywa | Dlaczego nie |
|---|---|
| Blokowanie wyłącznie po domenie | Psuje serwisy, które wyjątki DuckDuckGo utrzymują przy życiu (allegro.pl, forbes.com). |
| Oddanie reguł Chromium | QtWebEngine daje aplikacji osadzającej przechwytywacz żądań i nic deklaratywnego, więc reguły trzeba oceniać w ZTBrowserze. |
| Łatanie Chromium, by bramkować preconnecty | Oznacza utrzymywanie forka Chromium; proxy go nie potrzebuje. |
| `--host-resolver-rules` lub flagi funkcji wyłączające preconnect | Zmierzone: brak efektu za przechwytywaczem i brak efektu po ustawieniu proxy. |
| Sam skrypt strony | Nie widzi podpowiedzi w HTML, które parser czyta jako pierwsze. |
| Zapora lub filtr DNS w całym systemie | Wymaga uprawnień i obejmuje inne programy; to ma działać u zwykłego użytkownika. |

## 8. Znane ograniczenia

- Proxy widzi docelowy host każdego połączenia (nigdy treści połączenia TLS). Nie zapisuje go w dzienniku.
- Respektowane są tylko zwykłe proxy HTTP bez danych logowania; inne wartości są ignorowane, a nie stosowane połowicznie.
- Bramka ocenia host jako osobę trzecią na stronie zastępczej, więc reguła zależna od prawdziwej strony nie może dotyczyć
  połączenia bez świeżego zatwierdzenia; taki host jest oceniany wyłącznie według reguł domen.
- Pozostałe 5 hostów jest celowe (§4.2). Zablokowanie okna logowania Google jest możliwe, ale psuje logowanie przez
  Google w innych serwisach; płynniejsza obsługa to przyszła decyzja.
- W edycji Qt wciąż brakuje szumu odcisku przeglądarki, GPC i okna Tor ([plan migracji](../plans/2026-11-04-qtwebengine-migration-plan.md)).

## 9. Okna cookies: odrzucane, nie tylko ukrywane

**Problem.** Serwis, który potrzebuje zgody na zapis zbędnych ciasteczek, pokazuje okno. Ukrycie okna listą filtrów
(Guard też to robi) zostawia wybór niedokonany: serwis może uznać milczenie za zgodę albo strona zostaje nieużyteczna
za niewidocznym nakryciem. Osoba używająca przeglądarki chce, żeby wybór brzmiał *odmowa*, składana od razu i bez klikania.

**Słownik.** *Platforma zarządzania zgodami* (CMP) to produkt trzeciej strony, który rysuje okno i zapamiętuje odpowiedź
(Didomi, Cookiebot, OneTrust, Termly i setki innych; Facebook ma własną). *Autoconsent* to otwartoźródłowa biblioteka
DuckDuckGo (MPL-2.0), która rozpoznaje takie okna według reguł i naciska ich przycisk *odrzuć*; gdy żadna reguła nie
pasuje, heurystyka szuka okna, którego przyciski mówią „odrzuć wszystkie", i naciska go.

**Jak to się łączy.**

1. `assets/autoconsent/autoconsent.js` (biblioteka w wersji 18.0.0 plus `ztbrowser-entry.ts`, zob. README obok) jest
   wstrzykiwany przez `CookieNoticeScript` do własnego świata strony w każdej ramce przy tworzeniu dokumentu. Na razie
   tylko nasłuchuje. Działa w świecie strony, bo niektóre reguły wywołują interfejs JavaScript platformy zgód (na przykład
   `Didomi.setUserDisagreeToAll()`); wszystkie nazwy trzyma w jednym domknięciu i nic nie dopisuje do `window`.
2. Skrypt Guard (`assets/guard/page-hiding.js`, odizolowany świat z kanałem sieciowym) pyta Rusta
   `rejects_cookie_notices(adres)`, podając adres **strony najwyższego poziomu** (`location.ancestorOrigins`), bo ramka
   platformy zgód należy do innego serwisu niż ten, który ogląda osoba.
3. Rust odpowiada „tak" dokładnie wtedy, gdy tarcza jest włączona dla tego serwisu (`GuardPageHiding::rejects_cookie_notices`).
   Odpowiedź nie zależy od tego, czy listy filtrów zostały już zbudowane.
4. Przy „tak" skrypt Guard wysyła na dokumencie zdarzenie `ztbrowser-reject-cookie-notices`. Zdarzenie to jedyna rzecz,
   którą odizolowany świat i świat strony dzielą. Biblioteka startuje wtedy raz na dokument: najpierw reguły, potem
   heurystyka, **zawsze wyłącznie odrzucając** (`autoAction: 'optOut'`, tryb heurystyki `reject`).

Wyłączenie tarczy dla serwisu (przycisk tarczy przeładowuje kartę) zostawia więc jego okna w spokoju, tak jak każdą inną ochronę.

**Odrzucone alternatywy.** *Uruchomienie biblioteki w świecie odizolowanym* uniemożliwiłoby reguły potrzebujące
interfejsu JavaScript strony bez drugiego pomocnika w świecie strony. *Przekazanie listy niechronionych serwisów do
skryptu* skopiowałoby reguły dopasowania tarczy do JavaScriptu i się starzało po przełączeniu serwisu. *Zdarzenie z
ładunkiem* nie działa: `detail` zdarzenia `CustomEvent` nie przechodzi między światami, więc zdarzenie nie niesie danych,
a o jego wysłaniu decyduje tak/nie z Rusta. Strona może sama wysłać zdarzenie, co tylko uruchamia odrzucanie na stronie,
która i tak miała pokazać okno zgody.

**Znane ograniczenia.** Reguły są biblioteki; okna, którego nie zna ani nie zaklasyfikuje heurystyka, zostaje widoczne
(listy Guard mogą je jeszcze ukryć). Pakiet ma około 650 KB na ramkę chronionego serwisu; jego reguły pozostają
nieparsowanym tekstem do chwili zdarzenia. Odrzucenie wymagające własnej rundy do serwera strony zależy od serwisu.

## 10. Testy

| Przedmiot | Plik testu (`crates/…/tests/`) |
|---|---|
| Semantyka reguł DuckDuckGo | `ztbrowser-core/content_blocker_request_policy.rs` (16) |
| Ostrzejsza nakładka | `ztbrowser-core/advertising_tracker_request_policy.rs` (10) |
| Bramka połączeń | `ztbrowser-core/connection_gate.rs` (10) |
| Wymienialne polityki | `ztbrowser-core/replaceable_policies.rs` (4) |
| Skrypt podpowiedzi zasobów | `ztbrowser-core/resource_hint_blocking_script.rs` (5) |
| Strumień ochrony i pamięć podręczna | `ztbrowser-core/protection_feed.rs` (9) |
| Proxy (serwery na pętli zwrotnej) | `ztbrowser-engine-qt/gate_proxy.rs` (8), `upstream_proxy.rs` (7) |
| Przechwytywacz zatwierdza dopuszczone hosty | `ztbrowser-engine-qt/policy_decisions.rs` |
| Cała przeglądarka bez okna: obraz trackera zablokowany, zwykły wczytany | `ztbrowser-qt/tracker_blocking.rs` |
| Skrypt okien cookies (gwarancje pakietu) | `ztbrowser-core/cookie_notice_script.rs` (5), `guard_page_script.rs` |
| Komu każe się odrzucać | `ztbrowser-qt/guard_page_hiding.rs` |
| Cała przeglądarka: okno odrzucone, nigdy zaakceptowane; zostawione, gdy tarcza wyłączona | `ztbrowser-qt/tracker_blocking.rs` |

Test bez okna uruchamia prawdziwą przeglądarkę z lokalnym serwerem jako jej nadrzędnym proxy HTTP, który odpowiada za każdą
nazwę `.test`, więc strona, zwykła osoba trzecia i tracker z listy przechodzą przez bramkę.
