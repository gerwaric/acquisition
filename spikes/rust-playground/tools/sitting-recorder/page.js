// Runs in a sitting page (a local file: the extension needs "Allow access to
// file URLs"). It tells the page the recorder is here, and marks each search
// with what worker.js says was saved for its row. It changes two attributes
// of a search's list item and nothing else; the page's own script and style
// read them.

document.documentElement.dataset.recorder = 'here';

const items = new Map(Array.from(document.querySelectorAll('ul.searches li'),
  li => [li.querySelector('a').textContent, li]));

// `held` is everything the worker holds, sent at each connection: it must not
// paint an earlier answer over a search opened again on this page.
function mark(rows, held) {
  for (const [row, { state, detail }] of Object.entries(rows)) {
    const li = items.get(row);
    if (!li || (held && li.dataset.opened !== undefined && !li.dataset.capture)) continue;
    li.dataset.capture = state;
    li.dataset.detail = detail || '';
  }
}

// The worker sleeps when idle and the port closes with it: connect again,
// and it sends every row it holds.
function connect() {
  let port;
  try {
    port = chrome.runtime.connect({ name: 'sitting' });
  } catch {
    // The extension was reloaded under this page: reload the page.
    document.documentElement.dataset.recorder = 'gone';
    return;
  }
  port.onMessage.addListener(message => {
    if (message.clear) items.forEach(li => { delete li.dataset.capture; delete li.dataset.detail; });
    mark(message.rows || {}, message.held);
  });
  port.onDisconnect.addListener(() => setTimeout(connect, 1000));
}
connect();
