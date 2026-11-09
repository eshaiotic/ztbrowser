//! ZTBrowser's error pages in a web view, in place of WebKit's plain built-in ones.
//!
//! WebKit, libsoup and `GLib` report a failed load as an error code from one of several domains (resolver, I/O,
//! TLS, WebKit's own). This adapter turns those codes into the few user-facing causes of `ztbrowser-core`,
//! renders the matching page in the browser's current light or dark style and accent colour, and loads it as
//! "alternate HTML" for the failed address, so the address bar, Reload and Back behave as for the failed page.

use crate::interface_messages::InterfaceMessages;
use gio::prelude::NetworkMonitorExt;
use gio::prelude::TlsCertificateExt;
use webkit6::NetworkError;
use webkit6::PolicyError;
use webkit6::WebView;
use webkit6::prelude::WebViewExt;
use ztbrowser_core::navigation::error_page::ErrorPage;
use ztbrowser_core::navigation::error_page::PageAppearance;
use ztbrowser_core::navigation::load_failure::CertificateProblem;
use ztbrowser_core::navigation::load_failure::FailureCause;
use ztbrowser_core::navigation::load_failure::LoadFailure;
use ztbrowser_core::navigation::search_engine::SearchEngine;

pub struct ErrorPageView;

impl ErrorPageView {
    const TEMPLATE: &'static str = include_str!("../../../assets/error-page/error-page.html");

    /// Shows the page explaining `failure` under the failed address; a mistyped name is searched with
    /// `search_engine`.
    pub fn show(web_view: &WebView, failure: &LoadFailure, search_engine: SearchEngine) {
        let html = ErrorPage::new(Self::TEMPLATE).render(
            failure,
            &Self::appearance(),
            search_engine,
            InterfaceMessages::catalogue(),
        );
        web_view.load_alternate_html(&html, &failure.address, None);
    }

    /// Why a load failed, or `None` when the failure is not one to show: the user stopped it, or it became a
    /// download.
    pub fn cause_of(error: &glib::Error) -> Option<FailureCause> {
        if error.matches(NetworkError::Cancelled) || error.matches(PolicyError::FrameLoadInterruptedByPolicyChange) {
            return None;
        }
        let cause = if error.matches(PolicyError::CannotUseRestrictedPort) {
            FailureCause::RestrictedPort
        } else if error.matches(NetworkError::FileDoesNotExist) || error.matches(gio::IOErrorEnum::NotFound) {
            FailureCause::FileNotFound
        } else if error.kind::<gio::TlsError>().is_some() {
            FailureCause::UntrustedCertificate(Vec::new())
        } else if Self::is_network_error(error) && !gio::NetworkMonitor::default().is_network_available() {
            FailureCause::Offline
        } else if error.kind::<gio::ResolverError>().is_some() {
            FailureCause::HostNotFound
        } else {
            match error.kind::<gio::IOErrorEnum>() {
                Some(gio::IOErrorEnum::ConnectionRefused) => FailureCause::ConnectionRefused,
                // libsoup gives up on a server that never answers by cancelling the connection attempt.
                Some(gio::IOErrorEnum::TimedOut | gio::IOErrorEnum::Cancelled) => FailureCause::TimedOut,
                Some(_) => FailureCause::ConnectionInterrupted,
                None if error.matches(NetworkError::Transport) => FailureCause::ConnectionInterrupted,
                None => FailureCause::Unknown,
            }
        };
        Some(cause)
    }

    /// Every reason `GLib` gave for rejecting a certificate.
    pub fn certificate_cause(errors: gio::TlsCertificateFlags) -> FailureCause {
        let reasons = [
            (gio::TlsCertificateFlags::EXPIRED, CertificateProblem::Expired),
            (gio::TlsCertificateFlags::NOT_ACTIVATED, CertificateProblem::NotYetValid),
            (gio::TlsCertificateFlags::UNKNOWN_CA, CertificateProblem::UnknownIssuer),
            (gio::TlsCertificateFlags::BAD_IDENTITY, CertificateProblem::WrongSite),
            (gio::TlsCertificateFlags::REVOKED, CertificateProblem::Revoked),
            (gio::TlsCertificateFlags::INSECURE, CertificateProblem::Insecure),
            (gio::TlsCertificateFlags::GENERIC_ERROR, CertificateProblem::Unverifiable),
        ];
        let problems = reasons.into_iter().filter(|(flag, _)| errors.contains(*flag)).map(|(_, problem)| problem);
        FailureCause::UntrustedCertificate(problems.collect())
    }

    /// Who the certificate was issued to, by whom and until when: what an administrator needs to fix it.
    pub fn certificate_detail(certificate: &gio::TlsCertificate) -> String {
        let mut parts = Vec::new();
        if let Some(subject) = certificate.subject_name() {
            parts.push(tr!("Certificate for {name}", name = Self::common_name(&subject)));
        }
        if let Some(issuer) = certificate.issuer_name() {
            parts.push(tr!("Issued by {name}", name = Self::common_name(&issuer)));
        }
        if let Some(until) = certificate.not_valid_after().and_then(|date| date.format("%Y-%m-%d").ok()) {
            parts.push(tr!("Valid until {date}", date = until));
        }
        parts.join(" · ")
    }

    /// `*.badssl.com` of `OU=Domain Control Validated,CN=*.badssl.com`; the whole name when it has no `CN`.
    fn common_name(distinguished_name: &str) -> &str {
        distinguished_name.split(',').find_map(|part| part.trim().strip_prefix("CN=")).unwrap_or(distinguished_name)
    }

    fn is_network_error(error: &glib::Error) -> bool {
        error.kind::<gio::ResolverError>().is_some()
            || error.kind::<gio::IOErrorEnum>().is_some()
            || error.matches(NetworkError::Transport)
    }

    fn appearance() -> PageAppearance {
        let style = adw::StyleManager::default();
        let accent = style.accent_color_rgba();
        PageAppearance {
            is_dark: style.is_dark(),
            accent: [Self::channel(accent.red()), Self::channel(accent.green()), Self::channel(accent.blue())],
            language_tag: InterfaceMessages::language().tag(),
            is_right_to_left: InterfaceMessages::language().is_right_to_left(),
        }
    }

    /// A colour channel from 0.0–1.0 to 0–255, without a lossy cast: the first level at or above the value.
    fn channel(value: f32) -> u8 {
        let scaled = (value.clamp(0.0, 1.0) * 255.0).round();
        (0..=u8::MAX).find(|level| f32::from(*level) >= scaled).unwrap_or(u8::MAX)
    }
}
