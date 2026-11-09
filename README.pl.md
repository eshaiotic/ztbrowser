[English](README.md) | **Polski**

<p align="center">
  <img src="assets/icons/io.github.ztbrowser.ZTBrowser.svg" alt="ZTBrowser: fioletowa tarcza z białym łańcuchem" width="160">
</p>

# ZTBrowser

**Przeglądarka dla Linuksa, w której prywatność jest na pierwszym miejscu.** Odtwarza ochronę z otwartej
przeglądarki DuckDuckGo dla macOS na silniku **QtWebEngine** (Chromium, silnik Chrome i Brave), z interfejsem
**Qt 6 Quick** i rdzeniem domenowym napisanym w **Rust**. Bez telemetrii, bez rozszerzeń do pilnowania, każdy
profil i każda zainstalowana aplikacja webowa są od siebie odizolowane.

[![tests](https://github.com/eshaiotic/ztbrowser/actions/workflows/tests.yml/badge.svg)](https://github.com/eshaiotic/ztbrowser/actions/workflows/tests.yml)
[![release](https://img.shields.io/github/v/release/eshaiotic/ztbrowser?color=6A5ACD)](https://github.com/eshaiotic/ztbrowser/releases/latest)
![cargo nextest](https://img.shields.io/badge/cargo%20nextest-899%20test%C3%B3w-success)
![stack](https://img.shields.io/badge/stack-Rust%20%E2%80%A2%20Qt%206%20%E2%80%A2%20QtWebEngine-8E7FE8)
![telemetria](https://img.shields.io/badge/telemetria-brak-FFB300)
[![licencja](https://img.shields.io/badge/licencja-GPL--3.0--or--later-2E1A87)](LICENSE)

ZTBrowser to niezależny projekt. Nie jest powiązany z Duck Duck Go, Inc. ani przez nią wspierany.

---

## Zmierzone wobec Chrome i Brave, porównane z DuckDuckGo

2026-11-06 świeże profile Brave 1.98 (Chromium 156) i ZTBrowsera 1.2.3 działały jednocześnie, każdy we własnym oknie,
za proxy zapisującym wyłącznie nazwy hostów: 5 minut bezczynności, potem po 20 sekund na każdym z 12 popularnych
serwisów. Trackery ocenia neutralna lista (Disconnect), każdy serwer liczony raz na serwis. ZTBrowser uruchomiono dwa
razy: ze świeżym profilem (bez danych ochrony, więc pobiera je przy pierwszym uruchomieniu) i z danymi już na dysku.
Google Chrome 157 zmierzono tak samo 2026-11-05, w dwóch przebiegach, które pokazują, jak bardzo wynik waha się
między uruchomieniami.

| | Chrome 157 | Brave 1.98 | ZTBrowser, świeży profil | ZTBrowser, dane ochrony na dysku |
|---|---|---|---|---|
| Trackery przepuszczone na 12 serwisach | 111 / 109 | 8 | **1** | **1** |
| … w tym serwery reklamowe | 90 / 88 | 3 | **0** | **0** |
| Połączenia w pierwszych 5 minutach | 35 / 36 z 14 hostami | 46 z 12 hostami | 13 z 8 hostami | 13 z 8 hostami |

Dwie liczby w komórce Chrome to jego pierwszy i drugi przebieg.

**Jak to czytać.** ZTBrowser przepuszcza najmniej trackerów, zarówno na świeżym profilu, jak i z danymi ochrony już na
dysku: 1 na 12 serwisach, wobec 8 w Brave i 109 lub 111 w Chrome. Tym jednym jest `m.stripe.network` (płatności),
które reguły DuckDuckGo dopuszczają, bo jego zablokowanie psuje płacenie; z tego samego powodu dopuszczone jest okno
logowania Google (`accounts.google.com`, `www.google.com`), gdy serwis je pokazuje. Strony nadal się wczytują.
Osiąga to dzięki własnym regułom DuckDuckGo ocenianym tak, jak ocenia je DuckDuckGo, ostrzejszej warstwie dla
trackerów reklamowych i analitycznych, lokalnej bramce, która odrzuca połączenia otwierane przez stronę, zanim wyśle
jakiekolwiek żądanie (`preconnect`, którego filtry żądań nie widzą), oraz pobieraniu reguł przy pierwszym
uruchomieniu. Koszt pobierania widać w ruchu bezczynności: ZTBrowser łączył się ze stroną startową DuckDuckGo, jej
hostem pingu i sześcioma hostami serwującymi dane ochrony (13 połączeń z 8 hostami); Chrome z Google, a Brave z
własnymi serwerami (ping użycia, P3A). Odcisku przeglądarki nie zmierzono: ZTBrowser nie ma jeszcze ochrony przed
fingerprintingiem. Chrome 157 i Brave 1.98 to różne wersje Chromium, 157 i 156, a Chrome zmierzono 2026-11-05.
Jedno uruchomienie z jednego kraju w jednej sieci to nie ranking; metoda i ograniczenia są w
[porównaniu śledzenia](docs/pl/reports/tracking-comparison.md).

### Porównanie z przeglądarką DuckDuckGo

DuckDuckGo nie wydaje przeglądarki na Linuksa (tylko Windows, macOS, iOS i Androida), więc jej przeglądarki **nie
mierzono**. Tabela zestawia ZTBrowser z tym, co DuckDuckGo mówi o własnej przeglądarce, w liście funkcji ze
[strony porównania](https://duckduckgo.com/compare-privacy) („See how DuckDuckGo compares”) i na
[stronie pomocy o ochronie przed śledzeniem](https://duckduckgo.com/duckduckgo-help-pages/privacy/web-tracking-protections),
obie czytane w listopadzie 2026. Kolumna DuckDuckGo to jego własne twierdzenia, dla aplikacji na Windows i macOS
(gdzie się różnią, nazwano macOS); kolumna ZTBrowser to to, co ZTBrowser robi, sprawdzone w kodzie.
Niezależne liczby o przeglądarce DuckDuckGo są na [privacytests.org](https://privacytests.org/), które testuje
DuckDuckGo, Brave i Chrome na komputerach i nie testuje ZTBrowsera.

| | Przeglądarka DuckDuckGo mówi | ZTBrowser |
|---|---|---|
| **Skrypty śledzące** | | |
| Blokuje trackery stron trzecich, zanim się wczytają | tak, według swojej listy trackerów | tak, według tej samej listy i list filtrów Guarda, także w żądaniach do domeny samego serwisu; reguły są pobierane przy pierwszym uruchomieniu i odświeżane (okno raz na nie czeka) |
| Blokuje ciasteczka trackerów stron trzecich | tak | ciasteczka z listy trackerów są odrzucane; profil *Strict* (jeszcze bez interfejsu) odrzuca wszystkie |
| Wygasza ciasteczka własne | macOS: po 24 godzinach lub 7 dniach; Windows: w planach | nie |
| Chroni przed odciskiem przeglądarki | listowane skrypty blokowane, listowane API nadpisywane; obrona canvas i audio wyłączona na macOS | **jeszcze nie** |
| Chroni przed maskowaniem CNAME | podzbiór swojej listy na macOS | według nazw hostów zamaskowanych trackerów z listy |
| Chroni przed śledzeniem przez referer | referer skracany do nazwy hosta | tylko domyślne ustawienie Chromium |
| Blokuje śledzenie w linkach | tylko pierwsza nawigacja na macOS; Windows: w planach | każda nawigacja najwyższego poziomu |
| Blokuje śledzenie Google AMP | macOS: linki AMP zastępowane | nie |
| Blokuje wyskakujące okna logowania Google | tak | nie |
| Blokuje osadzone treści Facebooka | tak | Facebook jest na liście trackerów; brak zastępnika „kliknij, by wczytać” |
| **Reklamy i wyskakujące okna** | | |
| Blokuje okna zgody na cookies | zamyka okna z listy, wybierając opcję prywatną | odrzuca je od razu biblioteką autoconsent DuckDuckGo (także okno Facebooka), resztę ukrywa listą filtrów; zostawia w spokoju serwis z wyłączoną tarczą |
| Blokuje reklamy ukierunkowane i nieśledzące | mówi, że próbuje blokować wszystkie reklamy, z udokumentowanym wyjątkiem Microsoftu: `bat.bing.com` ładuje się na stronie reklamodawcy po kliknięciu reklamy w wyszukiwarce DuckDuckGo | listy filtrów Guarda (EasyList, uBlock Origin i inne), bez wyjątku dla kliknięć w reklamy |
| Odtwarza YouTube bez reklam | tak (Duck Player) | nie |
| **Kontrola** | | |
| Global Privacy Control | tak (macOS: nagłówek tylko do serwisów, o których wiadomo, że go respektują) | **jeszcze nie** |
| Usuwa dane przeglądania jednym przyciskiem | przycisk Fire | przycisk Wymaż dane przeglądania |
| Podnosi do HTTPS | serwisy z jego listy (około 90 % kliknięć w wyszukiwarce) | każda nawigacja, poza serwisami wyłączonymi w konfiguracji DuckDuckGo |
| Szyfruje zapytania DNS | tak | jeszcze nie (bezpieczny DNS Chromium nie jest skonfigurowany) |
| Przechowuje hasła lokalnie, synchronizuje prywatnie | tak | nie |
| Dane o użyciu wysyłane do twórcy | anonimowe metryki użycia | brak |
| **Poza przeglądarką** | | |
| Prywatne wyszukiwanie domyślnie | tak | DuckDuckGo jest domyślną wyszukiwarką |
| Ochrona przed oszustwami i phishingiem | Scam Blocker, lista Netcraft sprawdzana na urządzeniu | nie |
| Ochrona poczty | darmowe aliasy `@duck.com`, wymaga rejestracji | nie |
| VPN | w płatnej subskrypcji | jeszcze nie; okno Tor jest planowane |
| Usuwa dane osobowe z serwisów brokerów danych, pomaga odzyskać tożsamość | w płatnej subskrypcji (usługa tożsamości dla mieszkańców USA) | nie; zaplanowane, zob. *Następne wydanie* niżej |
| Działa na Linuksie | nie | tak |

W skrócie: przeglądarka DuckDuckGo wygrywa gotowymi ochronami, a ZTBrowser tym, czego nie zrobi (telemetria,
wyjątki dla kliknięć w reklamy, płatne poziomy) i działaniem na Linuksie; luka w gotowych ochronach to praca, która
stoi przed tym projektem, a nie coś, co powyższe liczby ukrywają.

---

## Przed czym chroni

ZTBrowser działa na **QtWebEngine** (Chromium), a interfejs jest w Qt Quick. Ostatnia kolumna mówi, czy dana
ochrona jest wdrożona.

| Ochrona | Jak działa | Stan |
|---|---|---|
| Trackery stron trzecich | Tracker Data Set DuckDuckGo rozstrzyga, żądanie po żądaniu, które żądania stron trzecich są odrzucane (port `TrackerRadarKit`). | działa; pobierane przy pierwszym uruchomieniu, odświeżane co 6 godzin |
| Psucie stron | Konfiguracja prywatności DuckDuckGo dostarcza wyjątki dla serwisów, żeby działały; przycisk tarczy (w aplikacji webowej: *Chroń tę witrynę* w menu) wyłącza ochronę dla jednego serwisu. | działa |
| Fingerprinting, GPC | Content Scope Scripts DuckDuckGo działają w każdej ramce; ZTBrowser Guard włącza obronę canvas, audio, ekranu i referera, którą DuckDuckGo zostawia wyłączoną, i wysyła nagłówek `Sec-GPC: 1`. | jeszcze nie |
| Reklamy, trackery, komunikaty o cookies | ZTBrowser Guard pobiera listy EasyList, EasyPrivacy, uBlock Origin, Petera Lowe'a, polskie i listy komunikatów o cookies, i stosuje je także do własnych zapytań serwisu. | działa; pobierane razem z resztą danych |
| Okna zgody na cookies | Biblioteka autoconsent DuckDuckGo rozpoznaje okna zgody setek dostawców (także Facebooka) i od razu naciska *odrzuć*; nigdy nie akceptuje. Pomijana dla serwisu z wyłączoną tarczą ([szczegóły](docs/pl/architecture/qt-protection.md)). | działa |
| Ciasteczka między serwisami | Ciasteczka stron trzecich z listy trackerów są odrzucane; profil *Strict*, który odrzuca wszystkie, istnieje w kodzie, ale nie ma jeszcze interfejsu. | działa |
| Linki śledzące | Nawigacja z `utm_*`, `fbclid`, `gclid`, ... jest zatrzymywana, zanim opuści komputer, i ładowana od nowa bez nich. | działa |
| Zwykłe HTTP | Nawigacje są podnoszone do HTTPS. | działa |
| Pozostawione dane | Przycisk Wymaż dane przeglądania jednym kliknięciem kasuje ciasteczka, magazyn, pamięć podręczną i favikony profilu. | działa |
| Mieszanie tożsamości | Każdy profil i każda zainstalowana aplikacja webowa działa jako osobny proces z własnymi ciasteczkami i danymi. | działa |
| Twój adres IP | Okno Tor wysyła wszystko przez własnego Tora ZTBrowsera; jego sesja żyje tylko w pamięci. | jeszcze nie |

Rozszerzeń nie ma — celowo.

**Aplikacje webowe.** *Zainstaluj witrynę jako aplikację* (przycisk na prawym końcu paska adresu albo menu okna) zamienia
dowolny serwis — bank, pocztę — w aplikację na pulpicie z ikoną serwisu. Każda aplikacja ma własną,
odizolowaną sesję; link do innego serwisu kliknięty w aplikacji otwiera się w przeglądarce, więc logowanie
aplikacji nigdy nie spotyka obcych stron. *Zainstalowane aplikacje…* w menu pokazuje je jako kafelki z przyciskami
Otwórz i Odinstaluj.

**Okno Tor (jeszcze niedostępne).** Jest planowane jako okno, którego każde żądanie idzie przez Tora, więc serwisy
widzą przekaźnik wyjściowy Tora zamiast Twojego adresu, przez własnego `tora` ZTBrowsera, a nie ogólnosystemową
usługę, bez powrotu do połączenia bezpośredniego i bez zapisu czegokolwiek na dysku. Ukryje Twój adres, ale nie to,
że używasz ZTBrowser: gdy liczy się nieodróżnialność wśród użytkowników Tora, używaj Tor Browsera.

**Preferencje** (`Ctrl+,`), osobno w każdym profilu: które przyciski pokazuje pasek narzędzi, czy pasek adresu
jest szeroki, czy krótki i wyśrodkowany, **gdzie są karty** (*Położenie kart*: góra, dół, lewo lub prawo; domyślnie
góra), **jak wyglądają przyciski okna** (*Przyciski okna*: domyślne okrągłe po prawej albo trzy kolorowe kropki po
lewej jak w macOS), płynne przewijanie kółkiem, folder pobierania i język interfejsu (68 języków).

**Domyślna przeglądarka.** Dopóki linki z innych programów (poczty, komunikatora, dokumentów) otwiera inna
przeglądarka, pierwsze okno raz pyta na pasku pod paskiem narzędzi, czy mają się otwierać w ZTBrowserze. Nic się nie
zmienia, dopóki nie wybierzesz *Ustaw jako domyślną*, a *Nie pytaj więcej* wyłącza pasek na dobre. *Preferencje →
Interfejs* pokazują, która przeglądarka otwiera linki, proponują tę samą zmianę i pozwalają włączyć pytanie przy
starcie z powrotem. Zmianę wykonuje systemowe `xdg-settings`, więc działa w KDE Plasma, GNOME, Xfce i innych
środowiskach; wersja uruchomiona ze źródeł nie ma wpisu w menu i nie może zostać wybrana.

---

## Jak wycinamy śledzenie i co to potwierdza

Strona, reklamodawca albo sam producent przeglądarki mogą Cię śledzić na siedem sposobów. Tabela nazywa każdy
z nich, mechanizm użyty przeciwko niemu przez ZTBrowser, miejsce tego mechanizmu w kodzie i test lub pomiar,
który go sprawdza. Decyzje mieszkają w niezależnej od silnika bibliotece domenowej; czego jeszcze brakuje,
wymienia [tabela ochron](#przed-czym-chroni) wyżej i
[plan migracji](docs/pl/plans/2026-11-04-qtwebengine-migration-plan.md).

| Sposób śledzenia | Co robi ZTBrowser | Kod (`crates/ztbrowser-core/src`) | Sprawdza to |
|---|---|---|---|
| Przeglądarka raportuje do producenta | W ogóle nie ma kodu telemetrii: żadnego pingu użycia, raportów awarii, ziarna eksperymentów ani konta. Jedyne połączenia, jakie program sam nawiązuje, to zwykłe `GET` po dane ochronne, bez ciasteczek i bez identyfikatora. | `protection/`, `guard/` (tylko pobieranie) | Ruch bezczynności zmierzony na poziomie nazw hostów: ZTBrowser łączy się z 8 hostami: stroną startową DuckDuckGo, jej hostem pingu i sześcioma serwującymi dane ochrony ([raport §4.5](docs/pl/reports/tracking-comparison.md)) |
| Trackery stron trzecich | Tracker Data Set DuckDuckGo rozstrzyga każde żądanie (przechwytywacz żądań w QtWebEngine oraz lokalna bramka połączeń odrzucająca podpowiedzi `preconnect`), razem z wyjątkami DuckDuckGo dla poszczególnych witryn, by strony działały. Ostrzejsza warstwa obejmuje trackery reklamowe i analityczne. | `protection/tracker_data_set.rs`, `protection/content_blocker_rules_builder.rs`, `network/content_blocker_request_policy.rs`, `network/advertising_tracker_request_policy.rs`, `network/connection_gate.rs` | Testy scenariuszowe `tracker_data_set`, `content_blocker_rules_builder`, `content_blocker_request_policy`, `advertising_tracker_request_policy`, `connection_gate`, `rule_exceptions`, `gate_proxy`, `tracker_blocking` ([architektura](docs/pl/architecture/qt-protection.md)) |
| Reklamy i trackery, których pierwsza lista nie zna | Guard pobiera dziewięć list społeczności (EasyList, EasyPrivacy, uBlock Origin, Petera Lowe'a, polskie i listy komunikatów o ciasteczkach), kompiluje je i stosuje także do żądań własnej witryny. Komunikaty o ciasteczkach są odrzucane (autoconsent), a gdy to się nie uda, ukrywane. | `guard/guard_engine.rs`, `guard/guard_rule_list.rs`, `network/guard_request_policy.rs` | Testy `guard_engine`, `guard_rule_list`, `guard_request_policy`, `cookie_notice_script`, `tracker_blocking`; trackery na 12 stronach zmierzone wyżej |
| Ciasteczka i zapisane identyfikatory | Ciasteczka stron trzecich z listy trackerów są odrzucane, każdy profil i aplikacja internetowa to osobny proces z własnym magazynem, a Wymaż dane przeglądania kasuje profil jednym kliknięciem. | `storage/cookie_policy.rs`, `storage/tracker_cookie_policy.rs`, `storage/burn_planner.rs` | Testy `cookie_policies`, `burn_planner` |
| Identyfikatory w linkach | `utm_*`, `fbclid`, `gclid` i inne parametry z listy DuckDuckGo są usuwane z każdej nawigacji najwyższego poziomu, zanim żądanie opuści komputer. | `protection/tracking_parameter_remover.rs`, `network/link_cleaning_request_policy.rs` | Testy `tracking_parameter_remover`, `link_cleaning_request_policy` |
| Odciski przeglądarki | Jeszcze niewdrożone. Biblioteka domenowa ma klocki (konfigurację Content Scope Scripts DuckDuckGo i zabezpieczenia Guarda: canvas, dźwięk, ekran, referer), ale przeglądarka ich jeszcze nie uruchamia, więc serwis odczytuje to, co wydaje Chromium. | `guard/fingerprint_hardening.rs`, `protection/content_scope_script.rs` | Testy `fingerprint_hardening`, `content_scope_script` |
| Co widzi sieć i serwer strony | Nawigacje są podnoszone do HTTPS. Nagłówek `Sec-GPC: 1` i okno Tor ukrywające adres IP nie są jeszcze wdrożone. | `navigation/https_upgrade_policy.rs`, `network/https_upgrade_request_policy.rs` | Testy `https_upgrade_policy`, `https_upgrade_request_policy` |

Każda lista i zestaw danych jest pobierany, nigdy dołączany, a przed użyciem ograniczany rozmiarem i
walidowany. Lista filtrów może tylko blokować albo ukrywać: konwersja nie zachowuje skryptletów, więc złośliwa
lista mogłaby zepsuć stronę, ale nigdy nie uruchomi w niej kodu. Cały zestaw (`./run-tests`, 899 testów razem z testami Qt, oraz
clippy i `cargo audit` w CI) uruchamia się przy każdym wypchnięciu.

### Dowody

Pomiar stojący za [pierwszą tabelą](#zmierzone-wobec-chrome-i-brave-porównane-z-duckduckgo) uruchomił 2026-11-06
obok siebie Brave 1.98 i ZTBrowser 1.2.3 (Chrome 157 — 2026-11-05), na świeżych profilach, każdy we własnym oknie,
przez proxy zapisujące wyłącznie nazwy hostów. Liczby, powtórzone tu obok mechanizmów:

- **Trackery osiągnięte na 12 stronach:** Chrome 111 i 109 w dwóch przebiegach, Brave 8, ZTBrowser 1 na świeżym
  profilu i 1 z danymi ochrony na dysku, według listy Disconnect, której żadna z nich nie używa.
- **Wysłane do producenta w 5 minut bezczynności:** Chrome łączył się wyłącznie z Google (14 hostów), Brave
  głównie z własnymi serwerami (ping użycia, P3A; 12 hostów), ZTBrowser ze stroną startową DuckDuckGo, jej hostem
  pingu i sześcioma hostami serwującymi dane ochrony (8 hostów).
- **Metoda, ograniczenia i przepis na powtórzenie pomiaru:** [porównanie śledzenia](docs/pl/reports/tracking-comparison.md), §3 i §8.

### Czego to nie dowodzi

Pomiar potwierdza, że „ZTBrowser nie daje własnemu producentowi nic do roboty, a gdy ma dane ochrony na miejscu,
daje trackerom niewiele”. Nie potwierdza, że „ZTBrowser jest bardziej anonimowy niż Brave lub DuckDuckGo”, i projekt
tego nie twierdzi.

- **Przewaga na jednej próbce to nie prawo.** ZTBrowser blokuje tu mocniej niż Brave, bo stosuje ostrzejszą
  warstwę dla trackerów reklamowych i analitycznych oraz odrzuca `preconnect`; ta sama ostrość może zepsuć stronę,
  którą łagodniejsze tarcze Brave zostawiają działającą, a lekarstwem jest przełącznik tarczy dla witryny. Okna
  logowania i trackery płatności są celowo zostawione wyjątkom DuckDuckGo.
- **Jedno uruchomienie na 12 stronach to nie ranking.** Dwa przebiegi Chrome różnią się o 2; strony zmieniają się
  między wczytaniami, a okno logowania Google serwis pokazuje w jednych przebiegach, w innych nie.
- **Przeglądarki DuckDuckGo nie zmierzono**, bo nie istnieje na Linuksa.
  [Tabela funkcji](#porównanie-z-przeglądarką-duckduckgo) porównuje udokumentowane zachowanie, nie ruch.
- **Blokowanie to nie anonimowość.** ZTBrowser to silnik Chromium jak Brave i Chrome, co dobrze robi na wtapianie się
  w tłum, ale ZTBrowser nie zmienia jeszcze tego, co serwis może odczytać o przeglądarce: brak szumu odcisku, brak
  GPC. Zainstalowane czcionki pozostają widoczne, tak jak w Brave i Chrome.
- **Brak ostrzeżeń przed phishingiem i złośliwym oprogramowaniem** (brak Safe Browsing), brak aliasów e-mail i
  VPN na razie. Planowane okno Tor ukryje Twój adres, nie fakt używania ZTBrowsera; gdy liczy się
  nieodróżnialność, użyj Tor Browser.
- **Brakuje niezależnej weryfikacji:** [privacytests.org](https://privacytests.org/) nie testuje ZTBrowsera.

---

## Stan

Projekt w toku, ale używalny, na QtWebEngine: karty (u góry, na dole, z lewej lub z prawej; kartę można przenieść do osobnego okna z jej menu pod prawym przyciskiem
myszy lub przeciągając ją poza pasek), profile z odizolowanymi
danymi i kafelkowym oknem wyboru jak w Chrome, aplikacje webowe z kafelkową stroną, pobieranie do wybranego folderu,
powiększenie osobno dla witryny, wyszukiwanie na stronie, przełącznik ochrony dla witryny, automatyczne odrzucanie
okien zgody na cookies, Wymaż dane przeglądania, 68 języków interfejsu. Jeszcze nie ma: ochrony przed odciskiem
przeglądarki i GPC, okna Tor, pytań o uprawnienia, przywracania sesji, zakładek i historii. Pełna lista z uzasadnieniem jest w dokumencie architektury i w planie migracji.

### Następne wydanie

Zaplanowane, niezaczęte i do zaprojektowania przed budową: usuwanie Twoich danych osobowych z serwisów brokerów
danych, pomoc w odzyskaniu tożsamości, gdy zostanie skradziona, oraz VPN — wszystko za darmo. DuckDuckGo sprzedaje te
trzy rzeczy w płatnej subskrypcji; ZTBrowser chce je dać bez opłat. Potrzebują serwerów lub partnerów i ustaleń
prawnych, więc nie ma daty.

W przyszłości ZTBrowser będzie częścią nowej dystrybucji Linuksa. Pozostanie samodzielnym projektem i zachowa
paczki dla innych dystrybucji.

---

## Instalacja

Pobierz paczkę z [najnowszego wydania](https://github.com/eshaiotic/ztbrowser/releases/latest).

| Plik | Dla |
|---|---|
| `ztbrowser-*.fc44.x86_64.rpm` | Fedora 44 |
| `ztbrowser-*-x86_64.pkg.tar.zst` | Arch Linux, CachyOS, EndeavourOS, Manjaro |
| `ZTBrowser-*-x86_64.AppImage` | każda inna dystrybucja z glibc 2.43 lub nowszą (około 230 MB: zawiera Chromium 140) |

Każde wydanie ma `SHA256SUMS` i atestację pochodzenia (build provenance), która dowodzi, że plik zbudował
GitHub Actions tego repozytorium:

```sh
sha256sum --check --ignore-missing SHA256SUMS
gh attestation verify ZTBrowser-*-x86_64.AppImage --repo eshaiotic/ztbrowser
```

Paczki rpm i pacmana korzystają z systemowych Qt i QtWebEngine, więc silnik przeglądarki dostaje aktualizacje
bezpieczeństwa Twojej dystrybucji; zależą od pakietów Qt Quick, QtWebChannel, QtWebEngine, SVG i xdg-utils, które menedżer
pakietów doinstaluje. AppImage niesie własne Qt i QtWebEngine (Chromium), grafikę bierze z hosta, a aktualizacje
silnika dostaje tylko z nowym wydaniem ZTBrowsera. Nie wymaga instalacji:
`chmod +x ZTBrowser-*-x86_64.AppImage && ./ZTBrowser-*-x86_64.AppImage`. Pulpity z Waylandem biorą ikonę okna z
zainstalowanego pliku `.desktop`, którego AppImage nie ma, więc przy starcie zapisuje go sobie sam
(`~/.local/share/applications/io.github.ztbrowser.ZTBrowser.desktop` i ikonę obok, wskazujące plik AppImage; po
przeniesieniu pliku uruchom go jeszcze raz). Wpis menu uruchamia ten plik później, więc wskazuje prawdziwy plik
(po rozwiązaniu dowiązań) i powstaje tylko wtedy, gdy ten plik i każdy katalog nad nim, aż do Twojego katalogu
domowego, należą do roota lub do Ciebie, a grupa ani inni nie mogą do nich pisać (nic pod `/tmp` ani innym katalogiem
współdzielonym nie spełnia tego warunku). Wpis znika, gdy jego plik przestaje spełniać warunek albo gdy zainstalujesz
paczkę (Twój wpis ma pierwszeństwo przed systemowym, więc bez tego menu dalej uruchamiałoby AppImage zamiast paczki; wpis
znika przy następnym starcie AppImage, a do tego czasu usuń go ręcznie). Wpisu napisanego przez Ciebie nigdy nie
ruszamy (wygenerowany ma `X-ZTBrowser-AppImage=<path>`), a przy zainstalowanej paczce nic nie powstaje. Nie ma jeszcze okna
Tor, więc żadna paczka nie dociąga Tora.

Z kopii tego repozytorium:

```sh
./install                 # Arch: buduje paczkę pacmana i ją instaluje; Fedora: instaluje dist/;
                          # potem pyta, czy ZTBrowser ma być domyślną przeglądarką (domyślnie: nie)
./uninstall               # usuwa paczkę; profile i aplikacje webowe zostają
./uninstall --purge       # kasuje też wszystkie dane ZTBrowsera i skróty aplikacji webowych
```

---

## Budowanie paczek wydania

```sh
packaging/release.sh                # wszystkie formaty do dist/, z SHA256SUMS
packaging/release.sh appimage rpm   # albo tylko wybrane: arch, appimage, rpm
```

AppImage i `.rpm` powstają w kontenerze Fedory 44 (rootless podman albo `CONTAINER_ENGINE=docker`), więc linkują
się z bibliotekami systemów, na które są przeznaczone, i niosą albo używają Chromium 140. Nie ma `.deb`: Qt
WebEngine Debiana 13 (Qt 6.8, Chromium 122) jest starszy niż próg silnika ZTBrowsera, a żaden stabilny Debian ani
Ubuntu nie ma nowszego. Paczkę dla Archa buduje `makepkg` na hoście, z flagami jego `makepkg.conf` (bez
optymalizacji w czasie łączenia, której linker Rusta nie czyta w części C++). Runtime AppImage jest przypięty do
wydania i sprawdzany po SHA-256.

Wypchnięcie taga wersji publikuje wydanie: `git tag vX.Y.Z && git push origin vX.Y.Z`.
[Workflow wydania](.github/workflows/release.yml) uruchamia te same skrypty (paczkę Archa w kontenerze Arch
Linux), odrzuca zależności ze znanymi podatnościami (`cargo audit`) i wysyła cztery paczki z `SHA256SUMS` oraz
atestacjami. Tag musi zgadzać się z wersją w `Cargo.toml`.

---

## Uruchamianie ze źródeł

ZTBrowser potrzebuje Qt 6 z QtWebEngine, QtWebChannel i Quick Controls (na Archu: `qt6-base qt6-declarative
qt6-webengine qt6-webchannel qt6-svg`), kompilatora C++ i Rusta.

```sh
sudo pacman -S --needed rust qt6-base qt6-declarative qt6-webengine qt6-webchannel qt6-svg
cargo run --release -p ztbrowser-qt                          # przeglądarka, profil domyślny
cargo run --release -p ztbrowser-qt -- --profile work        # przeglądarka, profil "work"
cargo run --release -p ztbrowser-qt -- https://example.com/  # otwórz stronę
```

Zainstalowane aplikacje webowe uruchamia się z menu aplikacji pulpitu (`ztbrowser-qt --web-app <id>`).

---

## Testy

Stos: **cargo-nextest** — 899 obiektowych scenariuszy Given/When/Then: 683 dla domeny w
`crates/ztbrowser-core/tests/` (82 plików, jeden na typ domenowy), 146 dla interfejsu Qt i jego adaptera silnika, 61 dla
narzędzia raportu w `crates/ztbrowser-test-report/tests/` i 9 dla narzędzia benchmarków w
`crates/ztbrowser-bench/tests/`. CI uruchamia je wszystkie w kontenerze Arch z Qt 6, testy Qt pod Xvfb z Mesą (QtWebEngine potrzebuje kontekstu
OpenGL nawet bez okna); same testy Qt uruchom przez
`xvfb-run -a cargo test -p ztbrowser-qt -p ztbrowser-engine-qt` (albo `QT_QPA_PLATFORM=offscreen` na pulpicie).

```sh
./run-tests                     # wszystkie testy + raport HTML w tests/reports/ (otwiera się w przeglądarce)
cargo test -p ztbrowser-core     # tylko testy domeny (szybkie, bez Qt)
cargo clippy --all-targets      # linty (pedantic, unsafe zabroniony)
cargo fmt --check               # formatowanie
cargo audit                     # podatności zależności (polityka zero podatności)
```

**Raport testów:** `./run-tests` zawsze zapisuje `tests/reports/test-report-YYYYMMDD-HHmm.html` — dashboard
historii przebiegów z ostatnim przebiegiem, ostatnimi 20 przebiegami, rankingami błędów, funkcjonalnościami i
najwolniejszymi testami, a gdy zainstalowany jest `cargo-llvm-cov`, także z pokryciem `ztbrowser-core`. Zobacz
[Raport testów HTML](docs/pl/tests/test-report.md).

**Architektura testów:**
- Domena (`ztbrowser-core`) nie zależy od Qt, więc każda decyzja ochrony — budowanie reguł,
  konwersja list filtrów, podnoszenie do HTTPS, czyszczenie linków, profile, aplikacje webowe, zoom — jest
  testowana bez ekranu.
- Każdy scenariusz to obiekt: `given_…` go buduje, `when_…` wykonuje akcję i zwraca wynik, `then_…` sprawdza
  jedną rzecz, zgodnie z [Object Oriented Programming First](docs/pl/engineering-rules.md):
  `LinkCleaningScenario::given_tracking_parameters_protection().when_link_is_opened(url).then_opens_unchanged()`.
- Nieudany test jest powtarzany raz: jeśli wtedy przejdzie, raport oznacza go jako FLAKY — to błąd do
  naprawy, nigdy do zignorowania.
- [Workflow testów](.github/workflows/tests.yml) uruchamia fmt, clippy, `cargo audit` i `./run-tests` przy
  każdym pushu do `main` i przy każdym pull requeście, i zachowuje raport każdego przebiegu jako artefakt.

---

## Struktura projektu

```
crates/ztbrowser-core/    logika domeny bez Qt: reguły ochrony, listy filtrów Guarda,
                         profile, aplikacje webowe, zoom, uruchamianie, położenie kart i styl przycisków okna
crates/ztbrowser-core/tests/
                         jeden plik scenariuszy OO na typ domenowy
crates/ztbrowser-qt/      przeglądarka: interfejs Qt Quick (QML) na QtWebEngine, spięty z Rustem przez cxx-qt
crates/ztbrowser-engine-qt/
                         adapter QtWebEngine: polityki żądań i ciasteczek, przekładka w C++
crates/ztbrowser/         aplikacja GTK 4 + libadwaita na WebKitGTK, zostaje, dopóki okno Tor i ochrona przed odciskiem przeglądarki nie powstaną w wersji Qt
crates/ztbrowser-test-report/
                         narzędzie deweloperskie za ./run-tests: raport testów HTML (nie trafia do paczek)
assets/                  dołączone Content Scope Scripts (Apache-2.0) i autoconsent (MPL-2.0), skrypt strony Guarda, ikony,
                         szablon strony błędu; nigdy Tracker Data Set ani listy filtrów
packaging/               PKGBUILD, spec rpm, budowa i składanie AppImage, plik .desktop, metadane AppStream
docs/en, docs/pl         architektura, zasady inżynierskie i raporty (kanoniczny angielski, polskie lustro)
```

- [Architektura](docs/pl/architecture/overview.md) — procesy, izolacja, potok ochrony, aplikacje webowe, ograniczenia.
- [Zasady inżynierskie](docs/pl/engineering-rules.md) — Object Oriented Programming First, testy OO Given/When/Then.

---

## Licencja

ZTBrowser to wolne oprogramowanie na licencji [GNU General Public License w wersji 3 lub późniejszej](LICENSE)
(`GPL-3.0-or-later`). Atrybucje są w [NOTICE](NOTICE).

### Licencje danych stron trzecich

- Content Scope Scripts 20.0.0 — Apache-2.0, dołączone w `assets/content-scope-scripts` razem z licencją.
- Konfiguracja prywatności — Apache-2.0, pobierana w czasie działania.
- Tracker Data Set — **CC BY-NC-SA 4.0** (niekomercyjna), pobierany w czasie działania, nigdy nie commitowany.
- Listy filtrów ZTBrowser Guard — GPL-3.0, CC BY-SA 3.0, CC BY 3.0, CC BY-NC-SA 4.0 lub warunki wydawcy,
  pobierane w czasie działania, nigdy nie commitowane.
- adblock-rust 0.13.5 — MPL-2.0, wkompilowany bez zmian (źródło: <https://github.com/brave/adblock-rust>).
- autoconsent 18.0.0 — MPL-2.0, spakowany z własnym punktem wejścia ZTBrowsera w `assets/autoconsent` (licencja, źródło
  punktu wejścia i przepis na przebudowę obok; źródło biblioteki: <https://github.com/duckduckgo/autoconsent>).
