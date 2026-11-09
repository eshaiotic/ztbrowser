//! The filter lists the downloader cached are what Guard builds its engine from.

use std::fs;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use tempfile::TempDir;
use ztbrowser_core::guard::shared_guard_engine::SharedGuardEngine;
use ztbrowser_qt::guard_lists::GuardLists;

use ztbrowser_qt as _;

struct CacheScenario {
    cache: TempDir,
}

impl CacheScenario {
    fn given_a_cache_with(list_file: &str, text: &str) -> Self {
        let cache = TempDir::new().expect("cache");
        fs::create_dir(cache.path().join("filter-lists")).expect("directory");
        fs::write(cache.path().join("filter-lists").join(list_file), text).expect("list");
        Self { cache }
    }
}

#[test]
fn should_read_the_lists_that_were_downloaded() {
    let scenario = CacheScenario::given_a_cache_with("easylist.txt", "||ads.example^");
    assert_eq!(GuardLists::in_cache(scenario.cache.path()).downloaded(), vec!["||ads.example^".to_owned()]);
}

#[test]
fn should_ignore_files_that_are_not_one_of_the_lists() {
    let scenario = CacheScenario::given_a_cache_with("notes.txt", "||ads.example^");
    assert_eq!(GuardLists::in_cache(scenario.cache.path()).downloaded(), Vec::<String>::new());
}

#[test]
fn should_have_no_lists_before_anything_was_downloaded() {
    let empty = TempDir::new().expect("cache");
    assert_eq!(GuardLists::in_cache(empty.path()).downloaded(), Vec::<String>::new());
}

#[test]
fn should_install_the_engine_once_it_is_built() {
    let scenario = CacheScenario::given_a_cache_with("easylist.txt", "||ads.example^");
    let engine = Arc::new(SharedGuardEngine::default());
    GuardLists::in_cache(scenario.cache.path()).build_into(Arc::clone(&engine));
    for _ in 0..100 {
        if engine.get().is_some() {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }
    assert!(engine.get().is_some());
}
