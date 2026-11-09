// ZTBrowser's entry to DuckDuckGo's autoconsent: rejects cookie notices, and only when Guard says the site is protected.
//
// Runs in the page's own world (the engine evaluates the library's eval rules there), so it keeps every name in
// this closure and puts nothing on `window`. It stays idle until Guard, which runs in an isolated world and knows
// whether the shield is on for the site, dispatches REJECT_EVENT on the document.
import AutoConsent from './lib/web';
import { filterCompactRules, type IndexedCMPRuleset } from './lib/encoding';
// The rules stay a string until a notice is to be rejected: parsing them in every frame would cost every page.
import compactRulesText from './rules/compact-rules.json';

const REJECT_EVENT = 'ztbrowser-reject-cookie-notices';

document.addEventListener(
    REJECT_EVENT,
    () => {
        const consent = new AutoConsent(() => {});
        consent.initialize(
            {
                enabled: true,
                autoAction: 'optOut',
                isMainWorld: true,
                enablePrehide: true,
                enableCosmeticRules: true,
                enableHeuristicDetection: true,
                heuristicMode: 'reject',
            },
            {
                autoconsent: [],
                compact: filterCompactRules(JSON.parse(compactRulesText) as IndexedCMPRuleset, {
                    url: window.location.href,
                    mainFrame: window.top === window,
                }),
            },
        );
    },
    { once: true },
);
