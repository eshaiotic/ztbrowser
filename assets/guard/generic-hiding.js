// ZTBrowser Guard: generic element hiding, applied lazily.
//
// Runs at document start in every frame, in a script world of its own: the page can neither see this script
// nor call its message handler. Generic selectors (`##.ad-banner`, naming no site) stay out of WebKit's rule
// lists, because WebKit matches their style sheets against every element at every style recalculation. This
// script reports the classes and ids the page uses and hides only the selectors needing them. The selectors no
// class or id narrows down it tests on the elements the page adds, keeping only the ones that match.
(() => {
  "use strict";
  const LISTS = /*GUARD_LISTS*/[];
  const PROBE_ELEMENT = "/*GUARD_PROBE_ELEMENT*/";
  const HANDLER = "/*GUARD_HANDLER*/";
  const FLUSH_DELAY_MS = 50;

  const root = document.documentElement;
  const handler = window.webkit?.messageHandlers?.[HANDLER];
  if (!(document instanceof HTMLDocument) || !root || !handler) return;

  // WebKit hides a list's probe exactly where it applies the list's element hiding, exceptions and the user's
  // shield included. No script of the page has run yet, so the page never sees the probe.
  const probe = document.createElement(PROBE_ELEMENT);
  root.append(probe);
  const lists = LISTS.filter((list) => {
    probe.className = list;
    return getComputedStyle(probe).display === "none";
  });
  probe.remove();
  if (lists.length === 0) return;

  const sheet = new CSSStyleSheet();
  const hidden = new Set();
  const reported = new Set();
  let pendingTokens = [];
  let pendingElements = new Set();
  // The id and classes each element was tested with: pages toggle the same few states over and over (scroll
  // animations do it every frame), and a state that was tested once hides nothing new the next time.
  const testedStates = new WeakMap();
  let unkeyed = null;
  let flushing = false;
  let timer = 0;

  function hide(selector) {
    if (hidden.has(selector)) return;
    hidden.add(selector);
    try {
      sheet.insertRule(`${selector}{display:none!important}`, sheet.cssRules.length);
    } catch {
      // A selector WebKit does not parse hides nothing.
    }
  }

  // The page may replace its adopted style sheets; Guard's goes back in after every change it makes.
  function adopt() {
    if (!document.adoptedStyleSheets.includes(sheet)) {
      document.adoptedStyleSheets = [...document.adoptedStyleSheets, sheet];
    }
  }

  function note(element) {
    const tokens = element.id ? [`#${element.id}`] : [];
    for (const name of element.classList) tokens.push(`.${name}`);
    for (const token of tokens) {
      if (!reported.has(token)) {
        reported.add(token);
        pendingTokens.push(token);
      }
    }
  }

  function isNewState(element) {
    const state = `${element.getAttribute("id")}|${element.getAttribute("class")}`;
    const seen = testedStates.get(element);
    if (!seen) {
      testedStates.set(element, new Set([state]));
      return true;
    }
    if (seen.has(state)) return false;
    seen.add(state);
    return true;
  }

  function survey(element) {
    note(element);
    for (const descendant of element.querySelectorAll("[id],[class]")) note(descendant);
    pendingElements.add(element);
    schedule();
  }

  function schedule() {
    if (!timer) timer = setTimeout(flush, FLUSH_DELAY_MS);
  }

  // Selectors without a class or id to ask for are tested on the elements added since the last flush, once
  // each: the parser adds a page element by element, so that is one pass over the page in all.
  function testUnkeyed(elements) {
    const candidates = unkeyed.filter((selector) => !hidden.has(selector));
    if (candidates.length === 0) return;
    const any = candidates.join(",");
    for (const element of elements) {
      if (!element.isConnected || !(element.matches(any) || element.querySelector(any))) continue;
      for (const selector of candidates) {
        if (!hidden.has(selector) && (element.matches(selector) || element.querySelector(selector))) hide(selector);
      }
    }
  }

  async function flush() {
    timer = 0;
    if (flushing) return schedule();
    flushing = true;
    const tokens = pendingTokens;
    const elements = pendingElements;
    pendingTokens = [];
    pendingElements = new Set();
    try {
      if (tokens.length > 0 || unkeyed === null) {
        const asksUnkeyed = unkeyed === null;
        const answer = JSON.parse(await handler.postMessage({ host: location.hostname, lists, tokens, unkeyed: asksUnkeyed }));
        if (asksUnkeyed) unkeyed = answer.unkeyed.filter((selector) => CSS.supports(`selector(${selector})`));
        answer.selectors.forEach(hide);
      }
      testUnkeyed(elements);
      if (hidden.size > 0) adopt();
    } catch {
      // The browser side went away (the page is closing): nothing left to hide.
    } finally {
      flushing = false;
    }
  }

  new MutationObserver((records) => {
    for (const record of records) {
      if (record.type === "attributes") {
        note(record.target);
        if (isNewState(record.target)) pendingElements.add(record.target);
      } else {
        for (const node of record.addedNodes) if (node.nodeType === Node.ELEMENT_NODE) survey(node);
      }
    }
    schedule();
  }).observe(root, { childList: true, subtree: true, attributes: true, attributeFilter: ["class", "id"] });
  survey(root);
})();
