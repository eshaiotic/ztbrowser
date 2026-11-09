// ZTBrowser Guard: resource hints that open a connection without any request.
//
// <link rel="preconnect"> and rel="dns-prefetch" make the browser open a connection to a host (and tell it
// the visitor's address) before a request exists, so no request filter ever gets to refuse it. Sites and
// their advertising scripts use them to warm up connections to the very hosts the filters block.
// rel="prefetch" and "prerender" fetch pages ahead of time with the same effect. "preload" is left alone:
// its requests pass through the filters like any other.
//
// The script runs in the page's own world, before any page script, and removes the four tokens from every
// way a page can give a <link> its relationship. A <link> written into the HTML file itself is read by the
// engine's parser before any script runs; those hints are outside what a page script can reach.
(function () {
  'use strict';
  const HINTS = new Set(['preconnect', 'dns-prefetch', 'prefetch', 'prerender']);
  const HINT_IN_TEXT = /\b(preconnect|dns-prefetch|prefetch|prerender)\b/gi;

  const withoutHints = (value) =>
    String(value).split(/\s+/).filter((token) => !HINTS.has(token.toLowerCase())).join(' ');

  const isLink = (node) => node instanceof HTMLLinkElement;

  // 1. link.rel = '...'
  const rel = Object.getOwnPropertyDescriptor(HTMLLinkElement.prototype, 'rel');
  if (rel && rel.set) {
    Object.defineProperty(HTMLLinkElement.prototype, 'rel', {
      configurable: true,
      enumerable: rel.enumerable,
      get: rel.get,
      set(value) { rel.set.call(this, withoutHints(value)); },
    });
  }

  // 2. link.setAttribute('rel', '...') and its namespaced twin
  const setAttribute = Element.prototype.setAttribute;
  Element.prototype.setAttribute = function (name, value) {
    return setAttribute.call(this, name, isLink(this) && String(name).toLowerCase() === 'rel' ? withoutHints(value) : value);
  };
  const setAttributeNS = Element.prototype.setAttributeNS;
  Element.prototype.setAttributeNS = function (namespace, name, value) {
    return setAttributeNS.call(this, namespace, name, isLink(this) && /(^|:)rel$/i.test(String(name)) ? withoutHints(value) : value);
  };

  // 3. link.relList.add('...'), .toggle, .replace and .value. Only the lists of <link> elements are touched:
  // other token lists (class names) may legitimately hold the same words.
  const linkLists = new WeakSet();
  const relList = Object.getOwnPropertyDescriptor(HTMLLinkElement.prototype, 'relList');
  if (relList && relList.get) {
    Object.defineProperty(HTMLLinkElement.prototype, 'relList', {
      configurable: true,
      enumerable: relList.enumerable,
      get() { const list = relList.get.call(this); linkLists.add(list); return list; },
    });
  }
  const isHint = (token) => HINTS.has(String(token).toLowerCase());
  const listMethods = {
    add: (original, list, tokens) => original.apply(list, tokens.filter((token) => !isHint(token))),
    toggle: (original, list, tokens) => (isHint(tokens[0]) ? false : original.apply(list, tokens)),
    replace: (original, list, tokens) => (isHint(tokens[1]) ? false : original.apply(list, tokens)),
  };
  for (const [method, guarded] of Object.entries(listMethods)) {
    const original = DOMTokenList.prototype[method];
    DOMTokenList.prototype[method] = function (...tokens) {
      return linkLists.has(this) ? guarded(original, this, tokens) : original.apply(this, tokens);
    };
  }
  const listValue = Object.getOwnPropertyDescriptor(DOMTokenList.prototype, 'value');
  if (listValue && listValue.set) {
    Object.defineProperty(DOMTokenList.prototype, 'value', {
      ...listValue,
      set(value) { listValue.set.call(this, linkLists.has(this) ? withoutHints(value) : value); },
    });
  }

  // 4. Markup parsed from a string: strip the hints from the text before the engine reads it.
  const scrubMarkup = (markup) => String(markup).replace(/<link\b[^>]*>/gi, (tag) =>
    /\brel\s*=/i.test(tag) ? tag.replace(HINT_IN_TEXT, '') : tag);
  const guardMarkupProperty = (prototype, name) => {
    const descriptor = Object.getOwnPropertyDescriptor(prototype, name);
    if (descriptor && descriptor.set) {
      Object.defineProperty(prototype, name, { ...descriptor, set(value) { descriptor.set.call(this, scrubMarkup(value)); } });
    }
  };
  guardMarkupProperty(Element.prototype, 'innerHTML');
  guardMarkupProperty(Element.prototype, 'outerHTML');
  const insertAdjacentHTML = Element.prototype.insertAdjacentHTML;
  Element.prototype.insertAdjacentHTML = function (position, markup) {
    return insertAdjacentHTML.call(this, position, scrubMarkup(markup));
  };

  // 5. A <link> built elsewhere and then inserted: the engine reads its relationship when it joins a document.
  const scrubNode = (node) => {
    if (!node || typeof node !== 'object') return;
    if (isLink(node)) {
      const own = node.getAttribute('rel');
      if (own !== null) setAttribute.call(node, 'rel', withoutHints(own));
    } else if (node.querySelectorAll) {
      for (const link of node.querySelectorAll('link[rel]')) scrubNode(link);
    }
  };
  const guardInsertion = (prototype, name) => {
    const original = prototype[name];
    if (typeof original !== 'function') return;
    prototype[name] = function (...nodes) {
      nodes.forEach(scrubNode);
      return original.apply(this, nodes);
    };
  };
  for (const name of ['appendChild', 'insertBefore', 'replaceChild']) guardInsertion(Node.prototype, name);
  for (const name of ['append', 'prepend', 'before', 'after', 'replaceWith', 'replaceChildren', 'insertAdjacentElement']) {
    guardInsertion(Element.prototype, name);
  }
})();
