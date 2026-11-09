//! A page that could not be shown, and why, in terms a person can act on.
//!
//! The application translates the error codes of WebKit and `GLib` into a [`FailureCause`]; the error page then
//! explains that cause. Keeping the causes few and user-facing (not one per error code) is what lets every
//! page name the problem and the way out.

use std::path::PathBuf;

use url::Url;

/// Why a page could not be shown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FailureCause {
    /// The computer has no network connection at all.
    Offline,
    /// The name in the address does not exist (DNS found no server): usually a typo.
    HostNotFound,
    /// The server exists but turned the connection away.
    ConnectionRefused,
    /// The server did not answer in time.
    TimedOut,
    /// The connection broke for another network reason.
    ConnectionInterrupted,
    /// The address uses a port that belongs to non-web services (mail, FTP, ...), which browsers never contact.
    RestrictedPort,
    /// The site's TLS certificate failed verification, for every listed reason.
    UntrustedCertificate(Vec<CertificateProblem>),
    /// The process drawing the page crashed.
    PageCrashed,
    /// The page used more memory than a page may.
    OutOfMemory,
    /// The server answered with an HTTP error status and an empty page.
    HttpStatus(u16),
    /// A `file:` address that points at nothing.
    FileNotFound,
    /// Anything else; the technical detail says what.
    Unknown,
}

/// One reason a TLS certificate was rejected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CertificateProblem {
    Expired,
    NotYetValid,
    /// Not signed by a certificate authority this computer trusts (self-signed, private CA, interception).
    UnknownIssuer,
    /// Issued for a different host name than the one in the address.
    WrongSite,
    Revoked,
    /// Signed with an algorithm too weak to trust.
    Insecure,
    /// Verification failed without a more specific reason.
    Unverifiable,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadFailure {
    pub address: String,
    pub cause: FailureCause,
    /// What WebKit or `GLib` said, shown small for people who need to report or search the problem.
    pub detail: String,
}

impl LoadFailure {
    #[must_use]
    pub fn new(address: &str, cause: FailureCause, detail: &str) -> Self {
        Self { address: address.to_owned(), cause, detail: detail.trim().to_owned() }
    }

    /// The host of a web address (`mbank.pl`), else the address itself.
    #[must_use]
    pub fn host(&self) -> String {
        self.url().and_then(|url| url.host_str().map(str::to_owned)).unwrap_or_else(|| self.address.clone())
    }

    /// The local path of a `file:` address.
    #[must_use]
    pub fn file_path(&self) -> Option<PathBuf> {
        self.url().filter(|url| url.scheme() == "file").and_then(|url| url.to_file_path().ok())
    }

    /// Whether following the address again is a meaningful retry, and safe to offer as a link.
    #[must_use]
    pub fn can_be_retried(&self) -> bool {
        self.url().is_some_and(|url| matches!(url.scheme(), "http" | "https" | "file"))
    }

    /// The site's home page, when the failed page is not the home page already.
    #[must_use]
    pub fn site_home(&self) -> Option<String> {
        let url = self.url().filter(|url| matches!(url.scheme(), "http" | "https"))?;
        let home = format!("{}/", url.origin().ascii_serialization());
        (home != url.as_str()).then_some(home)
    }

    /// The port of the address, when it names one.
    #[must_use]
    pub fn port(&self) -> Option<u16> {
        self.url().and_then(|url| url.port())
    }

    fn url(&self) -> Option<Url> {
        Url::parse(&self.address).ok()
    }
}
