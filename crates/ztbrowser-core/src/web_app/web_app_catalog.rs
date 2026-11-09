//! The sites installed as web apps.

use std::fmt;

use serde_json::Value;
use serde_json::json;
use url::Url;

use crate::document::lenient_json::LenientJson;
use crate::identity::slug::Slug;
use crate::web_app::installed_web_app::InstalledWebApp;
use crate::web_app::web_app_id::WebAppId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebAppInstallError {
    /// Only `http` and `https` pages can become web apps.
    NotAWebAddress,
}

impl fmt::Display for WebAppInstallError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAWebAddress => formatter.write_str("only http and https pages can be installed as apps"),
        }
    }
}

impl std::error::Error for WebAppInstallError {}

#[derive(Debug, Clone, Default)]
pub struct WebAppCatalog {
    web_apps: Vec<InstalledWebApp>,
}

impl WebAppCatalog {
    /// Reads the saved catalog; entries with unsafe identifiers or non-web addresses are skipped.
    #[must_use]
    pub fn from_json(document: &Value) -> Self {
        let mut catalog = Self::default();
        for entry in LenientJson::new(document).field("webApps").items() {
            let Some(Ok(id)) = entry.field("id").as_str().map(WebAppId::parse) else { continue };
            let Some(start_url) = entry.field("startUrl").as_str().and_then(Self::web_address) else { continue };
            let name = entry.field("name").as_str().unwrap_or(id.as_str()).to_owned();
            if catalog.web_app(&id).is_none() {
                catalog.web_apps.push(InstalledWebApp { id, name, start_url: start_url.into() });
            }
        }
        catalog
    }

    #[must_use]
    pub fn to_json(&self) -> Value {
        let web_apps: Vec<Value> = self
            .web_apps
            .iter()
            .map(|app| json!({"id": app.id.as_str(), "name": app.name, "startUrl": app.start_url}))
            .collect();
        json!({"webApps": web_apps})
    }

    pub fn web_apps(&self) -> impl Iterator<Item = &InstalledWebApp> {
        self.web_apps.iter()
    }

    #[must_use]
    pub fn web_app(&self, id: &WebAppId) -> Option<&InstalledWebApp> {
        self.web_apps.iter().find(|app| &app.id == id)
    }

    /// Registers `start_url` as a new web app named `name` (the host when blank).
    ///
    /// # Errors
    /// [`WebAppInstallError::NotAWebAddress`] for anything but an `http`/`https` URL with a host.
    pub fn install(&mut self, name: &str, start_url: &str) -> Result<&InstalledWebApp, WebAppInstallError> {
        let start_url = Self::web_address(start_url).ok_or(WebAppInstallError::NotAWebAddress)?;
        let host = start_url.host_str().unwrap_or_default().to_owned();
        let slug = Slug::from_text(&host)
            .first_unused(|candidate| self.web_app(&WebAppId::from_slug(candidate.clone())).is_some());
        let id = WebAppId::from_slug(slug);
        let name = if name.trim().is_empty() { host } else { name.trim().to_owned() };
        let position = self.web_apps.len();
        self.web_apps.push(InstalledWebApp { id, name, start_url: start_url.into() });
        Ok(&self.web_apps[position])
    }

    /// Forgets a web app. Removing its desktop file and profile directories is the caller's job.
    pub fn uninstall(&mut self, id: &WebAppId) {
        self.web_apps.retain(|app| &app.id != id);
    }

    fn web_address(text: &str) -> Option<Url> {
        Url::parse(text).ok().filter(|url| matches!(url.scheme(), "http" | "https") && url.host_str().is_some())
    }
}
