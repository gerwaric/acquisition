// Writes each call the recorder kept as a file of its own under the
// browser's downloads folder, acq-sitting/<row>_<time>-<kind>, and counts the
// searches on the toolbar badge. tools/trade-split.py reads the folder.
// The badge turns red, and stays red, at a call that was refused, lost its
// body or was asked to slow down; its title says which. A click on it
// starts the count again and forgets every row.
//
// It also tells the sitting page (page.js) what is saved for each row, once
// the browser says the file is written:
//
//   zero     the search is saved and found nothing: no fetch will follow
//   pending  the search is saved and found items: their fetch is owed
//   saved    the search and a fetch of its items are saved
//   failed   a call was refused, lost its body, or its file was not written
//
// "Saved" is a file on disk and no more: whether it answers its row is
// tools/trade-split.py's to say.

const FOLDER = 'acq-sitting';
const CLEAN = { searches: 0, fault: '', rows: {} };

chrome.downloads.setUiOptions({ enabled: false });

function show({ searches, fault }) {
  chrome.action.setBadgeBackgroundColor({ color: fault ? '#a00' : '#024' });
  chrome.action.setBadgeText({ text: String(searches) });
  chrome.action.setTitle({ title: fault || 'Searches saved this sitting' });
}

const ports = new Set();
const tell = message => ports.forEach(port => port.postMessage(message));

// One change at a time: the state is read and written back around each.
let turn = Promise.resolve();
const change = with_state => {
  turn = turn.then(async () => {
    const state = await chrome.storage.session.get(CLEAN);
    const message = await with_state(state);
    await chrome.storage.session.set(state);
    show(state);
    if (message) tell(message);
  });
};

chrome.runtime.onConnect.addListener(port => {
  if (port.name !== 'sitting') return;
  ports.add(port);
  port.onDisconnect.addListener(() => ports.delete(port));
  chrome.storage.session.get(CLEAN).then(state => port.postMessage({ rows: state.rows, held: true }));
});

chrome.action.onClicked.addListener(() => change(state => {
  Object.assign(state, structuredClone(CLEAN));
  return { clear: true };
}));

// What the row's mark becomes when this call's file is written, or is not.
function after(was, call, written) {
  if (!written) return { state: 'failed', detail: 'not saved' };
  if (call.slowed) return { state: 'failed', detail: 'slow down' };
  if (call.status !== 200) return { state: 'failed', detail: String(call.status) };
  if (!call.body) return { state: 'failed', detail: 'no body' };
  if (call.kind === 'search') return call.found === 0 ? { state: 'zero' } : { state: 'pending' };
  return was && was.state === 'failed' ? was : { state: 'saved' };
}

// Downloads on their way, by the browser's number for each.
const writing = new Map();

chrome.downloads.onChanged.addListener(delta => {
  if (!delta.state || delta.state.current === 'in_progress' || !writing.has(delta.id)) return;
  const call = writing.get(delta.id);
  writing.delete(delta.id);
  if (!call.row) return;
  change(state => {
    state.rows[call.row] = after(state.rows[call.row], call, delta.state.current === 'complete');
    return { rows: { [call.row]: state.rows[call.row] } };
  });
});

chrome.runtime.onMessage.addListener(call => {
  const bytes = new TextEncoder().encode(JSON.stringify(call.har));
  let binary = '';
  for (let i = 0; i < bytes.length; i += 0x8000) binary += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  delete call.har;
  // The browser names the file by its type: .json, whatever is asked.
  chrome.downloads.download({
    url: 'data:application/json;base64,' + btoa(binary),
    filename: `${FOLDER}/${call.row || 'unknown'}_${call.at.replace(/[:.]/g, '-')}-${call.kind}.json`,
    conflictAction: 'uniquify',
    saveAs: false,
  }).then(id => writing.set(id, call), () => {
    if (call.row) change(state => {
      state.rows[call.row] = after(state.rows[call.row], call, false);
      return { rows: { [call.row]: state.rows[call.row] } };
    });
  });
  change(state => {
    if (call.kind === 'search') state.searches += 1;
    const what = call.slowed ? 'THE SITE ASKED THE SITTING TO SLOW DOWN'
      : call.status !== 200 ? `a ${call.kind} answered ${call.status}`
      : !call.body ? `a ${call.kind} with no body` : '';
    if (what && (call.slowed || !state.fault)) state.fault = `${what}, at ${call.at}`;
  });
});
