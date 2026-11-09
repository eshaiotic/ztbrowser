//! The state of the profile this process runs: the shield's list, the downloads and the Fire button.
//!
//! QML creates the bridge objects itself, so the session is installed once at start-up and the bridge finds it
//! here.

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::OnceLock;

use ztbrowser_core::guard::shared_guard_engine::SharedGuardEngine;
use ztbrowser_core::profile::profile_directories::ProfileLocation;

use crate::burn_restart::BurnRestart;
use crate::download_center::DownloadCenter;
use crate::interface_language_setting::InterfaceLanguageSetting;
use crate::profile_preferences::ProfilePreferences;
use crate::site_protection::SiteProtection;
use crate::site_zoom_store::SiteZoomStore;

pub struct ProfileSession {
    pub protection: SiteProtection,
    pub burn: BurnRestart,
    /// ZTBrowser Guard's engine, filled in once the filter lists are read.
    pub guard: Arc<SharedGuardEngine>,
    downloads: Mutex<DownloadCenter>,
    zoom: Mutex<SiteZoomStore>,
    preferences: Mutex<ProfilePreferences>,
    language: Mutex<InterfaceLanguageSetting>,
}

static CURRENT: OnceLock<ProfileSession> = OnceLock::new();

impl ProfileSession {
    #[must_use]
    pub fn new(
        location: &ProfileLocation,
        protection: SiteProtection,
        downloads: DownloadCenter,
        zoom: SiteZoomStore,
        preferences: ProfilePreferences,
        language: InterfaceLanguageSetting,
        guard: Arc<SharedGuardEngine>,
    ) -> Self {
        Self {
            protection,
            burn: BurnRestart::new(location.clone()),
            guard,
            downloads: Mutex::new(downloads),
            zoom: Mutex::new(zoom),
            preferences: Mutex::new(preferences),
            language: Mutex::new(language),
        }
    }

    /// Makes this the session of the process; a second call is ignored.
    pub fn install(self) {
        let _ = CURRENT.set(self);
    }

    #[must_use]
    pub fn current() -> Option<&'static Self> {
        CURRENT.get()
    }

    pub fn zoom(&self) -> MutexGuard<'_, SiteZoomStore> {
        self.zoom.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub fn preferences(&self) -> MutexGuard<'_, ProfilePreferences> {
        self.preferences.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub fn language(&self) -> MutexGuard<'_, InterfaceLanguageSetting> {
        self.language.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub fn downloads(&self) -> MutexGuard<'_, DownloadCenter> {
        self.downloads.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}
