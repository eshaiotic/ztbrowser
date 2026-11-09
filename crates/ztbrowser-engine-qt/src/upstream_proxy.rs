//! The proxy the person's environment names (`https_proxy`, `http_proxy`, `no_proxy`), used behind the gate.
//!
//! The browser's own connections now go through the connection gate's local proxy, so a proxy the environment
//! asks for must be reached from there. Only plain HTTP proxies without credentials are understood; a value
//! that is anything else is ignored rather than half-honoured.

pub struct UpstreamProxy {
    http: Option<String>,
    https: Option<String>,
    excluded: Vec<String>,
}

impl UpstreamProxy {
    const DEFAULT_PORT: &'static str = "80";

    /// No proxy: every connection goes straight to its host.
    #[must_use]
    pub fn direct() -> Self {
        Self { http: None, https: None, excluded: Vec::new() }
    }

    /// Reads the proxy variables through `variable`, trying the lower-case name before the upper-case one.
    #[must_use]
    pub fn from_variables(variable: impl Fn(&str) -> Option<String>) -> Self {
        let read = |lower: &str| variable(lower).or_else(|| variable(&lower.to_ascii_uppercase()));
        Self {
            http: read("http_proxy").and_then(|value| Self::authority(&value)),
            https: read("https_proxy").and_then(|value| Self::authority(&value)),
            excluded: read("no_proxy")
                .map(|list| {
                    list.split(',').map(|entry| entry.trim().to_ascii_lowercase()).filter(|e| !e.is_empty()).collect()
                })
                .unwrap_or_default(),
        }
    }

    /// The proxy (`host:port`) a tunnel to `host` goes through, `None` for a direct connection.
    #[must_use]
    pub fn for_tunnel(&self, host: &str) -> Option<String> {
        self.https.clone().filter(|_| !self.is_excluded(host))
    }

    /// The proxy (`host:port`) a plain `http://` request to `host` goes through, `None` for a direct connection.
    #[must_use]
    pub fn for_plain_http(&self, host: &str) -> Option<String> {
        self.http.clone().filter(|_| !self.is_excluded(host))
    }

    fn is_excluded(&self, host: &str) -> bool {
        let host = host.to_ascii_lowercase();
        self.excluded.iter().any(|entry| {
            let domain = entry.trim_start_matches('*').trim_start_matches('.');
            entry == "*" || host == domain || host.ends_with(&format!(".{domain}"))
        })
    }

    fn authority(value: &str) -> Option<String> {
        let value = value.trim();
        let value = match value.split_once("://") {
            Some(("http", rest)) => rest,
            Some(_) => return None,
            None => value,
        };
        let value = value.trim_end_matches('/');
        if value.is_empty() || value.contains('@') || value.contains('/') {
            return None;
        }
        Some(if value.rsplit_once(':').is_some_and(|(_, port)| port.parse::<u16>().is_ok()) {
            value.to_owned()
        } else {
            format!("{value}:{}", Self::DEFAULT_PORT)
        })
    }
}
