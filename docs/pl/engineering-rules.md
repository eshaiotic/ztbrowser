# Zasady inżynierskie ZTBrowsera

Te zasady dotyczą wyłącznie ZTBrowsera i uzupełniają reguły globalne z `~/.claude/rules/general.md`.
W razie sprzeczności w tym repozytorium obowiązuje ten dokument.

## 1. Object Oriented Programming First

ZTBrowser jest pisany w Ruście. Rust nie ma klas ani dziedziczenia, ale ma wszystko, czego trzeba do
programowania obiektowego: **struktury i enumy przechowują stan**, **bloki `impl` przypisują do tego stanu
zachowanie**, **traity wyrażają polimorfizm**, a **widoczność (`pub` albo prywatne) daje hermetyzację**.
Korzystamy z tego świadomie i konsekwentnie. Chodzi o to, żeby każde zachowanie miało oczywistego
właściciela: na pytanie „kto za to odpowiada?” odpowiedzią zawsze jest jakiś typ.

### 1.1 Każda funkcja należy do typu

- Każda funkcja mieszka w bloku `impl` typu, który za nią odpowiada. Może to być pojęcie domenowe
  (`TrackerDataSet`, `PrivacyConfiguration`, `HttpsUpgradePolicy`), obiekt wartości (`DomainName`) albo
  usługa (`ContentBlockerRulesBuilder`).
- **Wolne funkcje na poziomie modułu są zabronione**. Wyjątki są dwa: `fn main()` i funkcje `#[test]`.
  Małe, bezstanowe przekształcenie (escapowanie wyrażenia regularnego, porównanie wersji) staje się funkcją
  powiązaną albo metodą typu, który go potrzebuje. Może też być metodą obiektu wartości modelującego dane
  pojęcie, np. typ `DomainName` zamiast funkcji `normalize_domain(&str)`.
- Domknięcia wewnątrz metod są w porządku: to wyrażenia, a nie zachowanie wiszące w próżni.

### 1.2 Pojęcia jako typy, a nie napisy i luźne dane

- Pojęcie, które ma własne reguły, dostaje własny typ, nawet jeśli opakowuje jeden `String`. Przykłady:
  nazwa domeny zawsze zapisana małymi literami w punycode, wersja aplikacji porównywana liczbowo,
  identyfikator aplikacji webowej zawsze bezpieczny jako nazwa pliku.
- Wartości tworzymy przez konstruktory (`new`, `parse`, `from_json`), które pilnują niezmienników.
  Dzięki temu nieprawidłowa wartość w ogóle nie może powstać.

### 1.3 Hermetyzacja

- Encje i usługi mają pola **prywatne** i udostępniają metody, których nazwy mówią, co robią
  (`is_enabled_for_site`, `record_failed_upgrade`). Nie dają setterów, przez które wywołujący mógłby złamać
  niezmienniki.
- Proste, niezmienne obiekty wartości, które tylko przenoszą sparsowane dane (np. regułę odczytaną z JSON-a
  DuckDuckGo), mogą mieć pola `pub`. Nie mają niezmienników poza samymi typami.

### 1.4 Polimorfizm zamiast przełączania po typie

- Gdy kilka wariantów pełni tę samą rolę, ale zachowuje się inaczej, rolę wyrażamy **traitem**, a każdy
  wariant dostaje własną implementację. Unikamy dopasowywania znacznika typu w wielu miejscach.
- Na granicach (porty) zależymy od traitów. Crate domenowy definiuje, czego potrzebuje (np. „zapisz gdzieś
  JSON”), a crate aplikacji dostarcza adaptery (dysk, WebKit, GTK). To zasada odwrócenia zależności
  (Dependency Inversion) i dzięki niej `ztbrowser-core` nie zależy od GTK ani WebKit.
- Rust nie ma dziedziczenia. Zachowanie współdzielimy przez **kompozycję** (struktura zawiera inną
  strukturę) i domyślne metody traitów.

### 1.5 Testy obiektowe (Given / When / Then)

Testy też są obiektami. Każdy plik testów definiuje **strukturę scenariusza**, która posiada wszystko,
czego test potrzebuje, oraz **strukturę wyniku**, która posiada rezultat:

- funkcje powiązane `given_*` budują scenariusz (*Given*, „mając”);
- metody `with_*` / `after_*` przyjmują i zwracają `self`, dokładając kontekst w stylu buildera;
- metody `when_*` wykonują testowaną akcję i zwracają obiekt wyniku (*When*, „gdy”);
- metody `then_*` obiektu wyniku sprawdzają asercje (*Then*, „wtedy”). Jedna logiczna asercja na test;
- w plikach testów nie ma luźnych funkcji pomocniczych. Pomocnicze rzeczy są metodami scenariusza albo wyniku.

```rust
LinkCleaningScenario::given_tracking_parameters_protection()
    .with_link_clicked_on_page("https://needs-utm.com/page")
    .when_link_is_opened("https://shop.com/?fbclid=1")
    .then_opens_unchanged();
```

Scenariusz jest właścicielem danych, a testowany kod je od niego pożycza. To przy okazji usuwa problemy
z czasem życia krótkotrwałych wartości w testach.

## 2. Granice crate'ów

- `crates/ztbrowser-core` zawiera logikę domenową niezależną od przeglądarki i nie zależy od żadnego
  crate'a GUI ani WebKit.
- `crates/ztbrowser` to aplikacja GTK4 / libadwaita / WebKitGTK, która dopasowuje rdzeń do platformy.
- Zależności wskazują tylko do środka: `ztbrowser` → `ztbrowser-core`, nigdy odwrotnie.

## 3. Importy

- Jeden `use` na importowany element (`use a::B;` i `use a::C;`, nigdy `use a::{B, C};`).
- Importujemy z modułu, który element definiuje. Moduły takie jak `protection.rs` tylko deklarują
  podmoduły (`pub mod x;`) i nigdy nic nie reeksportują (`pub use`).
