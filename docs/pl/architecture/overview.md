# Architektura ZTBrowsera

Ten dokument opisuje, jak zbudowany jest ZTBrowser i dlaczego właśnie tak. Jest pisany dla osoby, która
programuje, ale nigdy nie pracowała nad przeglądarką. Każde pojęcie spoza wiedzy ogólnej jest wyjaśnione w
[Słowniku](#2-słownik).

## 1. Problem

Większość stron wczytuje kod firm, które zarabiają na śledzeniu ludzi między witrynami: analitykę, reklamy
i „społecznościowe” widżety. Na Linuksie przeglądarki, które dobrze to blokują, bazują na Chromium (Brave)
albo Firefoksie. Obie przyjmują rozszerzenia, a rozszerzenie samo w sobie może czytać każdą odwiedzaną
stronę.

DuckDuckGo publikuje jako open source ochronę przed śledzeniem ze swojej przeglądarki dla macOS: listę
trackerów, reguły decydujące, co blokować, wyjątki per witryna, dzięki którym strony działają, oraz skrypty
przeciw fingerprintingowi. Tamta przeglądarka używa silnika **WebKit**, a Linux ma ten sam silnik jako
**WebKitGTK**. ZTBrowser przenosi tę ochronę do przeglądarki WebKitGTK na Linuksa, **całkowicie bez obsługi
rozszerzeń**.

Projekt kształtują jeszcze dwa cele:

- **Izolowane profile.** „Praca” i „Prywatny” nigdy nie dzielą cookies, logowań ani cache.
- **Każda strona jako aplikacja.** mBank, Gmail czy dowolną inną stronę można zainstalować jako osobną
  aplikację pulpitu, z ikoną w menu aplikacji. Każda aplikacja ma własną sesję, więc link phishingowy kliknięty
  w przeglądarce nigdy nie dosięgnie logowania do banku. Aplikacje webowe (PWA) w Brave dzielą cookies z
  profilem przeglądarki, co daje słabszą izolację.

## 2. Słownik

| Pojęcie | Znaczenie |
|---|---|
| WebKitGTK | Linuksowe wydanie WebKit, silnika, który pobiera, renderuje i uruchamia strony. ZTBrowser używa jego API dla GTK 4, `webkitgtk-6.0`. |
| Reguła blokowania treści | Reguła JSON, którą WebKit kompiluje i sprawdza, zanim żądanie opuści komputer: „jeśli URL pasuje do wzorca, zablokuj go” (albo podnieś do HTTPS). Reguły są grupowane w *listy reguł*. |
| Tracker Data Set (TDS) | Lista domen śledzących DuckDuckGo: kto jest ich właścicielem i które ich żądania są dozwolone. Licencja CC BY-NC-SA 4.0, więc ZTBrowser pobiera ją w trakcie działania i nigdy jej nie dystrybuuje. |
| Konfiguracja prywatności | „Rozdzielnia” DuckDuckGo: która ochrona jest włączona i na których witrynach musi być wyłączona, bo je psuje (*wyjątki*). |
| Content Scope Scripts (C-S-S) | JavaScript DuckDuckGo działający w każdej stronie i ramce: blokuje fingerprinting i ogłasza GPC. Apache-2.0, wersja 20.0.0 leży w `assets/`. |
| GPC (Global Privacy Control) | Sygnał „nie sprzedawaj i nie udostępniaj moich danych”; w niektórych stanach USA wiążący prawnie. |
| ITP (Intelligent Tracking Prevention) | Wbudowane w WebKit ograniczenia cookies i magazynu witryn uznanych za trackery. |
| Upgrade HTTPS | Wczytanie `https://`, gdy strona prosi o `http://`, żeby nikt w sieci nie mógł czytać ani zmieniać strony. |
| Parametr śledzący | Parametr zapytania dopisywany do linków, by śledzić kliknięcie, np. `utm_source`, `fbclid`, `gclid`. |
| Profil | Osobna tożsamość przeglądarki z własnymi cookies, magazynem, cache, historią i uprawnieniami witryn. |
| Aplikacja webowa (PWA) | Strona zainstalowana jako aplikacja pulpitu: własne okno, ikona i pozycja w menu. |
| Sufiks publiczny, domena rejestrowalna | `co.uk` i `github.io` to *sufiksy publiczne*: każdy może zarejestrować nazwę pod nimi. *Domena rejestrowalna* to sufiks plus jedna etykieta (`bbc.co.uk`). Dwa hosty z tą samą domeną rejestrowalną to *ta sama witryna*. |
| Identyfikator aplikacji | Nazwa D-Bus działającej aplikacji GTK, np. `io.github.ztbrowser.ZTBrowser`. GTK uruchamia tylko jeden proces na identyfikator. |
| Plik desktop | Plik `.desktop` w `~/.local/share/applications`, który dodaje aplikację do menu pulpitu. |
| Ogień (Fire) | Przycisk, który jednym kliknięciem usuwa dane przeglądania profilu. |
| ZTBrowser Guard | Własne zabezpieczenia ZTBrowsera ponad DuckDuckGo: listy filtrów społeczności, ochrona przed fingerprintingiem, którą DuckDuckGo zostawia wyłączoną, i nagłówek GPC (§5.6). |
| Lista filtrów | Plik tekstowy z regułami blokowania i ukrywania w składni Adblock Plus (`\|\|ads.example^` blokuje domenę, `##.ad` ukrywa elementy, `@@` oznacza wyjątek), utrzymywany przez wolontariuszy. Te same listy czytają uBlock Origin, Brave i AdGuard. |
| Fingerprinting | Rozpoznawanie przeglądarki bez cookies, po tym, co zdradza: jak rysuje canvas, jak jej stos audio zaokrągla liczby, jaki ma ekran i fonty. |
| Ping kliknięcia linku | Żądanie, które `<a ping="…">` wysyła po kliknięciu linku; mówi analityce strony, który link kliknięto. |
| Tor | Sieć anonimizująca złożona z przekaźników prowadzonych przez wolontariuszy. Połączenie przechodzi przez trzy z nich (*obwód*: strażnik, środkowy, wyjściowy), a każdy zna tylko sąsiadów, więc strona widzi adres przekaźnika wyjściowego i nikt po drodze nie widzi jednocześnie użytkownika i strony. |
| Usługa onion | Strona dostępna tylko wewnątrz Tora (`….onion`, RFC 7686). Połączenie nigdy nie opuszcza Tora i jest szyfrowane od końca do końca, nawet przez zwykłe `http://`. |
| Most, transport wtykowy | *Most* (bridge) to nieopublikowany przekaźnik Tora dla sieci, które blokują te opublikowane. *Transport wtykowy* (pluggable transport: `obfs4`, `snowflake`, `webtunnel`, `meek_lite`) to program, który dodatkowo maskuje połączenie z mostem. |
| Port SOCKS, port kontrolny | Dwa lokalne porty działającego `tor`: aplikacje wysyłają przez *port SOCKS* swoje połączenia, a program, do którego Tor należy, konfiguruje go i obserwuje przez *port kontrolny*. |

## 3. Dwa crate'y

Kod jest podzielony na dwa crate'y Rust, a zależność biegnie w jedną stronę: `ztbrowser` → `ztbrowser-core`.

- **`crates/ztbrowser-core`** zawiera logikę domenową: budowanie list reguł z TDS, czytanie konfiguracji
  prywatności, decyzje o upgrade HTTPS, wykrywanie parametrów śledzących, parsowanie linii poleceń, katalogi
  profili i aplikacji webowych, wybór ikon, generowanie plików desktop i nazw pobranych plików. Nie wie nic o
  GTK ani WebKit, dlatego każdą regułę sprawdzają szybkie testy jednostkowe (`cargo test -p ztbrowser-core`).
- **`crates/ztbrowser`** to aplikacja GTK 4 + libadwaita. Jej typy są adapterami: biorą decyzję od rdzenia i
  wykonują ją przez WebKitGTK, libsoup (HTTP) albo GIO (pliki, procesy).

Zasady inżynierskie obu crate'ów (Object Oriented Programming First, obiektowe testy Given/When/Then, jeden
`use` na element) są w [engineering-rules.md](../engineering-rules.md).

## 4. Procesy i izolacja

![Każdy profil i każda aplikacja webowa to osobny proces ZTBrowsera z własną sesją WebKit; wspólny, tylko do odczytu cache danych ochrony; drugie uruchomienie przekazywane przez D-Bus; link do innej witryny kliknięty w aplikacji webowej otwiera się w przeglądarce](../../assets/diagrams/architecture-process-isolation.pl.svg)

Każdy profil i każda zainstalowana aplikacja webowa działa jako **osobny proces z własnym identyfikatorem
aplikacji**:

| Co działa | Identyfikator aplikacji | Katalog profilu |
|---|---|---|
| Przeglądarka, profil domyślny | `io.github.ztbrowser.ZTBrowser` | `profiles/default` |
| Przeglądarka, profil „work” | `io.github.ztbrowser.ZTBrowser.Profile.work` | `profiles/work` |
| Aplikacja webowa „mBank” | `io.github.ztbrowser.ZTBrowser.WebApp.mbank_pl` | `profiles/app_mbank_pl` |
| Okno Tor | `io.github.ztbrowser.ZTBrowser.Tor` | brak: sesja żyje w pamięci (§4.1) |

`LaunchRequest` (rdzeń) zamienia linię poleceń `ztbrowser [--profile NAZWA | --web-app ID | --tor] [URL …]` na jeden z
tych identyfikatorów. GTK pyta następnie D-Bus, czy ten identyfikator już działa. Jeśli tak, nowa linia
poleceń trafia do działającego procesu (który otwiera URL w oknie), a nowy proces kończy pracę.

Każdy proces ma dokładnie jedną *sesję sieciową* WebKit, zapisaną w katalogach profilu
(`~/.local/share/ztbrowser/profiles/<id>` i `~/.cache/ztbrowser/profiles/<id>`). Są tam baza cookies, local
storage, IndexedDB, cache, favikony i wpisy HSTS. Cookies stron trzecich są odrzucane, a ITP jest włączone.
Nic z tej sesji nie jest widoczne dla innego profilu ani aplikacji.

**Dlaczego proces na profil, a nie kilka sesji w jednym procesie?** Granica procesu systemu operacyjnego to
najsilniejsza separacja, jaką Linux daje bez kontenerów: awaria, wyciek pamięci czy błąd WebKit w aplikacji
banku nie dotknie przeglądarki. Pulpit widzi też każdą aplikację jako osobny program (własna ikona w doku,
grupowanie okien przez `StartupWMClass`), a tego właśnie obiecuje „zainstaluj jako aplikację”.

Jedyne wspólne dane to **kanał ochrony** w `~/.cache/ztbrowser/protections`: TDS, konfiguracja prywatności i
listy filtrów Guarda (`filter-lists/*.txt`). Czyta je każdy proces. Pobranie podmienia plik atomowo (zapis `nazwa.part`, potem zmiana nazwy), więc proces
nigdy nie przeczyta pół pliku.

Przed usunięciem profilu albo odinstalowaniem aplikacji ZTBrowser pyta D-Bus, czy jej identyfikator nadal
działa (`RunningInstances`), i odmawia usunięcia danych, których używa żywy proces.

### 4.1 Okno Tor

![Proces okna Tor ma okno przeglądarki ze stroną łączenia, sesję trzymaną w pamięci i klienta HTTP, które czekają na Tora; TorConnection zapisuje konfigurację, uruchamia własny tor ZTBrowsera i śledzi go przez port kontrolny; cały ruch idzie przez lokalny port SOCKS tor, opcjonalnie przez transport mostu, do sieci Tor; systemowa usługa Tor, systemowy resolver DNS i połączenia bezpośrednie nie są nigdy używane](../../assets/diagrams/architecture-tor-window.pl.svg)

`ztbrowser --tor` (także **New Tor Window** w menu, `Alt+Shift+N` albo akcja w pliku desktop) otwiera **okno
Tor**: okno przeglądarki, którego każde żądanie idzie przez Tora, więc odwiedzane strony widzą adres
przekaźnika wyjściowego Tora zamiast adresu użytkownika. To osobny proces, `io.github.ztbrowser.ZTBrowser.Tor`,
tak jak profil, ale bez katalogu profilu: jego sesja żyje w pamięci i znika razem z procesem.

**Własny Tor ZTBrowsera, nigdy systemowy.** Proces uruchamia własny program `tor`, z którego nie korzysta nic
innego na komputerze; systemowa usługa Tor nie jest potrzebna ani ruszana. `TorPrograms` (rdzeń) szuka `tor`
najpierw w `lib/ztbrowser/tor/` obok `bin/` ZTBrowsera (tam swoją kopię kładą paczka Debiana i AppImage), potem
w `/usr/bin`, `/usr/sbin` i `/usr/local/bin`. Nigdy nie przeszukuje `PATH`: program podrzucony do katalogu
zapisywalnego przez użytkownika, który jest w `PATH`, nie może zostać Torem okna Tor. `TorConfiguration`
(rdzeń) zapisuje plik konfiguracji, a Tor startuje jako `tor -f <katalog danych>/torrc --defaults-torrc
/dev/null`, więc ani `/etc/tor/torrc`, ani plik domyślnych ustawień nie mogą go zmienić. Tor działa wyłącznie
jako klient, sam wybiera wolne porty pętli zwrotnej dla SOCKS i portu kontrolnego (`auto`), zapisuje port
kontrolny do pliku, na który czeka ZTBrowser, chroni port kontrolny plikiem cookie i kończy pracę razem z
ZTBrowserem (`__OwningControllerProcess`, a połączenie kontrolne przejmuje go poleceniem `TAKEOWNERSHIP`). Jego
katalog danych, `~/.local/share/ztbrowser/tor`, jest trwały: przechowuje strażników wejściowych Tora
(wybieranie nowych przy każdym uruchomieniu zwiększałoby szansę trafienia na złośliwego) i zapisany katalog
przekaźników, dzięki któremu kolejne połączenia trwają sekundy.

**Śledzenie połączenia.** `TorConnection` (aplikacja) uruchamia Tora, czyta plik portu kontrolnego,
udowadnia dostęp cookie (`AUTHENTICATE`), przejmuje Tora, subskrybuje zdarzenia stanu klienta i pyta, gdzie
Tor nasłuchuje SOCKS. `ControlReplyReader` (rdzeń) składa linie odpowiedzi Tora, `BootstrapStatus` czyta
raporty `BOOTSTRAP PROGRESS=… SUMMARY=…`, a `TorEndpoint` przyjmuje tylko adresy pętli zwrotnej. Każde okno
procesu zasłania karty stroną łączenia (`TorConnectionPage`), która pokazuje postęp Tora i zgłaszane problemy,
aż Tor dojdzie do 100 %. Wtedy sesja dostaje adres proxy i karty się pokazują. Gdy Tor nie wystartuje albo
później się zatrzyma, strona wraca z powodem i przyciskiem **Try Again**.

**Domyślnie zamknięte (fail closed).** Nic nie opuszcza okna, zanim Tor się połączy, i nic nigdy go nie
omija. `ProfileSession::start_tor` buduje sesję, której wczytania czekają, aż ochrona będzie aktywna *i*
trasa przez Tora ustawiona; `HttpClient`, który pobiera dane ochrony, też do tego czasu wstrzymuje żądania.
Nie ma powrotu do połączenia bezpośredniego. Zawsze, gdy Tor nie jest połączony (zanim połączy się pierwszy
raz, gdy startuje od nowa z innymi mostami, gdy się zatrzymał), proxy sesji to `ClosedProxy`: port pętli
zwrotnej, który ZTBrowser sam trzyma przez cały czas działania i który od razu zamyka każde połączenie. Adresy
otwierane w tym czasie czekają na Tora; żądania stron już otwartych (ich własne żądania, przeładowanie,
kliknięty odnośnik) od razu się nie udają i kończą stroną błędu, której **Try Again** wczytuje je, gdy Tor
wróci. Żaden inny lokalny program nie przejmie zwolnionego portu Tora, żeby je odczytać. Zmierzone na
AppImage: po zabiciu Tora żądanie przez `ClosedProxy` zostało odrzucone w 4 ms, a przeładowanie skończyło się
stroną błędu. Nazwy hostów jadą wewnątrz żądania SOCKS (klient SOCKS5 GLib wysyła nazwy, nie adresy), więc
rozwiązuje je Tor, a systemowy resolver DNS nie dowiaduje się, jakie strony okno odwiedza. Zmierzone na
AppImage podczas otwierania usługi onion DuckDuckGo: przez 60 sekund, próbkując co 0,2 s, procesy interfejsu i
sieci WebKit miały połączenia wyłącznie z portami SOCKS i kontrolnym Tora na 127.0.0.1 i żadnego z
`systemd-resolved` (próbkowanie może przeoczyć bardzo krótko żyjące gniazdo, więc to dowód poszlakowy, nie
ścisły).

**Nic na dysku.** Sesja WebKit jest efemeryczna: cookies, magazyn, cache i wpisy HSTS żyją w pamięci. Powiększenie
witryn, witryny z wyłączoną ochroną i wyszukiwarka też są trzymane w pamięci (`StoredFile::discarded`).
Skompilowane listy reguł i własna kopia danych ochrony okna trafiają do `$XDG_RUNTIME_DIR/ztbrowser/tor-session`
(pamięć, którą system czyści przy wylogowaniu) i są usuwane przy starcie i przy wyjściu, bo listy reguł
zawierają nazwy odwiedzonych witryn (hosty, którym nie udało się HTTPS, witryny z wyłączoną ochroną). Dane
ochrony są kopiowane z cache profili bez plików `ETag`, klient okna Tor nigdy nie wysyła `ETag`, a jego
pobrania nigdy nie trafiają do cache profili: walidator, który serwer dał jednej stronie, pozwoliłby mu
inaczej powiązać okno Tor z bezpośrednim połączeniem użytkownika. Zapisywane są tylko własny stan Tora i wybór
mostów (`tor-connection.json`).

**Wycieki i łączenie aktywności.** WebRTC i przechwytywanie multimediów są wyłączone: WebRTC wysyła własny
ruch UDP z pominięciem każdego proxy i może zdradzić adresy lokalne. Okno prosi strony o angielski
(`Accept-Language: en-US`), tak jak Tor Browser, bo język systemu wyróżniałby jego użytkowników. Usługi onion
nigdy nie są podnoszone do HTTPS (`DomainName::is_onion`), a ich strony przez zwykłe `http://` dostają
kłódkę tylko w tym oknie, bo Tor szyfruje je od końca do końca. Pasek adresu wyszukuje w usłudze onion DuckDuckGo. Aplikacje
webowe, profile i wybór wyszukiwarki nie są oferowane: wyprowadziłyby przeglądanie poza Tora. Z tego samego
powodu odnośniki do innych aplikacji (`mailto:`, `zoommtg:` i podobne) nie otwierają się z okna Tor, a prośby
o lokalizację są odrzucane bez pytania: położenie zdradziłoby to, co Tor ukrywa.

**Nowa tożsamość.** W oknie Tor przycisk Ognia daje nową tożsamość: usuwa wszystkie dane sesji (razem z HSTS),
zamyka karty, wysyła `SIGNAL NEWNYM` (nowe obwody dla nowych połączeń) i przenosi sesję na nowe dane
logowania SOCKS (`identityN:tor`). Tor kieruje połączenia z różnymi danymi logowania SOCKS na różne obwody
(`IsolateSOCKSAuth`, domyślnie włączone), więc nic, co nastąpi, nie dzieli obwodu z tym, co było wcześniej.

**Mosty.** Tam, gdzie Tor jest blokowany, **Tor Connection…** (menu albo strona łączenia) oferuje Snowflake,
obfs4 z mostami, które dostarcza Tor Browser (`BuiltInBridges`, skopiowane z `tor-browser-build`), albo mosty
wklejone przez użytkownika. `BridgeLine` (rdzeń) przyjmuje wklejoną linię tylko w dokładnym kształcie linii
mostu: linie trafiają do pliku konfiguracji Tora, gdzie znak nowej linii, `#` (komentarz) albo `\`
(kontynuacja linii) mogłyby przemycić własne opcje. Zapisanie wyboru restartuje z nim Tora. `TorPrograms`
znajduje programy transportów (`lyrebird` albo `obfs4proxy`, `snowflake-client`) w tych samych katalogach co
`tor`, a okno ustawień podaje pakiet do zainstalowania, gdy któregoś brakuje.

**Paczki.** Paczki dla Arch i Fedory zależą od `tor` z dystrybucji, który sam nie uruchamia usługi
systemowej. Paczka `tor` Debiana uruchamiałaby przy każdym starcie systemu ogólnosystemową usługę Tor, więc
paczka ZTBrowsera dla Debiana zamiast tego niesie własną kompilację `tor` Debiana w `/usr/lib/ztbrowser/tor`
(jej biblioteki są zwykłymi zależnościami paczki) i poleca `obfs4proxy` oraz `snowflake-client`. AppImage
niesie `tor`, potrzebne mu biblioteki poza biblioteką C, `obfs4proxy` i `snowflake-client`, z licencjami w
`usr/share/doc/ztbrowser/third-party`. Zmierzone na AppImage: Tor połączył się bezpośrednio w około 5 s, przez
wbudowane mosty obfs4 w około 30 s, a przez Snowflake w około 50 s, i za każdym razem `check.torproject.org`
potwierdził, że okno korzysta z Tora.

## 5. Od danych DuckDuckGo do chronionych stron

![Tracker Data Set i konfiguracja prywatności są pobierane z CDN DuckDuckGo, walidowane i zapisywane w cache; rdzeń zamienia je na trzy listy reguł i skrypt content scope; aplikacja kompiluje listy w magazynie filtrów WebKit i instaluje je razem ze skryptem w każdej stronie](../../assets/diagrams/architecture-protection-pipeline.pl.svg)

### 5.1 Pobieranie danych

`ProtectionFeed` pobiera `macos-tds.json` i `macos-config.json` z `staticcdn.duckduckgo.com` przez `HttpClient`
(libsoup, biblioteka HTTP, której używa sam WebKitGTK). Sieć jest traktowana jako wroga:

- żądanie nie niesie cookies, ma limit czasu 30 sekund i przestaje czytać po 32 MiB;
- odpowiedź zastępuje kopię w cache tylko wtedy, gdy jest poprawnym JSON z oczekiwanym kluczem najwyższego
  poziomu (`trackers`, `features`);
- odświeżenia wysyłają zapisany `ETag`, więc niezmieniony plik kosztuje jedno małe żądanie. Odświeżanie
  działa co 12 godzin, a co 30 sekund, dopóki profil nie ma żadnych danych ochrony.

Strony nie wczytują się, dopóki ochrona nie jest aktywna (`ProfileSession::when_protected`), więc pierwsza
strona nowego profilu nigdy nie wczyta się bez ochrony.

`FilterListFeed` pobiera listy filtrów Guarda według tych samych zasad (§5.6). Oba kanały korzystają z
`FeedDocument`, który trzyma ostatnią dobrą kopię jednego dokumentu i podmienia ją tylko na pobranie, które
rozpoznaje.

### 5.2 Trzy listy reguł

Akcja `ignore-previous-rules` w WebKit (wyjątek) anuluje tylko reguły *z tej samej listy*. Dlatego ZTBrowser
buduje trzy osobne listy, żeby wyjątki jednej funkcji nie wyłączyły innej:

1. **`trackers`**: `ContentBlockerRulesBuilder`, port `TrackerRadarKit` z DuckDuckGo, blokuje żądania stron
   trzecich do domen śledzących. Respektuje reguły zezwalające TDS, wyjątki `contentBlocking` z konfiguracji,
   allowlistę trackerów oraz witryny, dla których użytkownik wyłączył ochronę (przycisk tarczy; w aplikacji
   webowej „Protect This Site” w jej menu).
2. **`https`**: `HttpsUpgradePolicy` tworzy jedną regułę `make-https` dla każdego URL `http://`, a po niej
   wyjątki dla adresów IP, nazw jednoczłonowych i lokalnych (`localhost`, `.local`, `.lan`, `.internal`,
   `.home.arpa`), wyjątków `https` z konfiguracji i hostów, dla których upgrade wcześniej zawiódł. Każdy
   wzorzec wyjątku kończy się na hoście i opcjonalnym porcie, więc dane logowania w URL nie mogą udawać
   zwolnionego hosta: `http://localhost:1@example.com/` prowadzi do `example.com` i jest podnoszony do HTTPS.
3. **`links`**: `TrackingParameterRemover` tworzy jedną regułę na parametr śledzący. Reguła blokuje nawigację
   *dokumentu w ramce głównej*, która go niesie, poza witrynami zwolnionymi przez konfigurację.

`PageProtections` kompiluje każdą listę przez `UserContentFilterStore` WebKit. Kompilacja listy trackerów
trwa około sekundy, więc wynik jest trzymany na dysku obok odcisku (hasha) źródła. Niezmieniona lista jest
wczytywana zamiast kompilowana. ZTBrowser Guard dodaje trzy własne listy (§5.6).

Nawet niezmieniona lista trackerów kosztuje: zbudowanie jej 7000 reguł i serializacja (4,7 MB JSON-a) tylko po
to, by policzyć odcisk, zajmuje 0,1–0,2 s procesora, a na obciążonej maszynie więcej. Ta praca idzie na wątek
roboczy (`gio::spawn_blocking`), a start na nią nie czeka. Tak jak aplikacja DuckDuckGo na macOS, sesja
najpierw włącza listę trackerów skompilowaną poprzednio, więc strony ładują się od razu, a nową listę podmienia,
gdy jest gotowa i się różni. Listy `https` i `links` są zawsze budowane od nowa przed pierwszą stroną. Zajmują
znacznie poniżej milisekundy, a zapisana lista `https` mogłaby wciąż zwalniać hosty, które użytkownik pozwolił
wczytać bez szyfrowania w poprzedniej sesji. Tylko pierwszy w historii start, bez niczego skompilowanego, czeka
na całą kompilację.

### 5.3 Reakcja na zablokowane i nieudane wczytania

Reguły działają, zanim żądanie opuści komputer. Dwie z nich potrzebują potem pomocy strony (`PageView`):

- **Linki śledzące.** Gdy lista `links` zablokuje nawigację, WebKit zgłasza nieudane wczytanie z błędem
  polityki. `PageView` wczytuje wtedy ten sam adres bez parametrów śledzących. Strażnik pamięta ostatni
  oczyszczony adres, więc niezgodność reguły i czyściciela nigdy nie spowoduje pętli.
- **Witryny bez HTTPS.** Gdy podniesione wczytanie zawiedzie, baner informuje, że „*host* nie oferuje
  bezpiecznego połączenia”, i pokazuje przycisk **Load Insecure Page**. Dopiero po tym kliknięciu host trafia
  do nieudanych hostów profilu, lista `https` jest przebudowana i wczytywany jest `http://`. Ciche cofnięcie do
  HTTP pozwoliłoby atakującemu w sieci zepsuć HTTPS dowolnej witryny i zmusić ją do zwykłego HTTP.

**Strony błędów.** Każda inna porażka dostaje stronę błędu ZTBrowsera zamiast wbudowanej strony WebKita, która
jest gołym nagłówkiem na białym tle. WebKit, libsoup i GLib zgłaszają porażki kodami błędów z kilku domen:
resolvera, wejścia-wyjścia, TLS oraz własnych błędów sieci i polityk WebKita. `ErrorPageView` (aplikacja)
zamienia te kody na kilka przyczyn, na które człowiek może zareagować (`FailureCause` w rdzeniu):

- brak sieci;
- nieznany host;
- połączenie odrzucone, przekroczony czas albo przerwane połączenie;
- zastrzeżony port;
- niezaufany certyfikat, z każdym powodem;
- awaria strony albo brak pamięci;
- status błędu HTTP z pustą treścią;
- brak pliku.

Strona nazywa problem jednym nagłówkiem, mówi, której witryny dotyczy, i podaje, co sprawdzić. Oferuje tylko
pomocne akcje: Try Again albo Reload, wyszukiwanie wyszukiwarką profilu dla nieistniejącego hosta i Go to Home
Page dla pustego 404. Jest rysowana w bieżącym jasnym albo ciemnym stylu przeglądarki, w kolorze akcentu pulpitu.
Strona jest wczytywana jako *alternate HTML* WebKita dla nieudanego adresu, więc pasek adresu, Reload i Wstecz
działają jak dla strony, która zawiodła.

Ważne są cztery szczegóły:

- Status błędu z `Content-Length: 0` jest zatrzymywany na etapie polityki odpowiedzi. Wczytanie wtedy się
  nie udaje, a jego strona błędu zajmuje wpis w historii jak przy każdym nieudanym wczytaniu. Pokazana dopiero
  po pustej stronie, dodałaby drugi wpis, a Wstecz wracałoby do pustej strony i objaśniało ją od nowa, bez
  końca.
- Porażki certyfikatu przychodzą przez `load-failed-with-tls-errors` ze wszystkimi powodami znalezionymi przez
  GLib. Strona wymienia je wszystkie. Nie ma „przejdź mimo to”: WebKit nigdy nie wczyta takiej strony, a
  obejście uczyłoby użytkowników przeklikiwać ostrzeżenie, na którym polega atakujący.
- Połączenie z serwerem, który nigdy nie odpowiada, libsoup anuluje (`G_IO_ERROR_CANCELLED`), zamiast zgłosić
  przekroczenie czasu. Dlatego jest objaśniane jako przekroczony czas, a wczytanie zatrzymane przez
  użytkownika (`WEBKIT_NETWORK_ERROR_CANCELLED`) nie pokazuje niczego.
- Nad stroną błędu pasek adresu nie pokazuje kłódki ani przycisku instalacji, bo strona niczego nie dowodzi
  o połączeniu. Zwykłe HTTP zachowuje ostrzeżenie: strona działa w pochodzeniu witryny, więc inne okna tej
  witryny nadal mogą ją przepisać.

Strona działa w pochodzeniu nieudanej witryny, więc nie może nikomu zaszkodzić. Wszystko, co pochodzi z
porażki, jest escapowane w HTML: adres, host i komunikat GLib. Szablon jest wypełniany w jednym przebiegu, więc
wartość zawierająca `{{…}}` nigdy nie zostanie rozwinięta. Content Security Policy (`default-src 'none'`)
zabrania skryptów, formularzy i wszelkich zdalnych wczytań. Linki ponowienia powstają tylko dla adresów
`http`, `https` i `file`.

### 5.4 Content Scope Scripts, GPC i cookies

`ContentScopeScript` wypełnia szablon C-S-S konfiguracją prywatności, niechronionymi witrynami użytkownika i
preferencjami (GPC włączone, losowy klucz sesji i sekret wiadomości na proces, język interfejsu). Skrypt
działa na starcie dokumentu w każdej ramce, w świecie JavaScript strony, tak jak w aplikacji DuckDuckGo na
macOS. Wbudowana w niego konfiguracja ma włączoną ochronę Guarda przed fingerprintingiem (§5.6).

Każda ramka parsuje osadzoną konfigurację od nowa, dlatego `ContentScopeScript` osadza tylko funkcje, które
uruchamia dołączony build `apple` (`platformSupport.apple` i ich `featureConfigDependencies`). Bundle sam
odrzuca wszystkie pozostałe funkcje na samym początku (`pruneConfigFeatures`), więc wynik jest identyczny, ale
każda ramka czyta teraz ok. 165 KB konfiguracji zamiast 840 KB (sam `autoconsent` to 520 KB), a skrypt maleje z
1,36 MB do 0,67 MB. Test odczytuje ten zbiór funkcji z dołączonego bundla, więc aktualizacja C-S-S nie może po
cichu wyłączyć żadnej funkcji. Wiadomości skryptu trafiają do handlera `contentScopeScripts`. Handler odpowiada `{}` na powiadomienia i
obiektem błędu na żądania, bo ZTBrowser nie implementuje natywnych funkcji, o które proszą.

### 5.5 Ogień

Przycisk Ognia (w nagłówku każdego okna) zamyka pozostałe okna profilu i usuwa wszystkie dane witryn profilu
(cookies, magazyn, cache, favikony, zoom nadany witrynom) z wyjątkiem wpisów **HSTS**. Te pamiętają, które witryny wymagają HTTPS,
a ich zapomnienie tylko osłabiłoby następną wizytę. Bieżące okno zaczyna od nowa z pustą kartą.

### 5.6 ZTBrowser Guard

![Listy filtrów są pobierane od wydawców, sprawdzane i trzymane w cache; rdzeń grupuje je w trzy listy reguł Guarda rozpoznawane po odcisku wejścia; PageProtections wczytuje aktualną skompilowaną kopię albo buduje i kompiluje zmienioną listę w wątku roboczym; konfiguracja prywatności jest wzmacniana przed wypełnieniem skryptu content scope; funkcja WebKit wysyła nagłówek GPC](../../assets/diagrams/architecture-guard-pipeline.pl.svg)

Reguły DuckDuckGo blokują trackery skatalogowane przez DuckDuckGo i tylko wtedy, gdy ładują się jako strona
trzecia. Brave Shields czytają też listy filtrów społeczności, dlatego w naszym pomiarze Brave blokował więcej.
Guard zamyka tę lukę i idzie dalej na cztery sposoby: więcej list, reguły działające też na własne żądania
strony, ochrona przed fingerprintingiem, którą DuckDuckGo zostawia wyłączoną, oraz nagłówek GPC.

**Listy filtrów.** `FilterListFeed` pobiera dziewięć list od ich wydawców. Część ich licencji nie pasuje do
paczki ZTBrowsera na GPL-3.0-or-later: CC BY-NC-SA 4.0 zabrania użytku komercyjnego, a CC BY-SA 3.0 jest
niezgodna z GPL. Dlatego, tak jak TDS, nigdy nie są dołączane:

| Lista | Co blokuje | Licencja |
|---|---|---|
| EasyList | reklamy | GPL-3.0 albo CC BY-SA 3.0 |
| EasyPrivacy | trackery i analitykę | GPL-3.0 albo CC BY-SA 3.0 |
| uBlock filters, Privacy, Badware risks | reklamy, trackery, strony oszustw i złośliwego oprogramowania | GPL-3.0 |
| uBlock filters – Unbreak | nic: wyjątki naprawiające strony psute przez inne listy | GPL-3.0 |
| Lista Petera Lowe'a | serwery reklam i śledzenia | warunki wydawcy (pgl.yoyo.org) |
| Polish Filters | reklamy polskich stron | CC BY-NC-SA 4.0 |
| EasyList Cookie List | banery zgody na cookies | CC BY 3.0 |

Pobranie podmienia kopię w cache tylko wtedy, gdy pierwsza linia jest nagłówkiem listy albo komentarzem, więc
strona logowania hotelowego Wi-Fi podana zamiast listy zostaje odrzucona. Lista, której nie udało się
odświeżyć, zostaje przy starej kopii i nie zatrzymuje pozostałych.

`GuardRuleList` (rdzeń) grupuje listy w trzy listy reguł WebKit: `guard-ads` (EasyList, uBlock filters,
Badware risks, Peter Lowe, Polish Filters), `guard-privacy` (EasyPrivacy, uBlock Privacy) i
`guard-annoyances` (lista cookies). Są trzy z tych samych dwóch powodów, które rozdzielają listy DuckDuckGo:
lista mieści najwyżej 150 000 reguł, a wyjątek anuluje tylko reguły własnej listy. Dlatego wyjątki uBlock
Unbreak trafiają do każdej listy i każda lista kończy się jedną regułą, która wyłącza ją na stronach, dla
których użytkownik wyłączył ochronę. Crate `adblock` od Brave (MPL-2.0, konwerter, którego Brave używa w
przeglądarce na iOS) zamienia składnię Adblock Plus na JSON WebKit. Filtry bez `$third-party` działają też
na własne żądania i strony witryny; Brave Shields robią to tylko w trybie „agresywnym”. `guard-privacy`
blokuje też każdy ping kliknięcia linku.

Trzy odkrycia ukształtowały konwersję:

- `adblock` kończy wynik wyjątkiem dla dokumentów pierwszej strony, żeby żaden filtr nie zablokował strony,
  o którą prosi użytkownik. WebKit decyduje, które elementy strona ukrywa, przy tym samym wczytaniu, więc na
  końcu listy ten wyjątek anuluje każdą regułę ukrywania. Na stronie testowej ukryty `.ad_banner` był
  widoczny, dopóki `GuardRuleList` nie przesunął wyjątku przed pierwszą regułę ukrywania.
- WebKit odrzuca listę ponad 150 000 reguł, a także pustą („Empty extension”). Gdyby listy przerosły limit,
  `GuardRuleList` usuwa reguły blokujące tuż przed pierwszym wyjątkiem i zachowuje każdy wyjątek. Lista bez
  niczego pobranego w ogóle nie jest kompilowana (`has_rules`).
- WebKitGTK zalicza `navigator.sendBeacon` do `raw`, razem z każdym innym żądaniem skryptu, a do `ping`
  tylko `<a ping>`. Beacony do znanych trackerów są blokowane po adresie jak każde żądanie. Blokada
  wszystkich beaconów zgubiłaby też to, co aplikacje webowe wysyłają przy zamykaniu karty (szkic,
  potwierdzenie przeczytania), więc Guard tego nie robi.

**Kompilacja bez spowalniania stron.** Konwersja trzech list zajmuje 0,2 s procesora i w szczycie ok. 75 MB
pamięci. Ich kompilacja w WebKit trwa ok. 9 s. Żadne z nich nie może opóźnić strony, więc listę Guarda
rozpoznaje **odcisk jej wejścia**: teksty jej list filtrów, niechronione strony użytkownika i rewizja
konwersji. Policzenie go trwa poniżej milisekundy. Gdy zapisana skompilowana kopia powstała z tego samego
wejścia, jest wczytywana i nic nie jest konwertowane: start z niezmienionymi listami chroni strony po 0,5 s.
Gdy lista się zmieniła, jest budowana w wątku roboczym i kompilowana w tle, a w tym czasie strony chroni
poprzednia skompilowana kopia. Wyjątkiem jest tarcza użytkownika, w obu kierunkach, bo każda lista Guarda
zawiera niechronione strony użytkownika. Gdy użytkownik wyłącza ochronę dla strony, poprzednia kopia dalej
blokowałaby właśnie tę stronę. Dlatego na te kilka sekund zmienione listy Guarda są pomijane, listy DuckDuckGo
zostają aktywne, a strona przeładowuje się od razu. Gdy użytkownik włącza ochronę z powrotem, poprzednia kopia
wciąż zwalnia tę stronę, więc strona przeładowuje się dopiero po skompilowaniu nowych list, ok. 10 s później.
Przebudowy nakładają się na siebie (odświeżenie list w tle może ruszyć, zanim skończy się kompilacja po
kliknięciu), a nowsza przebudowa zastępuje starszą. Dlatego te obietnice, czyli przeładowanie i pominięcie
Guarda, trzyma profil (`PendingRuleChanges`), a nie przebudowa, która je złożyła. Dotrzymuje ich ta przebudowa,
która aktywuje reguły. Bez tego odświeżenie, które wypadło w tych sekundach, gubiło przeładowanie. Serializacja
reguł pojedynczo, zamiast budowania najpierw `serde_json::Value` całej
listy, obniżyła szczyt pamięci z 243 MB do 84 MB. Ma to znaczenie, bo glibc nie oddaje pamięci wątku
roboczego.

**Ogólne ukrywanie elementów, stosowane leniwie.** WebKit dopasowuje każdy selektor `css-display-none`
blokera treści do każdego elementu przy każdym przeliczeniu stylów. Listy Guarda mają około 30 000 ogólnych
selektorów (`##.ad-banner`, bez nazwy witryny). Z nimi pełne przeliczenie stylów strony z 10 000 elementów trwało
229 ms zamiast 35 ms. Dlatego `GuardRuleList` nie przekazuje ogólnego ukrywania do WebKit, tylko zapisuje je jako
`<lista>.generic-hiding.json` (`GenericHiding`, indeksowane klasą lub id, których selektor potrzebuje). Skrypt
strony Guarda (`assets/guard/generic-hiding.js`) działa we własnym świecie skryptów w czterech krokach:

1. Na starcie dokumentu sprawdza jedną regułę sondy na listę. WebKit ukrywa sondę tam, gdzie dana lista działa,
   więc o wyjątkach i tarczy użytkownika decyduje sam WebKit.
2. Zgłasza klasy i id strony do handlera wiadomości `ztbrowserGuard`.
3. Odpowiedź wstawia do przyjętego arkusza stylów.
4. Selektory, których nie zawęża żadna klasa ani id, testuje na elementach w miarę ich dodawania.

Działają tu dwa rodzaje wyjątków, które konwersja `adblock` do WebKit gubi: `$generichide` i wyjątki elementów
`#@#`. Pomiar po zmianie (względem stałej porcji pracy JS): układ 0,64 i przeliczenie stylów 0,20 z Guardem
wobec 0,66 i 0,25 bez niego. Domeny wyłączone dla ogólnego selektora porównujemy z hostem ramki, w której działa
skrypt.

![Listy Guarda są dzielone: blokowanie i ukrywanie dla konkretnych witryn trafia do WebKit z jedną regułą sondy na listę; ogólne selektory trafiają do indeksu JSON; skrypt strony sprawdza sondę, zgłasza klasy i id strony do handlera w procesie UI i wstawia otrzymane selektory](../../assets/diagrams/architecture-guard-generic-hiding.pl.svg)

**Ochrona przed fingerprintingiem.** Konfiguracja DuckDuckGo dla macOS zostawia wyłączone zabezpieczenia
C-S-S dla canvas, audio, baterii, rozmiaru ekranu, pamięci tymczasowej i referrera, bo na macOS WebKit Safari
dodaje własny szum; WebKitGTK tego nie robi. `FingerprintHardening` (rdzeń) je włącza i zachowuje wyjątki stron
DuckDuckGo (np. banki, których kontrola nadużyć czyta canvas) oraz ustawienia. Szum canvas czeka dodatkowo na
ustawienie `additionalEnabledCheck`, którego w konfiguracji DuckDuckGo dla macOS nie ma. Zmierzone: odczyty
canvas i audio różnią się w każdej sesji i są identyczne w obrębie strony, więc strony, które rysują, się nie
psują.

**Nagłówek GPC.** C-S-S ustawia tylko `navigator.globalPrivacyControl`, którego serwery strony nigdy nie
widzą. Funkcja WebKit `GlobalPrivacyControl`, domyślnie wyłączona, wysyła `Sec-GPC: 1` z każdym żądaniem. W tej
postaci sygnał uznają przepisy takie jak kalifornijskie CCPA. Zmierzone: nagłówek dociera.

**Pomiar w porównaniu z Brave.** Dwanaście serwisów informacyjnych, zakupowych i pogodowych (sześć polskich,
sześć zagranicznych) otwarto po razie przez rejestrujące proxy. Proxy zapisuje wyłącznie nazwy hostów, z którymi
łączy się przeglądarka. Host liczy się jako śledzący, gdy lista Disconnect zalicza go do reklam, analityki,
fingerprintingu, sieci społecznościowych albo koparek kryptowalut. „Zewnętrzny” oznacza każdy host spoza domeny
rejestrowalnej odwiedzanej strony.

| Przeglądarka (listopad 2026) | Hosty śledzące | Hosty zewnętrzne |
|---|---|---|
| Brave 1.98.63, Shields w ustawieniu domyślnym („standard”) | 6 | 82 |
| ZTBrowser przed Guardem (tylko ochrona DuckDuckGo) | 14 | 146 |
| ZTBrowser z Guardem | 3 | 70 |

Przeszły trzy hosty. `pub.doubleverify.com` (BBC) i `securepubads.g.doubleclick.net` (weather.com) przepuścił też
Brave. Trzeci to `m.stripe.network` (CNN), czyli antyfraudowa weryfikacja płatności Stripe. Wczytanie CNN w Brave
nie dotarło do żadnego hosta zewnętrznego, co wygląda na to, że serwis odprawił przeglądarkę headless. Bez CNN w
obu sumach wychodzi 6 / 82 dla Brave i 2 / 57 dla Guarda. To jedno wczytanie na stronę, a nie benchmark, ale
kierunek jest wyraźny: Guard łączy się z o połowę mniejszą liczbą trackerów niż Brave.

**Czego Guard nie robi.**

- Scriptlety uBlock Origin (`##+js(…)`) oraz filtry `$redirect`, `$removeparam` i `$csp` nie mają
  odpowiednika w content blockerach i odpadają przy konwersji. Brave uruchamia scriptlety.
- Strony nadal mogą sprawdzić, jakie fonty są zainstalowane. Flaga WebKit `ShouldAllowUserInstalledFonts`
  niczego nie zmieniła, bo dystrybucje Linuksa instalują fonty dla całego systemu. Brave ogranicza fonty w
  trybie standardowym.
- Lista AdGuard Tracking Protection (406 000 reguł) przekracza limit WebKit dla jednej listy.
- Każdy profil i aplikacja webowa kompiluje własną kopię list Guarda, ok. 67 MB w swoim katalogu cache.

## 6. Aplikacje webowe

**Instalacja.** „Install Site as App” (przycisk na prawym końcu paska adresu na każdej stronie WWW albo menu
okna) czyta tytuł strony, link do manifestu i linki do ikon skryptem
działającym w *izolowanym świecie JavaScript*, którego sama strona nie widzi i nie może zmienić. Manifest i
ikony są pobierane bez cookies, z limitami rozmiaru (manifest 512 KiB, ikona 2 MiB). Dekodowane są tylko ikony
PNG i tylko po tym, jak `PngHeader` potwierdzi rozsądne wymiary (najwyżej 2048 px na bok), co zatrzymuje
„bomby dekompresyjne”. Wybrana ikona jest ponownie kodowana do PNG i zapisywana. Gdy żadna ikona się nie
pobierze, używana jest favikona, którą WebKit już zdekodował. Wynikiem jest wpis w katalogu, ikona i plik
desktop, którego linia `Exec=` uruchamia `ztbrowser --web-app <id>`.

Po instalacji aplikacja się uruchamia, a przeglądarka przekazuje jej witrynę: każde okno przeglądarki tego
profilu zamyka kartę, z której zaczęła się instalacja, i każdą kartę ze stroną aplikacji
(`InstalledWebApp::is_own_page`: to samo pochodzenie, czyli schemat, host i port, co adres startowy aplikacji).
Okno, w którym nie została żadna karta, się zamyka. Test jest celowo węższy niż witryna aplikacji: aplikacja Gmail
posiada `mail.google.com`, więc karta Dokumentów czy Kalendarza Google z tej samej witryny zostaje w przeglądarce.
Gdy aplikacji nie udało się uruchomić, nic się nie zamyka, a komunikat proponuje jej otwarcie.

**Nawigacja.** `WebAppNavigationPolicy` (rdzeń) decyduje, dokąd może prowadzić nawigacja w aplikacji:

- **link kliknięty przez użytkownika** do innej *witryny* (domeny rejestrowalnej) otwiera się w zwykłej
  przeglądarce, więc linki z maili i reklamy nigdy nie działają w sesji banku;
- **przekierowania i nawigacje skryptów** zostają, bo logowanie (Gmail przez `accounts.google.com`) i
  płatności kartą (strony 3-D Secure) przeskakują między witrynami;
- **wyskakujące okna** otwierane przez aplikację (OAuth, potwierdzenie płatności) otwierają się w małym oknie
  tej samej aplikacji. Dzielą jej sesję, więc mają tę samą bramkę, mierzoną względem witryny, dla której okno
  otwarto (linki na stronie 3-D Secure działają, kliknięty link do trzeciej witryny otwiera się w
  przeglądarce), i ten sam wskaźnik witryny w nagłówku.

Przekazanie strony do przeglądarki zamyka okno, *z którego strona się przeniosła*, i tylko je. **Open Page in
Browser** przenosi bieżącą stronę aplikacji, więc okno aplikacji się zamyka. Wyskakujące okno, którego własna
strona trafia do przeglądarki, też się zamyka, bo pokazywałoby już tylko stronę, którą opuściło. Okno aplikacji
zostaje, gdy link w nim otwiera się w przeglądarce, bo aplikacja nadal jest na swojej stronie (skrzynka pocztowa
musi przetrwać kliknięcie linku w wiadomości). Zostaje też wyskakujące okno, którego link prosił o *nowe* okno
(`target=_blank`), bo jego strona, np. potwierdzenie płatności, jest wciąż potrzebna. Okno zamyka się dopiero
po uruchomieniu przeglądarki.

Przekierowanie może wyprowadzić aplikację poza jej witrynę, dlatego podtytuł okna zawsze pokazuje host
widocznej strony. Gdy strona należy do innej witryny, podtytuł zmienia kolor na ostrzegawczy i dostaje
wyjaśniający tooltip. Aplikacja banku nie może więc niepostrzeżenie pokazać obcej strony pod nazwą banku.

Test „ta sama witryna” używa listy sufiksów publicznych z libsoup, tej samej, której WebKit używa dla cookies,
więc aplikacja zgadza się z WebKit co do tego, czym jest witryna. Wywoływana jest tylko
`soup::tld_domain_is_public_suffix`. Wiązanie `soup::tld_get_base_domain` z crate'a `soup3` (0.9) czyta wynik
przez wskaźnik do tymczasowego, już zwolnionego napisu i wywracało aplikacje webowe przy pierwszej nawigacji.

## 7. Pobieranie plików, uprawnienia i inne aplikacje

- **Pobrane pliki** trafiają do katalogu XDG Downloads pod oczyszczoną nazwą (`DownloadFileName`: bez
  separatorów ścieżek, znaków sterujących i kropek na początku, najwyżej 255 bajtów, `nazwa (2).ext` zamiast
  nadpisania). Nigdy nie są otwierane automatycznie; komunikat proponuje „Show in Folder”.
- **Uprawnienia** (lokalizacja, powiadomienia, schowek, kamera, mikrofon, udostępnianie ekranu, dostęp do
  magazynu) są za każdym razem pytane w oknie dialogowym i nigdy nie są przyznawane po cichu. Blokada kursora
  jest dozwolona, wszystko inne odrzucane.
- **Inne schematy** (`mailto:`, `tel:`, `zoommtg:` …) są przekazywane do pulpitu tylko po prawdziwym geście
  użytkownika, więc strona nie może sama otwierać aplikacji.
- **Wyszukiwarka**: domyślnie DuckDuckGo; w każdym profilu można wybrać Brave Search albo Google (menu okna →
  Search Engine, zapis w `search-engine.json` profilu). `SearchEngine` decyduje, dokąd trafia wyszukiwanie
  wpisane w pasek adresu lub podane w wierszu poleceń, jaką stronę główną otwiera nowa karta i jaką wyszukiwarkę
  strona błędu proponuje dla błędnie wpisanej nazwy. Wyszukiwanie jest zwykłą nawigacją, więc ochrona działa też
  na stronach samej wyszukiwarki. Zapisana wyszukiwarka, której ZTBrowser nie oferuje, jest czytana jako DuckDuckGo.
- **Przyciski paska narzędzi**: *Preferences…* (menu okna, `Ctrl+,`) włącza i wyłącza każdy przycisk wokół
  paska adresu osobno w każdym profilu (`ToolbarLayout`, zapis w `toolbar.json` profilu; `ToolbarPreferences`
  dzieli go ze wszystkimi oknami procesu, więc przełącznik działa we wszystkich naraz). Plik wymienia przyciski
  *ukryte*, więc przycisk dodany w późniejszej wersji pokaże się sam. Paska adresu i menu nie da się ukryć, bo
  inaczej nie dałoby się wrócić do preferencji. Przycisk z cebulą otwiera okno Tor (§4.1); samo okno Tor nie
  należy do żadnego profilu, pokazuje wszystkie przyciski i nie oferuje preferencji.
- **Zoom strony** przechodzi przez rozmiary DuckDuckGo (`ZoomLevel`: 50, 75, 85, 100, 115, 125, 150, 175, 200, 250
  i 300 %) skrótami Ctrl `+`/`-`/`0` (także na klawiaturze numerycznej), Ctrl z kółkiem myszy albo rzędem zoomu
  w menu okna. Rozmiar inny niż 100 % widać jako znaczek obok paska adresu (w aplikacji webowej w nagłówku);
  kliknięcie wraca do 100 %. Jak w przeglądarce DuckDuckGo na macOS rozmiar należy do *witryny*
  (`SiteZoomLevels`, klucz to domena rejestrowalna, zapis per profil w `site-zoom-levels.json`): każda strona
  witryny otwiera się w nim, a jego zmiana od razu przeskalowuje pozostałe otwarte karty tej witryny. Bez tego
  WebKit przenosiłby zoom jednej karty na każdą witrynę, którą karta odwiedzi później. Strony spoza sieci
  (`file:`, `about:`) otwierają się w 100 % i nie są zapamiętywane. Ctrl z kółkiem jest przechwytywany, zanim
  zobaczy go strona, więc strona (np. mapa) nie może użyć tego gestu dla siebie; rozsunięcie palców na touchpadzie
  nadal powiększa stronę tak, jak robi to WebKit.

## 8. Co WebKitGTK potrafi, a czego nie (zmierzone na 2.54)

Powyższy projekt wynika z tych pomiarów. Każdy sprawdzono na działającym WebKitGTK 2.54, zanim kod zaczął na
nim polegać.

| Ustalenie | Konsekwencja |
|---|---|
| Sygnał `decide-policy` nie odróżnia nawigacji ramki głównej od iframe i nie pokazuje nagłówków `Sec-Fetch-*`. | Upgrade HTTPS i czyszczenie linków działają przez reguły treści, nie w haku nawigacji. |
| `make-https` działa przed wysłaniem żądania, we wszystkich ramkach. | To jest mechanizm upgrade HTTPS. |
| `redirect` (z `remove-params`) i `modify-headers` kompilują się, ale nic nie robią. | Parametry śledzące są blokowane, a strona wczytuje oczyszczony URL. |
| Funkcja `GlobalPrivacyControl` (status „testable”, domyślnie wyłączona) wysyła `Sec-GPC: 1` z każdym żądaniem. | Guard ją włącza; tak jest wysyłany nagłówek GPC. |
| `block` z `resource-type: document` i `load-context: top-frame` blokuje tylko nawigacje ramki głównej, przed żądaniem, i zgłasza błąd polityki 104. | Tak działają reguły `links`. |
| Dla wczytań ramki głównej `unless-domain` jest porównywane z celem. | Wyjątki reguł `links` wskazują witryny docelowe. |
| `ignore-previous-rules` sięga wstecz tylko w obrębie własnej listy. | Osobne listy dla trackerów, HTTPS, linków i każdej grupy Guarda. |
| Reguła `ignore-previous-rules` pasująca do wczytania samego dokumentu strony anuluje też wcześniejsze reguły `css-display-none`. | Guard przesuwa wyjątek `adblock` dla dokumentów pierwszej strony przed reguły ukrywania. |
| Lista mieści najwyżej 150 000 reguł; pusta lista jest odrzucana („Empty extension”); `url-filter` nie zna alternatywy `\|`. | Guard grupuje listy filtrów w trzy listy i dopasowuje każdą do limitu. |
| `resource-type: ping` pasuje tylko do `<a ping>`; `navigator.sendBeacon` i `fetch(…, {keepalive})` to `raw`. | Guard blokuje pingi kliknięć linków; beacony tylko po adresie. |
| Kompilacja trzech list Guarda (91 000, 59 000 i 27 000 reguł) trwa ok. 3, 4 i 1,5 s. | Skompilowane listy są zapisywane i używane ponownie; strony nigdy na nie nie czekają. |
| Wyłączenie `gtk-enable-animations` wewnątrz aplikacji nie zmienia stronom `prefers-reduced-motion`. | Animacje interfejsu (karty, dialogi, banery, toasty, pasek wyszukiwania) są wyłączone w każdym procesie ZTBrowsera, więc interfejs reaguje od razu, a witryny nie dowiadują się z tego niczego nowego o użytkowniku. |

## 9. Znane ograniczenia

| Ograniczenie | Dlaczego | Wrócić do tego, gdy |
|---|---|---|
| Parametry śledzące nie są usuwane, gdy strona, która zaczęła nawigację, jest na liście wyjątków (aplikacja macOS sprawdza inicjatora). | Reguły treści widzą tylko cel. | WebKitGTK udostępni inicjatora w regułach lub w haku nawigacji. |
| Formularz `POST`, którego URL akcji ma parametr śledzący, jest ponawiany jako `GET` oczyszczonego URL. | Zablokowana nawigacja jest wczytywana ponownie przez `load_uri`. | Pojawi się zgłoszenie realnej awarii. |
| Odpowiedzi na prośby o uprawnienia nie są zapamiętywane. | Pytanie za każdym razem to bezpieczna wartość domyślna. | Powstanie projekt przechowywania uprawnień per profil. |
| Brak przywracania sesji, zakładek i okna historii. Guard ukrywa banery cookies, ale na nie nie odpowiada (autoconsent DuckDuckGo klika „odrzuć”), więc strona blokująca przewijanie do czasu odpowiedzi zostaje zablokowana. | Jeszcze niezbudowane. | Następny etap. |
| Guard nie uruchamia scriptletów uBlock Origin i nie ukrywa listy zainstalowanych fontów (§5.6). | Content blockery WebKit nie wyrażają scriptletów; WebKitGTK nie oferuje ograniczenia fontów. | WebKitGTK zyska któreś z nich. |
| Wszystkie tłumaczenia są maszynowe, także polskie (patrz [Język interfejsu](interface-language.md)). | Brak native speakerów dla 66 języków. | Współtwórcy poprawią pliki `.po`. |
| Każdy profil jest w doku osobną aplikacją. | Jeden identyfikator aplikacji na proces profilu. | Użytkownicy będą woleli grupowanie. |
| Logowanie, które w aplikacji webowej zaczyna się od *klikniętego linku* do innej witryny, otwiera się w przeglądarce. | Kliknięte linki poza witrynę celowo opuszczają aplikację (phishing). Większość logowań używa przekierowań lub okien i działa. | Realna aplikacja będzie tego potrzebować; wtedy allowlista per aplikacja. |
| Okno Tor ukrywa adres użytkownika, ale nie to, że to ZTBrowser: jego odcisk (WebKitGTK, rozmiar okna, fonty) różni się od Tor Browsera, którego użytkownicy są upodabniani do siebie. | WebKitGTK nie ma letterboxingu ani łatek ujednolicających Tor Browsera. | Gdy liczy się nieodróżnialność wśród użytkowników Tora, używaj Tor Browsera. |
| Witryny odwiedzane w jednej tożsamości dzielą obwody Tora (Tor Browser rozdziela je per witryna). | WebKitGTK ustawia jedno proxy, a z nim jeden zestaw danych logowania SOCKS, na sesję. | WebKitGTK udostępni dane logowania proxy per żądanie. |
| Paczka Debiana i AppImage niosą własny `tor`, aktualizowany z wydaniami ZTBrowsera, a nie z poprawkami bezpieczeństwa Debiana. | Paczka `tor` Debiana uruchamiałaby ogólnosystemową usługę Tor. | Debian dostarczy program bez włączania usługi. |

## 10. Granice zaufania

- **Dane z sieci** (TDS, konfiguracja, listy filtrów, manifesty, ikony) mają limity rozmiaru, są parsowane
  defensywnie i walidowane przed użyciem. TDS i konfiguracja są pobierane wyłącznie przez HTTPS z CDN
  DuckDuckGo, listy filtrów przez HTTPS od ich wydawców. Lista filtrów może tylko blokować albo ukrywać:
  konwersja nie zachowuje scriptletów, więc wroga lista mogłaby psuć strony, ale nigdy nie uruchomi w nich
  kodu.
- **Strony** nigdy nie widzą skryptu instalatora (izolowany świat). Wiadomości C-S-S dostają tylko stałe
  odpowiedzi, a strony nie mogą otwierać innych aplikacji bez gestu użytkownika.
- **Pliki** zapisywane przez ZTBrowser powstają z uprawnieniami `0600` w katalogach `0700`. Najpierw trafiają do
  pliku `.part`, a potem dostają docelową nazwę, więc awaria nie zostawi w połowie zapisanego katalogu.
- **Pliki desktop** generuje `DesktopEntry`, który escapuje nazwy pochodzące ze stron. Tytuł strony nie może
  więc wstrzyknąć do pliku linii takiej jak `Exec=`.
- **Tor okna Tor** nasłuchuje tylko na pętli zwrotnej, a ZTBrowser przyjmuje od niego tylko adresy pętli
  zwrotnej. Jego port kontrolny wymaga cookie, które Tor zapisuje w swoim katalogu danych `0700`. Linie mostów
  są sprawdzane słowo po słowie, zanim trafią do pliku konfiguracji Tora, a `tor` i jego transporty są
  uruchamiane tylko ze stałych katalogów, nigdy wyszukiwane przez `PATH`.
