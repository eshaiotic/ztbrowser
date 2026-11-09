//! What the About dialog shows, as the GTK edition's `AdwAboutDialog` showed it: the application with its
//! version, a details page, and a legal page that opens with the application's own licence and then names every
//! component ZTBrowser owes a licence to. Every text comes from the catalogue, so the dialog is in one language.

use serde_json::Value;
use serde_json::json;

use crate::language::message_catalogue::MessageCatalogue;

/// A licence the legal page links to, its name in the language of the dialog.
struct Licence<'a> {
    name: &'a str,
    url: &'static str,
}

pub struct AboutContent;

impl AboutContent {
    const NAME: &'static str = "ZTBrowser";

    /// `{"name", "version", "detailsTitle", "legalTitle", "comments", "sections": [{"title", "copyright",
    /// "body"}]}`; `body` is styled text (`<a>` links only). A section without a title is the application's own.
    #[must_use]
    pub fn build(catalogue: &MessageCatalogue, version: &str) -> Value {
        let sections = vec![
            Self::section(None, None, &Self::licence_sentence(catalogue, &Self::gpl(catalogue))),
            Self::section(
                Some(catalogue.text("DuckDuckGo protections")),
                Some("© DuckDuckGo, Inc."),
                &Self::licence_sentence(catalogue, &Self::apache(catalogue)),
            ),
            Self::section(
                Some(catalogue.text("Tracker Data Set")),
                Some("© DuckDuckGo, Inc."),
                &Self::escaped(catalogue.text("Downloaded at run time from DuckDuckGo; licensed CC BY-NC-SA 4.0.")),
            ),
            Self::section(
                Some("adblock-rust"),
                Some("© Brave Software, Inc."),
                &Self::licence_sentence(catalogue, &Self::mpl(catalogue)),
            ),
            Self::section(
                Some(catalogue.text("ZTBrowser Guard filter lists")),
                Some(catalogue.text("© their maintainers")),
                &Self::escaped(catalogue.text(
                    "Downloaded at run time from their publishers: EasyList, EasyPrivacy and EasyList Cookie List \
                     (GPL-3.0 or CC BY-SA 3.0, CC BY 3.0), uBlock Origin's lists (GPL-3.0), Peter Lowe's list \
                     (pgl.yoyo.org terms), Polish Filters (CC BY-NC-SA 4.0).",
                )),
            ),
            Self::section(
                Some("Tor"),
                Some("© The Tor Project, Inc."),
                &Self::licence_sentence(catalogue, &Self::bsd3(catalogue)),
            ),
        ];
        json!({
            "name": Self::NAME,
            "version": version,
            "detailsTitle": catalogue.text("Details"),
            "legalTitle": catalogue.text("Legal"),
            "comments": catalogue.text(
                "A privacy-first browser for Linux built on DuckDuckGo's open-source protections. \
                 Not affiliated with or endorsed by DuckDuckGo."
            ),
            "sections": sections,
        })
    }

    /// `text` safe to place in styled text: markup characters are written as entities.
    #[must_use]
    pub fn escaped(text: &str) -> String {
        text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
    }

    fn section(title: Option<&str>, copyright: Option<&str>, body: &str) -> Value {
        json!({ "title": title, "copyright": copyright, "body": body })
    }

    // Each name is written out where it is translated: the translation tooling finds messages by their literal text.
    fn gpl(catalogue: &MessageCatalogue) -> Licence<'_> {
        Licence {
            name: catalogue.text("GNU General Public License, version 3 or later"),
            url: "https://www.gnu.org/licenses/gpl-3.0.html",
        }
    }

    fn apache(catalogue: &MessageCatalogue) -> Licence<'_> {
        Licence {
            name: catalogue.text("Apache License, Version 2.0"),
            url: "https://www.apache.org/licenses/LICENSE-2.0",
        }
    }

    fn mpl(catalogue: &MessageCatalogue) -> Licence<'_> {
        Licence { name: catalogue.text("Mozilla Public License 2.0"), url: "https://www.mozilla.org/en-US/MPL/2.0/" }
    }

    fn bsd3(catalogue: &MessageCatalogue) -> Licence<'_> {
        Licence { name: catalogue.text("BSD 3-Clause License"), url: "https://opensource.org/licenses/BSD-3-Clause" }
    }

    /// "This application comes with absolutely no warranty. See the {license} for details."
    fn licence_sentence(catalogue: &MessageCatalogue, licence: &Licence<'_>) -> String {
        let link = format!("<a href=\"{}\">{}</a>", licence.url, Self::escaped(licence.name));
        catalogue.text_with(
            "This application comes with absolutely no warranty. See the {license} for details.",
            &[("license", &link)],
        )
    }
}
