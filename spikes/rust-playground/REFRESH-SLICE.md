# The refresh slice — closed record

The refresh slice (policy → plan → apply → replan, then characters in
the plan, then legible output) was built and run live between
2026-08-31 and 2026-09-02 and is closed. This file is its permanent
short record in the mold of `NETWORK-CLEANUP.md`: what landed, in which
commits, what the reviews found, and what the runs taught. The
narratives that used to sit in `CONTEXT.md` live in git: the file at
`d660d1f5` holds the full text of all three sections (the tracer's
review rounds for steps 3–6, pruned earlier, are at `35fb35d9`).

The rulings survive in `CONTEXT.md` (Decisions, and the three
"Interfaces" subsections); the properties are pinned by the tests named
below; live runs are rows in `LIVE-TESTING.md`'s run ledger; facts about
GGG are ground-truth claims N38–N45. Nothing here is a second authority.

## Final state

- Tip at closure: `d660d1f5` (legibility run recorded). Facts schema
  **v7**, sync policy **v3**, plan schema **6**; 247 tests, gate green.
- Reading a store change before reviewing one: the findings table below
  is the checklist — the same class of gap was found five times in one
  day.

## Step ledger

| Slice | Step | Commits | What landed |
| --- | --- | --- | --- |
| Tracer | 1 semantics | `8363f8b8` | the ruled packet (brainstorming-notes 06) harvested into `CONTEXT.md` |
| Tracer | 2 identity + intent | `04b06534`, `34b5549c` | annotation file under tombstoned CAS, uuid required at login, store lint ratchet |
| Tracer | 3 snapshots | `c8a251d4`..`4d785ce3` | neutral `stash_snapshot` (now `refresh_snapshot`); malformed 2xx bodies fail the job |
| Tracer | 4 planner | `dfbe58b4`..`8021fb08` | `acquisition-plan`, `RefreshPlan` compiled offline, strict re-serialize parse |
| Tracer | 5 quote | `c93d46db`..`a6bfb92d` | `quote` on the protocol, `with_quote` enrichment, plan schema 3 |
| Tracer | 6 CLI | `efa0795b`, `35fb35d9` | `acq refresh --plan`, `acq policy show\|set`, `tests/plan_json.rs` |
| Tracer | 7 apply | `5c5a18fc`, `c6a7df17` | the `apply` parent kind, staleness gate, `tests/apply_loop.rs` |
| Tracer | 8 MCP | `e5f9107b`, `397b2263` | plan-slice tools; shared semantics factored into `acquisition-plan`; `tests/plan_loop.rs`, `tests/ggg_refusal.rs` |
| Tracer | 9 live rung | `b56fff53`..`7cc77a25`, run `d774a4c3` | `tools/tracer-rung.sh` + `tools/tracer-verify.py`; five review rounds; pass 2026-09-01 |
| Tracer | rulings | `c6ca68fb`, `3d685c6d`, `d91704c8`, `fdc1462f` | binding confirmed; a policy id covers the tab and its children (plan schema 4); MCP spends in either mode; rerun pass; window warning |
| Characters | design | `953be323`, `1a1e04a6` | the 2026-09-02 rulings (id identity, realm above league, container, skips) |
| Characters | (1) realm | `959e7fea`, `a0b2561b`, `f819f006`, review `c9a07254` | realm on the wire (pc by omission), facts v3, policy v2, plan schema 5, `tests/realm_wire.rs` |
| Characters | (2) key | `93801bbb`, `e188c86f`, reviews `10debd6c`..`ba5909bb` | characters keyed by id (facts v4), container, drift tripwire; withholding (v5), substash liveness (v6) |
| Characters | (3) plan | `263b5ada`, `13fead8d` | policy v3 per facet, plan schema 6, apply vocabulary, driver `--characters`, `tests/characters_wire.rs` |
| Characters | (4) live pc | `6caa07bf` | 112-request cycle, pass |
| Characters | (5) PoE2 | `b9e236eb`, `c27c0b0f`, `21f1c515`, `c4c798ba`, `b37d3f93` | first contact; refused bodies kept (facts v7); granted-skill ruling; N41–N45 |
| Legibility | ruling + build | `695c1ec1` | grouped plan text, `--expand`, `--plan=FILE`, apply/result reports, store footers, brief driver summary |
| Legibility | live | `3d1c8936`, `d660d1f5` | read at the terminal 2026-09-02: approved; density open |

## Findings

One row per review round; the finding, then the property or test that
holds it now. Rounds before step 7 are counted, not itemised — their
text is at `35fb35d9`.

| Round | Commit | Findings | Held by |
| --- | --- | --- | --- |
| step 3 review | `9e5f76d4`, `76064c7e` | six snapshot issues; malformed-body and pairing gaps | store snapshot tests; `Outcome::Failure` on a refused body |
| step 4 review | `e8420a1b`, `8021fb08` | six planner issues; nested strictness, prerequisites | planner parse tests (unknown fields at any depth) |
| step 5 review | `03a3a110`, `e4ff89a6`, `a6bfb92d` | seven quote gaps; verifiable work basis, seeded ETA; `Quote::work` was a schema event | `with_quote` validation; schema stamp 3 |
| step 7 review | `c6a7df17` | the no-op apply must say so honestly; exactness pins | `tests/plan_json.rs` (empty plan spends nothing) |
| step 8 review | `397b2263` | real mode must never spawn; offline claims were not proven offline | `tests/ggg_refusal.rs`; `tests/plan_loop.rs` (socket checked dead) |
| rung round 1 | `b56fff53` | the ceiling reached after the last planned send is the expected end, not a failure; the envelope applied must be the quoted file; probe hits bounded by the run's own sends; readback failures fail the run | driver checks; the "policy id covers children" question surfaced here |
| rung round 2 | `78df033c` | same-plan check must ignore drifting `age_seconds`; probe hits bounded per window plus bucket; no-hold bound is 15; `ceiling + 1` fails the cycle; selector case-insensitive | verifier |
| rung round 3 | `1b2efaf4` | `all` needs a day-long window and a cycle at most half of it; bucket by window position; verifier extracted with `--self-test` | driver refusal; `tools/tracer-verify.py --self-test` |
| rung round 4 | `5e4c0d41` | a matching journal count is not evidence the rail was there — the daemon must report armed, ceiling, count, halt; verifier accepts 2xx only, probe-first per account route; self-contained bundle with `verify.sh`; preflight refuses working-tree changes | driver + verifier |
| rung round 5 | `7cc77a25` | daemon log sliced like the journal; the bundle carries its own verifier with checksums; readback limits explicit | bundle layout |
| realm review (external) | `c9a07254` | the handshake compared the fixed package version (a pre-realm daemon served a console job); migrated items had a null realm; the v2 policy parser lost top-level strictness as an untagged enum; the version story was implicit | build-stamp handshake (Decisions); `NOT NULL DEFAULT 'pc'`; per-stamp strict parse; `tests/realm_wire.rs` |
| characters round 2 | `10debd6c` | a dropped character or tab left its items live; a late fetch rolled `name`/`class`/`level` back; removal keyed on a timestamp missed two fetches in one second | retired locations take their items; listing-owned address; `items.seen_response` |
| characters round 3 | `6923401f` | a fetch revived a retired location; the same tab id under two realms collapsed to one place; a parent's fetch did not retire vanished substashes; removal ids from a timestamp match | a fetch never revives; `Location` is the full coordinate; parent fetch is the substashes' listing; `RETURNING` |
| characters round 4 | `cae6e527` | a withheld fetch still touched the row; a drop-then-revive left a live empty location the planner called fresh; substash liveness ignored the parent; a withheld body lost its arrays | facts v5; revival clears `fetched_at` (planner: a revived tab is planned as never fetched) |
| characters round 5 | `ba5909bb` | a retired parent's substashes kept their freshness; `withheld` could not mark an empty location; identity was checked after the withhold decision | facts v6; `withheld` nullable exact count; validate before deciding |
| step (3) build | `263b5ada` | serde ignores extra fields beside a unit variant's `kind`; `tabs: []` compiled to an empty plan | the envelope must re-serialize to exactly what was read; "names no work" after normalization |
| PoE2 first contact | `b9e236eb`, `21f1c515` | a refusal that destroys its evidence turns every malformed body into a re-fetch; item-granted skills are id-less by design | `refused` table (facts v7, Decisions); granted-skill subtree is a property of its host |
| legibility | `695c1ec1` | the only failure text was an unsorted id list, the cause only in the daemon log; the plan was a wall of explicit lines | rule 4 (failure lines name job, target, cause, evidence); grouping with `--expand` |

## What the runs taught

Properties, each now a line in `CONTEXT.md` or a claim in ground truth:

- Binding plans produced no owner friction; subset-only reconciliation
  and two-cycle substash discovery cost nothing observable. The parked
  dynamic fan-out trigger did not fire.
- A policy id covers the tab and its children — the one coverage
  question the first run surfaced, ruled and built the same day.
- The owner-truth channel is conversation: two runs offered prompts,
  zero notes were typed, both verdicts arrived to the agent and were
  recorded verbatim.
- Authority is the coarser observation's: a listing owns membership,
  address, league and liveness; a fetch owns contents; a fetch never
  revives what a listing retired. Five rounds found this shape in five
  places before it was stated once.
- A refusal keeps what it refused (`refused`, `withheld`, the journal
  line for a failed HEAD).
- Two policies on one account pace independently; a hold ends at the
  window's expiry, not at a constant (N45).
- PoE2: poe2-suffixed policy names with pc's windows, free HEAD,
  `Character.realm` `poe2`, per-realm lists, item `realm` on PoE2 only,
  id-less item-granted skills (N41–N44).
- Legible output: the 112-request plan rendered in 14 lines and was
  approved. C53 now defines the density model — default text is the
  decision view, `--expand` the audit view, JSON the contract — for the
  next live reading to validate.

## Observations still open

Agent observations from the runs that became neither a ruling nor a
finding; each is data for the next slice that touches it.

- The quote is structurally uninformative on a fresh daemon (every cycle
  of the rung runs on one): "no ETA until the policy is learned" for
  every scope. Whether it is useful in a long-lived daemon is untested.
- Facts fresh at one plan's compile can be stale by the next plan's when
  their age plus the cycle's duration passes the window (51 min + 13 min
  against 3600 s bought a 6-request cycle 2). The driver warns; the
  product question is whether a window is the right handle for "keep
  these fresh" when cycles are long.
- GGG names a map substash by tier and a unique substash not at all, and
  the parent's "(Remove-only)" suffix rides along into plan text.
- Stripped characters (empty arrays, `_split` 0/0/0) are a real shape; a
  reader of `acq store characters` sees `0` items either way.
- The first probe of a lifetime queues behind the token POST (4.5 s once,
  ~190 ms otherwise). Noted, not investigated.
- The parent-failure report was never exercised live; it stands on its
  unit tests and the mock rehearsal.
- All-zero change lines: 112 refetched bodies, `changed:` nothing,
  `0 events`. Correct; whether a row of zeros reads as reassurance or as
  "did it work" is for the next reader.
- The driver stops with the daemon up after a failed apply so the jobs
  stay readable; a scripted caller would want the stop automated once
  the evidence is copied.
- The debug binary is unsigned, so macOS Keychain prompts twice per
  login after every rebuild.

## The method-test verdict, as recorded in CONTEXT

**Method-test verdict (2026-09-01):** *pass on correctness, with the
owner-truth channel under-exercised.* The slice ran live first time with
no code change; every rail and check fired as designed, and the five
review rounds before the run did the catching rather than the run. But
the method's truth is the owner's experience, and the owner's one
friction note plus the remark that the driver's output was too dense to
read means the run was judged through the agent's reading, not the
owner's. Not a failure of pin-after-the-consumer — a caveat it carries
into the pricing session: budget for legibility (the parked output items
below) before that session's own live run, or say explicitly that agent
observations stand in for owner notes. Discharged 2026-09-02: the
legible output was built and read live by the owner ("Legible output for
the refresh slice" below).

## Process used

Build / owner review / fix, one step at a time, with rulings written in
`CONTEXT.md` before code; five same-day review rounds on the store
found real defects each time and were worth their cost. What was not
worth its cost: recording each round as a paragraph in `CONTEXT.md` —
the file grew from 52 KB to 110 KB over these commits. The routing rule
in `AGENTS.md` (a finding is a row here, a ruling is a line there, the
narrative is the commit) is this record's lesson.
