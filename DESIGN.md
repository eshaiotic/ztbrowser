---
name: ZTBrowser
description: Privacy-first Linux browser whose shared reports are measured scoreboards on a deep indigo field, and whose error pages stay native to GNOME.
colors:
  ink: "#1e1060"
  indigo: "#2e1a87"
  field-floor: "#120a3d"
  field-track: "rgb(255 255 255 / 9%)"
  field-lead: "#dcd7ff"
  field-muted: "#bdb5f2"
  lilac: "#8e7fe8"
  amber: "#ffb300"
  chrome: "#b9b5c9"
  brave: "#ff7a45"
  paper: "#ffffff"
  mist: "#f4f2fc"
  text: "#1d1a33"
  muted: "#575270"
  line: "#e1ddf2"
  good: "#17703a"
  bad: "#b3261e"
  weakness-wash: "#fff7e0"
  weakness-ink: "#5c3d00"
  weakness-marker: "#b97800"
  operate-accent-default: "#3584e4"
  operate-bg: "#fafafb"
  operate-fg: "rgb(0 0 6 / 80%)"
  operate-muted: "rgb(0 0 6 / 60%)"
  operate-ghost: "rgb(0 0 6 / 7%)"
  operate-ghost-strong: "rgb(0 0 6 / 26%)"
  operate-button: "rgb(0 0 6 / 8%)"
  operate-button-hover: "rgb(0 0 6 / 12%)"
  operate-button-active: "rgb(0 0 6 / 22%)"
  operate-danger: "#e01b24"
  operate-danger-text: "#c01c28"
  operate-warning: "#e5a50a"
  operate-bg-dark: "#222226"
  operate-fg-dark: "#ffffff"
  operate-muted-dark: "rgb(255 255 255 / 62%)"
  operate-ghost-dark: "rgb(255 255 255 / 7%)"
  operate-ghost-strong-dark: "rgb(255 255 255 / 28%)"
  operate-button-dark: "rgb(255 255 255 / 10%)"
  operate-button-hover-dark: "rgb(255 255 255 / 15%)"
  operate-button-active-dark: "rgb(255 255 255 / 26%)"
  operate-danger-dark: "#c01c28"
  operate-danger-text-dark: "#ff7b63"
  operate-warning-dark: "#cd9309"
typography:
  display:
    fontFamily: "ZTBrowser Sans, Adwaita Sans, system-ui, sans-serif"
    fontSize: "clamp(32px, 5.2vw, 56px)"
    fontWeight: 700
    lineHeight: 1.06
    letterSpacing: "-0.02em"
  figure:
    fontFamily: "ZTBrowser Sans, Adwaita Sans, system-ui, sans-serif"
    fontSize: "44px"
    fontWeight: 800
    lineHeight: 1
    fontFeature: "tnum"
  figure-compact:
    fontFamily: "ZTBrowser Sans, Adwaita Sans, system-ui, sans-serif"
    fontSize: "19px"
    fontWeight: 700
    lineHeight: 1.6
    fontFeature: "tnum"
  headline:
    fontFamily: "ZTBrowser Sans, Adwaita Sans, system-ui, sans-serif"
    fontSize: "clamp(24px, 3vw, 30px)"
    fontWeight: 700
    lineHeight: 1.2
    letterSpacing: "-0.01em"
  title:
    fontFamily: "ZTBrowser Sans, Adwaita Sans, system-ui, sans-serif"
    fontSize: "18px"
    fontWeight: 700
    lineHeight: 1.6
  lead:
    fontFamily: "ZTBrowser Sans, Adwaita Sans, system-ui, sans-serif"
    fontSize: "19px"
    fontWeight: 400
    lineHeight: 1.6
  body:
    fontFamily: "ZTBrowser Sans, Adwaita Sans, system-ui, sans-serif"
    fontSize: "17px"
    fontWeight: 400
    lineHeight: 1.6
  label:
    fontFamily: "ZTBrowser Sans, Adwaita Sans, system-ui, sans-serif"
    fontSize: "14px"
    fontWeight: 400
    lineHeight: 1.6
  operate-title:
    fontFamily: "system-ui, Cantarell, Noto Sans, sans-serif"
    fontSize: "1.875rem"
    fontWeight: 750
    lineHeight: 1.2
    letterSpacing: "-0.015em"
  operate-body:
    fontFamily: "system-ui, Cantarell, Noto Sans, sans-serif"
    fontSize: "15px"
    fontWeight: 400
    lineHeight: 1.55
  operate-detail:
    fontFamily: "ui-monospace, Source Code Pro, Noto Sans Mono, monospace"
    fontSize: "12.5px"
    fontWeight: 400
    lineHeight: 1.5
rounded:
  focus: "3px"
  bar-compact: "6px"
  bar: "10px"
  pill: "9999px"
  dot: "50%"
spacing:
  operate-gap: "12px"
  lane: "18px"
  gutter-narrow: "20px"
  gutter: "28px"
  operate-gutter: "40px"
  column: "48px"
  section: "56px"
  hero-top: "64px"
components:
  report-container:
    width: "1040px"
    padding: "0 28px"
  hero-field:
    backgroundColor: "{colors.ink}"
    textColor: "{colors.paper}"
    padding: "64px 0 56px"
  race-track:
    backgroundColor: "{colors.field-track}"
    rounded: "{rounded.bar}"
    height: "46px"
  race-fill-ztbrowser:
    backgroundColor: "{colors.amber}"
    rounded: "{rounded.bar}"
    height: "46px"
  race-fill-brave:
    backgroundColor: "{colors.brave}"
    rounded: "{rounded.bar}"
    height: "46px"
  race-fill-chrome:
    backgroundColor: "{colors.chrome}"
    rounded: "{rounded.bar}"
    height: "46px"
  race-count-ztbrowser:
    textColor: "{colors.amber}"
    typography: "{typography.figure}"
  race-count:
    textColor: "{colors.paper}"
    typography: "{typography.figure}"
  mini-track:
    backgroundColor: "{colors.mist}"
    rounded: "{rounded.bar-compact}"
    height: "24px"
  mini-figure-ztbrowser:
    textColor: "{colors.indigo}"
    typography: "{typography.figure-compact}"
  comparison-cell:
    textColor: "{colors.text}"
    padding: "14px 12px"
  comparison-cell-ztbrowser:
    backgroundColor: "{colors.mist}"
    textColor: "{colors.ink}"
    padding: "14px 12px"
  weakness-band:
    backgroundColor: "{colors.weakness-wash}"
    textColor: "{colors.text}"
    padding: "44px 0 40px"
  weakness-heading:
    textColor: "{colors.weakness-ink}"
    typography: "{typography.headline}"
  link:
    textColor: "{colors.indigo}"
  footer:
    textColor: "{colors.muted}"
    typography: "{typography.label}"
    padding: "36px 0 48px"
  operate-page:
    backgroundColor: "{colors.operate-bg}"
    textColor: "{colors.operate-fg}"
    typography: "{typography.operate-body}"
    width: "640px"
  operate-page-dark:
    backgroundColor: "{colors.operate-bg-dark}"
    textColor: "{colors.operate-fg-dark}"
    typography: "{typography.operate-body}"
    width: "640px"
  operate-button:
    backgroundColor: "{colors.operate-button}"
    textColor: "{colors.operate-fg}"
    rounded: "{rounded.pill}"
    padding: "9px 26px"
    height: "40px"
  operate-button-hover:
    backgroundColor: "{colors.operate-button-hover}"
  operate-button-active:
    backgroundColor: "{colors.operate-button-active}"
  operate-button-primary:
    backgroundColor: "{colors.operate-accent-default}"
    textColor: "{colors.paper}"
    rounded: "{rounded.pill}"
    padding: "9px 26px"
    height: "40px"
---

# Design System: ZTBrowser

## Overview

**Creative North Star: "The Honest Scoreboard"**

ZTBrowser has two kinds of web surface, and they deliberately do not wear the same clothes. **Persuasion surfaces** are the HTML reports the author shares with people who evaluate the browser (today `docs/pl/reports/tracking-summary.html` and its English mirror `docs/en/reports/tracking-summary.html`). They behave like a scoreboard: a deep indigo field taken from the logo, measured counts at hero scale, every browser in its own recognisable colour, and the logo's amber reserved for ZTBrowser alone. Nothing decorates the data; the bars and their printed figures are the composition, and the page states where ZTBrowser is weaker in a band of its own instead of in fine print.

**Operate surfaces** are pages the browser itself shows while someone is working, today the error pages rendered from `assets/error-page/error-page.html`. They step back into GNOME's libadwaita world: near-neutral greys written as alpha over near-black, a light and a dark scheme, the desktop's own accent colour delivered at runtime, pill buttons and the system face. A failed page load is not a moment to advertise, so these pages carry no brand colour at all.

What ties the two together is the shield. Both surfaces draw the same shield geometry with identical path data from `assets/icons/io.github.ztbrowser.ZTBrowser.svg`: the report shows the full logo (a purple shield with a white chain) beside ZTBrowser's bar, and the error page shows it as a ghost-grey silhouette with the chain cut out, whose round badge takes the state colour (accent, danger or warning). Brand colour belongs to the claim; the operate surface belongs to the user's desktop. Density is calm on both: a 17px or 15px body, measures between 62ch and 75ch, generous vertical rhythm, and a plain, factual voice.

**Provenance.** No concept roll (concept-seed) was run for the report surface because the user imposed a time limit on 2026-11-02; its form was chosen directly (a data-led report with comparison bars at hero scale). The North Star name is the documenter's reading of the report's direction contract, recorded from the built pages; no separate language interview was held in this pass.

**Key Characteristics:**
- Two surface families: a branded persuasion surface and a native, GNOME-like operate surface.
- The deep indigo field and amber exist only on persuasion surfaces; amber marks ZTBrowser and nothing else.
- Numbers are the heaviest type on a report, always tabular, always printed beside their bar.
- Flat everywhere: depth comes from tonal fields and washes, never from shadows.
- Self-contained pages: fonts embedded or taken from the system, no network assets.
- One shared shield silhouette is the visual signature on both surfaces.

## Colors

A deep indigo field with one warm amber voice for ZTBrowser, competitors in their own colours, cool lilac-tinted neutrals on paper, and a separate libadwaita-neutral palette for the operate surface.

### Primary
- **Deep Indigo Ink** (#1e1060): the dominant colour of the hero field, the colour of every section headline on paper, and the text colour of ZTBrowser's highlighted table column. The logo's shield face is a brighter purple of the same family (#4d0c96).
- **Logo Indigo** (#2e1a87): the origin of the hero field's radial gradient and the colour of links. On paper it is ZTBrowser's text colour in charts and table headers (11.7:1 on mist), standing in for amber where amber would be illegible.
- **Field Floor** (#120a3d): the outermost stop of the hero field gradient, so the field darkens away from the top-left corner where the headline sits.
- **Field Track** (rgb(255 255 255 / 9%)): the empty lane behind each hero bar; a faint white glaze so the track reads as a slot in the field rather than a box on top of it.

### Secondary
- **ZTBrowser Amber** (#ffb300): ZTBrowser's own mark. It fills ZTBrowser's bars, colours ZTBrowser's name and count on the field (9.0:1 on ink), and is the keyboard focus ring for links. The logo itself has no amber; amber is the reports' own signal for ZTBrowser.
- **Logo Lilac** (#8e7fe8): a softer neighbour of the logo's rim (#864ce3); in the report it is the text-selection background.

### Tertiary
- **Brave Ember** (#ff7a45): Brave's bars, close to Brave's own orange so a reader recognises it without a legend.
- **Chrome Neutral Grey** (#b9b5c9): Chrome's bars. Chrome is shown neutral, neither branded nor punished with red, so the gap in length carries the comparison by itself.

### Neutral
- **Paper** (#ffffff): the report's page background below the hero, and the text colour on the field.
- **Lilac Mist** (#f4f2fc): the track behind compact bars and the wash behind ZTBrowser's table column.
- **Indigo Night Text** (#1d1a33): body text on paper; a near-black tinted toward the brand hue.
- **Dusk Muted** (#575270): section intros, table headers, the method paragraph and the footer (7.4:1 on paper).
- **Hairline Lilac** (#e1ddf2): the 1px rules between table rows and above the footer.
- **Field Lead** (#dcd7ff): the lead paragraph on the hero field.
- **Field Muted** (#bdb5f2): secondary text on the field: browser version labels and the measurement source line (8.5:1 on ink).

### Status and the weakness band
- **Fact Green** (#17703a) and **Fact Red** (#b3261e): the 9px dots beside yes/no facts in the comparison table, and the text colour of those cells on narrow screens. They judge a fact ("sends usage data: yes"), never a browser.
- **Weakness Wash** (#fff7e0), **Weakness Ink** (#5c3d00) and **Weakness Marker** (#b97800): the warm band that lists where ZTBrowser is weaker; the wash is the band, the ink is its headline (9.3:1 on the wash), the marker colours the list bullets. It is a tint of ZTBrowser's own amber, because the weaknesses are ZTBrowser's.

### Operate surface (error pages)
- **Desktop Accent** (runtime; #3584e4 is libadwaita's stock blue): written into `--accent` from the style manager's `accent_color_rgba()` each time a page renders. It fills the primary button and the illustration badge for ordinary failures.
- **Operate Background** (#fafafb light, #222226 dark): libadwaita's window colours.
- **Operate Foreground and Muted** (rgb(0 0 6 / 80%) and rgb(0 0 6 / 60%) light; #ffffff and rgb(255 255 255 / 62%) dark): text, as alpha over near-black or white so it sits on either scheme.
- **Ghost and Ghost Strong** (7% and 26% light; 7% and 28% dark): the shield silhouette and its features, and the bullet dots.
- **Button, Hover, Active** (8%, 12%, 22% light; 10%, 15%, 26% dark): flat button fills that deepen with state.
- **Danger** (#e01b24 / #c01c28 dark) with **Danger Text** (#c01c28 / #ff7b63 dark): certificate and security failures; the badge and the bullet dots turn danger and the points switch to full foreground.
- **Warning** (#e5a50a / #cd9309 dark): crashes; the badge glyph turns 80% black for contrast on yellow.

### Named Rules
**The Amber Is Us Rule.** Full-strength amber (#ffb300) marks only ZTBrowser: its bar and its name and count on the field. The single functional exception is the keyboard focus ring. A competitor, a headline, a link or a button never turns amber.

**The Indigo On Paper Rule.** Amber text is legible on the field (9.0:1 on ink) and illegible on paper (1.6:1 on mist). When ZTBrowser's name or figure sits on paper, it speaks in Logo Indigo (#2e1a87), and its table column gets the Lilac Mist wash.

**The Their Own Colours Rule.** Each competitor appears in a colour a reader already associates with it (Brave in ember orange, Chrome in neutral grey). Red and green are reserved for yes/no facts and are never used to colour a browser.

**The Desktop Owns The Operate Surface Rule.** Operate surfaces carry no brand colour. Their accent and their light or dark scheme come from the desktop at render time, so a user with a purple accent gets purple buttons. Indigo and amber never enter an error page.

## Typography

**Display Font:** ZTBrowser Sans, an embedded subset of Adwaita Sans (Latin and Polish; Inter-derived, SIL Open Font License 1.1), variable weight 100 to 900 (with Adwaita Sans, system-ui, sans-serif)
**Body Font:** ZTBrowser Sans on persuasion surfaces; system-ui, Cantarell, Noto Sans on operate surfaces
**Label/Mono Font:** ui-monospace with DejaVu Sans Mono (report) or Source Code Pro and Noto Sans Mono (error pages)

**Character:** One sans family carries the whole hierarchy through size and weight alone. It is GNOME's own face, so the report about a GNOME browser speaks in the browser's voice; the subset is embedded so a shared file renders the same on a reader's Windows or macOS machine, with no network fetch. Operate surfaces do not embed it: they use whatever face the desktop uses, as a native page should.

### Hierarchy
- **Display** (700, clamp(32px, 5.2vw, 56px), 1.06, -0.02em): the hero claim, one sentence, at most 22ch wide with balanced wrapping.
- **Figure** (800, 44px, 1, tabular figures): the count at the end of each hero lane; 34px under 760px.
- **Headline** (700, clamp(24px, 3vw, 30px), 1.2, -0.01em): section headlines in Deep Indigo Ink, balanced wrapping.
- **Title** (700, 18px): chart titles above compact bar groups.
- **Lead** (400, 19px, 1.6): the hero's method sentence in Field Lead, at most 62ch. Browser names in hero lanes use the same size at 700.
- **Body** (400, 17px, 1.6): running text; intros at most 66ch, the weakness list at most 72ch, the method paragraph at 15px and at most 75ch.
- **Figure Compact** (700, 19px, tabular figures): counts beside compact bars.
- **Label** (400, 14px): version labels, the source line, table headers (600) and the footer.
- **Operate Title** (750, 1.875rem, 1.2, -0.015em): the error page headline; 1.5rem under 520px.
- **Operate Body** (400, 15px, 1.55): the error page lead and points, at most 62ch, pretty wrapping; the failing host is set in 700.
- **Operate Detail** (400, 12.5px mono, 1.5): the raw error detail at the bottom, in muted colour, breaking anywhere.

### Named Rules
**The Figures Outrank Words Rule.** On a scoreboard the count is the heaviest type on the page (800 against the headline's 700), and every count uses tabular figures and sits right-aligned so 94, 5 and 2 line up in one column.

**The Embedded Face Rule.** Persuasion surfaces embed their face as a subset; operate surfaces never embed a face and use the desktop's.

## Layout

**Persuasion surface.** A single centred column at most 1040px wide with a 28px gutter (20px under 760px). The hero is a full-bleed indigo field with 64px above and 56px below its content (44px and 40px on narrow screens). Below it, sections follow a 56px top rhythm. Every measured chart is a stack of lanes on an 18px gap; a hero lane is a three-column grid (150px browser name, flexible track, 96px count), and under 760px it reflows to name and count on one row over a full-width 30px track. Bar length is proportional to the count, with the largest value at 100% of the track; the fill keeps a 10px minimum (6px for compact bars) so a single-digit value never disappears. Compact charts sit in a two-column split with a 48px gap that becomes one column with a 32px gap. The comparison table keeps a 640px minimum width inside a horizontal scroller on desktop; under 760px each row becomes a stacked block with a 92px browser-label column, the headers hidden and each cell labelled from its `data-browser` attribute. Browsers always appear in the same order, Chrome, Brave, ZTBrowser, in every chart and table. A measurement source line (date and sites) sits directly under the hero chart, and a method section with a link to the full report closes the page. In print the colours are forced exact and sections avoid breaking across pages.

**Operate surface.** A single left-aligned column at most 640px wide, with top padding of max(12vh, 48px), a 40px gutter and 64px below (40px, 24px and 48px under 520px). The order is fixed: illustration (104px, 88px narrow), headline, lead, points, actions on a 12px wrapping row 36px below the text, then the raw detail 44px further down.

## Elevation & Depth

Both surfaces are flat; no element anywhere casts a shadow. Depth on the report comes from tonal fields: the hero's radial gradient (Logo Indigo at the top-left, through Deep Indigo Ink at 55%, to Field Floor) against white paper, then the Lilac Mist wash and the warm Weakness Wash as quieter layers below. Hero tracks are a 9% white glaze cut into the field. On the operate surface, state is expressed by alpha: buttons deepen from 8% to 12% to 22% of near-black, and the accent button lightens by a 10% white overlay on hover and darkens by a 20% black overlay when pressed, exactly as libadwaita's flat buttons do.

### Named Rules
**The Tone Not Shadow Rule.** Layers are told apart by tone (field, paper, mist, wash), never by drop shadows. The only gradients are the hero field and the logo; bars and tracks are solid.

## Shapes

Gently rounded bars carry the report: hero tracks and fills share a 10px radius and compact ones a 6px radius, so a fill's end matches its track. Yes/no facts are marked with 9px circular dots, and the link focus ring has a 3px radius with a 3px offset. Borders exist only as 1px Hairline Lilac rules between table rows and above the footer. The operate surface uses libadwaita's forms: fully rounded pill buttons at least 40px tall, 6px round bullets, and the shield silhouette with a circular badge (radius 21 inside a 26 ring cut from the background colour). The shield is the only figurative shape on either surface, and it is always drawn from the logo's own path data.

## Components

### Race lanes (signature)
The hero comparison: one lane per browser, the measured count at hero scale.
- **Track:** Field Track glaze, 46px tall (30px narrow), 10px radius.
- **Fill:** solid browser colour, 10px radius, 10px minimum width, width proportional to the count.
- **Name:** 700 at 19px with the browser version below in 14px Field Muted; ZTBrowser's name is preceded by the 26px logo mark and turns amber.
- **Count:** Figure style, right-aligned; Paper for competitors, amber for ZTBrowser.
- **Motion:** fills grow from the left over 1.1s on cubic-bezier(.16, 1, .3, 1), staggered by 0.12s in the order Chrome, Brave, ZTBrowser; no animation under `prefers-reduced-motion` or in print.
- **Accessibility:** the whole chart is one `role="img"` with an `aria-label` stating every count in words.

### Compact comparison rows
- **Row:** 92px name, flexible track, 44px count on a 12px gap.
- **Track:** Lilac Mist, 24px tall, 6px radius; fills in the browser colours with a 6px minimum width.
- **ZTBrowser row:** name and count in Logo Indigo at 700 (the Indigo On Paper Rule).

### Comparison table
- **Cells:** 16px text, 14px by 12px padding, 1px Hairline Lilac rule below each row.
- **Headers:** 14px, 600, Dusk Muted; ZTBrowser's header in Logo Indigo.
- **ZTBrowser column:** Lilac Mist wash, Deep Indigo Ink text at 600.
- **Facts:** a Fact Green or Fact Red dot before each value; on narrow screens the dot gives way to a browser label and the cell text takes the fact colour.

### Weakness band
- **Background:** Weakness Wash, full-bleed, 44px above and 40px below, 56px above the band.
- **Headline:** Headline style in Weakness Ink.
- **List:** body-size items, each opening with a bold one-line statement, bullets in Weakness Marker, at most 72ch.

### Links
- **Style:** Logo Indigo, underlined with a 3px offset.
- **Focus:** a 3px amber outline at 3px offset with a 3px radius.

### Footer
- **Style:** 14px Dusk Muted on paper, 1px Hairline Lilac rule above, 36px above and 48px below the text; it carries the non-affiliation line.

### Operate buttons
- **Shape:** pill (9999px), at least 40px tall, 9px by 26px padding, 700 weight, no underline.
- **Default:** Button alpha fill with Operate Foreground text; hover and active deepen the alpha.
- **Primary:** Desktop Accent fill with white text; hover adds a 10% white overlay, active a 20% black overlay.
- **Focus:** a 2px outline at 2px offset in the accent at 55% opacity.

### Error header
- **Illustration:** the logo's shield in Ghost with its chain cut out (an SVG mask, so the page background shows through in either scheme), and a round badge at the lower right carrying one of eight stroked glyphs (search, offline, alert, clock, lock, file, blocked, crash) in white.
- **Tone:** the badge is the Desktop Accent for ordinary failures, Danger for security failures and Warning for crashes.
- **Text:** Operate Title, then the lead in Operate Body, then muted bullet points.

## Do's and Don'ts

### Do:
- **Do** draw every measured comparison as bars with the count printed beside each bar in tabular figures, the largest value at 100% of the track.
- **Do** order browsers Chrome, Brave, ZTBrowser in every chart and table.
- **Do** place the measurement date and sources directly under a measured chart, and link the full method from the page.
- **Do** give ZTBrowser's weaknesses their own Weakness Wash band at body size.
- **Do** switch ZTBrowser's text to Logo Indigo (#2e1a87) whenever it sits on paper or mist.
- **Do** draw the shield from the logo's own path data on every surface.
- **Do** take the accent and the light or dark scheme from the desktop on operate surfaces.
- **Do** stop the bar growth under `prefers-reduced-motion` and force exact colours in print.

### Don't:
- **Don't** use amber (#ffb300) on a competitor, a headline, a link or a button; it marks ZTBrowser and the focus ring only.
- **Don't** set amber text on paper or mist.
- **Don't** colour a browser red or green; those colours judge yes/no facts.
- **Don't** bring indigo, amber or the embedded face into an error page.
- **Don't** add drop shadows to bars, tracks or bands, or gradients to bars.
- **Don't** restate a measured comparison as metric tiles or feature cards.
- **Don't** load fonts, images or scripts from the network on either surface.
