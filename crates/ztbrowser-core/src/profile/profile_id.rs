//! Identifier of an isolated browsing profile; also the name of its directories on disk.

use std::fmt;

use crate::identity::application_id::ApplicationId;
use crate::identity::slug::InvalidSlug;
use crate::identity::slug::Slug;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ProfileId {
    slug: Slug,
}

impl ProfileId {
    const DEFAULT: &'static str = "default";
    /// Profiles of installed web apps live in the same directory tree; this prefix keeps them apart.
    pub const WEB_APP_PREFIX: &'static str = "app";

    #[must_use]
    pub fn default_profile() -> Self {
        Self { slug: Slug::from_text(Self::DEFAULT) }
    }

    /// The private profile of one installed web app.
    #[must_use]
    pub fn for_web_app(web_app: &Slug) -> Self {
        Self { slug: web_app.with_prefix(Self::WEB_APP_PREFIX) }
    }

    /// # Errors
    /// [`InvalidSlug`] when the stored identifier is not a safe slug.
    pub fn parse(text: &str) -> Result<Self, InvalidSlug> {
        Slug::parse(text).map(|slug| Self { slug })
    }

    pub(crate) fn from_slug(slug: Slug) -> Self {
        Self { slug }
    }

    #[must_use]
    pub fn is_default(&self) -> bool {
        self.slug.as_str() == Self::DEFAULT
    }

    #[must_use]
    pub fn belongs_to_web_app(&self) -> bool {
        self.slug.as_str().starts_with(&format!("{}_", Self::WEB_APP_PREFIX))
    }

    /// The default profile is "the browser"; every other profile is its own application and process.
    #[must_use]
    pub fn application_id(&self) -> ApplicationId {
        if self.is_default() { ApplicationId::browser() } else { ApplicationId::browser_profile(&self.slug) }
    }

    #[must_use]
    pub fn slug(&self) -> &Slug {
        &self.slug
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        self.slug.as_str()
    }
}

impl fmt::Display for ProfileId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.slug.fmt(formatter)
    }
}
