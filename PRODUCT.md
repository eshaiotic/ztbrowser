# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Stack

The browser is Rust with GTK 4, libadwaita and WebKitGTK (fixed by the codebase). Its own web surfaces are
static, self-contained HTML/CSS with no external assets:

- error pages, from the template in `assets/error-page/`;
- reports, such as `docs/*/reports/*.html`. The user asked for HTML pages they can share.

## Users

- **People on Linux desktops who want less tracking than Brave gives them, and no extensions to manage.** The
  project's author is the first such user.
- **Non-technical decision makers who evaluate the browser,** for example the author's Product Owner.
  - The author shares an HTML page with them, and they read it on their own.
  - What they compare is how well ZTBrowser blocks tracking scripts and ads, against Chrome and Brave
    (confirmed 2026-11-02).

## Product Purpose

ZTBrowser is a privacy-first browser for Linux. It is built on DuckDuckGo's open-source protections:

- the Tracker Data Set;
- the privacy configuration;
- Content Scope Scripts.

ZTBrowser Guard adds the rest on top:

- community filter lists;
- fingerprinting noise;
- the Global Privacy Control header.

Success means blocking more tracking than Brave, without telemetry and without breaking everyday sites such as
banking or Gmail.

## Positioning

- **No telemetry at all.** There is no usage ping, no product analytics and no experiment seed.
- **DuckDuckGo's protections plus Guard's lists,** and the lists also apply to a site's own requests.
- **Fingerprint noise that changes per site and per session.**
- **Isolation for profiles and installed web apps:**
  - every profile is isolated;
  - every installed web app (PWA) runs in its own process with its own data.

## Operating Context

- A Linux desktop: deb, rpm, Arch and AppImage packages.
- Users compare it with Chrome and Brave, which they already know.
- Claims about blocking come from measurements. Measurements are taken with fresh profiles, at the same time
  for every browser, and judged by a neutral tracker list (Disconnect's).

## Capabilities and Constraints

- Blocking is done by WebKit content blockers.
- WebKitGTK has no Safe Browsing (no phishing or malware warnings).
- It cannot run uBlock Origin scriptlets and cannot hide installed fonts.
- WebKitGTK is a rare engine, which makes its users stand out.
- The Tracker Data Set and the filter lists are downloaded at run time and never bundled, because of their
  licences.

## Brand Commitments

- **Name:** ZTBrowser.
- **Colours:** purple (the logo's shield face `#4D0C96`, its rim `#864CE3`, a white chain); the reports add an amber accent (`#FFB300`) that marks ZTBrowser.
- **Affiliation:** ZTBrowser is not affiliated with DuckDuckGo, and says so wherever DuckDuckGo is named.
- **Voice:** plain and factual. It states weaknesses openly.

## Evidence on Hand

- `docs/en/reports/tracking-comparison.md` (Polish mirror in `docs/pl/`), measured 2026-11-02:
  - trackers on 12 sites: Chrome 94, Brave 5, ZTBrowser 2;
  - ad servers: 75, 4 and 1;
  - what each browser sends home while idle;
  - fingerprint behaviour.
- `docs/en/architecture/overview.md` §5.6: the earlier Guard measurement against Brave.
- There are no testimonials, customers, user counts or market data. Do not invent them.

## Product Principles

1. Measure before claiming. Every number links to how it was measured.
2. Say plainly where ZTBrowser is weaker.
3. The browser itself sends nothing about the user.
4. Protection must not break the sites people depend on, such as banks and mail.
