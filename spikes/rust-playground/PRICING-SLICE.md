# The pricing slice — record (open)

The pricing slice (the packet, `brainstorming-notes/10-pricing-design-packet.md`
§2: ten steps, offline, validated by real use) opened on 2026-09-03. This
file is its record in the mold of `REFRESH-SLICE.md`: the step ledger,
the findings table (one row per review round, each finding with what
holds it now), what the census and the readings taught, and the
observations still open. It closes at step 10.

The rulings are `decisions/pricing.md` (C64–C75, C78), `decisions/plans.md`
(C76, C77) and `CONTEXT.md` (C79); properties are pinned by the tests
named below; facts about the trade site and the forum become `T<n>`
claims authored master-side. Nothing here is a second authority.

## State

- Opened at step 2 on 2026-09-03. Annotations schema **v2**, facts **v7**,
  sync policy **v3**, plan schema **6**; no pricing code yet.
- Reading a store or intent change before reviewing one: `REFRESH-SLICE.md`'s
  findings table is the checklist, plus the rows below.

## Step ledger

| Slice | Step | Commits | What landed |
| --- | --- | --- | --- |
| Pricing | 1 harvest | `ecdb9d3d`, `889c941d`, `0e4a4d9f` | the accepted packet into the registry (C64–C79); parks routed to `decisions/<area>.md` |
| Pricing | 2a clause audit | `8aa7a507` | C35–C44 and C52 cited from the tests that hold them; C44's fact-drift half pinned; the dependency direction (C34, C39, C41) a `docs-check` check |
| Pricing | 2b annotations review | `3e234fa5` | FULL sync and an atomic, checked export on the intent file; eight constraints recorded for step 3 |
| Pricing | 2c census | `cdfd02cc` | `tools/census.py` (read-only, immutable open, WAL guard); the census below |
| Pricing | 2d currency source | (this commit) | `SURFACES.md`, the register under C79 (the trade site: `browser`, no automation); `crates/acquisition-plan/reference/currency-v1.toml`, 19 rows from the C++ list with a citation each, alias and display columns marked pending the owner's browser read |

## Findings

One row per review round; the finding, then the property or test that
holds it now.

| Round | Commit | Findings | Held by |
| --- | --- | --- | --- |
| 2a clause audit | `8aa7a507` | (1) C41's caller-asserted freshness read ("fails with the exact `RefreshPlan` it would take") has no consumer and no code — held by nothing; its first consumer is C72's report at step 8. (2) "The daemon never reads the store" was described as graph-enforced, but the daemon crate links the store crate to *write* facts (C28); what the graph can enforce is daemon ∌ planner and daemon never names the intent API — now a check. (3) C44's "fact drift does not refuse" was stated in the doc and held by the shape of `check_spendable`, pinned by nothing. (4) The uncited ids were the ones whose pins existed under descriptive names; the fix was citation, not tests — 30 tests now name their id. | (1) `decisions/plans.md` C41 *Pinned:* says so; (2) `tools/docs-check.sh` check 4; (3) `c44_fact_drift_never_refuses`; (4) doc comments on the tests, `docs-check`'s uncited report |
| 2b annotations review, fixed now | `3e234fa5` | (1) `synchronous=NORMAL` under WAL keeps the file consistent but lets the last commits before a power loss roll back — on the one file with no server to refetch from. (2) `export` wrote `dest` directly and never fsynced it: an interrupted export left a partial file that looked like a backup and, under never-overwrite, blocked every retry. | (1) FULL; `c35_the_intent_file_is_fully_synchronous`; a single-row put measured 87 µs on this machine. (2) `<dest>.partial` → `quick_check` → fsync → link; an existing `dest` refuses before anything is written; `export_is_a_consistent_snapshot_and_never_overwrites` |
| 2b annotations review, constraints on step 3 | — | (3) Tombstone generations are invisible: `get`/`list` hide a tombstoned row and `Conflict { current: None }` hides its revision, so C71's precondition "absent at generation *g*" can be neither stated nor checked, and `put(None)` conflates never-existed (revision 0) with tombstoned. (4) One IMMEDIATE transaction per `put` and no batch: all-or-none for a plan of hundreds is impossible through the API — cost is not the issue (10k single puts in 449 ms under NORMAL), atomicity is. (5) The migration shape `0 \| 1 => CREATE TABLE IF NOT EXISTS` cannot add a column; v3 (`written_via`, `actor`, `applied_plan`, the receipt table) needs stepwise `ALTER TABLE … ADD COLUMN … NOT NULL DEFAULT 'unknown_legacy'` inside the same IMMEDIATE transaction; a v3 file is already refused by a v2 build. (6) The `tab` key is realm-less and a substash is the caller's `parent/id` convention; C67's targets carry realm and the owner's file holds zero tab-scoped rows — the realm-bearing key must be defined before the first row lands (P3). (7) `list` filters by scope only; pricing reads want one `kind` across four scopes; at 10k rows `list` takes 35 ms, `get` 27 µs, export 7 ms — a kind filter is needed, an index is not. (8) A writer blocked past the 5 s busy timeout surfaces as `Db(…)`, indistinguishable by kind from any other SQLite error, though a driver must tell "retry later" from "re-read and retry". (9) Receipt growth: an import of this owner's file is ~1.4k mutations, ~300 KB per receipt at ~200 B each — megabytes a year at human pace; a `receipts` + `receipt_mutations` layout indexed by target keeps "since *T*" a query and is a representation of C78's receipt, not the parked event log. (10) `open_for` creates the file and stamps the uuid on read paths — a read that writes; harmless, noted. | step 3 builds against these rows; each becomes a pin there |
| 2c census | `cdfd02cc` | (1) A `file:` URI truncates at `#`, so the owner's `userstore-GERWARIC#7694.db` opened as an empty file; a stray zero-byte `userstore-GERWARIC` beside it, dated 2026-08-13, is the same trap met by someone else. (2) A `mode=ro` open of a WAL file created `-shm`/`-wal` in the owner's data directory. | `tools/census.py`: percent-encoded `immutable=1` open, refusal of an uncheckpointed WAL; the side files removed |
| 2d currency source | (this commit) | No official GGG data export for the currency vocabulary is known to the agent, and the trade site's static endpoint is rejected as a tooling source (packet §1(d)); so v1's only cited source today is the C++ tables plus the census, and every alias/display column is `browser:pending` until the owner records a dated read. | `currency-v1.toml` says so per row; `SURFACES.md` row for the trade site; question 3 above |

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
neither. So the desired state C73 imports is **two manual rows**; the
other 1353 item rows and 17 location rows are non-actions with their
reasons. `~c/o`: 0 rows. Currencies on rows: `chaos` 1206, `divine` 105,
`exa` 37, `blessed` 6, `jew` 1. Every amount is integral (1…4321, 18
distinct values); none fractional, none ≤ 0. `last_update` is ISO-8601
text with milliseconds and no zone (`2025-06-18T06:51:41.000` …
`2026-08-14T12:39:43.368`) in a column declared INTEGER. Item ids are 64
hex; stash location ids 10 hex; the character row's location id is the
64-hex character id. Every stash location id resolves in `stashes`.

The note corpus: 26,286 items sit in the stash bodies; 121 carry a
note, 35 distinct, 120 tilde-prefixed (the one other note is `50`).
Under the C++ regex (`(~\S+)\s+(\d+\.?\d*)\s+(\w+)`, searched, not
matched): `~price N divine` 44, `~price N exalted` 37, `~price N chaos`
36, and **three unparsed**: `~price 22/10 chaos`, `~price 55/600 chaos`,
`~price 10/80 chaos`. Amount texts are all integers 1…150. `~b/o` and
`~c/o` appear in no note; the word is `exalted`, never `exa`. The 120
notes sit in 17 tabs, 14 of them remove-only ("Lab Enchants #2
(Remove-only)" holds 31). Tilde tab names: 17 (Standard 13, Hardcore 3,
Allflame 1), all `~price N chaos|divine`, all parse; 12 carry a
"(Remove-only)" suffix, 8 a letter "(A)"…"(G)", and "~price 30 chaos
(C)" exists once in Standard and once in Allflame under different ids.

**The current facts** (spike, 2026-09-03). 402 tabs listed in pc /
Standard, 69 fetched, 0 removed: 16 folders, 61 `MapStash` (46
substashes), 46 `UniqueStash` (36 substashes), 163 `PremiumStash`. `idx`
is present on all 402. `metadata.public` is present on 12 tabs and true
on all 12 — absent means not public. Metadata keys seen: `colour` 338,
`layout` 3, `public` 12, `map` 57, `folder` 16, `items` 64. The 13
Standard tilde tab names are in the facts and parse; the Hardcore and
Allflame ones are not (only Standard is listed). Items: 816 stash rows
(the 69 fetched tabs) and 1244 character rows; `x`/`y` are null exactly
for socketed items (24 stash, 531 character); containers: stash `items`
816; character `equipment` 870, `inventory` 126, `jewels` 123, `skills`
42, `guardian` 18, plus 65 pre-v4 rows with no container. **Zero item
notes in the facts**: none of the 17 note-bearing sale tabs is in the
five-tab policy. Characters: 65 rows over 9 (realm, league) pairs, 47
fetched (poe2 included). The intent file holds one row (the sync policy,
revision 9) and no tab or item rows. The account index maps
`GERWARIC#7694` to `cac319d8-…`; a second account, `_vagabond#6960`, has
its own facts file.

**What it means for the rulings.**

- C67, the amount grammar's edge: the ratio form exists in the owner's
  own notes (3 rows). The C++ app never parsed them, so those items were
  priced by their tab or not at all. C67's canonical text already
  represents `a/b`; what `22/10 chaos` *means* on the trade site is a
  `T<n>` claim, not a local inference. The parked entry's trigger has
  fired: a ruling is due before Buyout v1 freezes (step 4).
- C67, `current_offer`: zero rows; the park stands.
- C67 / C73: source `[ignore]` rows carry an amount and a currency; the
  import reads them as non-semantic and the value refuses them if
  present. `last_update` is parsed as zoneless local ISO text.
- C68, currency v1: tags in use are `chaos`, `divine`, `exa`, `blessed`,
  `jew`; the alias `exalted` appears in 37 notes; the C++ list of 19
  tags covers everything observed.
- C69, note precedence, from the C++ code as rule evidence
  (`ItemsManager::ApplyAutoItemBuyouts`): an item note beats the tab
  name, and a note that stops parsing *clears* a game-set item price so
  the tab applies — C69 instead reports the unparseable note and does
  not substitute the tab.
- C73, binding: the file's username maps to this account's uuid in the
  index — `verified`. `not_in_facts` will be large: the source holds
  1080 Standard stashes (978 remove-only) against 402 listed today.
- C74, eligibility: `metadata.public` exists only when true (12 of
  402); positions are absent for socketed items; "(Remove-only)" and
  letter suffixes ride along in priced tab names, so the parser must
  tolerate trailing text as the C++ regex did.
- The two manual rows read as test values (2222 `jew`, 4321 `blessed`);
  validation reading 1 will be about non-actions, `not_in_facts` and
  the binding statement, not about hundreds of manual prices. The
  "batch of hundreds" (C78) will come from the game rows only if the
  owner chooses to import game-derived prices as intent — C73 says no.

## Observations still open

Agent observations that became neither a ruling nor a finding; each is
data for the step that touches it.

- C38's annotation-revision basis is one row (the policy); a `PricePlan`'s
  is many (C71's precondition set). The envelope discipline C75 tests
  will meet this first at step 6.
- The game-side parser's real corpus is in the userstore, not the facts,
  until the sale tabs are refreshed; step 5's tripwire will read empty
  on the current policy.
- The C++ userstore stored `last_update` as text in an INTEGER column
  (Qt bound a `QDateTime`); SQLite's affinity kept the text. Nothing to
  fix here; the import parses what is there.

## Questions for the owner

1. `~price 22/10 chaos` and its two siblings: keep them as ratio amounts
   (representable under C67) pending a trade claim on what the form
   means, or refuse them at import as unparsed?
2. Are the two manual rows real intent (a character item at 2222 `jew`;
   one tab ignored) or test residue? The import handles either; what
   reading 1 can show changes.
3. Is there an official GGG data export for the currency vocabulary?
   None is known to the agent; without one the alias column comes from
   the trade site read in a browser (C79, `SURFACES.md`).
