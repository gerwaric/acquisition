// The sitting recorder (C79; SURFACES.md, the trade site's rows: access
// method `browser`). Loaded by the developer tools of the tab they are open
// on, it is told of each request that tab's page finished and keeps two
// kinds: the trade search and the fetch of its items. It sends nothing,
// opens nothing, and reads no cookie: it copies what tools/trade-split.py
// reads — the request's address and posted query, the response's status,
// rate-limit headers and body — and no other header of either side. Of a
// search's answer it reads one thing, how many results it lists, so that the
// sitting page can show a search that found nothing as finished.
//
// Each call is handed to worker.js as a recording of one entry, at once, so
// no body waits in the panel to be dropped.
//
// A file is named for its row. rows.json, written beside this file by
// tools/trade-pages.py, says which row each of the sheet's links is; the tab
// was sent to one of them, and the calls its page then makes are that row's.
// The name is a convenience: tools/trade-split.py names a capture by the
// query it answers and never reads a file's name.

const KEPT = ['/api/trade/search/', '/api/trade/fetch/'];
const LIMIT = name => name.startsWith('x-rate-limit') || name === 'retry-after';

let rows = {};
fetch(chrome.runtime.getURL('rows.json')).then(r => r.json()).then(read => { rows = read; }, () => {});
const plain = address => address.split(/[?#]/)[0];
let row = null;
chrome.devtools.network.onNavigated.addListener(address => { row = rows[plain(address)] || null; });

chrome.devtools.network.onRequestFinished.addListener(finished => {
  const url = new URL(finished.request.url);
  if (url.hostname !== 'www.pathofexile.com' || !KEPT.some(part => url.pathname.includes(part))) return;
  finished.getContent((text, encoding) => {
    const posted = finished.request.postData;
    const content = { size: finished.response.content.size, mimeType: finished.response.content.mimeType };
    // A body the browser did not keep is left out, as an export leaves it.
    if (text !== null && text !== undefined) content.text = text;
    if (encoding) content.encoding = encoding;
    const entry = {
      startedDateTime: new Date(finished.startedDateTime).toISOString(),
      request: {
        method: finished.request.method,
        url: finished.request.url,
        headers: [],
        cookies: [],
        ...(posted ? { postData: { mimeType: posted.mimeType, text: posted.text } } : {}),
      },
      response: {
        status: finished.response.status,
        headers: finished.response.headers.filter(h => LIMIT(h.name.toLowerCase())),
        cookies: [],
        content,
      },
    };
    // A search says which page asked it; a fetch is its search's.
    const from = finished.request.headers.find(h => h.name.toLowerCase() === 'referer');
    if (from && rows[plain(from.value)]) row = rows[plain(from.value)];
    let found = null;
    if (url.pathname.includes('/api/trade/search/') && 'text' in content && !encoding) {
      try { found = JSON.parse(text).result.length; } catch { found = null; }
    }
    chrome.runtime.sendMessage({
      row,
      found,
      kind: url.pathname.includes('/api/trade/search/') ? 'search' : 'fetch',
      status: entry.response.status,
      slowed: entry.response.status === 429 || entry.response.headers.some(h => h.name.toLowerCase() === 'retry-after'),
      body: 'text' in content,
      at: entry.startedDateTime,
      har: { log: { version: '1.2', creator: { name: 'acquisition sitting recorder', version: '1' }, entries: [entry] } },
    });
  });
});
