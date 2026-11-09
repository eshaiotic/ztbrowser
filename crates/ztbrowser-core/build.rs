//! Embeds every `locales/<tag>.po` catalogue in the library, so adding a language means adding a file.

use std::env;
use std::fs;
use std::path::PathBuf;

struct CatalogueEmbedding;

impl CatalogueEmbedding {
    fn write() {
        let directory =
            PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("cargo sets the manifest directory")).join("locales");
        println!("cargo:rerun-if-changed={}", directory.display());
        let mut tags: Vec<String> = fs::read_dir(&directory)
            .expect("the locales directory exists")
            .filter_map(Result::ok)
            .filter_map(|entry| {
                let path = entry.path();
                (path.extension().is_some_and(|extension| extension == "po"))
                    .then(|| path.file_stem().map(|stem| stem.to_string_lossy().into_owned()))
                    .flatten()
            })
            .collect();
        tags.sort();
        let entries: String = tags
            .iter()
            .map(|tag| {
                let file = directory.join(format!("{tag}.po"));
                [format!("    (\"{tag}\", include_str!(r\"{}\")),\n", file.display())].concat()
            })
            .collect::<Vec<_>>()
            .concat();
        let table = format!("pub const CATALOGUES: [(&str, &str); {}] = [\n{entries}];\n", tags.len());
        let out = PathBuf::from(env::var("OUT_DIR").expect("cargo sets the output directory")).join("catalogues.rs");
        fs::write(out, table).expect("the output directory is writable");
    }
}

fn main() {
    CatalogueEmbedding::write();
}
