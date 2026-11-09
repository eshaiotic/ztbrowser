//! Generic element hiding (`##.ad-banner`: a filter naming no site), applied lazily by Guard's page script.
//!
//! WebKit matches every selector of a content blocker's style sheet against every element at every style
//! recalculation. With the thirty thousand generic selectors of Guard's lists in it, styles took six times
//! longer (measured on a 10 000-element page). So generic selectors stay out of WebKit's lists, as in uBlock
//! Origin and Brave. Nearly all of them need a class or id on the page (`.ad-banner`, `div.sponsored > a`):
//! they are indexed by it, and a page gets only those whose class or id it uses, usually a handful. The rest
//! (`[data-ad-slot]`, `a:not(.safe)`) go to the page script, which tests them on the page and keeps only the
//! ones that match anything.
//!
//! Exceptions are honoured here, which `adblock`'s WebKit conversion does not do: an element exception
//! (`bank.pl#@#.ad`) arrives as a second copy of the selector excluding the site, and is merged into the
//! first; a `$generichide` exception (`@@||bank.pl^$generichide`, a site generic hiding breaks) switches
//! generic hiding off on the site.

use std::collections::HashMap;

use serde_json::Value;
use serde_json::json;

/// One generic selector and the domains its filter excludes (`~bank.pl##.ad`).
struct HidingSelector {
    selector: String,
    unless_domains: Vec<String>,
}

impl HidingSelector {
    /// `*bank.pl` excludes the domain and its subdomains, `bank.pl` the domain alone, as in WebKit's
    /// `unless-domain`.
    fn applies_on(&self, host: &str) -> bool {
        !self.unless_domains.iter().any(|domain| match domain.strip_prefix('*') {
            Some(parent) => GenericHiding::is_on_site(host, parent),
            None => host == domain,
        })
    }
}

#[derive(Default)]
pub struct GenericHiding {
    selectors: Vec<HidingSelector>,
    /// Selector text → its entry, so that every filter naming a selector lands in one entry.
    by_selector: HashMap<String, usize>,
    /// `.class` or `#id` → the selectors needing it.
    keyed: HashMap<String, Vec<usize>>,
    unkeyed: Vec<usize>,
    /// Sites (and their subdomains) a `$generichide` exception switches generic hiding off on.
    switched_off_on: Vec<String>,
}

impl GenericHiding {
    /// Hides `selector` everywhere but `unless_domains`; a selector added again is excluded from the domains
    /// of both, since a second copy is how an element exception arrives.
    pub fn add(&mut self, selector: &str, unless_domains: &[String]) {
        if let Some(&index) = self.by_selector.get(selector) {
            let excluded = &mut self.selectors[index].unless_domains;
            for domain in unless_domains {
                if !excluded.contains(domain) {
                    excluded.push(domain.clone());
                }
            }
            return;
        }
        let index = self.selectors.len();
        match Self::needed_token(selector) {
            Some(token) => self.keyed.entry(token).or_default().push(index),
            None => self.unkeyed.push(index),
        }
        self.by_selector.insert(selector.to_owned(), index);
        self.selectors.push(HidingSelector { selector: selector.to_owned(), unless_domains: unless_domains.to_vec() });
    }

    /// Hides nothing generic on `site` and its subdomains.
    pub fn switch_off_on(&mut self, site: &str) {
        if !self.switched_off_on.iter().any(|known| known == site) {
            self.switched_off_on.push(site.to_owned());
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.selectors.is_empty()
    }

    /// The selectors that apply on `host` and need one of `tokens` (`.class`, `#id`: what the page uses).
    #[must_use]
    pub fn selectors_for(&self, host: &str, tokens: &[String]) -> Vec<&str> {
        if self.is_switched_off_on(host) {
            return Vec::new();
        }
        tokens
            .iter()
            .filter_map(|token| self.keyed.get(token))
            .flatten()
            .filter_map(|&index| self.on(host, index))
            .collect()
    }

    /// The selectors that apply on `host` but no class or id narrows down: the page tests them itself.
    #[must_use]
    pub fn unkeyed_selectors_for(&self, host: &str) -> Vec<&str> {
        if self.is_switched_off_on(host) {
            return Vec::new();
        }
        self.unkeyed.iter().filter_map(|&index| self.on(host, index)).collect()
    }

    /// The stored form: `{"selectors": [[selector, [excluded domains]], ...], "switchedOffOn": [sites]}`; the
    /// index is rebuilt on reading.
    #[must_use]
    pub fn to_json(&self) -> String {
        let selectors: Vec<Value> =
            self.selectors.iter().map(|hiding| json!([hiding.selector, hiding.unless_domains])).collect();
        json!({ "selectors": selectors, "switchedOffOn": self.switched_off_on }).to_string()
    }

    /// Reads a stored copy; `None` when it is damaged.
    #[must_use]
    pub fn from_json(text: &str) -> Option<Self> {
        let document: Value = serde_json::from_str(text).ok()?;
        let mut hiding = Self::default();
        for entry in document.get("selectors")?.as_array()? {
            let selector = entry.get(0)?.as_str()?;
            let domains: Option<Vec<String>> =
                entry.get(1)?.as_array()?.iter().map(|domain| domain.as_str().map(str::to_owned)).collect();
            hiding.add(selector, &domains?);
        }
        for site in document.get("switchedOffOn")?.as_array()? {
            hiding.switch_off_on(site.as_str()?);
        }
        Some(hiding)
    }

    fn is_switched_off_on(&self, host: &str) -> bool {
        self.switched_off_on.iter().any(|site| Self::is_on_site(host, site))
    }

    /// Whether `host` is `site` or one of its subdomains.
    fn is_on_site(host: &str, site: &str) -> bool {
        host.strip_suffix(site).is_some_and(|rest| rest.is_empty() || rest.ends_with('.'))
    }

    fn on(&self, host: &str, index: usize) -> Option<&str> {
        let hiding = &self.selectors[index];
        hiding.applies_on(host).then_some(hiding.selector.as_str())
    }

    /// The class or id an element of the page must carry for `selector` to match anything: the first one
    /// outside brackets, parentheses and quotes (a class inside `:not(...)` is not needed, a dot inside
    /// `[href*=".net"]` is no class). `None` for a selector list, whose alternatives need different ones, and
    /// for escaped names, which the page would report unescaped.
    fn needed_token(selector: &str) -> Option<String> {
        if selector.contains('\\') {
            return None;
        }
        let characters: Vec<char> = selector.chars().collect();
        let (mut depth, mut quote, mut token) = (0_usize, None, None);
        for (position, &character) in characters.iter().enumerate() {
            if let Some(open) = quote {
                if character == open {
                    quote = None;
                }
                continue;
            }
            match character {
                '"' | '\'' => quote = Some(character),
                '(' | '[' => depth += 1,
                ')' | ']' => depth = depth.saturating_sub(1),
                ',' if depth == 0 => return None,
                '.' | '#' if depth == 0 && token.is_none() => token = Self::name_at(&characters, position),
                _ => {}
            }
        }
        token
    }

    /// `.ad-banner` from the sigil at `position`, if a CSS name follows it.
    fn name_at(characters: &[char], position: usize) -> Option<String> {
        let is_name = |character: char| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_') || !character.is_ascii()
        };
        let name: String = characters[position + 1..].iter().take_while(|&&character| is_name(character)).collect();
        let starts_well = name.chars().next().is_some_and(|first| !first.is_ascii_digit());
        (starts_well && name != "-").then(|| format!("{}{name}", characters[position]))
    }
}
