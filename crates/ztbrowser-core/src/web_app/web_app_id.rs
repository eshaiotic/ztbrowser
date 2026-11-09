//! Identifier of an installed web app, from which its profile and desktop identity derive.

use std::fmt;

use crate::identity::application_id::ApplicationId;
use crate::identity::slug::InvalidSlug;
use crate::identity::slug::Slug;
use crate::profile::profile_id::ProfileId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WebAppId {
    slug: Slug,
}

impl WebAppId {
    /// # Errors
    /// [`InvalidSlug`] when the identifier (from storage or the command line) is not a safe slug.
    pub fn parse(text: &str) -> Result<Self, InvalidSlug> {
        Slug::parse(text).map(|slug| Self { slug })
    }

    pub(crate) fn from_slug(slug: Slug) -> Self {
        Self { slug }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        self.slug.as_str()
    }

    /// The web app's private profile: cookies and storage shared with nothing else.
    #[must_use]
    pub fn profile_id(&self) -> ProfileId {
        ProfileId::for_web_app(&self.slug)
    }

    /// Its own application id, so it runs as its own process with its own dock icon.
    #[must_use]
    pub fn application_id(&self) -> ApplicationId {
        ApplicationId::web_app(&self.slug)
    }
}

impl fmt::Display for WebAppId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.slug.fmt(formatter)
    }
}
