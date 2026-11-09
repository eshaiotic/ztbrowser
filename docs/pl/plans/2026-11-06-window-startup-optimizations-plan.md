# Optymalizacje startu okna — plan

**Status:** propozycje. **Wymaganie (właściciel, 2026-11-06): okno pojawia się w mniej niż 0,3 s.** Nic z sekcji 1–6
nie jest zatwierdzone ani rozpoczęte; wdrożenie danej sekcji zaczyna się dopiero po wyraźnej akceptacji. Angielski oryginał: `docs/en/plans/2026-11-06-window-startup-optimizations-plan.md`.

## Problem, prostymi słowami

Po kliknięciu ZTBrowsera w menu aplikacji musi pojawić się okno. Czas od kliknięcia do pierwszego obrazu na ekranie to
**czas do pierwszej klatki**. To opóźnienie, które człowiek czuje, więc właśnie je ten plan ma skrócić. Przeglądarka
startuje wolno z przyczyn strukturalnych: zanim pokaże stronę, musi uruchomić toolkit interfejsu (Qt), uruchomić
Chromium (silnik www działający w kilku procesach) i złożyć reguły prywatności, według których sprawdzane jest każde
żądanie. Strona nie może się wczytać, zanim te reguły zaczną obowiązywać, więc ostatniego kroku nie da się pominąć,
tylko przyspieszyć lub nałożyć na inną pracę.

## Słownik

- **Pierwsza klatka** — pierwszy moment, w którym program oddaje kompozytorowi Wayland gotowy obraz swojego okna.
  Mierzona jako pierwsza linia `wl_surface#N.attach(wl_buffer#M…)` w wyniku `WAYLAND_DEBUG=client`.
- **Ścieżka krytyczna** — łańcuch kroków, które muszą kończyć się jeden po drugim; cały start przyspiesza tylko
  skrócenie kroku *na tej ścieżce*. Kroki działające obok (w wątku roboczym) nic nie kosztują, dopóki same nie staną
  się najdłuższym łańcuchem.
- **Polityki** — obiekty, które dla każdego żądania i ciasteczka decydują, czy może przejść (reguły trackerów, listy
  filtrów Guard, lista wyjątków witryn). Chromium odrzuca wszystko, dopóki `QtWebEngine::install_policies` ich nie
  przekaże; to odrzucanie sprawia, że w międzyczasie można bezpiecznie robić inne rzeczy.
- **Silnik Guard** — skompilowana postać społecznościowych list filtrów (`SharedGuardEngine`); powstaje we własnym
  wątku, a strony wczytane przed jego gotowością nie są filtrowane.
- **AppImage / squashfs / FUSE** — AppImage to jeden plik z skompresowanym systemem plików tylko do odczytu
  (squashfs). Uruchomienie montuje ten system plików przez FUSE (sterownik systemu plików w przestrzeni użytkownika)
  i rozpakowuje pliki programu na żądanie. Paczka dystrybucji nie ma nic z tego: jej pliki leżą na dysku zwyczajnie.
- **Zimna pamięć podręczna stron** — system operacyjny trzyma w pamięci niedawno czytane zawartości plików. Świeżo
  zamontowany AppImage startuje bez niczego w pamięci, więc każde uruchomienie płaci za odczyt i rozpakowanie od nowa.
- **Proces renderera i GPU** — Chromium wykonuje treść stron i grafikę w osobnych procesach, uruchamianych mniej
  więcej wtedy, gdy powstaje pierwszy widok www (sekcja 1 powinna potwierdzić dokładny moment).

## Stan wyjściowy (zmierzone 2026-11-06, KDE Plasma na Waylandzie, NVMe)

Czas do pierwszej klatki, uruchomienia przeplatane, każde z własnymi katalogami `XDG_*` i kopią pamięci podręcznej
ochrony (żeby nie liczyć pobierania przy pierwszym starcie):

| Wersja | Uruchomień | Mediana | Zakres |
|---|---|---|---|
| Paczka Arch (`/usr/bin/ztbrowser`, budowana z `target-cpu=native` i `lto = "fat"` z profilu Cargo) | 4 | 0,72 s | 0,55–0,89 s |
| AppImage 1.2.3 z `dist/` | 4 | 1,73 s | 1,23–2,12 s |

Wcześniej tego samego dnia, w innej serii: paczka 0,61 s, AppImage 1,47 s, a *te same pliki rozpakowane do katalogu*
0,65 s. Samo okno Qt Quick otwiera się w 0,11 s, a okno z pustym widokiem www w 0,50 s (sekcja o wymaganiu niżej; 176–275 ms
z `docs/pl/reports/performance-analysis.md` §4.2 to starszy pomiar porównania z czasów GTK), więc około 0,6 s startu
paczki to więcej, niż potrzebuje gołe okno, i około 0,2 s więcej niż potrzebuje gołe okno z widokiem www.
**Jeszcze nie zmierzono:** startu przez launcher KDE, drugiego uruchomienia, które tylko przekazuje adres do działającej
instancji, ani tego, gdzie w tych 0,72 s ucieka czas (sekcja 1).

![Start okna: ścieżka krytyczna i miejsca, w których działają planowane prace](../../assets/diagrams/plan-window-startup-critical-path.pl.svg)

## Wymaganie: poniżej 0,3 s

Właściciel wymaga startu poniżej **0,3 s**. Plan rozumie to jako **czas do pierwszej klatki** (miara ze stanu
wyjściowego), a nie czas do ukazania się strony; strona nie może pojawić się tak wcześnie (zob. niżej). Proszę
potwierdzić lub poprawić tę interpretację, zanim sekcja 3 zostanie zatwierdzona.

**Czy to osiągalne? Zmierzone dolne granice** (2026-11-06, ten sam protokół co stan wyjściowy, po 6 uruchomień,
program `qml6` z Qt, więc to dolne granice, a nie prognozy dla własnego kodu startu ZTBrowsera):

| Minimalny program | Mediana | Zakres |
|---|---|---|
| Samo okno Qt Quick | 0,11 s | 0,10–0,15 s |
| Okno z pustym `WebEngineView` (`about:blank`) | 0,50 s | 0,33–0,54 s |
| ZTBrowser, paczka Arch (stan wyjściowy wyżej) | 0,72 s | 0,55–0,89 s |

Co z tego wynika:

1. **Sam QtWebEngine kosztuje około 0,4 s**, zanim okno, które go zawiera, pokaże pierwszą klatkę. Pierwsza klatka
   czekająca na widok www nie zmieści się więc w 0,3 s, choćby cała reszta była dopracowana; ZTBrowser jest już tylko
   około 0,2 s ponad gołym oknem z widokiem www.
2. **Cel jest osiągalny tylko przez pokazanie okna najpierw i uruchomienie silnika www za nim** — pojawia się pasek
   narzędzi i pasek kart, obszar treści jest jeszcze pusty. To sekcja 3, którą cel zamienia z opcji w główną pracę.
   Budżet na wszystko, co jest na drodze do tej pierwszej klatki (exec i dynamiczne łączenie dużych bibliotek
   Chromium, Qt, nasz QML, sprawdzenie jednej instancji), to około 0,3 − 0,11 ≈ 0,19 s; sekcja 1 musi zmierzyć, ile
   z tego zużywa dziś.
3. **Strona i tak pojawi się później**, po instalacji polityk i utworzeniu widoku www (według powyższej dolnej
   granicy około 0,5 s lub więcej). Plan śledzi *czas do strony* jako drugą liczbę obok pierwszej klatki, żeby cel
   nigdy nie został osiągnięty przez ukrycie oczekiwania.
4. **AppImage nie osiągnie 0,3 s żadną dotąd zmierzoną opcją** (najlepiej: 0,65 s z rozpakowanymi plikami, sekcja 5).
   Czy start z oknem na pierwszym miejscu usuwa też z drogi do pierwszej klatki koszt montowania i rozpakowywania
   AppImage, nie jest zmierzone. Dopóki nie jest, wymaganie odnosi się do paczek.

## Już zrobione

- **Reguły i profil Chromium nakładają się** (commit `4fee5c9`). Reguły są składane w wątku roboczym, gdy wątek
  główny otwiera profil Chromium; okno otwiera się dopiero po zakończeniu obu. Zapisane dokumenty są czytane raz,
  a nie dwa razy.
- **Silnik Guard powstaje po pobraniu przy pierwszym starcie.** Nakładanie najpierw budowało go z pamięci podręcznej,
  zanim wątek roboczy cokolwiek pobrał, więc pierwszy start działałby z pustymi listami Guard aż do następnego
  sprawdzenia po sześciu godzinach. `ProtectionUpdates::starting_protection` buduje go teraz po pobraniu; dwa testy
  scenariuszowe w `crates/ztbrowser-qt/tests/protection_updates.rs` zawodzą, jeśli ta kolejność zostanie utracona.
- **Wpis menu nie przesłania już paczki** (commit `d92f9a0`). Wpis zapisany wcześniej przez AppImage miał
  pierwszeństwo przed wpisem zainstalowanej paczki i dalej uruchamiał wolniejszy AppImage; AppImage wycofuje teraz
  swój wpis, gdy paczka ma wpis w całym systemie.

## Sprawdzone i odrzucone dla AppImage (zmierzone wcześniej, nie powtarzać)

| Pomysł | Wynik |
|---|---|
| Kompresja zstd na poziomie 3 | −10 % czasu startu, +28 MB |
| Bloki squashfs 1 MiB | bez zysku |
| Czytanie dużych bibliotek w tle z `AppRun` | bez zysku |
| Kompresja lz4 | runtime typu 2 obsługuje tylko zlib i zstd |
| Squashfs bez kompresji | 0,79 s, ale 542 MB |

## 1. Najpierw zmierzyć: nazwane punkty czasowe

**Co.** Zapisywać monotoniczny czas w nazwanych punktach startu — start procesu, utworzenie `QGuiApplication`,
otwarcie sesji, start i dołączenie wątku roboczego, powrót z `open_profile`, instalacja polityk, utworzenie okna,
pierwsza klatka — i wypisywać je, gdy poprosi o to zmienna środowiskowa. Pomiar z logu Wayland, użyty do stanu
wyjściowego, zamienić w małe, zatwierdzone narzędzie deweloperskie obok `ztbrowser-test-report`, żeby każdą kolejną
liczbę dało się odtworzyć.

**Dlaczego.** Stan wyjściowy mówi, że paczka potrzebuje około 0,6 s więcej niż samo okno Qt Quick, ale nie *gdzie*.
Sekcje 2–4 są zgadywaniem, dopóki nie ma tego rozkładu, a zgadnięcie kroku poza ścieżką krytyczną niczego nie zmienia.
Ta sekcja jest bramką dla 2 i 4: każda rusza tylko wtedy, gdy jej krok stanowi zauważalną część drogi do pierwszej
klatki. Sekcja 3 jest wymagana przez cel 0,3 s w każdym przypadku; rozkład powie, ile z budżetu 0,19 s jej zostaje.
Dodaje też *czas do strony* jako drugi punkt.

**Gotowe, gdy.** Tabela w raporcie wydajności pokazuje udział każdego punktu, z co najmniej dziesięciu uruchomień.

## 2. Skrócić to, co działa przed startem wątku roboczego

**Co.** `BrowserApplication::install_policies` wywołuje `open_session` w wątku głównym, zanim uruchomi wątek roboczy.
`open_session` kończy spalenie danych zleconych przyciskiem Fire i czyta zapisane wybory profilu; wątek roboczy
potrzebuje z niej tylko listy witryn bez ochrony i uchwytu Guard sesji. Jeśli sekcja 1 pokaże `open_session` jako
realny koszt, podzielić ją tak, by wątek roboczy mógł ruszyć wcześniej.

**Ryzyko.** Spalenie danych musi nadal kończyć się, zanim cokolwiek otworzy pliki profilu; podział musi zachować tę
kolejność.

## 3. Pokazać okno najpierw, stronę potem (wymagane dla celu 0,3 s)

**Co.** Dziś okno powstaje po złączeniu, więc pierwsza klatka czeka na profil Chromium, reguły i widok www. Nowa
kolejność: Qt startuje, pokazywane jest okno **bez żadnego widoku www** (pasek narzędzi, pasek kart, pusty obszar
treści — to pierwsza klatka), a dopiero potem `open_profile` działa obok wątku roboczego. Po złączeniu i
`install_policies` powstaje pierwszy widok www (wtedy startują jego procesy renderera i GPU) i zaczyna się
wczytywanie pierwszej strony. W QML widok www karty musi więc powstawać na żądanie (loader, który aplikacja włącza
po `install_policies`), a nie być częścią okna od początku.

**Dlaczego.** Zmierzone dolne granice (sekcja o wymaganiu) pokazują, że pierwsza klatka z widokiem www nie może być
szybsza niż około 0,5 s, a gołe okno potrzebuje około 0,11 s. Raport z czasów GTK proponował ten sam pomysł z własnych
powodów (`performance-analysis.md` §6, propozycja 2, „do 200 ms do strony”).

**Ryzyko i reguły, które zostają.** Dopóki polityki nie są zainstalowane, Chromium odrzuca każde żądanie, a okno do
tego czasu nie ma widoku www, więc nic nie może się wczytać bez ochrony. Wczytanie pierwszej strony trzeba *zacząć*
po `install_policies`; pierwszy start nadal musi pobrać dane przed jakąkolwiek stroną; okno Tor nadal nie może
niczego wczytać przed połączeniem z Tor. Widoczne okno z pustym obszarem treści przez kilkaset milisekund to decyzja
interfejsu: pusty obszar musi wyglądać na zamierzony (własne tło karty i już wypełniony pasek adresu), a nie na
zepsuty, a Fire, wyszukiwanie, powiększenie i pobieranie nie mogą sięgać po widok www, którego jeszcze nie ma.

**Gotowe, gdy.** Pierwsza klatka jest poniżej 0,3 s w rozkładzie z sekcji 1, a *czas do strony* nie jest gorszy niż
dziś.

## 4. Sprawdzić, czy QML jest kompilowany z wyprzedzeniem

**Co.** `crates/ztbrowser-qt/build.rs` wylicza pliki QML interfejsu. Qt potrafi skompilować QML do kodu bajtowego lub
C++ podczas budowania (`qmlcachegen`), żeby start nie parsował i nie kompilował go od nowa; czy nasza budowa to robi,
jest dziś **nieznane**. Ustalić (kategorie logowania QML w Qt mogą pokazać, czy wczytano skompilowane jednostki),
a jeśli nie, włączyć.

**Dlaczego może mieć znaczenie.** Interfejs ma około dwudziestu plików QML; parsowanie ich przy każdym starcie byłoby
czystą, powtarzaną pracą. **Mierzyć** punktem „okno utworzone” z sekcji 1 przed zmianą i po niej.

## 5. AppImage: rozpakować raz, uruchamiać z pamięci podręcznej

**Co.** AppImage kosztuje około sekundę więcej niż paczka (1,73 s wobec 0,72 s), bo każde uruchomienie montuje
squashfs przez FUSE i rozpakowuje na żądanie; te same pliki rozpakowane do katalogu startują w 0,65 s. `AppRun`
mógłby przy pierwszym starcie skopiować drzewo do `${XDG_CACHE_HOME:-~/.cache}/ztbrowser/appimage-<build>/`
(rozpakować do sąsiedniego katalogu, potem zmienić mu nazwę na docelową), a przy kolejnych startach robić `exec`
z kopii.

**Koszty do rozstrzygnięcia z właścicielem.** Około 540 MB dysku na wersję (obraz bez kompresji ma 542 MB), wolniejszy
pierwszy start o czas kopiowania (niezmierzony), sprzątanie starych wersji oraz zaufanie: skopiowane drzewo jest
zapisywalne przez osobę, więc obowiązuje ta sama reguła co dla wpisu menu (`AppImageIntegration`: tylko ścieżki, które
należą do roota lub osoby i do których nikt inny nie może pisać). Paczki pozostają szybką drogą na Archu i Fedorze;
to pomaga tylko użytkownikom AppImage.

## 6. Zmierzyć ścieżkę przez launcher i drugie uruchomienie

**Co.** Zmierzyć start przez launcher KDE (`kioclient exec <plik desktop>`) w porównaniu z bezpośrednią binarką oraz
czas od drugiego wywołania `ztbrowser` do okna już działającej instancji (gniazdo jednej instancji przekazuje
adresy). Może tu nie być nic do optymalizowania; dopóki nie zmierzono, raport tylko by zgadywał.

## Reguły, które musi zachować każdy kandydat

- Żadne żądanie ani ciasteczko nie przechodzi przed instalacją polityk; żadna strona nie wczytuje się bez ochrony
  (odrzucanie przez Chromium do `install_policies` sprawia, że nakładanie jest bezpieczne).
- Pierwszy start nadal pobiera dane przed pierwszą stroną, a silnik Guard powstaje z tego, co pobrano, nigdy
  z pustej pamięci podręcznej.
- Okno Tor nigdy nie wczytuje się przed połączeniem z Tor i nigdy nie wraca do połączenia bezpośredniego.
- Piaskownica Chromium, izolacja witryn i proces GPU zostają włączone: żadnego `--no-sandbox` ani `--single-process`
  dla szybkości.
- Każda zmiana ma najpierw test scenariuszowy (Given/When/Then), potem implementację.

## Jak mierzyć (protokół za stanem wyjściowym)

1. Każde uruchomienie dostaje własne `XDG_DATA_HOME`, `XDG_CACHE_HOME`, `XDG_CONFIG_HOME` i `XDG_RUNTIME_DIR`, żeby
   prawdziwe profile osoby i gniazdo jednej instancji pozostały nietknięte; `WAYLAND_DISPLAY` ustawić na bezwzględną
   ścieżkę gniazda, bo prywatny katalog uruchomieniowy go nie ma.
2. Skopiować prawdziwą pamięć podręczną ochrony do prywatnego katalogu pamięci podręcznej, inaczej liczy się
   pobieranie przy pierwszym starcie.
3. Uruchamiać z `WAYLAND_DEBUG=client`, zapisać monotoniczny czas startu procesu i pierwszej linii
   `wl_surface#N.attach(wl_buffer#M`. Format logu różni się między wersjami libwayland (tu `#`, w starszych `@`):
   przed zaufaniem tabeli obejrzeć jeden log.
4. Przeplatać porównywane wersje, uruchamiać każdą co najmniej cztery razy (dziesięć przy decyzji), podawać medianę
   i zakres, a proces zabijać po PID grupy (nigdy `pkill -f` ze wzorcem, który występuje też w poleceniu go
   uruchamiającym).

## Dziennik decyzji

- Nakładanie wybrano, bo reguły i profil są niezależne i były dwoma najdłuższymi krokami wątku głównego.
- Sekcja 3 jest wymagana przez cel 0,3 s, bo zmierzone dolne granice wykluczają każdą pierwszą klatkę czekającą na
  widok www. Sekcje 2 i 4 pozostają zależne od sekcji 1: prawdopodobne optymalizacje, żadna niezmierzona, to sposób,
  w jaki start robi się bardziej skomplikowany, a nie szybszy.
- Sekcja 5 to wymiana miejsca na dysku za szybkość, którą może zrobić tylko właściciel; jest wypisana, by wybór był
  widoczny, a nie dlatego, że jest bezwarunkowo polecana.
