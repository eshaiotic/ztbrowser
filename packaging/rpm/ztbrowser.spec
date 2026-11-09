# Built by packaging/release.sh inside a Fedora container (packaging/rpm/build.sh). Source0 is a tarball of
# the working tree. Library dependencies (libQt6WebEngineQuick.so.6, ...) are found by rpm in the ELF binary;
# the QML modules, the SVG plugin and the web engine's helper process are loaded at run time, so they are listed.

# The release profile strips symbols, so there is nothing for a debuginfo package.
%global debug_package %{nil}
# Fedora's link-time optimisation flags would leave the C++ part of the Qt bridge unreadable for Rust's linker
# (undefined references to Qt's inline functions).
%global _lto_cflags %{nil}

Name:           ztbrowser
Version:        1.3.0
Release:        1%{?dist}
Summary:        Privacy-first browser on QtWebEngine (Chromium), built on DuckDuckGo's open-source tracker protections
License:        GPL-3.0-or-later
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  rust
BuildRequires:  gcc-c++
BuildRequires:  clang
BuildRequires:  lld
BuildRequires:  cmake(Qt6Core)
BuildRequires:  cmake(Qt6Gui)
BuildRequires:  cmake(Qt6Network)
BuildRequires:  cmake(Qt6Qml)
BuildRequires:  cmake(Qt6Quick)
BuildRequires:  cmake(Qt6WebEngineCore)
BuildRequires:  cmake(Qt6WebEngineQuick)
BuildRequires:  cmake(Qt6WebChannel)
Requires:       hicolor-icon-theme
Requires:       qt6-qtdeclarative
Requires:       qt6-qtsvg
Requires:       qt6-qtwebchannel
Requires:       qt6-qtwebengine
# xdg-settings makes ZTBrowser the default browser when the person asks for it.
Requires:       xdg-utils
# ZTBrowser was called InDuckGo up to 1.2.4: installing ZTBrowser removes the InDuckGo package, and the first
# start moves InDuckGo's data over (crates/ztbrowser-qt/src/induckgo_migration.rs).
Obsoletes:      induckgo <= 1.2.4

%description
ZTBrowser blocks third-party trackers with DuckDuckGo's open-source protections, upgrades connections to
HTTPS and removes tracking parameters from links. Every profile and every site installed as an app runs in
its own process with its own cookies and storage.

%prep
%autosetup
if ! grep -qx 'version = "%{version}"' Cargo.toml; then
    echo "Version %{version} differs from the version in Cargo.toml" >&2
    exit 1
fi

%build
cargo build --release --locked -p ztbrowser-qt

%check
cargo test --locked -p ztbrowser-core

%install
install -Dm0755 target/release/ztbrowser-qt %{buildroot}%{_bindir}/ztbrowser
install -Dm0644 packaging/linux/io.github.ztbrowser.ZTBrowser.desktop -t %{buildroot}%{_datadir}/applications
install -Dm0644 packaging/linux/io.github.ztbrowser.ZTBrowser.metainfo.xml -t %{buildroot}%{_metainfodir}
install -Dm0644 assets/icons/io.github.ztbrowser.ZTBrowser.svg -t %{buildroot}%{_datadir}/icons/hicolor/scalable/apps

%files
%license LICENSE NOTICE
%doc README.md
%{_bindir}/ztbrowser
%{_datadir}/applications/io.github.ztbrowser.ZTBrowser.desktop
%{_metainfodir}/io.github.ztbrowser.ZTBrowser.metainfo.xml
%{_datadir}/icons/hicolor/scalable/apps/io.github.ztbrowser.ZTBrowser.svg

%changelog
* Mon Nov 09 2026 Michał Ligus <eshaiotic@eshaiotic.io> - 1.3.0-1
- InDuckGo is now ZTBrowser, with a new logo. The first start moves InDuckGo's profiles, logins, installed web apps and settings over, and this package replaces the induckgo package.
- ZTBrowser can become the default browser: it offers the change once at start and in Preferences, and never makes it without you.

* Sun Nov 08 2026 Michał Ligus <eshaiotic@eshaiotic.io> - 1.2.4-1
- Fixed: the toolbar icons were invisible on a dark bar under Plasma's Breeze style; they are now drawn in the colour of the bar's text.
- A tab can be moved to a window of its own: right-click it and choose "Move to New Window", or drag it off the tab strip. The page keeps running.

* Thu Nov 05 2026 Michał Ligus <eshaiotic@eshaiotic.io> - 1.2.3-1
- Cookie dialogs are rejected at once (DuckDuckGo's autoconsent, Facebook's included) and never accepted; a site whose shield is off is left alone.
- Fixed: the page scripts were not applied, so DuckDuckGo offered a browser extension the browser does not install and Guard hid no page elements.

* Thu Nov 05 2026 Michał Ligus <eshaiotic@eshaiotic.io> - 1.2.2-1
- The browser now runs on QtWebEngine (Chromium) with a Qt Quick interface; it replaces the WebKitGTK edition.
- Tracker protection: DuckDuckGo's rules evaluated as DuckDuckGo evaluates them, a stricter layer for advertising and analytics trackers, a local gate that refuses connections a page opens before any request, and protection data downloaded on first start and kept current. Measured on 12 popular sites: 5 trackers against 6 for the WebKitGTK edition, 8 for Brave and 109 for Chrome.
- Not yet in this edition: fingerprint and GPC protection, the Tor window, automatic rejection of cookie pop-ups, bookmarks and history.

* Thu Nov 05 2026 Michał Ligus <eshaiotic@eshaiotic.io> - 1.2.1-1
- The WebKitGTK browser is unchanged in behaviour; the shared core gained tab placement, window button style and profile tile preferences for the Qt edition (QtWebEngine), which is not packaged yet.
- Interface texts for the new features were added to all 66 translations.

* Thu Nov 05 2026 Michał Ligus <eshaiotic@eshaiotic.io> - 1.2.0-1
- The WebKitGTK browser is unchanged from 1.1.2 in behaviour; the shared core gained the groundwork of the Qt edition (QtWebEngine), which is not packaged yet.
- Interface texts for the new Qt features were added to all 66 translations.

* Wed Nov 04 2026 Michał Ligus <eshaiotic@eshaiotic.io> - 1.1.2-1
- Smooth mouse-wheel scrolling: every wheel step is animated like in Chromium (switch in Preferences → Appearance).
- Faster scrolling on pages with scroll animations: Guard's page script no longer retests the same elements.

* Wed Nov 04 2026 Michał Ligus <eshaiotic@eshaiotic.io> - 1.1.1-1
- Interface language: choose among 68 languages in the preferences.
- Downloads: toolbar list, save-folder choice, clear list, and a manager with the whole history.
- Faster pages: drawing with GTK's OpenGL renderer instead of copying every frame through memory.

* Tue Nov 03 2026 Michał Ligus <eshaiotic@eshaiotic.io> - 1.1.0-1
- Tor window: browsing through ZTBrowser's own Tor, with bridges and New Identity.
- Toolbar: an onion button for the Tor window, and preferences to choose the toolbar's buttons.

* Tue Nov 03 2026 Michał Ligus <eshaiotic@eshaiotic.io> - 1.0.1-1
- New application icon.

* Mon Nov 02 2026 Michał Ligus <eshaiotic@eshaiotic.io> - 1.0.0-1
- First release.
