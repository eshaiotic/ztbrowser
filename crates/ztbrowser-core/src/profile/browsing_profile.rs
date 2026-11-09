//! A user-created browsing profile, such as "Work" or "Banking".

use crate::profile::profile_id::ProfileId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowsingProfile {
    pub id: ProfileId,
    /// What the user called it; shown in menus, never used as a path.
    pub name: String,
}
