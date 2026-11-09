//! The browsing profiles the user created. The default profile always exists and cannot be removed.
//!
//! Profiles of installed web apps are not listed here; each web app owns exactly one profile, derived from
//! its identifier (see `WebAppId::profile_id`).

use serde_json::Value;
use serde_json::json;

use crate::document::lenient_json::LenientJson;
use crate::identity::slug::Slug;
use crate::profile::browsing_profile::BrowsingProfile;
use crate::profile::profile_id::ProfileId;

#[derive(Debug, Clone)]
pub struct ProfileCatalog {
    profiles: Vec<BrowsingProfile>,
}

impl Default for ProfileCatalog {
    fn default() -> Self {
        Self {
            profiles: vec![BrowsingProfile {
                id: ProfileId::default_profile(),
                name: Self::DEFAULT_PROFILE_NAME.to_owned(),
            }],
        }
    }
}

impl ProfileCatalog {
    const DEFAULT_PROFILE_NAME: &'static str = "Default";
    /// Namespace for user profiles whose names would otherwise look like web-app profiles.
    const USER_PROFILE_PREFIX: &'static str = "profile";

    /// Reads the saved catalog; entries with unsafe or reserved identifiers are skipped.
    #[must_use]
    pub fn from_json(document: &Value) -> Self {
        let mut catalog = Self::default();
        for entry in LenientJson::new(document).field("profiles").items() {
            let Some(Ok(id)) = entry.field("id").as_str().map(ProfileId::parse) else { continue };
            let name = entry.field("name").as_str().unwrap_or(id.as_str()).to_owned();
            if !id.belongs_to_web_app() && !catalog.contains(&id) {
                catalog.profiles.push(BrowsingProfile { id, name });
            }
        }
        catalog
    }

    #[must_use]
    pub fn to_json(&self) -> Value {
        let profiles: Vec<Value> =
            self.profiles.iter().map(|profile| json!({"id": profile.id.as_str(), "name": profile.name})).collect();
        json!({"profiles": profiles})
    }

    pub fn profiles(&self) -> impl Iterator<Item = &BrowsingProfile> {
        self.profiles.iter()
    }

    #[must_use]
    pub fn profile(&self, id: &ProfileId) -> Option<&BrowsingProfile> {
        self.profiles.iter().find(|profile| &profile.id == id)
    }

    /// Adds a profile and returns its new, unique identifier.
    pub fn create_profile(&mut self, name: &str) -> ProfileId {
        let mut slug = Slug::from_text(name);
        if ProfileId::from_slug(slug.clone()).belongs_to_web_app() {
            slug = slug.with_prefix(Self::USER_PROFILE_PREFIX);
        }
        let id = ProfileId::from_slug(
            slug.first_unused(|candidate| self.contains(&ProfileId::from_slug(candidate.clone()))),
        );
        self.profiles.push(BrowsingProfile { id: id.clone(), name: name.trim().to_owned() });
        id
    }

    /// Forgets a profile. The default profile stays. Deleting its files is the caller's job.
    pub fn remove_profile(&mut self, id: &ProfileId) {
        if !id.is_default() {
            self.profiles.retain(|profile| &profile.id != id);
        }
    }

    fn contains(&self, id: &ProfileId) -> bool {
        self.profiles.iter().any(|profile| &profile.id == id)
    }
}
