//! Every language's catalogue holds exactly the messages the code uses, so no screen mixes two languages.
//!
//! The code's messages are found in the sources (`tr!("...")` in the application, `catalogue.text("...")` in this
//! crate, `bridge.message("...")` in the Qt edition's QML files). `locales/template.pot` lists them for translators; run
//! `ZTBROWSER_BLESS_TEMPLATE=1 cargo test -p ztbrowser-core --test message_catalogues` after adding or changing a
//! message to rewrite it.

use std::collections::BTreeSet;
use std::fmt::Write;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use regex::Regex;
use ztbrowser_core::language::interface_language::InterfaceLanguage;
use ztbrowser_core::language::message_catalogue::MessageCatalogue;
use ztbrowser_core::language::translation_sources::TranslationSources;

struct CatalogueScenario {
    root: PathBuf,
}

impl CatalogueScenario {
    fn given_the_repository() -> Self {
        Self { root: Path::new(env!("CARGO_MANIFEST_DIR")).join("..") }
    }

    /// Languages whose messages are the source text, so they need no catalogue.
    fn is_written_in_the_source_language(tag: &str) -> bool {
        tag.starts_with("en-")
    }

    fn used_messages(&self) -> BTreeSet<String> {
        let pattern =
            Regex::new(r#"(?:\btr!|catalogue\.text(?:_with)?)\(\s*"((?:[^"\\]|\\.)*)""#).expect("a valid pattern");
        let mut messages = BTreeSet::new();
        for crate_name in ["ztbrowser/src", "ztbrowser-core/src"] {
            Self::rust_files(&self.root.join(crate_name), &mut |source| {
                let code: String =
                    source.lines().filter(|line| !line.trim_start().starts_with("//")).collect::<Vec<_>>().join("\n");
                for capture in pattern.captures_iter(&code) {
                    messages.insert(Self::unescape_rust(&capture[1]));
                }
            });
        }
        let qml_pattern = Regex::new(r#"\bbridge\.message\(\s*"((?:[^"\\]|\\.)*)""#).expect("a valid pattern");
        let qml_directory = self.root.join("ztbrowser-qt/qml");
        for entry in fs::read_dir(&qml_directory).expect("a readable QML directory").filter_map(Result::ok) {
            if entry.path().extension().is_some_and(|extension| extension == "qml") {
                let source = fs::read_to_string(entry.path()).expect("a readable QML file");
                let code: String =
                    source.lines().filter(|line| !line.trim_start().starts_with("//")).collect::<Vec<_>>().join("\n");
                for capture in qml_pattern.captures_iter(&code) {
                    messages.insert(Self::unescape_rust(&capture[1]));
                }
            }
        }
        messages
    }

    fn rust_files(directory: &Path, visit: &mut dyn FnMut(&str)) {
        for entry in fs::read_dir(directory).expect("a readable source directory").filter_map(Result::ok) {
            let path = entry.path();
            if path.is_dir() {
                Self::rust_files(&path, visit);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                visit(&fs::read_to_string(&path).expect("a readable source file"));
            }
        }
    }

    /// A Rust string literal's text: `\"`, `\\`, `\n`, `\'` resolved and a backslash at a line's end joining it
    /// with the next line (leading blanks of that line dropped).
    fn unescape_rust(literal: &str) -> String {
        let mut text = String::new();
        let mut letters = literal.chars().peekable();
        while let Some(letter) = letters.next() {
            if letter != '\\' {
                text.push(letter);
                continue;
            }
            match letters.next() {
                Some('n') => text.push('\n'),
                Some('t') => text.push('\t'),
                Some('\n') => while letters.next_if(|next| next.is_whitespace()).is_some() {},
                Some(other) => text.push(other),
                None => {}
            }
        }
        text
    }

    fn template_path(&self) -> PathBuf {
        self.root.join("ztbrowser-core/locales/template.pot")
    }

    fn template_text(messages: &BTreeSet<String>) -> String {
        let mut text = String::from(
            "# Messages of ZTBrowser's interface, written in English. Translate each msgid into the msgstr below it;\n\
             # keep every {placeholder} exactly as written (it may move within the sentence).\n\
             msgid \"\"\nmsgstr \"\"\n\"Content-Type: text/plain; charset=UTF-8\\n\"\n",
        );
        for message in messages {
            let escaped = message.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\t', "\\t");
            write!(text, "\nmsgid \"{escaped}\"\nmsgstr \"\"\n").expect("writing to a string");
        }
        text
    }

    fn catalogue_messages(tag: &str) -> BTreeSet<String> {
        let source = TranslationSources::source_of(tag).expect("a compiled-in catalogue");
        MessageCatalogue::parse(source).messages().into_iter().map(str::to_owned).collect()
    }

    fn untranslated_in(tag: &str, used: &BTreeSet<String>) -> Vec<String> {
        let translated = Self::catalogue_messages(tag);
        used.difference(&translated).cloned().collect()
    }

    fn obsolete_in(tag: &str, used: &BTreeSet<String>) -> Vec<String> {
        let translated = Self::catalogue_messages(tag);
        translated.difference(used).cloned().collect()
    }

    fn placeholder_mismatches_in(tag: &str, used: &BTreeSet<String>) -> Vec<String> {
        let catalogue = MessageCatalogue::parse(TranslationSources::source_of(tag).expect("a compiled-in catalogue"));
        used.iter()
            .filter(|message| {
                let wanted: BTreeSet<&str> = MessageCatalogue::placeholders_of(message).into_iter().collect();
                let translated = catalogue.text(message);
                let found: BTreeSet<&str> = MessageCatalogue::placeholders_of(translated).into_iter().collect();
                wanted != found
            })
            .cloned()
            .collect()
    }
}

#[test]
fn should_list_every_message_the_code_uses_in_the_template() {
    let scenario = CatalogueScenario::given_the_repository();
    let expected = CatalogueScenario::template_text(&scenario.used_messages());
    if std::env::var_os("ZTBROWSER_BLESS_TEMPLATE").is_some() {
        fs::write(scenario.template_path(), &expected).expect("a writable template");
    }
    let actual = fs::read_to_string(scenario.template_path()).unwrap_or_default();
    assert!(actual == expected, "locales/template.pot is out of date: run with ZTBROWSER_BLESS_TEMPLATE=1");
}

#[test]
fn should_have_a_catalogue_for_every_language_that_is_not_the_source_language() {
    let missing: Vec<&str> = InterfaceLanguage::ALL
        .iter()
        .map(|language| language.tag())
        .filter(|tag| !CatalogueScenario::is_written_in_the_source_language(tag))
        .filter(|tag| TranslationSources::source_of(tag).is_none())
        .collect();
    assert_eq!(missing, Vec::<&str>::new());
}

#[test]
fn should_have_no_catalogue_for_a_language_the_interface_does_not_offer() {
    let unknown: Vec<&str> =
        TranslationSources::tags().into_iter().filter(|tag| InterfaceLanguage::from_tag(tag).is_none()).collect();
    assert_eq!(unknown, Vec::<&str>::new());
}

#[test]
fn should_translate_every_message_in_every_catalogue() {
    let scenario = CatalogueScenario::given_the_repository();
    let used = scenario.used_messages();
    let incomplete: Vec<String> = TranslationSources::tags()
        .into_iter()
        .filter_map(|tag| {
            let missing = CatalogueScenario::untranslated_in(tag, &used);
            (!missing.is_empty()).then(|| format!("{tag}: {} missing, e.g. {:?}", missing.len(), missing.first()))
        })
        .collect();
    assert_eq!(incomplete, Vec::<String>::new());
}

#[test]
fn should_keep_no_message_in_a_catalogue_that_the_code_no_longer_uses() {
    let scenario = CatalogueScenario::given_the_repository();
    let used = scenario.used_messages();
    let stale: Vec<String> = TranslationSources::tags()
        .into_iter()
        .filter_map(|tag| {
            let obsolete = CatalogueScenario::obsolete_in(tag, &used);
            (!obsolete.is_empty()).then(|| format!("{tag}: {} obsolete, e.g. {:?}", obsolete.len(), obsolete.first()))
        })
        .collect();
    assert_eq!(stale, Vec::<String>::new());
}

#[test]
fn should_keep_the_placeholders_of_every_message_in_every_translation() {
    let scenario = CatalogueScenario::given_the_repository();
    let used = scenario.used_messages();
    let broken: Vec<String> = TranslationSources::tags()
        .into_iter()
        .filter_map(|tag| {
            let mismatches = CatalogueScenario::placeholder_mismatches_in(tag, &used);
            (!mismatches.is_empty()).then(|| format!("{tag}: {:?}", mismatches.first()))
        })
        .collect();
    assert_eq!(broken, Vec::<String>::new());
}
