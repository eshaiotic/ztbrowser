# Autoconsent (cookie notices)

`autoconsent.js` is DuckDuckGo's [autoconsent](https://github.com/duckduckgo/autoconsent) library, version
**18.0.0**, licence **MPL-2.0** (`LICENSE` next to it), bundled with ZTBrowser's own entry point
`ztbrowser-entry.ts`. The library recognises the common cookie-consent dialogs (rules for hundreds of consent
managers plus a heuristic for unknown ones) and presses their *reject* button.

`ztbrowser-entry.ts` differs from the library's own standalone entry in three ways, all on purpose:

- it rejects only (`autoAction: 'optOut'`, heuristic mode `reject`): nothing here ever accepts a notice;
- it stays idle until Guard dispatches the `ztbrowser-reject-cookie-notices` event, which Guard does only for a
  site the shield is on for (see `docs/en/architecture/qt-protection.md`, "Cookie notices");
- it keeps the rules as a string until then, so a frame that never rejects anything does not pay for parsing them.

## Rebuilding

The bundle is a vendored build artefact, like `assets/content-scope-scripts/contentScope.js`. To rebuild it:

```sh
npm pack @duckduckgo/autoconsent@18.0.0 && tar -xzf duckduckgo-autoconsent-18.0.0.tgz   # into an empty directory
cp assets/autoconsent/ztbrowser-entry.ts package/
cd package && npx esbuild@0.25.0 ztbrowser-entry.ts --bundle --format=iife --target=es2021 --minify \
    --legal-comments=eof --loader:.json=text --outfile=autoconsent.js
```

Upgrading the library: change the version above, rebuild, run `./run-tests`, and note the new version here.
