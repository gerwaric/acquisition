# Local Control CLI

Status: **IMPLEMENTED AND LOCALLY VERIFIED** (August 2026).
This document is frozen to the coherent local branch before its eventual
pull-request boundaries are chosen. It records the implemented contract, not an
upstream commitment.

## Purpose

Give scripts and coding-agent skills a stable way to inspect Acquisition's
published inventory and ask the already-running application to refresh it.
Acquisition remains the sole owner of OAuth credentials, API rate-limit state,
persistence, and the refresh lifecycle.

The user-facing automation surface is a small CLI, `acquisitionctl`. Agent
integration is a skill that invokes that CLI; Acquisition does not embed MCP or
another agent-specific protocol.

## Scope and decisions

1. **The GUI process is the service.** `acquisitionctl` connects to an open
   Acquisition process. It never starts another synchronizer and never opens
   Acquisition's databases. If no application is listening it reports
   `not_running`.
2. **Local IPC only.** The application listens through `QLocalServer` with
   same-user access. There is no TCP port, remote API, or network discovery.
3. **One request and response vocabulary.** Both directions use framed JSON
   with an explicit protocol version. Human diagnostics go to stderr; stdout
   remains machine-readable.
4. **Viewing means published application state.** Item results come from
   `ItemsManager` and effective prices from `BuyoutManager`, matching what the
   GUI consumes rather than exposing persistence schemas or OAuth material.
5. **Refreshes belong to the application.** A client starts a refresh and gets
   an operation id. Disconnecting the client does not cancel it. Another client
   can inspect or wait for that operation later.
6. **Existing shop behavior is unchanged.** The control path invokes the same
   refresh command as the GUI. A clean completion triggers automatic shop work
   only when the user's existing setting already enables it. Refresh completion
   does not claim that asynchronous forum work completed.
7. **No total refresh timeout.** Valid refreshes may take hours. Client-side
   waiting may have a caller-selected timeout, but it never owns or cancels the
   application operation.
8. **Integration-first development.** The complete local result is tested,
   reviewed commit-by-commit, dogfooded through the skill, and brought to a
   clean release-quality checkpoint before PR slices are derived from the real
   diff.

## Non-goals for version 1

- Running without the GUI, daemon mode, cron auto-start, or a second
  Acquisition process.
- MCP, HTTP, TCP, or remote-machine control.
- Editing buyouts, settings, searches, or forum threads.
- A refresh-cancel command.
- Raw OAuth tokens, POESESSID values, raw wire payloads, logs, or settings.
- Reproducing the visual tree, tooltip rendering, or screen automation.
- Executing saved searches or defining a second query language.
- Changing the application's existing multi-instance policy. Only the process
  that owns the data-directory control endpoint is addressable through it.

## Architecture

### Components

`LocalControlServer`
: Owned by `Application`. It binds one endpoint derived from the canonical
  application data directory, accepts local connections, validates framing and
  protocol version, and delegates commands. It does not own domain state.

`ControlService`
: The application-facing command service. Before login it can answer service
  and authentication status. After `UserSession` construction it is attached
  to `ItemsManager`, `ItemsManagerWorker`, and `BuyoutManager`. It serializes
  published state and owns refresh operation records.

`acquisitionctl`
: A `QCoreApplication` executable linked only to the protocol/client support it
  needs. It discovers the endpoint from the same canonical `--data-dir`, sends
  one command, prints JSON, and exits. `refresh wait` polls with bounded
  single-command connections; its lifetime never controls the refresh. Release
  packages keep it beside the GUI executable on Windows, inside the app bundle
  on macOS, and in a separate CLI AppImage on Linux; documentation and the skill
  resolve those paths rather than assuming a system-wide installation.

`acquisition-cli` skill
: A separately reviewable skill that teaches agents command ordering,
  revision-safe pagination, refresh semantics, and credential boundaries. It
  contains no privileged integration and calls only `acquisitionctl`.

### Endpoint identity and ownership

The endpoint identity is a digest of the canonical data-directory path,
namespaced by application and protocol identity. Different test or user data
roots do not collide. Unix places the socket and lock in a short, owner-only
OS-managed user runtime root (Darwin's user temp directory or
`/run/user/<uid>`), with a validated passwd-home fallback when that root is
unavailable. Discovery does not depend on the caller's `HOME` or
`XDG_RUNTIME_DIR`. Windows uses the digest as the named-pipe identity
and keeps the lock under the data directory. The server requests
`QLocalServer::UserAccessOption` and holds the endpoint lock while listening.

A listen failure must not delete an endpoint blindly. After taking the lock,
the application probes before calling `listen` because Qt may otherwise unlink
a live Unix socket. A successful connection means another process owns the
endpoint. Only a refused or missing-endpoint probe while holding the lock
permits stale endpoint removal and one listen retry. A second GUI retries ownership every two
seconds so it can take over if the owner exits.

The current multi-GUI policy remains out of scope. If two application processes
use one data directory, exactly the listener owner is controlled.

### Framing

Each message is:

1. a four-byte unsigned big-endian payload length;
2. exactly that many UTF-8 JSON bytes.

Version 1 rejects zero-length frames, frames above a fixed small request limit,
invalid JSON, non-object roots, missing protocol/command fields, and unsupported
protocol versions. A connection dispatches at most its first complete request
and returns one response; already-buffered or later trailing data is never
dispatched as another command before the server closes the connection. The
server bounds unread buffered data, concurrent connections, request time, and
response size, and closes on framing violations.

Every response has this envelope:

```json
{
  "protocol": 1,
  "request_id": "client supplied opaque id",
  "ok": true,
  "result": {}
}
```

Errors use `ok: false` and a stable object:

```json
{
  "code": "invalid_request",
  "message": "human-readable diagnostic"
}
```

Unknown fields are ignored within a protocol version so additive evolution is
possible. Changed semantics or removed fields require a new protocol version.

## Status contract

`status` is available before login and never exposes credentials. Version 1
returns:

- protocol and application version;
- service state: `starting`, `needs_login`, `loading_cache`, or `ready`;
- account and league only after a session exists;
- inventory revision and counts when available;
- refresh state and active operation id;
- the most recently retained terminal operation outcome.

The service does not infer successful authentication merely from a stored token;
it reports lifecycle state established by `Application`.

## Published inventory and revisions

`ItemsManager` is the authority for viewed items. The control service never
reads repository tables behind it.

A monotonically increasing `quint64` inventory revision changes after every
published-state mutation:

- initial cached snapshot;
- per-source tab delta;
- child reconciliation;
- final snapshot reconciliation;
- effective buyout changes that alter viewed price data.

A process restart starts a new revision epoch. Responses therefore pair the
numeric revision with a process-unique `instance_id`; clients compare both.

### Tabs

`tabs` returns the current display locations in `ItemsManager`'s published
`LocationInventory`, including:

- kind (`stash` or `character`), stable display id, label/name, index, and tab
  type;
- colour where represented by `ItemLocation`;
- remove-only status;
- refresh checked and locked state;
- current published item count.

The schema must not use display-location equality as a fetch-source identity.
Special-tab items expose both display id and fetch-source id. Tab responses are
bounded to 100 entries and use the same revision-safe continuation model as item
responses.

### Items

`item` and `items` expose only stable, already-modeled fields needed for
inventory inspection:

- id, name, type line, category, item level, stack count, frame type;
- identified/corrupted/crafted/enchanted/fractured/split/synthesized/mutated
  flags and influence names;
- dimensions, socket groups, properties, requirements, and parsed modifiers;
- note and normalized location (kind, display id, fetch-source id, label,
  position, dimensions, inventory/character metadata);
- effective buyout value, type, currency, source, inherited flag, and update
  time from `BuyoutManager::Get`.

Absent or null values remain distinct from empty strings where the model can
represent that distinction. Enum values use stable lowercase strings, never
Qt/C++ ordinal values.

### Pagination consistency

`items` accepts a bounded `limit`, an optional tab id, and an opaque cursor. The
first page captures `(instance_id, revision)` and the deterministic item order.
Every subsequent page supplies that pair. If published state changed, the
server returns `revision_changed`; it never silently combines pages from two
states.

The implementation uses an opaque source-index cursor over `ItemsManager`'s
source-keyed published buckets. It does not materialize the dirty flat view.
The cursor embeds the original limit, filters, source position, `instance_id`,
and revision and is authenticated with a process-private HMAC; cursor requests
cannot alter or override that state. Revision is
checked before reading each page and serialization runs in the application
thread. Unfiltered first pages report `total`; filtered pages omit it rather
than scanning the full result set. A page examines at most 10,000 source items,
so a sparse filter can return an empty item array with a continuation cursor.
Clients continue until the cursor is null. Tab item counts aggregate the
source-keyed published buckets rather than flattening or scanning every item.
Single-item lookup uses an index maintained with snapshots, source replacements,
and child reconciliation. Page and scan limits bound UI-thread work and response
memory. The measured cost did not justify an immutable snapshot.

## Refresh operations

### Commands

`refresh.start`
: Starts the application's `UpdateScope::All` path. The client generates a
  request id that also becomes the operation id. The command returns
  immediately with `accepted` and that id, or `busy`. A busy
  response includes `active_refresh_id` only when the active refresh was started
  through control; GUI-originated worker activity has no control operation id.
  The request id is also the operation id and idempotency key. The service
  retains bounded start responses, including rejections, so retrying an
  ambiguous transport outcome cannot change the decision or start duplicate
  work. It queues the actual update after sending the response so no terminal
  signal can nest inside request handling.

`refresh.status`
: Returns the operation's state (`queued`, `running`, `completed`, `failed`),
  latest cosmetic progress, and terminal outcome when available.

`refresh wait` (CLI workflow)
: Polls `refresh.status` until a retained operation reaches a terminal outcome.
  It is deliberately not a server command. Client timeout or disconnect ends
  only observation; the operation continues.

### Identity and retention

The control service assigns operation ids from refresh-start request ids; worker
internals do not need to expose update identity. At most one operation is active because the worker is
single-update. An in-memory history retains the 32 most recent records for
reconnection. Process restart invalidates ids and is reported as
`operation_not_found` under the new `instance_id`.

### Outcomes

- A clean `CompletedRefresh` maps to `completed` with no skips.
- `CompletedRefresh` with skipped sources maps to `completed` plus structured
  skips and a non-clean flag; it is not mislabeled as a full success.
- `FailedRefresh` maps to `failed` with the typed error kind and message.

CLI `refresh wait` uses distinct exit statuses for clean completion,
completed-with-skips, failed refresh, busy/not accepted, not running, invalid
request, and client-side wait timeout. The JSON body, not the number, remains
the primary contract.

## Skill behavior

The distributable skill lives at `contrib/skills/acquisition-cli/`. It instructs
an agent to:

1. call `status` before inventory commands;
2. distinguish initialization, live incremental refresh state, and idle state;
3. use bounded pages and restart after `revision_changed`;
4. request refresh only with user intent because it performs network work and
   may preserve already-enabled automatic shop behavior;
5. start long work, retain the operation id, and inspect/poll rather than hold
   an agent tool call for hours;
6. treat client interruption as loss of observation, not refresh cancellation;
7. distinguish clean, skipped, and failed outcomes;
8. use normalized effective price fields rather than parsing notes;
9. never seek credentials or unsupported mutations.

## Security and robustness

- Same-user local socket access is requested on every platform. On Windows the
  client additionally compares the named-pipe server process token SID with its
  own before sending a request. This prevents cross-user impersonation and
  request disclosure; another user who guesses the pipe name can still deny
  service by occupying it first.
- No credential-bearing object is reachable through command serialization.
- Request and response frames, page limits, retained jobs, connection counts,
  idle request time, and connection buffers are bounded. A conservative JSON
  size walk rejects oversized responses before serialization allocates the
  payload.
- Malformed input cannot terminate the GUI process.
- Disconnect cleanup removes sockets, never accepted refresh jobs.
- Application shutdown closes listeners and clients before their referenced
  domain services are destroyed.
- Logs avoid full requests and item payloads by default.

## Verification contract

### Unit and component tests

- endpoint naming is stable per canonical data root and distinct across roots;
- frame fragmentation, single-dispatch behavior for pipelined/trailing data,
  oversize lengths, invalid JSON, unsupported versions, unknown commands, and
  request-id echo;
- service status before and after session attachment;
- item, tab, enum, location, and effective-price serialization;
- revision increments for snapshots, deltas, reconciliations, and buyout changes;
- bounded pagination, filters, cursor validation, and revision changes;
- refresh accepted/busy, queued start, progress, clean/skipped/failed outcomes,
  bounded retention, wait timeout, and disconnect survival;
- no secret fields in any response.

### Integration tests

- real `QLocalServer`/`QLocalSocket` round trips with fragmented writes;
- stale endpoint recovery without removing a live listener;
- multiple clients, connection caps, idle timeout, and owner-only Unix socket
  permissions;
- application/service shutdown ordering and session destruction;
- process-level CLI/server status round trip, parser validation, stdout JSON,
  and stderr diagnostics;
- refresh dispatch uses the same application path as the GUI, preserving its
  existing shop gating.

### Scale and review

The checked-in `control_benchmark` is excluded from normal builds and measures a
complete cursor traversal of deterministic published collections. A local
Release run on an Apple M4 Max measured:

- 101,048 items in 1,011 pages: 1,064.366 ms total, 4.562 ms maximum page,
  0.756 ms sparse-filter page; 20 tab pages took 8.198 ms total and 0.477 ms
  maximum per page;
- 975,711 items in 9,758 pages: 9,593.230 ms total, 5.442 ms maximum page,
  0.453 ms sparse-filter page; 26 tab pages took 9.989 ms total and 0.400 ms
  maximum per page.

The local checkpoint completed a clean RelWithDebInfo build and all 39 tests.
The four control-focused tests also passed an AddressSanitizer build
(`detect_leaks=0`, as required on macOS). Fresh-agent skill dogfood against an
offscreen GUI with temporary data verified status, not-ready viewing and refresh
errors, stopped-application handling, help, parser errors, JSON envelopes, and
exit statuses.

The final checkpoint requires a clean configure/build and complete `ctest`,
applicable sanitizer checks, safe-data GUI/CLI dogfood, and a clean Diffwarden
review of every final commit and the complete branch. Valid findings are fixed
and re-reviewed; declined findings are recorded with the invariant they
misread.

Passing tests establish exercised behavior only. Windows named-pipe ACLs and
server-token authentication, plus native Windows/Linux release packaging,
remain validation gaps until CI or a native host verifies them. A live authenticated refresh and forum update are
not run against user data during local validation.

## Local implementation sequence

1. Protocol codec, endpoint identity, server/client foundation, `status`, and
   removal of the standalone headless prototype.
2. Inventory revision and viewing (`tabs`, `items`, `item`) with effective
   pricing and pagination.
3. Application-owned refresh operation registry and start/status/wait.
4. CLI skill and dogfood scenarios.
5. Full build, tests, scale checks, security review, and independent review.
6. Freeze this document to the verified implementation (**complete**).

Only after step 6 is the completed diff inspected for reviewer-aligned PR
boundaries. A recoverable backup is created before any history surgery. Each
resulting independent or stacked branch must compile and pass tests on its own.
No branch is pushed and no issue or pull request is posted without explicit user
approval.
