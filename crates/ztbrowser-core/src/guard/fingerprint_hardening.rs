//! The fingerprinting defences of DuckDuckGo's Content Scope Scripts that ZTBrowser Guard always runs.
//!
//! Fingerprinting recognises a browser without cookies, from what it gives away: how it draws a canvas, how
//! its audio stack rounds numbers, its exact screen size. DuckDuckGo's macOS configuration leaves most of the
//! matching defences off, because Safari's WebKit adds noise of its own there; WebKitGTK does not, so Guard
//! switches them on. Referrer trimming joins them: the full referrer tells a site which page you came from.
//!
//! DuckDuckGo's per-site exceptions, settings and minimum versions stay as published: they record where a
//! defence breaks a page and which script release it needs.

use serde_json::Map;
use serde_json::Value;

pub struct FingerprintHardening;

impl FingerprintHardening {
    pub const FEATURES: [&'static str; 6] = [
        "fingerprintingAudio",
        "fingerprintingBattery",
        "fingerprintingCanvas",
        "fingerprintingScreenSize",
        "fingerprintingTemporaryStorage",
        "referrer",
    ];
    /// Settings a defence checks before it runs, on top of its state: canvas noise starts only once
    /// `additionalEnabledCheck` is on (`FingerprintingCanvas.init` in the bundled script).
    const RUN_CHECKS: [(&'static str, &'static str); 1] = [("fingerprintingCanvas", "additionalEnabledCheck")];

    /// The privacy configuration `document` with every defence it describes switched on.
    #[must_use]
    pub fn applied_to(mut document: Value) -> Value {
        if let Some(features) = document.get_mut("features").and_then(Value::as_object_mut) {
            for name in Self::FEATURES {
                if let Some(feature) = features.get_mut(name).and_then(Value::as_object_mut) {
                    feature.insert("state".to_owned(), Value::from("enabled"));
                    for (_, check) in Self::RUN_CHECKS.iter().filter(|(checked, _)| *checked == name) {
                        let settings = feature.entry("settings").or_insert_with(|| Value::Object(Map::new()));
                        if let Some(settings) = settings.as_object_mut() {
                            settings.insert((*check).to_owned(), Value::from("enabled"));
                        }
                    }
                }
            }
        }
        document
    }
}
