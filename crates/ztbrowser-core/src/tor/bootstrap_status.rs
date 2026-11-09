//! How far Tor has got connecting to the Tor network ("bootstrapping"), from its `BOOTSTRAP` status reports.
//!
//! Tor reports each step twice over: as the answer to `GETINFO status/bootstrap-phase` and as asynchronous
//! `STATUS_CLIENT` events while it works. Both carry the same text:
//! `NOTICE BOOTSTRAP PROGRESS=50 TAG=loading_descriptors SUMMARY="Loading relay descriptors"`. When something
//! goes wrong the severity becomes `WARN` and the report adds `WARNING="Connection refused"` and a
//! `RECOMMENDATION`: `warn` when the user should be told, `ignore` when Tor expects to recover by itself.

use crate::language::message_catalogue::MessageCatalogue;
use crate::tor::keyword_arguments::KeywordArguments;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootstrapStatus {
    progress: u8,
    tag: String,
    summary: String,
    /// Tor warns and wants the user to know.
    is_problem: bool,
    warning: Option<String>,
}

impl BootstrapStatus {
    const BOOTSTRAP: &'static str = "BOOTSTRAP";
    const COMPLETE: u8 = 100;

    /// The report in `text` (the part after `STATUS_CLIENT ` or `status/bootstrap-phase=`); `None` for any other
    /// status report.
    #[must_use]
    pub fn from_report(text: &str) -> Option<Self> {
        let report = KeywordArguments::parse(text);
        let [severity, kind, ..] = report.words() else { return None };
        if kind != Self::BOOTSTRAP {
            return None;
        }
        let progress = report.value("PROGRESS")?.parse::<u8>().ok().filter(|progress| *progress <= Self::COMPLETE)?;
        let is_warning = matches!(severity.as_str(), "WARN" | "ERR");
        let wants_attention = report.value("RECOMMENDATION") != Some("ignore");
        Some(Self {
            progress,
            tag: report.value("TAG").unwrap_or_default().to_owned(),
            summary: report.value("SUMMARY").unwrap_or_default().to_owned(),
            is_problem: is_warning && wants_attention,
            warning: report.value("WARNING").map(str::to_owned),
        })
    }

    /// Per cent of the way, 0 to 100.
    #[must_use]
    pub fn progress(&self) -> u8 {
        self.progress
    }

    /// What Tor is doing (`Loading relay descriptors`). Tor documents a fixed set of step tags, and each is shown
    /// in the interface language; a step Tor adds later is shown in Tor's own (English) words.
    #[must_use]
    pub fn summary<'a>(&'a self, catalogue: &'a MessageCatalogue) -> &'a str {
        match self.tag.as_str() {
            "starting" => catalogue.text("Starting"),
            "conn_pt" => catalogue.text("Connecting to pluggable transport"),
            "conn_done_pt" => catalogue.text("Connected to pluggable transport"),
            "conn_proxy" => catalogue.text("Connecting to proxy"),
            "conn_done_proxy" => catalogue.text("Connected to proxy"),
            "conn_dir" | "conn" => catalogue.text("Connecting to a relay directory server"),
            "conn_done" => catalogue.text("Connected to a relay directory server"),
            "handshake_dir" | "handshake" => catalogue.text("Finishing handshake with a relay directory server"),
            "handshake_done" => catalogue.text("Finished handshake with a relay directory server"),
            "onehop_create" => catalogue.text("Establishing an encrypted directory connection"),
            "requesting_status" => catalogue.text("Asking for the network status"),
            "loading_status" => catalogue.text("Loading the network status"),
            "loading_keys" => catalogue.text("Loading authority certificates"),
            "requesting_descriptors" => catalogue.text("Asking for relay descriptors"),
            "loading_descriptors" => catalogue.text("Loading relay descriptors"),
            "enough_dirinfo" => catalogue.text("Loaded enough directory information to build circuits"),
            "ap_conn_pt" => catalogue.text("Connecting to pluggable transport to build circuits"),
            "ap_conn_done_pt" => catalogue.text("Connected to pluggable transport to build circuits"),
            "ap_conn_proxy" => catalogue.text("Connecting to proxy to build circuits"),
            "ap_conn_done_proxy" => catalogue.text("Connected to proxy to build circuits"),
            "ap_conn" | "conn_or" => catalogue.text("Connecting to a relay to build circuits"),
            "ap_conn_done" => catalogue.text("Connected to a relay to build circuits"),
            "ap_handshake" | "handshake_or" => catalogue.text("Finishing handshake with a relay to build circuits"),
            "ap_handshake_done" => catalogue.text("Finished handshake with a relay to build circuits"),
            "circuit_create" => catalogue.text("Establishing a Tor circuit"),
            "done" => catalogue.text("Connected to the Tor network"),
            _ => &self.summary,
        }
    }

    /// What went wrong, when Tor wants the user to know (`Connection refused`, in Tor's own words); a warning
    /// without text is described in the interface language.
    #[must_use]
    pub fn problem<'a>(&'a self, catalogue: &'a MessageCatalogue) -> Option<&'a str> {
        self.is_problem.then(|| self.warning.as_deref().unwrap_or_else(|| catalogue.text("Tor reported a problem")))
    }

    /// Tor is connected: circuits can be built and the SOCKS port carries traffic.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.progress == Self::COMPLETE
    }
}
