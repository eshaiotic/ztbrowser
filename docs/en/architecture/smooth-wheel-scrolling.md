# Smooth mouse-wheel scrolling

ZTBrowser animates every step of the mouse wheel the way Chromium does: 120 px in about 180 ms, easing in and out,
instead of WebKitGTK's abrupt 93 px in about 90 ms.

## 1. The problem

WebKitGTK scrolls a wheel step at once, in six or seven frames, and the first frames are the fastest. Next to
Brave this reads as a jerk: the page "jumps" on every notch even though the frame rate is fine. WebKit offers no
setting for the length or shape of this animation, and replacing the engine (QtWebEngine, CEF) was measured and
rejected: QtWebEngine does not give Chromium's animation, and a Chromium engine would mean rewriting the interface
and the whole protection layer. So ZTBrowser plays the animation itself.

## 2. Vocabulary

| Term | Meaning |
|---|---|
| **Notch** | One click of a mouse wheel. WebKitGTK reports it as `deltaY` 93 and `wheelDeltaY` ±120. |
| **Easing** | How fast an animation runs at each moment: slow, fast, slow (*ease in-out*) or fast, then slowing (*ease out*). |
| **Progress table** | 65 numbers from 0 to 1: how far the animation has got after 0, 1/64, … 64/64 of its duration. |
| **Script world** | An isolated JavaScript environment in a page: it sees the page's DOM but the page cannot see its variables. |

## 3. How it fits together

![A wheel step goes from the wheel event through the page script to a frame-by-frame scroll](../../assets/diagrams/architecture-smooth-wheel.en.svg)

- **`ztbrowser-core`** holds the numbers: `SmoothWheelScroll` (120 px per notch, 180 ms, two progress tables built
  from `CubicBezierEasing`) and `ScrollingPreferences` (`smoothWheel`, on by default). Their tests
  (`tests/smooth_wheel_scroll.rs`, `tests/cubic_bezier_easing.rs`, `tests/scrolling_preferences.rs`) pin Chromium's
  measured profile: the first frame moves at most 4 px, no frame more than 22 px, the peak lies around the middle,
  the sum is exactly one notch, and a step that arrives while the page moves never slows down to start over.
- **`SmoothWheelScript`** (app) fills those numbers into `assets/smooth-wheel/smooth-wheel.js` and wraps it as a
  user script in a script world of its own, in every frame.
- **`PageProtections`** installs it with the other page scripts when the protection data is ready, so a page
  never starts without it. **`ProfileSession`** keeps the choice in `scrolling.json` of the profile (the Tor window
  keeps it in memory only), and *Preferences → Appearance → Smooth mouse wheel scrolling* changes it; pages
  loaded afterwards follow it.

## 4. What the page script does

It listens to `wheel` and takes over only a whole notch of a mouse wheel: `deltaMode` 0, no horizontal part,
`wheelDeltaY` a multiple of 120 (a touchpad sends fractions, so it is never touched), no Ctrl, Shift or Alt, the
event cancelable and not already cancelled by the page, and the system not asking for reduced motion. It then
looks for the element WebKit would scroll: the nearest ancestor that can scroll in that direction and has room
left (otherwise the event would chain to its parent), else the page, unless its overflow is hidden or it snaps
(`scroll-snap-type`). If there is none, the event goes to WebKit unchanged.

Otherwise it cancels the event and moves the element in `requestAnimationFrame` frames from where it is to the
target with `scrollTo({behavior: "instant"})` (the page's own `scroll-behavior: smooth` must not smooth it a second
time). A notch that arrives while the element moves adds 120 px to the unfinished target and continues with the
*ease out* table, so a quick series of notches flows on. When anyone else scrolls the element (keyboard, scrollbar,
the page's script), the animation stops.

## 5. Measured

One notch in the same test (X11 wheel events through XTest, pixels the page moved in each 60 Hz frame):

| Engine | Pixels | Frames | Time | Per frame |
|---|---|---|---|---|
| WebKitGTK (before) | 86–93 | 6–7 | ~90 ms | 5,12,34,21,10,3,1 |
| Brave / Chromium | 120 | 11–12 | 167–183 ms | 2,5,10,13,16,18,17,15,12,7,4,1 |
| ZTBrowser (smooth wheel) | 120 | 11–12 | 161–183 ms | 1,6,9,14,17,18,18,15,12,7,2,1 |

Also verified: a page with `scroll-behavior: smooth`, a scrollable box inside a page (the box scrolls first), and
15 notches 40 ms apart (continuous, no stop between notches).

## 6. Limits

The animation runs on the page's main thread, so on a page whose main thread is busy it is as steady as the
page, where WebKit's own scrolling would stay steady; turn it off in the preferences for such a page.
A touchpad is not covered: it already sends a continuous stream, and its gestures were not testable in the
development environment. The script has no automated test of its own: the numbers are tested in the core crate,
the behaviour by the measurements above.
