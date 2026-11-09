//! The community filter lists ZTBrowser Guard reads, downloaded at run time from their publishers.
//!
//! A filter list is a text file in Adblock Plus syntax: `||ads.example^` blocks a domain, `example.com##.ad`
//! hides matching elements, `@@` marks an exception. Volunteers maintain them; uBlock Origin, Brave and AdGuard
//! read the same files. Their licences (GPL-3.0, CC BY-SA, CC BY-NC-SA, the publisher's own terms) do not fit
//! an Apache-2.0 package, so, like DuckDuckGo's Tracker Data Set, they are never bundled.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FilterListSource {
    id: &'static str,
    name: &'static str,
    url: &'static str,
}

impl FilterListSource {
    pub const EASYLIST: Self =
        Self { id: "easylist", name: "EasyList", url: "https://easylist.to/easylist/easylist.txt" };
    pub const EASYPRIVACY: Self =
        Self { id: "easyprivacy", name: "EasyPrivacy", url: "https://easylist.to/easylist/easyprivacy.txt" };
    pub const UBLOCK_FILTERS: Self = Self {
        id: "ublock-filters",
        name: "uBlock filters",
        url: "https://ublockorigin.github.io/uAssets/filters/filters.txt",
    };
    pub const UBLOCK_PRIVACY: Self = Self {
        id: "ublock-privacy",
        name: "uBlock filters – Privacy",
        url: "https://ublockorigin.github.io/uAssets/filters/privacy.txt",
    };
    pub const UBLOCK_BADWARE: Self = Self {
        id: "ublock-badware",
        name: "uBlock filters – Badware risks",
        url: "https://ublockorigin.github.io/uAssets/filters/badware.txt",
    };
    /// Exceptions that undo blocks known to break sites; read into every Guard rule list.
    pub const UBLOCK_UNBREAK: Self = Self {
        id: "ublock-unbreak",
        name: "uBlock filters – Unbreak",
        url: "https://ublockorigin.github.io/uAssets/filters/unbreak.txt",
    };
    pub const PETER_LOWE: Self = Self {
        id: "peter-lowe",
        name: "Peter Lowe's Ad and tracking server list",
        url: "https://pgl.yoyo.org/adservers/serverlist.php?hostformat=adblockplus&showintro=0&mimetype=plaintext",
    };
    pub const POLISH_FILTERS: Self = Self {
        id: "polish-filters",
        name: "Polish Filters",
        url: "https://raw.githubusercontent.com/MajkiIT/polish-ads-filter/master/polish-adblock-filters/adblock.txt",
    };
    pub const COOKIE_NOTICES: Self = Self {
        id: "cookie-notices",
        name: "EasyList Cookie List",
        url: "https://secure.fanboy.co.nz/fanboy-cookiemonster_ubo.txt",
    };
    /// Every list ZTBrowser downloads, each once however many rule lists read it.
    pub const ALL: [Self; 9] = [
        Self::EASYLIST,
        Self::EASYPRIVACY,
        Self::UBLOCK_FILTERS,
        Self::UBLOCK_PRIVACY,
        Self::UBLOCK_BADWARE,
        Self::UBLOCK_UNBREAK,
        Self::PETER_LOWE,
        Self::POLISH_FILTERS,
        Self::COOKIE_NOTICES,
    ];

    /// A stable identifier (`easylist`), for the cached copy's file name.
    #[must_use]
    pub fn id(self) -> &'static str {
        self.id
    }

    #[must_use]
    pub fn name(self) -> &'static str {
        self.name
    }

    #[must_use]
    pub fn url(self) -> &'static str {
        self.url
    }

    /// Whether a download is a filter list rather than, say, a hotel Wi-Fi login page served in its place:
    /// its first non-blank line is a list header (`[Adblock Plus 2.0]`) or a comment (`! Title: …`).
    #[must_use]
    pub fn accepts(text: &str) -> bool {
        text.lines()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .is_some_and(|line| line.starts_with('[') || line.starts_with('!'))
    }
}
