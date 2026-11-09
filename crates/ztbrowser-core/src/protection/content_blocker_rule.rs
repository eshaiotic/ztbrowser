//! One WebKit content-blocking rule, in the JSON format shared by Safari, `WKWebView` and `WebKitGTK`.
//!
//! WebKit compiles a list of such rules into a bytecode automaton and evaluates it for every sub-resource
//! request in its network process. Rule order matters: an `ignore-previous-rules` action cancels every
//! earlier rule that matched the same request, which is how exceptions are layered on top of blocking.
//!
//! Format reference: <https://webkit.org/blog/3476/content-blockers-first-look/>

use serde_json::Map;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceType {
    Document,
    Image,
    StyleSheet,
    Script,
    Font,
    Raw,
    SvgDocument,
    Media,
    Popup,
    /// `<a ping>` reports, which tell a site's analytics which link you clicked. (WebKitGTK files
    /// `navigator.sendBeacon` under `raw`, with every other script request.)
    Ping,
}

impl ResourceType {
    /// Maps the TDS request-type vocabulary as `TrackerRadarKit` does, which leaves other types (e.g. `ping`)
    /// unmapped.
    #[must_use]
    pub fn from_tds_name(name: &str) -> Option<Self> {
        match name {
            "script" => Some(Self::Script),
            "xmlhttprequest" => Some(Self::Raw),
            "subdocument" => Some(Self::Document),
            "image" => Some(Self::Image),
            "stylesheet" => Some(Self::StyleSheet),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::Image => "image",
            Self::StyleSheet => "style-sheet",
            Self::Script => "script",
            Self::Font => "font",
            Self::Raw => "raw",
            Self::SvgDocument => "svg-document",
            Self::Media => "media",
            Self::Popup => "popup",
            Self::Ping => "ping",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LoadType {
    FirstParty,
    ThirdParty,
}

impl LoadType {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FirstParty => "first-party",
            Self::ThirdParty => "third-party",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LoadContext {
    TopFrame,
    ChildFrame,
}

impl LoadContext {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TopFrame => "top-frame",
            Self::ChildFrame => "child-frame",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuleAction {
    Block,
    IgnorePreviousRules,
    /// Rewrites `http://` to `https://` before the request is sent, in every frame.
    MakeHttps,
}

impl RuleAction {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Block => "block",
            Self::IgnorePreviousRules => "ignore-previous-rules",
            Self::MakeHttps => "make-https",
        }
    }
}

/// The regular expression a request URL must match for a rule to apply.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct UrlFilter {
    regex: String,
}

impl UrlFilter {
    /// Anchored, subdomain-tolerant prefix recommended by WebKit for fast domain matching:
    /// <https://webkit.org/blog/4062/targeting-domains-with-content-blockers/>
    pub const SUBDOMAIN_PREFIX: &'static str = "^[^:]+://+([^:/]+\\.)?";
    const DOMAIN_MATCH_SUFFIX: &'static str = "[:/]";
    /// Where the authority of an exempted plain-HTTP URL must end: an optional port, then the path, which WebKit
    /// always writes (at least `/`). Allowing anything else would let user info pose as the host:
    /// `http://localhost:1@example.com/` is `example.com`, not `localhost`.
    const PLAIN_HTTP_HOST_END: &'static str = "(:[0-9]+)?/";

    #[must_use]
    pub fn any_url() -> Self {
        Self { regex: ".*".to_owned() }
    }

    /// Every plain-HTTP URL.
    #[must_use]
    pub fn plain_http() -> Self {
        Self { regex: "^http://".to_owned() }
    }

    /// Plain-HTTP URLs whose host is an IPv4 address.
    #[must_use]
    pub fn plain_http_ipv4_host() -> Self {
        Self { regex: format!("^http://[0-9.]+{}", Self::PLAIN_HTTP_HOST_END) }
    }

    /// Plain-HTTP URLs whose host is an IPv6 address (user info cannot contain a literal `[`).
    #[must_use]
    pub fn plain_http_ipv6_host() -> Self {
        Self { regex: "^http://\\[".to_owned() }
    }

    /// Plain-HTTP URLs whose host has no dot (`http://nas/`), which only resolve on a local network.
    #[must_use]
    pub fn plain_http_single_label_host() -> Self {
        Self { regex: format!("^http://[^/:?#.@]+{}", Self::PLAIN_HTTP_HOST_END) }
    }

    /// Plain-HTTP URLs on `domain` or its subdomains.
    #[must_use]
    pub fn plain_http_on_domain(domain: &str) -> Self {
        Self { regex: format!("^http://([^/:?#@]+\\.)?{}{}", Self::regex_escaped(domain), Self::PLAIN_HTTP_HOST_END) }
    }

    /// Plain-HTTP URLs on exactly `host`, subdomains excluded.
    #[must_use]
    pub fn plain_http_on_exact_host(host: &str) -> Self {
        Self { regex: format!("^http://{}{}", Self::regex_escaped(host), Self::PLAIN_HTTP_HOST_END) }
    }

    /// URLs whose query (not fragment) contains the parameter `name`, with or without a value.
    #[must_use]
    pub fn carrying_query_parameter(name: &str) -> Self {
        Self { regex: format!("^[^#]*[?&]{}([=&#].*)?$", Self::regex_escaped(name)) }
    }

    /// Every URL on `domain` or its subdomains.
    #[must_use]
    pub fn matching_domain(domain: &str) -> Self {
        Self {
            regex: format!("{}{}{}", Self::SUBDOMAIN_PREFIX, Self::regex_escaped(domain), Self::DOMAIN_MATCH_SUFFIX),
        }
    }

    /// A TDS rule pattern, which is already a regex; relative patterns are anchored on any subdomain.
    #[must_use]
    pub fn matching_tds_pattern(pattern: &str) -> Self {
        let regex = if pattern.starts_with("http") {
            pattern.to_owned()
        } else {
            format!("{}{pattern}", Self::SUBDOMAIN_PREFIX)
        };
        Self { regex }
    }

    /// A plain `host/path` allowlist prefix, tolerating an explicit port after the host.
    #[must_use]
    pub fn matching_allowlisted_url(rule: &str) -> Self {
        let with_port = match rule.split_once('/') {
            Some((host, path)) => format!("{host}(:[0-9]+)?/{path}"),
            None => rule.to_owned(),
        };
        Self { regex: format!("{}{}.*", Self::SUBDOMAIN_PREFIX, Self::regex_escaped(&with_port)) }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.regex
    }

    fn regex_escaped(text: &str) -> String {
        text.replace('\\', "\\\\").replace('.', "\\.").replace('/', "\\/")
    }
}

/// Which requests a rule applies to. `None` fields are omitted from the JSON (they mean "any").
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Trigger {
    pub url_filter: UrlFilter,
    pub unless_domain: Option<Vec<String>>,
    pub if_domain: Option<Vec<String>>,
    pub resource_type: Option<Vec<ResourceType>>,
    pub load_type: Option<Vec<LoadType>>,
    pub load_context: Option<Vec<LoadContext>>,
}

impl Trigger {
    #[must_use]
    pub fn new(url_filter: UrlFilter) -> Self {
        Self { url_filter, ..Self::default() }
    }

    #[must_use]
    pub fn to_json(&self) -> Value {
        let mut fields = Map::new();
        fields.insert("url-filter".to_owned(), Value::from(self.url_filter.as_str()));
        Self::insert_list(&mut fields, "unless-domain", self.unless_domain.as_deref(), String::as_str);
        Self::insert_list(&mut fields, "if-domain", self.if_domain.as_deref(), String::as_str);
        Self::insert_list(&mut fields, "resource-type", self.resource_type.as_deref(), |kind| kind.as_str());
        Self::insert_list(&mut fields, "load-type", self.load_type.as_deref(), |kind| kind.as_str());
        Self::insert_list(&mut fields, "load-context", self.load_context.as_deref(), |kind| kind.as_str());
        Value::Object(fields)
    }

    fn insert_list<T>(fields: &mut Map<String, Value>, key: &str, items: Option<&[T]>, label: impl Fn(&T) -> &str) {
        if let Some(items) = items {
            fields.insert(key.to_owned(), items.iter().map(|item| Value::from(label(item))).collect());
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContentBlockerRule {
    pub trigger: Trigger,
    pub action: RuleAction,
}

impl ContentBlockerRule {
    #[must_use]
    pub fn new(trigger: Trigger, action: RuleAction) -> Self {
        Self { trigger, action }
    }

    #[must_use]
    pub fn to_json(&self) -> Value {
        let mut action = Map::new();
        action.insert("type".to_owned(), Value::from(self.action.as_str()));
        let mut rule = Map::new();
        rule.insert("trigger".to_owned(), self.trigger.to_json());
        rule.insert("action".to_owned(), Value::Object(action));
        Value::Object(rule)
    }
}
