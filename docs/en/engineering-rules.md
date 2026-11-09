# ZTBrowser Engineering Rules

These rules are specific to ZTBrowser and add to the global rules in `~/.claude/rules/general.md`. When the
two conflict, this document wins inside this repository.

## 1. Object Oriented Programming First

ZTBrowser is written in Rust. Rust has no classes or inheritance, but it has everything needed to program
in an object-oriented way: **structs and enums own state**, **`impl` blocks attach behaviour to that
state**, **traits express polymorphism**, and **privacy (`pub` vs. private) gives encapsulation**. We
use these deliberately and consistently. The goal is that every piece of behaviour has an obvious owner:
when you ask "who is responsible for this?", the answer is always a type.

### 1.1 Every function belongs to a type

- Every function lives in an `impl` block of the type responsible for it — a domain concept
  (`TrackerDataSet`, `PrivacyConfiguration`, `HttpsUpgradePolicy`), a value object (`DomainName`), or a
  service (`ContentBlockerRulesBuilder`).
- **Module-level free functions are not allowed**, with two exceptions: `fn main()` and `#[test]`
  functions. A small stateless transformation (escaping a regex, comparing versions) becomes an
  associated function or method of the type that needs it, or a method of a value object that models the
  concept (for example a `DomainName` type instead of a `normalize_domain(&str)` function).
- Closures are fine inside methods; they are expressions, not free-floating behaviour.

### 1.2 Model concepts as types, not as strings and loose data

- A concept with rules of its own gets its own type, even when it wraps a single `String`: a domain name
  that is always lower-case punycode, an app version compared numerically, a web-app identifier that is
  always safe to put in a file name.
- Construct values through constructors (`new`, `parse`, `from_json`) that establish their invariants, so
  an invalid value cannot exist.

### 1.3 Encapsulation

- Entities and services keep their fields **private** and expose intention-revealing methods
  (`is_enabled_for_site`, `record_failed_upgrade`), never setters that let callers break invariants.
- Plain immutable value objects that only carry parsed data (for example a rule read from DuckDuckGo's
  JSON) may expose `pub` fields, because they have no invariants beyond their types.

### 1.4 Polymorphism instead of type switches

- When several variants share a role but behave differently, express the role as a **trait** and give
  each variant its own implementation, instead of matching on a type tag in many places.
- Depend on traits at boundaries (ports): the domain crate defines what it needs (for example "store
  JSON somewhere"), the application crate provides adapters (disk, WebKit, GTK). This is the Dependency
  Inversion Principle and keeps `ztbrowser-core` free of GTK and WebKit.
- Rust has no inheritance; reuse behaviour through **composition** (a struct holding another struct) and
  trait default methods.

### 1.5 Object-oriented tests (Given / When / Then)

Tests are objects too. Each test file defines a **scenario struct** that owns everything the test needs,
and an **outcome struct** that owns the result:

- `given_*` associated functions construct the scenario (the *Given*).
- `with_*` / `after_*` methods consume and return `self` to add context, builder-style.
- `when_*` methods perform the action under test and return an outcome object (the *When*).
- `then_*` methods on the outcome object assert (the *Then*). One logical assertion per test.
- No free helper functions in test files; helpers are methods of the scenario or outcome type.

```rust
LinkCleaningScenario::given_tracking_parameters_protection()
    .with_link_clicked_on_page("https://needs-utm.com/page")
    .when_link_is_opened("https://shop.com/?fbclid=1")
    .then_opens_unchanged();
```

The scenario owns its data, so the code under test borrows it from the scenario. That also avoids
lifetime problems with short-lived values in tests.

## 2. Crate boundaries

- `crates/ztbrowser-core` holds browser-independent domain logic and depends on no GUI or WebKit crate.
- `crates/ztbrowser` is the GTK4 / libadwaita / WebKitGTK application and adapts the core to the platform.
- Dependencies point inward only: `ztbrowser` → `ztbrowser-core`, never the other way.

## 3. Imports

- One `use` per imported item (`use a::B;` and `use a::C;`, never `use a::{B, C};`).
- Import from the module that defines the item; modules such as `protection.rs` only declare
  submodules (`pub mod x;`) and never re-export (`pub use`).
