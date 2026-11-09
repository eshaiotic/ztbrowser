// ZTBrowser: DuckDuckGo's own pages offer a browser extension to every Chromium browser. ZTBrowser has the
// protections built in and runs no extensions, so on duckduckgo.com the offer is hidden instead of the page
// being told something untrue about this browser.
(function () {
  var host = location.hostname;
  if (host !== 'duckduckgo.com' && !host.endsWith('.duckduckgo.com')) return;
  var sheet = new CSSStyleSheet();
  // Only the extension's own card and links: the neighbouring "set as default search" card stays.
  sheet.replaceSync(
    'li:has([data-ssg-id="home-extension"]), li:has([data-testid="cta-extension-button"]),' +
    ' li:has(> a[data-testid="sidemenu-link-extension"]), [data-ssg-id="home-extension"],' +
    ' [data-testid="cta-extension-button"], a[data-testid="sidemenu-link-extension"],' +
    ' a[href*="chromewebstore.google.com"], a[href*="chrome.google.com/webstore"],' +
    ' a[href*="addons.mozilla.org"], a[href*="microsoftedge.microsoft.com/addons"]' +
    ' { display: none !important; }'
  );
  document.adoptedStyleSheets = document.adoptedStyleSheets.concat([sheet]);
})();
