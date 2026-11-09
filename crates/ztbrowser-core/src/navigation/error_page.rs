//! The page ZTBrowser shows instead of one that could not be loaded.
//!
//! Every page names the problem in one heading, says which site it concerns, lists what the user can check,
//! and offers the actions that can actually help (try again, search for a mistyped name, go to the site's
//! home page). The vendored HTML template carries the look; this module fills it, in the interface language.
//!
//! The page is shown under the failed address, so it runs in that site's origin: everything taken from the
//! failure (address, host, the `GLib` message) is escaped, the template forbids scripts and every remote load with
//! a Content Security Policy, and retry links are only emitted for `http`, `https` and `file` addresses.

use crate::language::message_catalogue::MessageCatalogue;
use crate::navigation::load_failure::CertificateProblem;
use crate::navigation::load_failure::FailureCause;
use crate::navigation::load_failure::LoadFailure;
use crate::navigation::search_engine::SearchEngine;

/// How the browser around the page looks and speaks, so the page matches it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PageAppearance {
    pub is_dark: bool,
    /// The desktop's accent colour (button background), as red, green, blue.
    pub accent: [u8; 3],
    /// The BCP 47 tag of the interface language, written to the page's `lang` attribute.
    pub language_tag: &'static str,
    /// Whether the language is written from right to left, which mirrors the page (`dir="rtl"`).
    pub is_right_to_left: bool,
}

pub struct ErrorPage {
    template: String,
}

impl ErrorPage {
    #[must_use]
    pub fn new(template: &str) -> Self {
        Self { template: template.to_owned() }
    }

    /// The complete HTML document explaining `failure`, worded by `catalogue`; a mistyped name is searched with
    /// `search_engine`.
    #[must_use]
    pub fn render(
        &self,
        failure: &LoadFailure,
        appearance: &PageAppearance,
        search_engine: SearchEngine,
        catalogue: &MessageCatalogue,
    ) -> String {
        let explanation = Explanation::of(failure, search_engine, catalogue);
        let [red, green, blue] = appearance.accent;
        Template::fill(&self.template, |placeholder| match placeholder {
            "scheme" => Some(if appearance.is_dark { "dark" } else { "light" }.to_owned()),
            "accent" => Some(format!("#{red:02x}{green:02x}{blue:02x}")),
            "language" => Some(Markup::escaped(appearance.language_tag)),
            "direction" => Some(if appearance.is_right_to_left { "rtl" } else { "ltr" }.to_owned()),
            "tone" => Some(explanation.tone.to_owned()),
            "glyph" => Some(explanation.glyph.to_owned()),
            "tab_title" => Some(Markup::escaped(&explanation.tab_title)),
            "title" => Some(Markup::escaped(explanation.title)),
            "lead" => Some(explanation.lead.clone()),
            "points" => Some(explanation.points_markup()),
            "actions" => Some(explanation.actions_markup()),
            "detail" => Some(explanation.detail_markup()),
            _ => None,
        })
    }
}

/// What the page says and offers for one failure.
///
/// Every sentence is looked up whole in the catalogue (`catalogue.text("...")`), with named placeholders for
/// the parts that vary, so a translator can reorder them. The values put into placeholders are escaped before
/// they are substituted; the catalogue's own text is compiled into the program and trusted.
struct Explanation<'a> {
    failure: &'a LoadFailure,
    search_engine: SearchEngine,
    catalogue: &'a MessageCatalogue,
    /// The host as markup, ready to be placed into the lead.
    host: String,
    tab_title: String,
    title: &'a str,
    /// Markup: its variable parts are already escaped.
    lead: String,
    points: Vec<&'a str>,
    note: Option<&'a str>,
    actions: Vec<PageAction>,
    detail: String,
    /// Which symbol the illustration's badge shows (a class in the template).
    glyph: &'static str,
    /// Badge colour: `accent` for ordinary failures, `danger` for security, `warning` for crashes.
    tone: &'static str,
}

struct PageAction {
    label: String,
    address: String,
    is_primary: bool,
}

impl<'a> Explanation<'a> {
    fn of(failure: &'a LoadFailure, search_engine: SearchEngine, catalogue: &'a MessageCatalogue) -> Self {
        let mut explanation = Self {
            failure,
            search_engine,
            catalogue,
            host: Markup::strong(&failure.host()),
            tab_title: failure.host(),
            title: catalogue.text("This site can’t be reached"),
            lead: String::new(),
            points: Vec::new(),
            note: None,
            actions: Vec::new(),
            detail: failure.detail.clone(),
            glyph: "alert",
            tone: "accent",
        };
        match &failure.cause {
            FailureCause::UntrustedCertificate(problems) => explanation.explain_certificate(problems),
            FailureCause::PageCrashed | FailureCause::OutOfMemory => explanation.explain_crash(),
            FailureCause::HttpStatus(status) => explanation.explain_http_status(*status),
            FailureCause::FileNotFound => explanation.explain_missing_file(),
            FailureCause::RestrictedPort => explanation.explain_restricted_port(),
            FailureCause::Offline
            | FailureCause::HostNotFound
            | FailureCause::ConnectionRefused
            | FailureCause::TimedOut
            | FailureCause::ConnectionInterrupted
            | FailureCause::Unknown => explanation.explain_connection(),
        }
        explanation
    }

    fn try_again(&self) -> &'a str {
        self.catalogue.text("Try Again")
    }

    fn reload(&self) -> &'a str {
        self.catalogue.text("Reload Page")
    }

    fn check_connection(&self) -> &'a str {
        self.catalogue.text("Check your internet connection.")
    }

    fn check_typo(&self) -> &'a str {
        self.catalogue.text("Check the address for a typo.")
    }

    fn check_network_tools(&self) -> &'a str {
        self.catalogue.text("If you use a VPN, firewall or proxy, check that it isn’t blocking the site.")
    }

    fn try_later(&self) -> &'a str {
        self.catalogue.text("Try again in a few minutes.")
    }

    fn explain_connection(&mut self) {
        let catalogue = self.catalogue;
        let host = self.host.clone();
        let values = [("host", host.as_str())];
        match self.failure.cause {
            FailureCause::Offline => {
                self.title = catalogue.text("You’re offline");
                self.lead = catalogue.text_with(
                    "ZTBrowser can’t open {host} because this computer isn’t connected to a network.",
                    &values,
                );
                self.points = vec![
                    catalogue.text("Check the network cable or the Wi-Fi connection."),
                    catalogue.text("Turn off airplane mode, if it is on."),
                    catalogue.text("Reconnect your VPN, if you use one."),
                ];
                self.glyph = "offline";
            }
            FailureCause::HostNotFound => {
                self.title = catalogue.text("This site can’t be found");
                self.lead = catalogue.text_with("ZTBrowser couldn’t find a server at {host}.", &values);
                self.points = vec![
                    self.check_typo(),
                    catalogue.text("If the address is right, the site may no longer exist."),
                    catalogue.text("If you use a VPN or your own DNS server, check that it works."),
                ];
                self.glyph = "search";
            }
            FailureCause::ConnectionRefused => {
                self.lead = catalogue.text_with("{host} refused the connection.", &values);
                self.points = vec![
                    catalogue.text("The site may be down for maintenance. Try again in a few minutes."),
                    catalogue.text("Check the address, including the port number after the colon, if there is one."),
                    self.check_network_tools(),
                ];
            }
            FailureCause::TimedOut => {
                self.lead = catalogue.text_with("{host} took too long to respond.", &values);
                self.points = vec![
                    self.check_connection(),
                    catalogue.text("The site may be overloaded. Try again in a few minutes."),
                    self.check_network_tools(),
                ];
                self.glyph = "clock";
            }
            FailureCause::ConnectionInterrupted => {
                self.lead = catalogue.text_with("The connection to {host} was interrupted.", &values);
                self.points = vec![self.check_connection(), self.try_later(), self.check_network_tools()];
            }
            _ => {
                self.title = catalogue.text("This page can’t be loaded");
                self.lead = catalogue.text_with("Something went wrong while loading {host}.", &values);
                self.points = vec![self.check_connection(), self.try_later()];
            }
        }
        self.offer_retry(self.try_again());
        if self.failure.cause == FailureCause::HostNotFound {
            self.offer_search();
        }
    }

    fn explain_certificate(&mut self, problems: &[CertificateProblem]) {
        let catalogue = self.catalogue;
        self.title = catalogue.text("This connection isn’t private");
        self.lead = catalogue.text_with(
            "ZTBrowser stopped loading {host} because its security certificate can’t be trusted. Someone may be \
             trying to read or change what you send to this site, such as passwords, messages or card numbers.",
            &[("host", self.host.as_str())],
        );
        let problems = if problems.is_empty() { &[CertificateProblem::Unverifiable][..] } else { problems };
        self.points = problems.iter().map(|problem| Self::certificate_problem(catalogue, *problem)).collect();
        self.note = Some(catalogue.text(
            "ZTBrowser never opens a site over a connection it can’t verify, so this warning can’t be skipped by \
             mistake.",
        ));
        self.glyph = "lock";
        self.tone = "danger";
        self.offer_retry(self.try_again());
    }

    fn certificate_problem(catalogue: &'a MessageCatalogue, problem: CertificateProblem) -> &'a str {
        match problem {
            CertificateProblem::Expired => catalogue
                .text("The certificate has expired. If this computer’s date is wrong, correct it and try again."),
            CertificateProblem::NotYetValid => {
                catalogue.text("The certificate isn’t valid yet. Check that this computer’s date and time are right.")
            }
            CertificateProblem::UnknownIssuer => {
                catalogue.text("The certificate wasn’t issued by an authority this computer trusts.")
            }
            CertificateProblem::WrongSite => catalogue.text("The certificate was issued for a different site."),
            CertificateProblem::Revoked => catalogue.text("The authority that issued the certificate has revoked it."),
            CertificateProblem::Insecure => {
                catalogue.text("The certificate is signed with an algorithm that is no longer safe.")
            }
            CertificateProblem::Unverifiable => catalogue.text("The certificate could not be verified."),
        }
    }

    fn explain_crash(&mut self) {
        let catalogue = self.catalogue;
        let host = self.host.clone();
        let values = [("host", host.as_str())];
        if self.failure.cause == FailureCause::OutOfMemory {
            self.title = catalogue.text("This page ran out of memory");
            self.lead =
                catalogue.text_with("{host} used more memory than a page may use, so ZTBrowser stopped it.", &values);
            self.points = vec![catalogue.text("Close tabs or apps you don’t need, then reload the page.")];
        } else {
            self.title = catalogue.text("This page crashed");
            self.lead = catalogue
                .text_with("Something went wrong while showing {host}. The rest of ZTBrowser keeps working.", &values);
            self.points = vec![catalogue.text("If it crashes again after reloading, the page itself may be broken.")];
        }
        self.glyph = "crash";
        self.tone = "warning";
        self.offer_retry(self.reload());
    }

    fn explain_http_status(&mut self, status: u16) {
        let catalogue = self.catalogue;
        let host = self.host.clone();
        let values = [("host", host.as_str())];
        match status {
            404 | 410 => {
                self.title = catalogue.text("This page can’t be found");
                self.lead = catalogue.text_with("{host} has no page at this address.", &values);
                self.points = vec![
                    self.check_typo(),
                    catalogue.text("Go to the site’s home page and look for the page from there."),
                ];
                self.glyph = "search";
            }
            401 | 403 => {
                self.title = catalogue.text("Access denied");
                self.lead = catalogue.text_with("{host} doesn’t allow you to see this page.", &values);
                self.points = vec![catalogue.text("If you have an account on this site, sign in and try again.")];
                self.glyph = "blocked";
            }
            500..=599 => {
                self.title = catalogue.text("This page isn’t working");
                self.lead = catalogue.text_with("{host} can’t handle the request right now.", &values);
                self.points = vec![catalogue.text("The problem is on the site’s side. Try again in a few minutes.")];
            }
            _ => {
                self.title = catalogue.text("This page isn’t working");
                self.lead = catalogue.text_with("{host} couldn’t answer this request.", &values);
            }
        }
        self.detail = format!("HTTP {status}");
        self.offer_retry(self.reload());
        if let Some(address) = self.failure.site_home() {
            let label = catalogue.text("Go to Home Page").to_owned();
            self.actions.push(PageAction { label, address, is_primary: false });
        }
    }

    fn explain_missing_file(&mut self) {
        let catalogue = self.catalogue;
        let path =
            self.failure.file_path().map_or_else(|| self.failure.address.clone(), |path| path.display().to_string());
        path.rsplit('/').find(|part| !part.is_empty()).unwrap_or(&path).clone_into(&mut self.tab_title);
        self.title = catalogue.text("File not found");
        self.lead = catalogue.text_with("There is no file at {path}.", &[("path", Markup::strong(&path).as_str())]);
        self.points = vec![catalogue.text("It may have been moved, renamed or deleted.")];
        self.glyph = "file";
        self.offer_retry(self.try_again());
    }

    /// Retrying cannot help: the port is refused before any connection is made.
    fn explain_restricted_port(&mut self) {
        let catalogue = self.catalogue;
        let host = self.host.clone();
        self.title = catalogue.text("This address is blocked");
        self.lead = match self.failure.port() {
            Some(port) => catalogue.text_with(
                "ZTBrowser doesn’t connect to {host} on port {port}, because that port belongs to services other \
                 than websites.",
                &[("host", host.as_str()), ("port", port.to_string().as_str())],
            ),
            None => {
                catalogue.text_with("ZTBrowser doesn’t connect to {host} on this port.", &[("host", host.as_str())])
            }
        };
        self.note = Some(catalogue.text(
            "Ports like this one serve mail, file sharing and other services. Browsers refuse them so that a web \
             page can’t talk to those services on your behalf.",
        ));
        self.glyph = "blocked";
    }

    fn offer_retry(&mut self, label: &str) {
        if self.failure.can_be_retried() {
            let address = self.failure.address.clone();
            self.actions.push(PageAction { label: label.to_owned(), address, is_primary: true });
        }
    }

    /// A mistyped name is most often found by searching for it.
    fn offer_search(&mut self) {
        let address = self.search_engine.search_url(&self.failure.host());
        let label = self.catalogue.text_with("Search {engine}", &[("engine", self.search_engine.name())]);
        self.actions.push(PageAction { label, address, is_primary: false });
    }

    fn points_markup(&self) -> String {
        let mut markup = String::new();
        if !self.points.is_empty() {
            markup.push_str("<ul class=\"points\">");
            for point in &self.points {
                Markup::push_element(&mut markup, "<li>", point, "</li>");
            }
            markup.push_str("</ul>");
        }
        if let Some(note) = self.note {
            Markup::push_element(&mut markup, "<p class=\"note\">", note, "</p>");
        }
        markup
    }

    fn actions_markup(&self) -> String {
        if self.actions.is_empty() {
            return String::new();
        }
        let mut markup = String::from("<div class=\"actions\">");
        for action in &self.actions {
            let opening =
                if action.is_primary { "<a class=\"button primary\" href=\"" } else { "<a class=\"button\" href=\"" };
            Markup::push_element(&mut markup, opening, &action.address, "\">");
            markup.push_str(&Markup::escaped(&action.label));
            markup.push_str("</a>");
        }
        markup.push_str("</div>");
        markup
    }

    fn detail_markup(&self) -> String {
        if self.detail.is_empty() {
            return String::new();
        }
        format!("<p class=\"detail\">{}</p>", Markup::escaped(&self.detail))
    }
}

/// HTML escaping for text taken from the failure, which an attacker can shape (the address, the `GLib` message).
struct Markup;

impl Markup {
    fn escaped(text: &str) -> String {
        let mut escaped = String::with_capacity(text.len());
        for character in text.chars() {
            match character {
                '&' => escaped.push_str("&amp;"),
                '<' => escaped.push_str("&lt;"),
                '>' => escaped.push_str("&gt;"),
                '"' => escaped.push_str("&quot;"),
                '\'' => escaped.push_str("&#39;"),
                other => escaped.push(other),
            }
        }
        escaped
    }

    fn strong(text: &str) -> String {
        format!("<strong>{}</strong>", Self::escaped(text))
    }

    /// Appends `opening`, the escaped `text` and `closing`.
    fn push_element(markup: &mut String, opening: &str, text: &str, closing: &str) {
        markup.push_str(opening);
        markup.push_str(&Self::escaped(text));
        markup.push_str(closing);
    }
}

/// `{{name}}` placeholders, replaced in one pass: a value that contains `{{...}}` is never expanded again.
struct Template;

impl Template {
    fn fill(template: &str, value_of: impl Fn(&str) -> Option<String>) -> String {
        let mut filled = String::with_capacity(template.len() + 4096);
        let mut rest = template;
        while let Some(start) = rest.find("{{") {
            filled.push_str(&rest[..start]);
            let after = &rest[start + 2..];
            if let Some((end, value)) =
                after.find("}}").and_then(|end| value_of(&after[..end]).map(|value| (end, value)))
            {
                filled.push_str(&value);
                rest = &after[end + 2..];
            } else {
                filled.push_str("{{");
                rest = after;
            }
        }
        filled.push_str(rest);
        filled
    }
}
