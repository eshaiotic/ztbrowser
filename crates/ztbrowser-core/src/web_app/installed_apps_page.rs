//! The page that lists the installed web apps as tiles, like the apps page of other browsers.
//!
//! It is a page of its own, made of static HTML: the browser loads it under [`InstalledAppsPage::ADDRESS`] and
//! recognises its two kinds of link (open an app, uninstall one) with [`InstalledAppsLink`]. Names and hosts come
//! from sites, so every text is escaped, and an icon is accepted only as a `data:image/…;base64,…` address.

use crate::language::message_catalogue::MessageCatalogue;
use crate::web_app::web_app_id::WebAppId;

/// One app on the page.
#[derive(Debug, Clone, Copy)]
pub struct AppTile<'a> {
    pub id: &'a WebAppId,
    pub name: &'a str,
    pub host: &'a str,
    /// The app's icon as a `data:` address; a tile without one shows the name's first letter.
    pub icon: Option<&'a str>,
}

/// What a click on the page asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstalledAppsLink {
    Open(WebAppId),
    Uninstall(WebAppId),
}

impl InstalledAppsLink {
    const OPEN_PREFIX: &'static str = "ztbrowser-app-open:";
    const UNINSTALL_PREFIX: &'static str = "ztbrowser-app-uninstall:";

    /// The link `address` is, or `None` for any other address and for an identifier that is not a safe one.
    #[must_use]
    pub fn parse(address: &str) -> Option<Self> {
        if let Some(id) = address.strip_prefix(Self::OPEN_PREFIX) {
            return WebAppId::parse(id).ok().map(Self::Open);
        }
        address.strip_prefix(Self::UNINSTALL_PREFIX).and_then(|id| WebAppId::parse(id).ok()).map(Self::Uninstall)
    }

    fn open_address(id: &WebAppId) -> String {
        format!("{}{}", Self::OPEN_PREFIX, id.as_str())
    }

    fn uninstall_address(id: &WebAppId) -> String {
        format!("{}{}", Self::UNINSTALL_PREFIX, id.as_str())
    }
}

/// The colours of the page, taken from libadwaita's palette so it sits in the window like the dialogs do.
struct Palette {
    background: &'static str,
    card: &'static str,
    text: &'static str,
    dimmed_text: &'static str,
    border: &'static str,
    accent: &'static str,
    destructive: &'static str,
}

impl Palette {
    const LIGHT: Self = Self {
        background: "#fafafb",
        card: "#ffffff",
        text: "#2e2e32",
        dimmed_text: "#6d6d73",
        border: "rgba(0,0,0,0.12)",
        accent: "#1c71d8",
        destructive: "#c01c28",
    };
    const DARK: Self = Self {
        background: "#222226",
        card: "#2e2e32",
        text: "#ffffff",
        dimmed_text: "#a8a8ae",
        border: "rgba(255,255,255,0.14)",
        accent: "#78aeed",
        destructive: "#ff7b63",
    };
}

pub struct InstalledAppsPage<'a> {
    title: &'a str,
    description: &'a str,
    open: &'a str,
    uninstall: &'a str,
    nothing_installed: &'a str,
    palette: &'static Palette,
}

impl<'a> InstalledAppsPage<'a> {
    /// What the address bar shows for the page.
    pub const ADDRESS: &'static str = "ztbrowser://apps";

    #[must_use]
    pub fn new(catalogue: &'a MessageCatalogue, is_dark: bool) -> Self {
        Self {
            title: catalogue.text("Installed Apps"),
            description: catalogue
                .text("Each app runs in its own window and process, with cookies and logins of its own."),
            open: catalogue.text("Open"),
            uninstall: catalogue.text("Uninstall"),
            nothing_installed: catalogue.text("No apps installed yet"),
            palette: if is_dark { &Palette::DARK } else { &Palette::LIGHT },
        }
    }

    /// The question asked before an app is removed.
    #[must_use]
    pub fn uninstall_heading(catalogue: &MessageCatalogue, app_name: &str) -> String {
        catalogue.text_with("Uninstall {name}?", &[("name", app_name)])
    }

    /// What removing an app costs, said before it happens.
    #[must_use]
    pub fn uninstall_warning(catalogue: &MessageCatalogue) -> &str {
        catalogue.text("Deletes the app with its cookies, logins and data. This cannot be undone.")
    }

    #[must_use]
    pub fn render(&self, tiles: &[AppTile<'_>]) -> String {
        let body = if tiles.is_empty() {
            format!("<p class=\"empty\">{}</p>", Self::escaped(self.nothing_installed))
        } else {
            let items: String = tiles.iter().map(|tile| self.tile(tile)).collect();
            format!("<ul class=\"grid\">{items}</ul>")
        };
        format!(
            "<!doctype html><html><head><meta charset=\"utf-8\">\
             <meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; img-src data:; style-src 'unsafe-inline'\">\
             <title>{title}</title><style>{style}</style></head><body><main><h1>{title}</h1>\
             <p class=\"lead\">{description}</p>{body}</main></body></html>",
            title = Self::escaped(self.title),
            style = self.style(),
            description = Self::escaped(self.description),
        )
    }

    fn tile(&self, tile: &AppTile<'_>) -> String {
        let initial = tile.name.chars().next().map(|letter| letter.to_uppercase().to_string()).unwrap_or_default();
        let icon = match tile.icon.filter(|address| Self::is_image_address(address)) {
            Some(address) => format!("<img src=\"{address}\" alt=\"\">"),
            None => format!("<span class=\"initial\">{}</span>", Self::escaped(&initial)),
        };
        format!(
            "<li class=\"tile\"><a class=\"launch\" href=\"{open_address}\">{icon}\
             <span class=\"name\">{name}</span><span class=\"host\">{host}</span></a>\
             <div class=\"actions\"><a href=\"{open_address}\">{open}</a>\
             <a class=\"destructive\" href=\"{uninstall_address}\">{uninstall}</a></div></li>",
            open_address = InstalledAppsLink::open_address(tile.id),
            uninstall_address = InstalledAppsLink::uninstall_address(tile.id),
            name = Self::escaped(tile.name),
            host = Self::escaped(tile.host),
            open = Self::escaped(self.open),
            uninstall = Self::escaped(self.uninstall),
        )
    }

    fn style(&self) -> String {
        let palette = self.palette;
        format!(
            "html{{color-scheme:{scheme}}}\
             body{{margin:0;background:{background};color:{text};font:15px system-ui,sans-serif}}\
             main{{max-width:960px;margin:0 auto;padding:48px 32px}}\
             h1{{margin:0 0 6px;font-size:26px}}\
             .lead{{margin:0 0 32px;color:{dimmed}}}\
             .empty{{color:{dimmed}}}\
             .grid{{list-style:none;margin:0;padding:0;display:grid;grid-template-columns:repeat(auto-fill,minmax(180px,1fr));gap:16px}}\
             .tile{{background:{card};border:1px solid {border};border-radius:16px;padding:20px 12px 12px;display:flex;flex-direction:column;align-items:center}}\
             a{{color:{accent};text-decoration:none}}\
             .launch{{display:flex;flex-direction:column;align-items:center;width:100%;color:{text}}}\
             .launch img,.initial{{width:64px;height:64px;border-radius:14px;object-fit:contain}}\
             .initial{{display:flex;align-items:center;justify-content:center;background:{accent};color:#fff;font-size:30px;font-weight:700}}\
             .name{{margin-top:12px;max-width:100%;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-weight:700}}\
             .host{{max-width:100%;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;color:{dimmed};font-size:13px}}\
             .actions{{display:flex;gap:6px;margin-top:14px;width:100%}}\
             .actions a{{flex:1;text-align:center;padding:7px 0;border-radius:8px;font-weight:600;font-size:14px}}\
             .actions a:hover{{background:{border}}}\
             .actions .destructive{{color:{destructive}}}",
            scheme = if std::ptr::eq(palette, &Palette::DARK) { "dark" } else { "light" },
            background = palette.background,
            card = palette.card,
            text = palette.text,
            dimmed = palette.dimmed_text,
            border = palette.border,
            accent = palette.accent,
            destructive = palette.destructive,
        )
    }

    /// Only a base64 `data:image/…` address goes into an attribute, so an icon can never close it.
    fn is_image_address(address: &str) -> bool {
        address.starts_with("data:image/")
            && address.chars().all(|character| character.is_ascii_alphanumeric() || "+/=:;,-.".contains(character))
    }

    fn escaped(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }
}
