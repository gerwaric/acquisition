# The control contract (PR #192) as the A0 seam

Status: review result, round 1 — written to be handed to Auro
Provenance: produced 2026-08-08 (Claude, Tom's session) by the
step-zero review that `topics/migration-order.md` names under A0.
Reviewed: the PR's spec `docs/design/local-control.md` **on the PR
branch** (`private-league-support` at `69cffb70`; the spec does not
exist on `master`/`redesign`), then the implementation, then a full
build-and-drive on Tom's Windows workstation against a scrubbed copy
of his Standard-league data. All `file:line` cites are to the PR
branch. Lanes: **measured** (observed at runtime here, or a cited
benchmark), **traced** (read in code), **inferred** (stated as such).
The merge-side review is a separate deliverable
(`docs/redesign/pr192-merge-review.md`, untracked, graded
blocking/question/nit); per the review brief the two verdicts are
decoupled, and nothing in this document is a merge objection.

## The answer in one paragraph

The PR #192 contract is a **good seed for the permanent UI/core
seam, and it is cheap to walk back** — the two properties the
migration-order plan actually needs from step zero. The generation
token the windowed protocol calls for already exists as the
`(instance_id, inventory_revision)` pair; pages are strictly bounded
by three independent mechanisms; the refresh terminal outcome is
exposed exactly in the shape the shop auto-post gate consumes
(answering shop-write-path §5's open question: **yes**); the command
side is not read-only by construction — `refresh.start` establishes
idempotent command semantics that `set_item_buyout` and the rest of
the A0 addendum can reuse; and an internals-leakage audit found
nothing a Rust core could not reproduce, down to the endpoint name
(re-derived here from an independent non-Qt client — measured). Two
things are genuinely missing, and neither is foreclosed: a written
"no endpoint may ever return all matching items" standing constraint
(R1-2), and any notification path — v1 is strictly poll-based
request/response, which is right for a CLI and insufficient for a
live UI (R1-7). R1-7 is the one piece of design work A0 owes before
a UI consumes this seam.

## 1. What the contract is (traced; measured where noted)

One framed-JSON request/response vocabulary over a local socket or
named pipe, protocol version 1 in every envelope, exact-match
version gate (`controlprotocol.cpp:22-25`; measured: a protocol-2
request answers `unsupported_version`). Commands: `status`, `tabs`,
`items`, `item`, `refresh.start`, `refresh.status`
(`controlservice.cpp:416-439`). Reads serve **published** state only
(`ItemsManager` buckets + `BuyoutManager` effective prices), never
persistence. Pagination is an opaque, HMAC-authenticated forward
cursor carrying `(instance_id, revision, source position, filters)`;
any published-state mutation bumps the revision and the next cursor
use answers `revision_changed` (measured mid-refresh: a pre-refresh
cursor was refused exactly this way while deltas streamed).
Refreshes are application-owned operations: client request id =
operation id = idempotency key, retained decisions, `busy` while one
is active, terminal outcomes retained in a 32-deep history.

Measured here (Windows, Release, 40,906 items / 379 locations):
full tabs traversal 8 pages in 0.31 s; items pages of 100 at
~55 ms/page including process spawn per page; the four control test
suites plus the rest of the checked-in suite pass 39/39.

## 2. Checklist verdicts

### R1-1 — Windowed-protocol fit: grows by addition, not breakage

The migration-order shape is query → (generation token, count,
windowed rows). Present already: the generation token is
`(instance_id, revision)` — instance for the epoch, string-encoded
u64 for the revision (`controlservice.cpp:441-446`); the count is
`total` on unfiltered pages plus exact per-tab `item_count` on every
tab row (aggregated from the source-keyed buckets, never scanned —
`itemsmanager.h:53-56`, `controlservice.cpp:552-555`); rows are
windowed. What v1 does **not** have is random access — the cursor is
forward-only over the bucket map, so a virtualized UI cannot jump to
row 800,000. That is additive to fix (a windowed-read command over
the same deterministic order; "unknown fields are ignored within a
protocol version" is the spec's stated evolution rule), and the
deterministic order it needs already exists (source-key map order ×
in-bucket index, `controlservice.cpp:621-667`). No v1 consumer
breaks. Traced; the additive path is inferred but uses only the
spec's own rules.

One design note for that future command (inferred): under streaming
deltas the revision moves constantly, so a scroll position pinned to
`(instance, revision)` invalidates on every delta. A live UI window
needs either snapshot isolation or delta-aware windows — exactly the
job `Search`/`ItemsModel` does in-process today. The spec records
that v1 measured and rejected immutable snapshots for its own needs
("The measured cost did not justify an immutable snapshot"); the
windowed-UI requirement will reopen that measurement with different
stakes. This belongs to the A1/B design, not to this contract's v1.

### R1-2 — Page bounding is real; write it down as a standing constraint

Three independent mechanisms bound every response: the page limit
(≤100, `controlservice.cpp:29-30`), the per-page scan cap (10,000
source items, so a sparse filter returns an empty page with a
continuation cursor rather than scanning 1M —
`controlservice.cpp:33`, spec'd), and a serialized-size budget
(4 MiB − 64 KiB envelope reserve) enforced by an exact pre-allocation
JSON size walk (`localcontrolserver.cpp:37-132`) with a lower-bound
fast path per item (`viewprojection.cpp:158-239`). Measured: a
999 MB frame-length header is refused; oversized-response handling
is tested at the exact boundary. **But the principle itself —
migration-order's "no endpoint may ever return all matching items
across IPC" — is enforced structurally, not stated.** The spec
should say it as a standing constraint, so a future convenience
endpoint cannot reintroduce the 13-second path by being individually
reasonable. Recommendation: one sentence in `local-control.md` at
merge time; the redesign side already owns the rationale
(migration-order, "conditions").

### R1-3 — Refresh terminal outcome: present, faithful, sufficient for the gate

`ProjectOutcome` (`controlservice.cpp:67-90`) maps `RefreshOutcome`
without loss: clean completion (`clean: true`, empty `skipped`),
completed-with-skips (structured per-source skips, each with the
typed `FetchError` kind/message/http_status/attempts — and
deterministically sorted), failed (typed error). The CLI splits exit
codes 0/6/5 on exactly this (`acquisitionctl.cpp:335-341`).
**Shop-write-path §5's open question is answered: yes**, the
contract exposes the `RefreshOutcome` equivalent the auto-post gate
consumes. The gate itself stays core-side (control `refresh.start`
drives the same `ItemsManager::Update` path the GUI uses,
`application.cpp:236-246`, so existing shop gating is preserved
untouched — spec decision 6). One scoping note (traced): the
operation registry retains only **control-originated** operations;
a GUI-originated refresh is visible as `refresh_state: "updating"`
but has no operation id and leaves no retained outcome
(`controlservice.cpp:767-771` — `busy` includes `active_refresh_id`
only for control-started work, per spec). Fine for A0, where gate
and refresh share the core side; a UI-on-A0 that wants to display
"last refresh had 3 skips" regardless of who started it would need
an additive status field.

### R1-4 — De-arming semantics: room exists; nothing is wired yet

Session rejection as a terminal, de-arming event (CG4,
credential-custody) is representable but absent from v1 — correctly,
since v1 excludes the shop surface entirely. What matters for the
seam is that the error/event model has room, and it does: error
`kind`/`code` sets are open string vocabularies (unknown values are
the server's to add; clients are told to ignore unknown fields), and
`status` can grow an auth/session section additively. What v1
cannot do is *push* the event — see R1-7; a de-arm would surface on
the next poll. The credential surface itself already matches
credential-custody §3's rules in v1: no secret appears in any
response (traced across every serializer in
`controlservice.cpp`/`viewprojection.cpp`; observed across every
runtime response here — status, tabs, items, item, refresh.*), and
the spec's non-goals exclude tokens/POESESSID/raw wire payloads
explicitly.

### R1-5 — Command-path fit: refresh.start is the template the A0 addendum needs

The investigations concluded the A0 command surface is small (buyout
edits with M3 one-change-set-per-command semantics, shop settings,
`post_shop`, `set_poesessid`). This contract extends there without a
second protocol. `refresh.start` already establishes the command
pattern a future write surface wants: client-generated request id as
idempotency key; **retained decisions** including rejections, so an
ambiguous transport retry cannot double-fire
(`controlservice.cpp:728-744`; the CLI's `start_unconfirmed` path
surfaces the operation id to query before retrying —
`acquisitionctl.cpp:278-298`, tested); dispatch queued after the
response so no terminal signal nests inside request handling
(`controlservice.cpp:796-857`). A `set_item_buyout` command lands on
machinery that is already listening: the service subscribes to
`BuyoutsChanged` and bumps the revision per change-set
(`controlservice.cpp:382-385`), which is M3 D1 rule 4's one-event-
per-command boundary arriving for free. `set_poesessid` fits as the
one inbound-only secret (credential-custody §3); nothing in the
envelope or transport would need to change.

### R1-6 — Internals leakage: none found

Audited every serialized field against the "could a Rust core
reproduce this?" bar:

- Enums are stable lowercase strings with **unknown → null**, never
  ordinals (`viewprojection.cpp:21-156`: frame types, buyout
  types/sources, property display modes and value types; spec
  requires exactly this).
- Timestamps are UTC ISO-8601 with milliseconds
  (`controlservice.cpp:791`, `viewprojection.cpp:105`).
- Revisions are u64 serialized as JSON **strings**, dodging the
  double-precision truncation a naive JSON number would hit
  (`controlservice.cpp:446`); cursor offsets likewise
  (`controlservice.cpp:162-178`).
- Cursors are opaque and HMAC-sealed with a process-private key
  (`controlservice.cpp:120-160`) — their contents are internals but
  **cannot ossify**: no client can depend on what it cannot read,
  and the spec says "must never be decoded or modified". This is the
  single best walk-back property in the contract.
- Item/tab ids are upstream API ids; special tabs expose display id
  and fetch-source id as distinct fields (observed in real data:
  a remove-only tab with `fetch_source_id` ≠ `id`).
- Endpoint identity is a SHA-256 digest of the canonical data-dir
  path (`controlendpoint.cpp:150-176`) — **measured**: an
  independent PowerShell (.NET) client re-derived the pipe name from
  the documented recipe and connected. Nothing Qt-specific survives
  in the wire identity.

One non-guarantee worth stating (inferred): QJson serializes object
keys in its own order; byte-identical output across implementations
is not promised anywhere and should not become an implicit contract
— semantic equivalence is the bar.

### R1-7 — The structural gap: no notification path

Version 1 is strictly one request → one response per connection; the
server dispatches at most the first complete frame and ignores the
rest (`localcontrolserver.cpp:384-403`, tested against pipelined
data). There are no events, no subscriptions, no long-lived
connections. For the CLI/agent consumer this is the right shape
(simplest possible client; the skill teaches poll-don't-hold). For
the **permanent UI seam** it is the missing piece: a live UI cannot
poll `status` at frame rate for revision changes, and Tauri-side
rendering wants push invalidation. Options, none foreclosed
(inferred): a long-poll command ("respond when revision ≠ X") fits
the existing one-request model but collides with the server's 5 s
request timeout (`localcontrolserver.h:28`), which would need a
per-command bound; or v2 changes connection lifecycle to allow a
subscription stream. Either way this is **design work A0 owes before
the Qt UI or a spike UI consumes the seam** — it should be decided
deliberately, not grown ad hoc. Cross-reference: migration-order's
open question on the A0 versioning story; this is the sharper form
of it.

### R1-8 — Versioning story: gate is real; the promise needs restating at merge

What exists: exact-match envelope version (measured rejection of
protocol 2), the additive rule ("unknown fields are ignored within a
protocol version; changed semantics or removed fields require a new
protocol version"), and rejection of unsupported versions with a
stable error code. What does not exist: capability discovery beyond
`application_version` in `status`, per-command versions, or any
compatibility promise — the spec explicitly disclaims one ("records
the implemented contract, not an upstream commitment"). Merging
changes that: the moment users exist, the document should say what
is promised (v1 semantics frozen; evolution additive within 1;
semantics changes cost a version) and what is explicitly unstable.
Candidates for an explicit **unstable** marker in v1: the cosmetic
`progress` string (already spec'd as cosmetic), the
`latest_refresh` retention depth (32), and the `not_started` outcome
kind that dispatch-time failures produce
(`controlservice.cpp:810-833`) — a shape the spec's outcome section
does not mention. Small list; worth writing down so v2 stays cheap.

### R1-9 — Restart error taxonomy (small, worth one sentence)

The cursor HMAC key rotates per process instance and per data-dir
reset (`controlservice.cpp:320-321,353-354`), so a cursor held
across a GUI restart fails signature verification and reports
`invalid_cursor` ("malformed") rather than `revision_changed` —
the tamper shape and the world-changed shape are conflated exactly
where the skill's "restart after `revision_changed`" guidance
(`SKILL.md:117`) doesn't reach. Traced; consequence inferred. Fix is
documentation (treat `invalid_cursor` on a previously-valid cursor
as a restart signal), or a v2 cursor carrying the instance id in
cleartext beside the sealed payload. Also filed as merge-review Q6.

## 3. Recommendations (all compatible with merging as-is)

1. Write R1-2's standing constraint into `local-control.md` at merge
   time — one sentence, redesign cites it thereafter.
2. Flip the spec's status line from "not an upstream commitment" to
   an explicit v1 promise + unstable list (R1-8) in the merge commit
   or immediately after.
3. Document the restart/`invalid_cursor` case in the skill (R1-9).
4. Treat R1-7 (notification path) as an A0 design item on the
   redesign side — no PR change requested; the decision belongs to
   the windowed-protocol design, with this contract as its base.

## Candidate findings (for `docs/cleanup/findings.md`, on master)

**None.** The review brief asked for new correctness problems in
code the PR does not touch; none were found this round. (One
cosmetic aside, not register-worthy: the refresh progress string
"Receieved N/M stash tabs…" is a pre-existing typo on master,
`itemsmanagerworker.cpp:1266`, and it now surfaces verbatim in
control responses as the cosmetic `progress` field.)

## Dead ends and rejected interpretations

- **Rejected: "the tab item counts can disagree with filtered item
  queries for special tabs."** Suspected because counts are keyed by
  the *embedded* item location while queries match on the
  *canonical* location. Both key by `(type, display id)` and
  `Canonical()` is a same-key metadata refresh
  (`locationinventory.h:23-44`), so they cannot diverge; runtime
  confirmed a tab filter returning exactly the tab row's
  `item_count`.
- **Rejected: "the busy-decision memoization can wedge a client
  out."** A rejected `refresh.start` is replayed forever **for that
  request id** (`controlservice.cpp:728-744`) — deliberate
  idempotency, and a fresh id is admitted normally (observed). The
  gap is only that no test covers fresh-id-after-busy; filed as
  merge-review Q4, not a contract defect.
- **Rejected: reading the one-request-per-connection rule as a codec
  limit.** It is server policy (`localcontrolserver.cpp:384-403`);
  the framing itself would carry streams. R1-7's options stay open.
- **Not pursued: exercising GUI-originated refresh concurrency**
  (control `busy` without `active_refresh_id`). Traced only
  (`controlservice.cpp:767-771`); driving the GUI's refresh action
  mid-review added risk for a branch already covered by the service
  tests' readiness matrix.
- **Expected but absent, deliberately: any shop/credential surface.**
  Verified absent rather than assumed: no command reaches `Shop`,
  no response field carries a credential. This is v1 scope working
  as intended, not a gap (the A0 addendum lands later per
  shop-write-path §5).

## Verification notes

- **Spec-read**: `local-control.md` in full, against the brief's
  checklist, before any implementation reading (brief method rule).
- **Code-traced** (PR branch at `69cffb70`): all of `src/control/`
  (protocol codec, endpoint identity, server, client, service,
  projection), `src/acquisitionctl.cpp`, the
  `ItemsManager`/`ItemsManagerWorker`/`Application`/`main.cpp`
  integration diffs, and spot-verification inside the four test
  suites. Two subagent passes (tests; packaging/CI/skill) fed the
  merge review; their load-bearing claims were re-verified directly
  before use.
- **Observed at runtime** (this workstation, Windows 11, MSVC 2022
  Release, Qt 6.11.1): 39/39 ctest; the full drive described in §1
  and the merge review — including the mid-refresh
  `revision_changed`, the raw-frame error shapes via an independent
  named-pipe client, endpoint re-derivation, `refresh wait` timeout
  semantics, and idempotent-start behavior. The one live refresh was
  deliberately ended early (~60/713 fetch sources; rate-limit pacing
  put completion 1–2 hours out at the account's shared budget), so
  the **terminal outcome projection is traced and unit-tested, not
  observed live** — everything before the terminal signal was.
  Scope guards honored:
  scrubbed data copy only (POESESSID absent, shop auto-post and
  auto-refresh disabled), no interactive login (the remembered
  token's normal startup refresh only), no forum or legacy-endpoint
  traffic, no credential values read or quoted, no GUI/source
  changes, nothing posted to GitHub.
- **Taken from repo docs, not re-verified**: M2/M3 delta-pipeline
  invariants (D-numbers), shop-write-path and credential-custody
  conclusions, the spec's M4 Max benchmark numbers.

## Open questions

- R1-7's answer: long-poll within v1's connection model, or a v2
  subscription stream? Owns the A0 windowed-protocol design's first
  page. (This supersedes migration-order's "does A0 need a formal
  versioning story before two consumers exist" — versioning exists;
  the notification path is the real open item.)
- Should the windowed-read command (R1-1) be designed together with
  R1-7 as one v1.x additive pair, or deferred until the A1 split
  makes the UI consumer real? (Inferred lean: together — they share
  the snapshot-isolation question.)
- Does `total`'s absence on filtered pages matter to any planned
  consumer? Per-tab counts already cover the common case (R1-1);
  a match-count for arbitrary future filters would reopen the
  scan-cost question the spec deliberately avoided.
