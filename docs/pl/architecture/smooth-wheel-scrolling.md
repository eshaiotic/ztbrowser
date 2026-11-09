# Płynne przewijanie kółkiem myszy

ZTBrowser animuje każdy ząbek kółka myszy tak jak Chromium: 120 px w około 180 ms, z łagodnym startem i
hamowaniem, zamiast gwałtownych 93 px w około 90 ms w WebKitGTK.

## 1. Problem

WebKitGTK przewija ząbek kółka od razu, w sześciu–siedmiu klatkach, a pierwsze klatki są najszybsze. Obok Brave
wygląda to jak szarpnięcie: strona „skacze” przy każdym ząbku, mimo że liczba klatek jest w porządku. WebKit nie ma
ustawienia długości ani kształtu tej animacji, a wymiana silnika (QtWebEngine, CEF) została zmierzona i odrzucona:
QtWebEngine nie daje animacji Chromium, a silnik Chromium oznaczałby przepisanie interfejsu i całej warstwy
ochrony. Dlatego ZTBrowser odtwarza animację sam.

## 2. Słownik

| Pojęcie | Znaczenie |
|---|---|
| **Ząbek** | Jedno kliknięcie kółka myszy. WebKitGTK zgłasza go jako `deltaY` 93 i `wheelDeltaY` ±120. |
| **Krzywa łagodzenia** | Jak szybko biegnie animacja w danej chwili: wolno, szybko, wolno (*ease in-out*) albo szybko, a potem hamowanie (*ease out*). |
| **Tabela postępu** | 65 liczb od 0 do 1: jak daleko dotarła animacja po 0, 1/64, … 64/64 swojego czasu. |
| **Świat skryptów** | Odizolowane środowisko JavaScript na stronie: widzi DOM strony, ale strona nie widzi jego zmiennych. |

## 3. Jak to jest połączone

![Ząbek kółka przechodzi ze zdarzenia wheel przez skrypt strony do przewijania klatka po klatce](../../assets/diagrams/architecture-smooth-wheel.pl.svg)

- **`ztbrowser-core`** zawiera liczby: `SmoothWheelScroll` (120 px na ząbek, 180 ms, dwie tabele postępu zbudowane
  z `CubicBezierEasing`) oraz `ScrollingPreferences` (`smoothWheel`, domyślnie włączone). Ich testy
  (`tests/smooth_wheel_scroll.rs`, `tests/cubic_bezier_easing.rs`, `tests/scrolling_preferences.rs`) utrwalają
  zmierzony profil Chromium: pierwsza klatka przesuwa najwyżej 4 px, żadna klatka więcej niż 22 px, szczyt wypada w
  okolicy środka, suma to dokładnie jeden ząbek, a ząbek, który przychodzi w ruchu, nie zwalnia, by zacząć od nowa.
- **`SmoothWheelScript`** (aplikacja) wpisuje te liczby do `assets/smooth-wheel/smooth-wheel.js` i pakuje go jako
  skrypt użytkownika we własnym świecie skryptów, w każdej ramce.
- **`PageProtections`** instaluje go razem z pozostałymi skryptami strony, gdy dane ochrony są gotowe, więc strona
  nigdy nie startuje bez niego. **`ProfileSession`** trzyma wybór w `scrolling.json` profilu (okno Tor tylko w
  pamięci), a zmienia go *Preferencje → Wygląd → Płynne przewijanie kółkiem myszy*; strony ładowane później
  się do niego stosują.

## 4. Co robi skrypt strony

Nasłuchuje `wheel` i przejmuje wyłącznie pełny ząbek kółka myszy: `deltaMode` 0, bez składowej poziomej,
`wheelDeltaY` będące wielokrotnością 120 (touchpad wysyła ułamki, więc nigdy go nie dotyka), bez Ctrl, Shift i Alt,
zdarzenie anulowalne i niezanulowane przez stronę, a system nie prosi o ograniczenie ruchu. Szuka elementu, który
przewinąłby WebKit: najbliższego przodka, który może się przewijać w tym kierunku i ma jeszcze miejsce (inaczej
zdarzenie przeszłoby do jego rodzica), a w końcu samej strony, o ile jej overflow nie jest ukryty i nie przyciąga
(`scroll-snap-type`). Jeśli nie ma żadnego, zdarzenie trafia do WebKit bez zmian.

W przeciwnym razie anuluje zdarzenie i przesuwa element w klatkach `requestAnimationFrame` od miejsca, w którym
stoi, do celu przez `scrollTo({behavior: "instant"})` (własne `scroll-behavior: smooth` strony nie może wygładzać
go drugi raz). Ząbek, który przychodzi, gdy element jest w ruchu, dodaje 120 px do niedokończonego celu i
kontynuuje z tabelą *ease out*, więc szybka seria ząbków płynie dalej. Gdy ktokolwiek inny przewinie element
(klawiatura, pasek przewijania, skrypt strony), animacja się zatrzymuje.

## 5. Zmierzone

Jeden ząbek w tym samym teście (zdarzenia kółka X11 przez XTest, piksele przesunięte w każdej klatce 60 Hz):

| Silnik | Piksele | Klatki | Czas | Na klatkę |
|---|---|---|---|---|
| WebKitGTK (przed) | 86–93 | 6–7 | ~90 ms | 5,12,34,21,10,3,1 |
| Brave / Chromium | 120 | 11–12 | 167–183 ms | 2,5,10,13,16,18,17,15,12,7,4,1 |
| ZTBrowser (płynne kółko) | 120 | 11–12 | 161–183 ms | 1,6,9,14,17,18,18,15,12,7,2,1 |

Sprawdzone też: strona z `scroll-behavior: smooth`, przewijane pudełko wewnątrz strony (przewija się najpierw
ono) oraz 15 ząbków co 40 ms (ciągły ruch, bez zatrzymania między ząbkami).

## 6. Ograniczenia

Animacja działa na głównym wątku strony, więc na stronie z zajętym głównym wątkiem jest tak równa jak sama
strona, tam gdzie własne przewijanie WebKit pozostałoby równe; dla takiej strony można ją wyłączyć w
preferencjach. Touchpad nie jest objęty: wysyła już ciągły strumień, a jego gestów nie dało się przetestować w
środowisku deweloperskim. Skrypt nie ma własnego testu automatycznego: liczby testuje crate domenowy, zachowanie
powyższe pomiary.
