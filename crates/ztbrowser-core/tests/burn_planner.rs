//! What burning a profile's data deletes.

use std::path::Path;

use ztbrowser_core::profile::profile_directories::ProfileDirectories;
use ztbrowser_core::profile::profile_id::ProfileId;
use ztbrowser_core::storage::burn_plan::BurnPlan;
use ztbrowser_core::storage::burn_planner::BurnPlanner;
use ztbrowser_core::storage::burn_scope::BurnScope;

struct BurnScenario {
    planner: BurnPlanner,
}

impl BurnScenario {
    fn given_profile_work() -> Self {
        let directories = ProfileDirectories::new(Path::new("/data/ztbrowser"), Path::new("/cache/ztbrowser"));
        let location = directories.for_profile(&ProfileId::parse("work").expect("valid profile id"));
        Self { planner: BurnPlanner::new(location) }
    }

    fn when_burning(&self, scope: &BurnScope) -> BurnPlan {
        self.planner.plan(scope)
    }
}

#[test]
fn should_delete_every_cookie_and_the_cache_when_burning_everything() {
    let plan = BurnScenario::given_profile_work().when_burning(&BurnScope::Everything);
    assert!(plan.delete_all_cookies && plan.clear_cache);
}

#[test]
fn should_delete_the_whole_engine_directory_and_the_cache_when_burning_everything() {
    let plan = BurnScenario::given_profile_work().when_burning(&BurnScope::Everything);
    assert_eq!(
        plan.storage_directories,
        vec![
            Path::new("/data/ztbrowser/profiles/work/engine").to_path_buf(),
            Path::new("/cache/ztbrowser/profiles/work").to_path_buf()
        ]
    );
}

#[test]
fn should_keep_the_profiles_own_settings_when_burning_everything() {
    let plan = BurnScenario::given_profile_work().when_burning(&BurnScope::Everything);
    assert!(plan.storage_directories.iter().all(|directory| directory != Path::new("/data/ztbrowser/profiles/work")));
}

#[test]
fn should_delete_only_the_cookies_of_one_site_when_burning_a_site() {
    let plan = BurnScenario::given_profile_work().when_burning(&BurnScope::Site("Shop.Example.org".to_owned()));
    assert_eq!(plan, BurnPlan { cookie_sites: vec!["shop.example.org".to_owned()], ..BurnPlan::default() });
}
