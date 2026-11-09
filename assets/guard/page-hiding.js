// ZTBrowser Guard: hides the elements the community filter lists name (cookie notices, ads, banners).
//
// Runs at document start in every frame, in an isolated script world: the page can neither see this script nor
// reach the web channel it asks. The browser answers with the selectors the lists name for the page's address
// and, for the classes and ids the page uses, the generic selectors they call for. Each selector is checked
// and inserted as a rule of its own, so one the engine does not parse hides nothing instead of breaking others.
// `QWebChannel` (Qt's own client) is placed in front of this script by the browser.
//
// It also decides, for the cookie-notice script that runs in the page's own world, whether the shield is on for the
// site: the answer reaches that script as an event on the document, the one thing both worlds share.
(() => {
  "use strict";
  if (!(document instanceof HTMLDocument) || typeof QWebChannel === "undefined" || !qt?.webChannelTransport) return;
  const FLUSH_DELAY_MS = 50;
  const REJECT_COOKIE_NOTICES_EVENT = "ztbrowser-reject-cookie-notices";
  const sheet = new CSSStyleSheet();
  const hidden = new Set();
  const reportedNames = new Set();
  let guard = null;
  let exceptions = [];
  let asksGeneric = false;
  let pendingClasses = [];
  let pendingIds = [];
  let timer = 0;

  function hide(selector) {
    if (hidden.has(selector) || typeof selector !== "string" || !CSS.supports(`selector(${selector})`)) return;
    hidden.add(selector);
    try {
      sheet.insertRule(`${selector}{display:none!important}`, sheet.cssRules.length);
    } catch {
      // A selector this browser does not parse hides nothing.
    }
  }

  // The page may replace its adopted style sheets; Guard's goes back in after every change it makes.
  function adopt() {
    if (!document.adoptedStyleSheets.includes(sheet)) {
      document.adoptedStyleSheets = [...document.adoptedStyleSheets, sheet];
    }
  }

  function note(element) {
    if (element.id && !reportedNames.has(`#${element.id}`)) {
      reportedNames.add(`#${element.id}`);
      pendingIds.push(element.id);
    }
    for (const name of element.classList) {
      if (!reportedNames.has(`.${name}`)) {
        reportedNames.add(`.${name}`);
        pendingClasses.push(name);
      }
    }
  }

  function survey(element) {
    note(element);
    for (const descendant of element.querySelectorAll("[id],[class]")) note(descendant);
    schedule();
  }

  function schedule() {
    if (!timer && guard && asksGeneric) timer = setTimeout(flush, FLUSH_DELAY_MS);
  }

  function flush() {
    timer = 0;
    if (pendingClasses.length === 0 && pendingIds.length === 0) return;
    const found = JSON.stringify({ classes: pendingClasses, ids: pendingIds, exceptions });
    pendingClasses = [];
    pendingIds = [];
    guard.generic_selectors(location.href, found, (answer) => {
      JSON.parse(answer).forEach(hide);
      if (hidden.size > 0) adopt();
    });
  }

  new MutationObserver((records) => {
    for (const record of records) {
      if (record.type === "attributes") note(record.target);
      else for (const node of record.addedNodes) if (node.nodeType === Node.ELEMENT_NODE) survey(node);
    }
    schedule();
  }).observe(document, { childList: true, subtree: true, attributes: true, attributeFilter: ["class", "id"] });

  // The site that decides is the top page's, also in a frame: a consent manager's frame belongs to another site.
  function topAddress() {
    const origins = location.ancestorOrigins;
    return origins && origins.length > 0 ? origins[origins.length - 1] : location.href;
  }

  new QWebChannel(qt.webChannelTransport, (channel) => {
    guard = channel.objects.guard;
    guard.rejects_cookie_notices(topAddress(), (rejects) => {
      if (rejects) document.dispatchEvent(new Event(REJECT_COOKIE_NOTICES_EVENT));
    });
    guard.page_hiding(location.href, (answer) => {
      const page = JSON.parse(answer);
      if (!page) return;
      page.selectors.forEach(hide);
      if (hidden.size > 0) adopt();
      exceptions = page.exceptions;
      asksGeneric = page.generic;
      if (document.documentElement) survey(document.documentElement);
    });
  });
})();
