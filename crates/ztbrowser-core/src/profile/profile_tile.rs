//! How a browsing profile is drawn in the profiles popup: a coloured circle with an initial, and a name.
//!
//! The colour is derived from the profile's identifier, so a profile keeps its colour for as long as it exists
//! and two windows never disagree about it.

use crate::language::message_catalogue::MessageCatalogue;
use crate::profile::profile_catalog::ProfileCatalog;
use crate::profile::profile_id::ProfileId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileTile {
    pub id: String,
    /// What the person called it; the default profile is named in the interface language.
    pub name: String,
    /// One letter or digit, upper case; `?` when the name has none.
    pub initial: String,
    /// Which of [`ProfileTiles::COLOUR_COUNT`] colours the circle takes.
    pub colour_index: usize,
    pub is_current: bool,
    pub is_default: bool,
}

pub struct ProfileTiles;

impl ProfileTiles {
    pub const COLOUR_COUNT: usize = 8;

    /// One tile per profile of the catalogue, the default profile first, as the catalogue lists them.
    #[must_use]
    pub fn of(catalog: &ProfileCatalog, current: &ProfileId, catalogue: &MessageCatalogue) -> Vec<ProfileTile> {
        catalog
            .profiles()
            .map(|profile| {
                let is_default = profile.id.is_default();
                let name = if is_default { catalogue.text("Default").to_owned() } else { profile.name.clone() };
                ProfileTile {
                    id: profile.id.as_str().to_owned(),
                    initial: Self::initial_of(&name),
                    colour_index: Self::colour_index_of(profile.id.as_str()),
                    is_current: &profile.id == current,
                    is_default,
                    name,
                }
            })
            .collect()
    }

    /// The label of the tile that adds a profile.
    #[must_use]
    pub fn add_label(catalogue: &MessageCatalogue) -> &str {
        catalogue.text("Add Profile")
    }

    /// The question before a profile is deleted.
    #[must_use]
    pub fn delete_heading(catalogue: &MessageCatalogue, name: &str) -> String {
        catalogue.text_with("Delete profile {name}?", &[("name", name)])
    }

    /// What deleting a profile does, in a sentence.
    #[must_use]
    pub fn delete_body(catalogue: &MessageCatalogue) -> &str {
        catalogue.text("Deletes the profile with its cookies, logins, history and cache. This cannot be undone.")
    }

    fn initial_of(name: &str) -> String {
        name.chars()
            .find(|character| character.is_alphanumeric())
            .map_or_else(|| "?".to_owned(), |character| character.to_uppercase().collect())
    }

    /// FNV-1a over the identifier: stable across runs and versions, unlike the standard library's hasher.
    fn colour_index_of(id: &str) -> usize {
        let hash = id
            .bytes()
            .fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3));
        usize::try_from(hash % Self::COLOUR_COUNT as u64).unwrap_or(0)
    }
}
