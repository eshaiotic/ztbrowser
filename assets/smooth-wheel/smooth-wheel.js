// ZTBrowser smooth wheel: animates each step of a mouse wheel the way Chromium does.
//
// WebKitGTK moves a wheel step at once and in about 90 ms; this script takes the step over, cancels it and
// scrolls the same distance in about 180 ms, easing in and out. The distance, the duration and the two curves
// come from `ztbrowser_core::scrolling::smooth_wheel_scroll::SmoothWheelScroll`, which tests them.
//
// Only a whole notch of a mouse wheel is animated. A touchpad (fractional deltas) and everything else is left
// to WebKit, which already scrolls it smoothly: modifier keys (zoom, horizontal scrolling), a page that handled
// the wheel itself, a person who asked their system for reduced motion, and containers that cannot scroll that
// way. Runs at document start in every frame, in a script world of its own.
(() => {
  "use strict";
  const CONFIGURATION = /*SMOOTH_WHEEL_CONFIGURATION*/{};
  const { pixelsPerNotch, durationMs, startCurve, continueCurve } = CONFIGURATION;
  if (!Array.isArray(startCurve) || !Array.isArray(continueCurve) || !(durationMs > 0)) return;

  // `wheelDeltaY` of one notch, in the legacy unit every engine still reports.
  const NOTCH_UNITS = 120;
  // The page scrolled itself by more than this (pixels) since our last frame: someone else is scrolling.
  const EXTERNAL_SCROLL_TOLERANCE = 1.5;
  const REACHES_END_TOLERANCE = 1;
  const reducedMotion = matchMedia("(prefers-reduced-motion: reduce)");
  const animations = new WeakMap();

  function isWholeNotch(event) {
    return (
      event.isTrusted &&
      event.cancelable &&
      !event.defaultPrevented &&
      event.deltaMode === 0 &&
      event.deltaX === 0 &&
      event.wheelDeltaX === 0 &&
      event.deltaY !== 0 &&
      event.wheelDeltaY % NOTCH_UNITS === 0 &&
      !event.ctrlKey &&
      !event.shiftKey &&
      !event.altKey &&
      !event.metaKey
    );
  }

  function hasRoomToScroll(element, direction) {
    return direction > 0
      ? element.scrollTop + element.clientHeight < element.scrollHeight - REACHES_END_TOLERANCE
      : element.scrollTop > 0;
  }

  // The page itself scrolls unless the root (or the body, when the root leaves it) hides its overflow.
  function viewportScrolls() {
    const root = getComputedStyle(document.documentElement);
    const overflow = root.overflowY === "visible" && document.body ? getComputedStyle(document.body).overflowY : root.overflowY;
    return overflow !== "hidden" && overflow !== "clip";
  }

  function scrollsByWheel(element, direction) {
    const style = getComputedStyle(element);
    return (
      /^(auto|scroll|overlay)$/.test(style.overflowY) &&
      element.scrollHeight > element.clientHeight &&
      hasRoomToScroll(element, direction)
    );
  }

  // The element WebKit would scroll for this step: the nearest ancestor with room, else the page. Nothing when
  // the step would not scroll anything, or when the scroller snaps (the snap would fight each frame).
  function scrollerFor(target, direction) {
    const root = document.scrollingElement;
    for (let node = target instanceof Element ? target : target?.parentElement; node; node = node.parentElement) {
      if (node === document.body || node === document.documentElement) break;
      if (scrollsByWheel(node, direction)) return getComputedStyle(node).scrollSnapType === "none" ? node : null;
    }
    if (!root || !viewportScrolls() || !hasRoomToScroll(root, direction)) return null;
    return getComputedStyle(document.documentElement).scrollSnapType === "none" ? root : null;
  }

  function progressAt(curve, time) {
    const position = time * (curve.length - 1);
    const below = Math.floor(position);
    const above = Math.min(below + 1, curve.length - 1);
    return curve[below] + (curve[above] - curve[below]) * (position - below);
  }

  function maximumOffset(element) {
    return Math.max(0, element.scrollHeight - element.clientHeight);
  }

  function frame(animation, now) {
    if (!animation.running) return;
    const element = animation.element;
    if (Math.abs(element.scrollTop - animation.expected) > EXTERNAL_SCROLL_TOLERANCE) {
      animation.running = false;
      return;
    }
    const target = Math.min(animation.target, maximumOffset(element));
    const time = Math.min(1, Math.max(0, (now - animation.started) / durationMs));
    const offset = time >= 1 ? target : animation.from + (target - animation.from) * progressAt(animation.curve, time);
    element.scrollTo({ top: offset, behavior: "instant" });
    animation.expected = element.scrollTop;
    if (time < 1) requestAnimationFrame((next) => frame(animation, next));
    else animation.running = false;
  }

  function animate(element, delta) {
    const previous = animations.get(element);
    const isMoving = previous?.running === true;
    const animation = {
      element,
      from: element.scrollTop,
      target: Math.min(maximumOffset(element), Math.max(0, (isMoving ? previous.target : element.scrollTop) + delta)),
      curve: isMoving ? continueCurve : startCurve,
      started: performance.now(),
      expected: element.scrollTop,
      running: true,
    };
    if (isMoving) previous.running = false;
    animations.set(element, animation);
    requestAnimationFrame((now) => frame(animation, now));
  }

  addEventListener(
    "wheel",
    (event) => {
      if (reducedMotion.matches || !isWholeNotch(event)) return;
      const direction = Math.sign(event.deltaY);
      const element = scrollerFor(event.target, direction);
      if (!element) return;
      event.preventDefault();
      animate(element, direction * (Math.abs(event.wheelDeltaY) / NOTCH_UNITS) * pixelsPerNotch);
    },
    { passive: false },
  );
})();
