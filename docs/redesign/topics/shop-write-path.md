# Shop/forum write path

Status: investigation result, unreviewed
Provenance: traced 2026-08-08 (Claude, Tom's session) against the
working tree on the `redesign` branch, answering the "Shop/forum write
path … untraced" open question in `topics/migration-order.md`. All
`file:line` cites are to that tree; the shop code is identical to
`master` at v0.18.0. No runtime or live-network verification was done
(see Verification notes).

## The answer in one paragraph

The write side is a **small addendum to A0, not a second protocol**.
`Shop` consumes only the published items snapshot and item-level
buyouts — it has **zero coupling to `Search`, `ItemsModel`, or any
view state** — and since items-pipeline M2 D8 it captures its entire
input **by value at request time**, so shop-text generation is already
a pure projection over (captured items × buyouts × template ×
realm/league × legacy tab index). The transport tail (CSRF scraping,
HTML-scraped forum rate limits, POESESSID rejection handling) is
effectful machinery that never crosses the UI boundary and stays
core-internal under any A0 split. The smallest command surface is
enumerated under "A0 verdict" below.

## 1. Data dependencies

What `Shop` reads, exhaustively (traced from `Shop::CaptureJob` and
the constructor):

- **Items**: `ItemsManager::items()` — the published flat snapshot
  (`itemsmanager.h:47`), iterated once at capture time
  (`shop.cpp:268`). Never the `Search`/`ItemsModel` layer: `shop.cpp`
  includes no search or model header, and no other item source exists
  in the file. The migration-order question "coupling to `Search`"
  has a clean answer: **there is none**.
- **Buyouts**: `BuyoutManager::Get(item)` only (`shop.cpp:269`),
  filtered by `Buyout::IsPostable()`. Tab buyouts are never read
  directly — they reach shop text indirectly, materialized as
  inherited item buyouts by the pricing passes (see §4).
- **Settings**: `realm`, `league`, `account`, `session_id`
  (`shop.cpp:263-264`, `shop.cpp:281`, `shop.cpp:237`);
  `shop_autoupdate` (`shop.cpp:105`).
- **Datastore key-value rows**: `shop` (thread list),
  `shop_template`, `shop_hash` (`shop.cpp:104-109`, `shop.cpp:403`,
  `shop.cpp:550`).
- **Legacy tab index**: per-job network fetch (§2), the one input not
  derivable from (items, buyouts). Forum `[linkItem]` codes address
  stash items by tab **ordinal**, which the OAuth API does not serve
  in that form; the legacy list's ordinal `i` is keyed by the first
  10 characters of the legacy tab id (`shop.cpp:327`) and looked up
  by `ItemLocation::id()` (`shop.cpp:463`), which is the modern API
  stash id (`itemlocation.cpp:31`). Inferred: the modern id equals
  the legacy id's 10-char prefix — consistent with the code and the
  test fixture (`tst_shop.cpp:131-133`), never confirmed against a
  live account from this session.

**Purity of shop-text generation: already achieved.** Every
submission renders from an immutable value capture (`ShopJob`,
`shop.h:93-115`; M2 D8/R2-1): item id, pretty name, `ItemLocation`,
and `Buyout` are copied per postable item, so a later refresh
rebasing the shared `Item` objects cannot mutate a job under
submission. `RenderJob` (`shop.cpp:428-495`) is a pure function of
the capture plus the job-local tab index: sort by buyout, group into
`[spoiler]` blocks, paginate at 50,000 characters, substitute into
the template, md5 the result. Its only Qt is string-level
(`QString`, `QRegularExpression`) plus `ItemLocation::GetForumCode`
(`itemlocation.cpp:115-133`, needs type/x/y/inventory-id/character)
and `Buyout` formatting. A non-Qt core could express this as
`render(items, buyouts, template, realm, league, tab_index) →
(pages[], hash)` with no design change — the C++ already treats it
that way.

## 2. Write surface

One submission job issues three kinds of requests, in sequence:

1. **Legacy stash index** — GET
   `www.pathofexile.com/character-window/get-stash-items`
   (`tabs=1&tabIndex=0`), built by
   `PoeApiClient::getLegacyStashIndex` (`poeapiclient.cpp:129-158`)
   and submitted through `RateLimiter::SubmitFuture` — i.e. **through
   the hub and the serialized gate**, as one of the five rate-limited
   endpoints (`network-redesign.md` D5). Header-driven rate-limit
   protocol per N21; the observed policy is `backend-item-request-limit`
   (F65 ledger entry). Carries the 10 s transfer timeout the gate's
   liveness depends on (F60). Auth: POESESSID cookie.
2. **CSRF/title fetch** — GET `/forum/edit-thread/<id>`
   (`shop.cpp:537-541`), one per configured thread, via
   `NetworkManager` directly — **deliberately outside the hub and
   gate** (`network-redesign.md` D5 scope rationale; N22). 5-minute
   transfer timeout (`replytimeout.h:6` — "pathofexile.com can be
   very slow"). The CSRF `hash` and the thread title are scraped out
   of the HTML (`shop.cpp:587-611`).
3. **Edit POST** — form-urlencoded
   `title/content/notify_owner/hash/submit` to the same URL
   (`shop.cpp:630-644`), also direct and outside the gate. The forum
   regime has **no rate-limit headers** (N22): limits arrive as HTML
   ("You must wait N seconds", `shop.cpp:84`) and are honored by a
   scrape-and-resubmit timer (`shop.cpp:715-743`). A fixed 500 ms
   delay separates the CSRF fetch from the POST (`shop.cpp:613`),
   which retired the "Security token has expired" failures
   (`shop.cpp:707-714`).

Threads are strictly sequential — one edit-page/POST pair at a time,
driven by a completion counter (`shop.cpp:521-559`) — and at most one
job is active with at most one capture waiting (§3). So the forum
adds one slow, serialized request stream on top of gate traffic,
which is exactly the D5 rationale for leaving it ungated.

**Authentication.** Everything on `www.pathofexile.com` authenticates
by the POESESSID cookie, installed domain-wide on `.pathofexile.com`
(`networkmanager.cpp:87-97`). The OAuth bearer is attached only for
`api.pathofexile.com` (`networkmanager.cpp:115-123`) — load-bearing,
not stylistic: a bearer on `www` actively breaks both the legacy and
forum regimes (N21, N22), and N21 already notes nothing pins this
host scoping. OAuth cannot post shops; the UI says so explicitly
(`shop.cpp:184-192`).

**The POESESSID guard (PR #190 hotfix, commit 8b761e2c).** All three
request kinds detect session rejection two ways: HTTP 401/403
(`shop.cpp:37-41`) and — because the forum redirects unauthenticated
requests to error pages whose final status is **200** — body markers
"Login Required" / "Permission Denied" / "Content Denied"
(`shop.cpp:43-47`, checked at `shop.cpp:578-585` and
`shop.cpp:678-682`). `RejectPoeSession` (`shop.cpp:333-344`) drops
the waiting capture, disables auto-update, and emits
`PoeSessionRejected`, which `Application::ClearSessionId` answers by
deleting both the cookie and the settings row (`application.cpp:128`,
`application.cpp:343-347`). Two implications for a future core:
credential rejection is a **de-arming terminal event, not a
retryable failure** (it must stop automation, or a dead cookie gets
hammered into the forum on every refresh), and **status codes alone
are insufficient** — the write path must classify 200-status HTML.

## 3. Trigger and lifecycle

- **Manual**: Shop → "Update shop" menu action →
  `SubmitShopToForum(true)` (`mainwindow.cpp:1705-1708`). Refused
  while a job is active (`shop.cpp:225-228`).
- **Automatic**: the typed terminal event
  `ItemsManager::RefreshFinished` (`application.cpp:127`), gated on a
  **clean** `CompletedRefresh` — a failed or completed-with-skips
  refresh never auto-posts (`shop.cpp:147-173`; M2 D8/R1-1). There is
  no shop-owned timer; auto-post cadence rides the items
  auto-refresh timer.
- **Desired state**: at most one active immutable job plus at most
  one waiting automatic capture, newest-clean-wins
  (`shop.h:160-165`; M2 D8/R3-1). Success — including the
  unchanged-hash no-post — drains the waiting capture; failure
  discards it (`shop.cpp:372-397`).

**Can a shop update run mid-refresh, and is it safe?** Yes, and yes —
by construction, not by luck. A manual submission can start during a
refresh, and refresh N+1 legally starts while a job posts. Safety is
the M2 D8 value capture: streamed deltas cannot reach an active job's
immutable input, pinned offline by
`shopSubmissionUsesCapturedSnapshot` and
`activeJobUnaffectedByLocalEdits` (`tst_shop.cpp`). The index fetch
adds one request to the gate alongside refresh traffic, bounded like
everything else by the cap-2/250 ms gate (D5). The known blind spot
is deliberate and documented: deltas do not advance the preview
cache's input revision (M2 D8/R4-2), so mid-refresh the
clipboard/preview can claim currency while published state has
streamed past it — submissions are unaffected because every job
re-renders its own capture.

Preview-cache expiry inputs: buyout edits (`mainwindow.cpp:574`),
thread and template changes (`shop.cpp:121`, `shop.cpp:144`) — these
also drop the waiting capture — and every published snapshot
(`application.cpp:435`), which keeps the waiting capture
(keep-and-drain, M2 D8/R5-6).

## 4. Buyout coupling

The edit flow, UI → persistence → shop text:

1. `MainWindow::OnBuyoutChange` (`mainwindow.cpp:571-646`) expires
   the shop preview first (`mainwindow.cpp:574`), then runs one
   `BuyoutBatch` over the selection: `SetTab` for top-level rows,
   `Set` for items (`mainwindow.cpp:618`, `mainwindow.cpp:633`),
   then `ItemsManager::PropagateTabBuyouts`
   (`mainwindow.cpp:644`).
2. **Persistence is already command-shaped.** `Set`/`SetTab` write
   through signals — `SetItemBuyout`/`SetLocationBuyout` →
   `BuyoutRepo::saveItemBuyout`/`saveLocationBuyout`
   (`application.cpp:159-160`); clears go inline through
   `m_repo.remove*` under the F52 no-op-delete guard
   (`buyoutmanager.cpp:104-120`, `buyoutmanager.cpp:178-184`).
   `refresh_checked_state` still persists separately via `DataStore`
   JSON (`buyoutmanager.cpp:384-390`) — the F22 split.
3. **Shop text sees buyouts only at item granularity.** Tab buyouts
   reach the forum post because `PropagateTabBuyouts`
   (`itemsmanager.cpp:98-131`) materializes them as inherited item
   buyouts at snapshot boundaries (and `ApplyScopedPricing` does the
   per-delta equivalent, `itemsmanager.cpp:192`). A core-side shop
   projection therefore depends on the **propagation pass's
   inheritance semantics**, not on raw tab buyouts — port the pass,
   not just the maps.

**F22's bearing on the A0 command path: none blocking.** The
buyout-edit commands map onto `BuyoutManager`'s existing surface
(`Set`/`SetTab` plus the M3 batch boundary that already yields
exactly one change-set event per command). F22's second persistence
lane holds refresh-*selection* state, not pricing state — it belongs
to the refresh command family, so buyout edits can go on A0 without
touching it. But if a future core takes custody of buyout
persistence anyway, that is precisely the "other work touches this
storage" trigger F22 names for unification — do it then. F54's
lesson transfers verbatim: any rekeying or migration in a new core
must write through persistence, never memory-only.

## 5. A0 verdict

**Small addendum.** The smallest command surface covering current
shop functionality:

- `set_item_buyout(item_id, buyout | null)` and
  `set_tab_buyout(tab_id, buyout | null)`, with batch semantics (one
  change-set event out per command — M3 D1 rule 4 already defines
  this contract).
- `set_shop_threads(list)`, `set_shop_template(text)`,
  `set_shop_auto_post(bool)`.
- `post_shop(force)` — refused (or queued, matching today's
  manual-vs-automatic split) while a job is active.
- `set_poesessid(secret)` — the one credential-bearing command (§6).

Read-side additions are equally small: the rendered preview pages
plus the outdated flag (snapshot-granular, per R4-2), job
status/progress, and user-warning / session-rejected events.

Everything else stays **inside** the core, invisible to the
protocol: the legacy index fetch, CSRF and title scraping, HTML
rate-limit scraping and resubmit timers, session-rejection
classification, and the shop-hash dedupe. Hidden state, enumerated
(nothing else was found): threads/template/`shop_hash` (datastore),
auto flag and realm/league/account/POESESSID (settings),
input/cache revisions, active job and waiting capture (memory). All
of it is shop-local; none couples to `Search` or the view layer.

One genuine boundary condition: the automatic gate consumes the
refresh terminal outcome (`RefreshOutcome`: clean / skips / failed).
That stays core-to-core **as long as refresh execution and shop
automation live on the same side of the boundary** — which is the
migration-order plan (PR #192's idempotent refresh operations are
core-side). Whether the PR #192 contract exposes an equivalent
terminal outcome is an open question for its review (it exists only
on that branch).

## 6. Custody implications (for the `credential-custody` spike)

- **At rest**: POESESSID is plaintext in `settings.ini`
  (`application.cpp:355`), reloaded into the cookie jar at startup
  (`application.cpp:303-306`). Log output masks it
  (`networkmanager.cpp:89-91`). That is the current guarantee the
  spike must meet; an OS keychain would exceed it.
- **In flight**: a domain cookie on `.pathofexile.com`
  (`networkmanager.cpp:93-96`), sent on the legacy index and every
  forum request. The write path is the **only** POESESSID consumer
  in the application.
- **Host-scoped auth is a hard requirement, currently unpinned**:
  bearer only to `api.`, POESESSID only useful on `www.`; mixing
  them actively breaks the legacy and forum regimes (N21, N22). A
  future core should pin this in its HTTP client, not inherit it as
  a convention.
- **Custody can be confined entirely to the core.** Under the A0
  split both POESESSID consumers are core-side, so the webview never
  needs the secret: it needs a `set_poesessid` command (user pastes
  the cookie in a dialog today, `mainwindow.cpp:1620-1651`) and a
  session-rejected event. The de-arming behavior from 8b761e2c —
  clear the credential, disable automation, notify — is a guarantee
  to preserve, and note it fires on 200-status HTML, not just 4xx.

## Candidate findings (for `docs/cleanup/findings.md`, on master)

Found while tracing; not fixed here per the register's rules.
Registered on master as **F68–F70** (August 8, 2026), in the order
listed below.

1. **F68. `src/poe/endpoints/website/` is dead code with wrong contents.**
   None of its four headers has an includer anywhere in `src/` or
   `tests/` (grep-verified), and two are copy-paste casualties:
   `webleagues.h` defines `AccountCharacters` and `webstashitems.h`
   defines `AccountLeagues`. The live website type is
   `src/poe/types/website/webstashtab.h`. Risk is misdirection — a
   repo survey (including the brief for this investigation) naturally
   lands there looking for the shop's endpoint surface.
2. **F69. `Shop::StashesIndexed` is a dead signal** — declared
   (`shop.h:77`), never emitted, never connected (grep-verified).
3. **F70. Stale user-facing recovery advice** at `shop.cpp:704`: the
   "Failed to find item" handler tells the user to try
   Shop → "Update stash index", a menu item that no longer exists
   (no such action in `mainwindow.ui`/`mainwindow.cpp`). The advice
   is also obsolete on the merits — every job fetches a fresh index
   (`shop.cpp:277-291`), so the recovery is simply resubmitting.

## Dead ends and rejected interpretations

- **`src/poe/endpoints/website/` as the write-path endpoint layer.**
  The brief's entry-point hints pointed there; it is unused (F68). The legacy index request is hand-built in
  `poeapiclient.cpp:129-158`.
- **"Do shop posts go through the rate limiter like API reads?" is a
  false binary.** The answer splits per request kind: the index GET
  does (hub + gate); the forum GET/POST pair deliberately does not.
  Spec history matters here: earlier network-redesign revisions
  *did* gate forum traffic and the IR round reversed that
  (`network-redesign.md` D5) — a rewrite should not re-gate it
  without new evidence.
- **Rejected reading: shop couples to `Search`/view state.** It
  doesn't (§1). The migration-order plan's read-path-heavy shape
  survives contact with the write side.
- **Considered flagging the title scrape as a correctness problem**
  (`shop.cpp:600-611` re-extracts the thread title from a specific
  `<input>` tag and posts it back). It is layout-fragile, but it is
  long-standing, failure is loud (job fails, nothing posted), and it
  is transport detail under any A0 split — noted, not registered.

## Verification notes

- **Traced in code** (working tree, `redesign`, 2026-08-08): all of
  `shop.h`/`shop.cpp`; `buyoutmanager.h`/`.cpp`;
  `PoeApiClient::getLegacyStashIndex` and the facade contract;
  `NetworkManager` auth scoping and cookie handling; the
  `Application` wiring (signal connections, `ClearSessionId`,
  startup POESESSID load); the `MainWindow` shop menu and
  buyout-edit paths; `ItemsManager`'s pricing passes and snapshot
  flow; commit 8b761e2c read as a diff and cross-checked against the
  current tree. The offline test suite (`tst_shop.cpp`, 20 tests)
  pins the D8 machinery end-to-end through a fake limiter and fake
  network.
- **Taken from docs, not independently re-verified**: gate
  properties (D5), the N21/N22 regime claims, and the legacy
  endpoint's policy identity (F65 ledger). No live network requests
  were made from this session.
- **Grep-verified absences**: no includers of `endpoints/website/`;
  no emit or connect of `StashesIndexed`; no "Update stash index"
  action anywhere in the UI; no search/model include in `shop.cpp`.
- **Lane summary**: everything above is traced-in-code unless
  marked; the two inferred claims are the 10-char tab-id prefix
  match (§1) and the 50,000-character forum post limit
  (`shop.cpp:34` — estimated: encoded in the client since the
  original implementation, no ground-truth N-number covers it).

## Open questions

- Does the PR #192 control contract expose a refresh terminal
  outcome equivalent to `RefreshOutcome`? Needed by the auto-post
  gate; only checkable on the PR #192 branch (step-zero review).
- Is the 10-char legacy-to-modern tab-id prefix match guaranteed for
  all realms/leagues, or an observed regularity? (Inferred lane, §1.)
- Is 50,000 characters still the forum's actual post limit, and is
  the multi-thread pagination behavior at that limit still what the
  forum expects? (Estimated lane; would need a deliberate live test.)
