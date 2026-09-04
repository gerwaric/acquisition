# The pricing slice — record (open)

The pricing slice opened on 2026-09-03 from the packet
(`brainstorming-notes/10-pricing-design-packet.md` §2: ten steps,
offline, validated by real use) and was **re-scoped on 2026-09-04** to
the render-first minimum in "Plan" below, after the forum-shop reading
(now T1–T15, `docs/design/trade-ground-truth.md`) and the owner's framing: pricing is niche,
so build the least that lets a price reach a forum thread, and keep the
three things that will change — currencies, parsing quirks, precedence
— cheap to change. This file is the slice's record in the mold of
`REFRESH-SLICE.md`: the plan, the step ledger, the findings table, what
the census and the runs taught, and what is still open. It closes at
the plan's done criterion.

The rulings are `decisions/pricing.md` (C64–C75, C78), `decisions/plans.md`
(C76, C77) and `CONTEXT.md` (C79); properties are pinned by the tests
named below; facts about the trade site and the forum become `T<n>`
claims authored master-side. Nothing here is a second authority.

## State

- Opened at step 2 on 2026-09-03; re-scoped 2026-09-04. Annotations
  schema **v2**, facts **v7**, sync policy **v3**, plan schema **6**; no
  pricing code yet.
- The forum is **write-only from our side**: `/character` returns no
  `forum_note` for a forum-listed item (price-notes run, 2026-09-04), so
  the game side of a listing is item note, then tab name (C69), and a
  forum listing is intent we hold, never a fact we observe.
- Reading a store or intent change before reviewing one: `REFRESH-SLICE.md`'s
  findings table is the checklist, plus the rows below.
- Read-only tools: `tools/census.py` (the 0.18 userstore and the facts;
  refuses an uncheckpointed WAL) and `tools/notes-check.py` (the price
  notes a refresh landed; reads through the WAL).

## Plan (re-scoped 2026-09-04)

Steps 1 and 2 of the packet are done (ledger below). What replaces its
steps 3–10, in order; each gate-green; observable behavior unchanged
until step 5. Parked in place with triggers: `PricePlan`/receipts/revert
(C71, C78), the import (C73), the forum-matrix instrument (the owner's
hand experiments replaced it), MCP adapters (after the CLI has been used
for real).

1. **The currency table v1** (C68): the 39 words the game writes
   (`crates/acquisition-plan/reference/price-notes-2026-09-04.txt`) as
   `emit`, the legacy C++ tags as aliases where they differ, the three
   retired tags kept and marked, `game:` evidence per row; display names
   from the owner's list in the fixture (2026-09-04); shipped in the
   binary with version and enumeration.
2. **Annotations v3 and the typed value** (C65, C66, C67): `written_via`
   and `actor` by the stepwise `ALTER`; the per-kind strict trait with
   `SyncPolicy` moved onto it unchanged; `PriceTarget` with the
   realm-bearing tab and substash keys defined before the first row;
   `Buyout` v1 (`exact`, `negotiable`, `no_price`, `ignore`; two-place
   decimal or lot pair; a tag that resolves); a `clear` then `set` on one
   target works through the tombstone; a kind filter on `list`; a
   busy-timeout error kind. The 2b constraints (5)–(8) and (3) are pinned
   here.
3. **The game-side parser** (C69) as one pure function over a note or a
   tab name — exact or negotiable price, `skip`, `invalid`, none —
   tolerant of the trailing text the game appends to tab names, tested
   against the fixture file; every later misread report is one more
   fixture line.
4. **The listing state** (C69, C70): one function over items, tabs,
   characters, `metadata.public` and the buyout rows — manual side, game
   side, relation, causes, basis, the raw note beside the parse; `acq
   price status | show | list` under C53's views.
5. **Set, clear, list** (`acq price set|clear <target> …`): single-row
   compare-and-swap writes returning the prior value; the owner sets the
   census's two manual rows by hand — validation reading 1.
6. **`shop render`** (C74, C72's report): per-item link codes with
   `realm=`, the price on the line after each link, grouped by price,
   pages labelled *n* of *N* under a size parameter (default 50,000),
   template from a file (default `[items]`), the policy table with its
   opening rows and everything else blocked and counted, the coverage,
   staleness and moved-since-basis lines — then **validation reading 2**:
   the owner pastes a page for their own shop and reads it against the
   trade site; whether the report lines change what they do next is the
   evidence.

**Done criterion:** the currency table committed; the buyout value
strict and pinned by id; the parser passing the fixture; the listing
state legible with the raw note beside the parse; the owner's two rows
set by hand; a page rendered, pasted and read correct on the trade site
in the owner's words, with every omission counted; the `T<n>` claims
authored master-side and cherry-picked; the always-loaded documents at
budget.

**The trade claims exist:** T1–T15 in `docs/design/trade-ground-truth.md`
(authored master-side, PR #224, cherry-picked here) — the About page's
two listing channels (T1) and bulk-ratio grammar (T2, T3), the site's
realms and leagues (T4), the `forum_note` field (T5) and its absence
from `/character` (T6), the link code the site emits today and its
resolution at post time (T7), a character item listed through the forum
(T8), the in-game vocabulary (T9) and note shapes (T10), the owner's
tab-price and forum-precedence observations (T11, T12), item addressing
(T13), the wiki (T14) and the code (T15); open questions Q1–Q11 there
are this file's "Observations still open". The evidence note
(`brainstorming-notes/12`) is retired. C67 cites T2 and T10, C69
cites T10 and T11, C74 cites T7, T11, T12 and T13 — the owner trimmed
each entry on 2026-09-04 to make room under the 800-byte limit; the
claims file's appendix maps T→C as well.

## Step ledger

| Slice | Step | Commits | What landed |
| --- | --- | --- | --- |
| Pricing | 1 harvest | `ecdb9d3d`, `889c941d`, `0e4a4d9f` | the accepted packet into the registry (C64–C79); parks routed to `decisions/<area>.md` |
| Pricing | 2a clause audit | `8aa7a507` | C35–C44 and C52 cited from the tests that hold them; C44's fact-drift half pinned; the dependency direction (C34, C39, C41) a `docs-check` check |
| Pricing | 2b annotations review | `3e234fa5` | FULL sync and an atomic, checked export on the intent file; eight constraints recorded for the intent step |
| Pricing | 2c census | `cdfd02cc` | `tools/census.py` (read-only, immutable open, WAL guard); the census below |
| Pricing | 2d currency source | `51a33751` | `SURFACES.md`, the register under C79 (the trade site: `browser`, no automation); `crates/acquisition-plan/reference/currency-v1.toml`, 19 rows from the C++ list with a citation each |
| Pricing | 2e forum reading, price-notes run, re-scope | `0d9fee64` | the owner's saved trade-site and wiki pages read, Procurement and the C++ shop compared, the API reference checked for `forum_note`; the price-notes run (ledger row 2026-09-04: the test tab and the forum-listed character); the note corpus committed as a fixture; C65, C67, C68, C69, C72, C74 amended, C71, C73, C78 parked in place; the plan above; `SURFACES.md` rows; `tools/notes-check.py` |

## Findings

One row per review round; the finding, then the property or test that
holds it now.

| Round | Commit | Findings | Held by |
| --- | --- | --- | --- |
| 2a clause audit | `8aa7a507` | (1) C41's caller-asserted freshness read ("fails with the exact `RefreshPlan` it would take") has no consumer and no code — held by nothing; its first consumer is C72's report at the render step. (2) "The daemon never reads the store" was described as graph-enforced, but the daemon crate links the store crate to *write* facts (C28); what the graph can enforce is daemon ∌ planner and daemon never names the intent API — now a check. (3) C44's "fact drift does not refuse" was stated in the doc and held by the shape of `check_spendable`, pinned by nothing. (4) The uncited ids were the ones whose pins existed under descriptive names; the fix was citation, not tests — 30 tests now name their id. | (1) `decisions/plans.md` C41 *Pinned:* says so; (2) `tools/docs-check.sh` check 4; (3) `c44_fact_drift_never_refuses`; (4) doc comments on the tests, `docs-check`'s uncited report |
| 2b annotations review, fixed now | `3e234fa5` | (1) `synchronous=NORMAL` under WAL keeps the file consistent but lets the last commits before a power loss roll back — on the one file with no server to refetch from. (2) `export` wrote `dest` directly and never fsynced it: an interrupted export left a partial file that looked like a backup and, under never-overwrite, blocked every retry. | (1) FULL; `c35_the_intent_file_is_fully_synchronous`; a single-row put measured 87 µs on this machine. (2) `<dest>.partial` → `quick_check` → fsync → link; an existing `dest` refuses before anything is written; `export_is_a_consistent_snapshot_and_never_overwrites` |
| 2b annotations review, constraints on the intent step | — | (3) Tombstone generations are invisible: `get`/`list` hide a tombstoned row and `Conflict { current: None }` hides its revision, and `put(None)` conflates never-existed (revision 0) with tombstoned. (4) One IMMEDIATE transaction per `put` and no batch: all-or-none for a plan of hundreds is impossible through the API (10k single puts in 449 ms under NORMAL — atomicity, not cost). (5) The migration shape `0 \| 1 => CREATE TABLE IF NOT EXISTS` cannot add a column; v3 needs stepwise `ALTER TABLE … ADD COLUMN … NOT NULL DEFAULT 'unknown_legacy'` inside the same IMMEDIATE transaction; a v3 file is already refused by a v2 build. (6) The `tab` key is realm-less and a substash is the caller's `parent/id` convention; C67's targets carry realm and the owner's file holds zero tab-scoped rows — the realm-bearing key must be defined before the first row lands (P3). (7) `list` filters by scope only; pricing reads want one `kind` across four scopes; at 10k rows `list` takes 35 ms, `get` 27 µs, export 7 ms — a kind filter is needed, an index is not. (8) A writer blocked past the 5 s busy timeout surfaces as `Db(…)`, indistinguishable by kind from any other SQLite error, though a driver must tell "retry later" from "re-read and retry". (9) Receipt growth: an import of this owner's file is ~1.4k mutations, ~300 KB per receipt at ~200 B each; a `receipts` + `receipt_mutations` layout indexed by target keeps "since *T*" a query. (10) `open_for` creates the file and stamps the uuid on read paths — a read that writes; harmless, noted. **Re-priced 2026-09-04:** (5), (6), (7), (8) and a reduced (3) — a `clear` then `set` on one target must work through the tombstone — are paid by plan step 2; (4) and (9) are parked with C71/C78. | step 2 of the plan builds against these rows; each becomes a pin there |
| 2c census | `cdfd02cc` | (1) A `file:` URI truncates at `#`, so the owner's `userstore-GERWARIC#7694.db` opened as an empty file; a stray zero-byte `userstore-GERWARIC` beside it, dated 2026-08-13, is the same trap met by someone else. (2) A `mode=ro` open of a WAL file created `-shm`/`-wal` in the owner's data directory. | `tools/census.py`: percent-encoded `immutable=1` open, refusal of an uncheckpointed WAL; the side files removed |
| 2d currency source | `51a33751` | No official GGG data export for the currency vocabulary is known, and the trade site's static endpoint is rejected as a tooling source (packet §1(d)); v1's only cited source was the C++ tables plus the census. **Superseded 2026-09-04:** the game's own client is the source for `emit` (the price-notes run) and the owner's reading of the dialog for display names (both in the fixture). | `currency-v1.toml` header; `SURFACES.md`; plan step 1 |
| 2e forum reading and price-notes run | `0d9fee64` | (1) C67's mechanism reduced `a/b` and compared rationals, but the trade site's ratio is `wanted/lot` with the denominator a lot size (`2/35` is not `1/17.5`; `3/1` is not `3`) — the amount is a pair or a two-place decimal, ruled. (2) The game writes `exalted`, `chrome`, `jewellers`, `fusing` where the C++ table emitted `exa`, `chrom`, `jew`, `fuse`; `chisel`, `coin`, `silver` are no longer in the game; 23 words are missing — the table's `emit` column was folklore. (3) The game side has four outcomes, not two: the dialog writes `~skip ` for "Do not index" and leaves `~price  chaos` (empty amount) after an invalid entry; a ratio in a tab name unlists the whole tab (owner, in game). (4) `/character` carries no `forum_note` for a forum-listed item: the forum is write-only from our side. (5) Every stash item's `inventoryId` is the literal `Stash1`; a link code's `Stash<n>` must come from the tab's index; socketed items have no position. (6) The forum resolves a `linkItem` into `[item post= index=]` at post time and the site emits `realm=` today — the C++ code had it, the wiki and Procurement do not. (7) A daemon exit leaves the facts file's WAL uncheckpointed (1.1 MB after this run), so `census.py`'s guard refuses the file; whether the stop path skips the final checkpoint is a store/daemon look before the import trigger ever fires. (8) The journal's default path is `<socket dir>/acquisition-playground.ggg.sends.jsonl`, not `<socket>.ggg…`; the owner copied the daemon log by mistake. | (1) C67; (2) C68, the fixture file, plan step 1; (3) C69; (4) `PRICING-SLICE.md` State, C69; (5), (6) C74's policy rows, the render step; (7) open — a finding for the store; (8) `LIVE-TESTING.md` ledger row, the live-run skill's wording |

## What the census taught (step 2c, 2026-09-03)

A read-only census of the owner's real data, by `tools/census.py`: the
0.18 userstore (`~/Library/Application Support/Acquisition/data/userstore-GERWARIC#7694.db`,
user_version 3, WAL, 57 MB, last written 2026-08-14 by the C++ app) and
the spike's facts and intent files for the same account
(`store/ggg/GERWARIC_7694.db`, facts v7; `cac319d8-….annotations.db`,
v2). Two traps the tool now guards: the `#` in the C++ filename is a URI
fragment (an un-encoded open reads an empty file), and a plain `mode=ro`
open of a WAL file creates `-shm`/`-wal` beside the owner's database
(removed by hand; the tool opens `immutable=1` and refuses an
uncheckpointed WAL). Counts and shapes only; no item id is recorded.

**The 0.18 userstore.**

| Table | Rows |
| --- | --- |
| `item_buyouts` | 1355 |
| `location_buyouts` | 18 |
| `stashes` | 1294 (Standard 1080, of which 978 remove-only; Hardcore 125; Allflame 51; Mirage 38) |
| `characters` | 58 |

| `item_buyouts` by type / source / inherited / location | Rows | Reading |
| --- | --- | --- |
| `price` / `game` / inherited / stash | 1231 | a tab-name price materialized onto each item — C64's rejected shape, as data |
| `price` / `game` / own / stash | 117 | an item note |
| `[ignore]` / `manual` / inherited / stash | 6 | the items under the one manually ignored tab |
| `b/o` / `manual` / own / character | 1 | the single manual item price: 2222 `jew` on a Standard character's item |

`location_buyouts`: 17 `price`/`game` (tab names) and 1 `[ignore]`/`manual`
— which carries amount 4321 and currency `blessed` although ignore has
neither. So the manual desired state is **two rows** (one character
item at 2222 `jew`, one tab ignored), set by hand at plan step 5; the
other 1353 item rows and 17 location rows are derivations the facts
will reproduce once the sale tabs are fetched. `~c/o`: 0 rows.
Currencies on rows: `chaos` 1206, `divine` 105, `exa` 37, `blessed` 6,
`jew` 1. Every amount is integral (1…4321, 18 distinct values); none
fractional, none ≤ 0. `last_update` is ISO-8601 text with milliseconds
and no zone in a column declared INTEGER. Item ids are 64 hex; stash
location ids 10 hex; the character row's location id is the 64-hex
character id. Every stash location id resolves in `stashes`.

The note corpus: 26,286 items sit in the stash bodies; 121 carry a
note, 35 distinct, 120 tilde-prefixed (the one other note is `50`).
Under the C++ regex (`(~\S+)\s+(\d+\.?\d*)\s+(\w+)`, searched, not
matched): `~price N divine` 44, `~price N exalted` 37, `~price N chaos`
36, and **three unparsed**: `~price 22/10 chaos`, `~price 55/600 chaos`,
`~price 10/80 chaos` — lot ratios (22 chaos per 10, and so on), kept
as pairs under C67. Amount texts are all integers 1…150. `~b/o` and
`~c/o` appear in no note; the word is `exalted`, never `exa` — the
game's own spelling, as the price-notes run confirmed. The 120 notes
sit in 17 tabs, 14 of them remove-only ("Lab Enchants #2 (Remove-only)"
holds 31). Tilde tab names: 17 (Standard 13, Hardcore 3, Allflame 1),
all `~price N chaos|divine`, all parse; 12 carry a "(Remove-only)"
suffix, 8 a letter "(A)"…"(G)", and "~price 30 chaos (C)" exists once
in Standard and once in Allflame under different ids.

**The current facts** (spike, 2026-09-03). 402 tabs listed in pc /
Standard, 69 fetched, 0 removed: 16 folders, 61 `MapStash` (46
substashes), 46 `UniqueStash` (36 substashes), 163 `PremiumStash`. `idx`
is present on all 402. `metadata.public` is present on 12 tabs and true
on all 12 — absent means not public; **none of the 12 is a priced tab,
and none of the 13 priced tabs is public**, so every in-game tab price
the owner has is invisible to the trade site and a forum post is its
only channel. Metadata keys seen: `colour` 338, `layout` 3, `public`
12, `map` 57, `folder` 16, `items` 64. The 13 Standard tilde tab names
are in the facts and parse; the Hardcore and Allflame ones are not
(only Standard is listed); none of the 13 is fetched. Items: 816 stash
rows (the 69 fetched tabs) and 1244 character rows; `x`/`y` are null
exactly for socketed items (24 stash, 531 character); every stash
item's `inventoryId` is `Stash1`; containers: stash `items` 816;
character `equipment` 870, `inventory` 126, `jewels` 123, `skills` 42,
`guardian` 18, plus 65 pre-v4 rows with no container. Characters: 65
rows over 9 (realm, league) pairs, 47 fetched (poe2 included). The
intent file holds one row (the sync policy, revision 9) and no tab or
item rows. The account index maps `GERWARIC#7694` to `cac319d8-…`; a
second account, `_vagabond#6960`, has its own facts file.

**What it means for the rulings** (as amended 2026-09-04).

- C67: the lot-ratio form exists in the owner's own notes (3 rows) and
  is represented as an unreduced pair; `current_offer` has zero rows,
  the park stands; source `[ignore]` rows carry a non-semantic amount
  and currency, which the value refuses.
- C68: tags in use are `chaos`, `divine`, `exa`, `blessed`, `jew`; the
  0.18 rows map 1:1 onto v1 tags; the game's spellings are the
  fixture's.
- C69: the C++ code (`ItemsManager::ApplyAutoItemBuyouts`) let a note
  that stopped parsing *clear* a game-set item price so the tab
  applied; C69 instead reports `invalid` and does not substitute the
  tab.
- C74: "(Remove-only)" and letter suffixes ride along in priced tab
  names, so the parser tolerates trailing text as the C++ regex did;
  positions are absent for socketed items; a real render needs the 13
  priced Standard tabs in coverage, and Hardcore and Allflame listed
  for their four — C72's report will say so.

## What the forum reading and the price-notes run taught (2026-09-03/04)

The evidence is T1–T15 in `docs/design/trade-ground-truth.md`
(quoted with sources, each tagged and dated); the note corpus is
`crates/acquisition-plan/reference/price-notes-2026-09-04.txt`; the run
is the 2026-09-04 ledger row. What it changed is in the registry
(C65, C67, C68, C69, C72, C74 amended; C71, C73, C78 parked in place)
and in the plan above. The owner's in-game observations, verbatim, are
T11 and T12 (Provisional) and will be cited by the render's policy rows
until a matrix cell upgrades each.

## Observations still open

Agent observations that became neither a ruling nor a finding; each is
data for the step that touches it. The hand experiments are the
owner's, one each, when the render step needs them; the forum preview
shows the item picture, so a wrong link is visible before posting.

- Which item `[linkItem location="Stash<n>" …]` resolves to when
  folders occupy tab indices — is `n` the tab's `index + 1` counting
  folders? Decides whether the first pasted page is trustworthy.
- Item note against tab name in game: rename the test tab to a price
  and see whether the noted items keep their own price on the site.
  C69's note-then-tab order rests on the C++ code until then.
- The link code for an item in a substash (map and unique tabs); the
  post size limit (50,000 is the C++ constant); whether an unpriced
  (`no_price`) forum link is indexed; what the indexer does with a
  ratio on a non-bulk item, and with `~b/o a/b`; a game `skip` against
  a manual price. All blocked-and-counted in the render's policy table
  until observed.
- The trade site's seller-account search, run in a browser for this
  account and league, is the oracle for the listing state as a whole:
  it shows what is listed, and so whether the remove-only priced tabs
  are as invisible as `metadata.public` says.
- The game-side parser's real corpus is the test tab (fixture) plus the
  userstore's 120 notes; the facts hold the sale tabs listed, not
  fetched, until the policy covers them.
- The C++ userstore stored `last_update` as text in an INTEGER column
  (Qt bound a `QDateTime`); nothing to fix, noted.
- `LIVE-TESTING.md` is over 90% of its budget; route at session close.

## Questions for the owner

1. Are the two manual rows (a character item at 2222 `jew`; one tab
   ignored) real intent to set by hand at step 5, or test residue to
   drop?
2. C73 parked as "a 0.18 user asks": pricing is niche, but the 0.18
   import is a product question for other users, not only yours. Park
   stands unless you say otherwise.
