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

The rulings are `decisions/pricing.md` (C64–C75, C78, C80, C81), `decisions/plans.md`
(C76, C77) and `CONTEXT.md` (C79); properties are pinned by the tests
named below; facts about the trade site and the forum become `T<n>`
claims authored master-side. Nothing here is a second authority.

## State

- Opened at step 2 on 2026-09-03; re-scoped 2026-09-04. Annotations
  schema **v3** (provenance on every row; v3 is the floor — the owner's
  v2 file is refused and its one policy row set again by hand), facts **v7**, sync policy **v3**, plan schema
  **6**; currency table **v1** (reviewed by the owner 2026-09-05);
  buyout value **v1** (`skip`, renamed from `ignore` 2026-09-06); game-side
  parser **v2** (v1 refused a note's suffix; the site reads through
  one, 2026-09-08); listing report schema **2** (1 never left development;
  the second step-4 review's count rename bumped it; the shape is pinned
  by a committed fixture); shop render schema **2** (pinned the same
  way; 1 measured staleness over every posted item). Pricing code so far: `acquisition-plan/src/currency.rs`,
  `price.rs`, `game_side.rs`, `listing.rs`, `shop.rs`; the store's pricing snapshot
  (`snapshot.rs`); `acq reference currency`, `acq price
  status | show | list | set | clear` (`price_cmd.rs`) and `acq shop
  render` (`shop_cmd.rs`). The write is one
  row through `price.rs`'s `set_buyout`/`clear_buyout` (plan step 5,
  2026-09-06, outside review fixed 2026-09-07); **validation reading 1**
  ran 2026-09-07 (below): the owner set rows by hand from an empty
  intent file and read them back; informal by the owner's own account,
  with rigorous testing scheduled after step 6. **Step 6 is built**
  (2026-09-07, `13f50f9d`; outside review fixed the same day,
  `9cfdef5a`, its second round the same evening): the render's policy table posts
  three cells and blocks the rest by name; **validation reading 2**
  ran 2026-09-07 in two passes (below): the owner posted a page,
  answered Q3, Q4 and Q5 (T20, T21, T23), set the page's shape (T22;
  `423d39aa`, shop render schema **3**), found the tab numbering wrong
  past a folder and supplied the website's own rule (T24; `1c85cdcb`),
  and read the renumbered page correct on the trade site: **step 6's
  done criterion is met** in the owner's words. What remained before the
  slice could close is done: PR #227 (T20–T24) merged 2026-09-07, the
  C68/MCP question answered the same day (question 4), and the rigorous
  test pass, **plan step 7**, complete 2026-09-09: item 1 (property
  tests on the three pure functions, proptest) landed 2026-09-07
  (`53421bd0`); item 2 landed the same day — the writer (`0f749018`),
  the owner's run reviewed and committed (`ab2b88f9`), and the suite's
  tests over it (`48d542eb`: the listing state, the render and the
  site's numbering over the owner's league, the audit views' time
  guards over it tiled). Items 3 and 4
  landed the same day (`1eaef2f4`): the story and its races through
  the binary, which found and fixed the intent file's creation race,
  and the daemon's checkpoint on exit. **Item 5 ran 2026-09-08** (the
  owner's eight saved search pages against the store of the same day;
  ledger row and "What the site taught" below): Q11 answered,
  `metadata.public` is the whole condition; no difference changes what
  a page does; the site facts for the owner to author as claims are
  drafted paste-ready in `brainstorming-notes/16` (T17 rewritten, T11
  amended, T25–T35, Q2/Q10/Q11 closed) — the closeout blocker the
  outside review of 2026-09-09 named. Every item of step 7 has its pin or its reading:
  the closed record can be cut when the owner has read this.
- The forum is **write-only from our side**: `/character` returns no
  `forum_note` for a forum-listed item (price-notes run, 2026-09-04), so
  the game side of a listing is item note, then tab name (C69), and a
  forum listing is intent we hold, never a fact we observe.
- Reading a store or intent change before reviewing one: `REFRESH-SLICE.md`'s
  findings table is the checklist, plus the rows below.
- Read-only tools: `tools/census.py` (the 0.18 userstore and the facts;
  refuses an uncheckpointed WAL), `tools/notes-check.py` (the price
  notes a refresh landed; reads through the WAL), `tools/site-listings.py`
  (the trade site's saved search pages → one table by item id;
  `--exchange-items` reads the bulk exchange's item groups from a saved
  page) and `tools/site-join.py` (that table against the listing state
  the `listing-report` example writes, unredacted, under `runs/`).

## Plan (re-scoped 2026-09-04)

Steps 1 and 2 of the packet are done (ledger below). What replaces its
steps 3–10, in order; each gate-green; observable behavior unchanged
until step 5 (one exception, accepted 2026-09-06 as development-only:
a v3 sync policy is held to its canonical spelling — findings, plan 2).
Parked in place with triggers: `PricePlan`/receipts/revert
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
   and `actor`; the per-kind strict trait with `SyncPolicy` moved onto
   it, its parse unchanged and its v3 spelling now canonical; `PriceTarget` with the
   realm-bearing tab and substash keys defined before the first row;
   `Buyout` v1 (`exact`, `negotiable`, `no_price`, `skip` — `ignore` until
   2026-09-06; four-place
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
   compare-and-swap writes returning the prior value; the owner prices
   a few real items by hand, from an empty intent file — validation
   reading 1 (the census's two manual rows are residue, dropped:
   question 1 below).
6. **`shop render`** (C74, C72's report): per-item link codes with
   `realm=`, the price on the line after each link, grouped by price,
   pages labelled *n* of *N* under a size parameter (default 50,000),
   template from a file (default `[items]`), the policy table with its
   opening rows and everything else blocked and counted, the coverage,
   staleness and moved-since-basis lines — then **validation reading 2**:
   the owner pastes a page for their own shop and reads it against the
   trade site; whether the report lines change what they do next is the
   evidence.

7. **The test pass** (added 2026-09-07 after reading 2, at the owner's
   ask: "more rigorous testing, but not yet" — now). In this order, each
   gate-green, the record open throughout:
   1. *Property tests on the three pure functions* — the amount's
      canonical text parses back to the same amount; the note parser
      yields a reading for any input and never panics (C47); for any
      entries and any template the page cutter keeps every page under
      the size, posts every item exactly once, and keeps each price
      group contiguous within a page. The workspace's first
      property-testing crate; the choice of crate is agent-owned.
   2. *A real-scale fixture* — a redacted snapshot of the owner's facts
      (names and ids hashed, notes and tab names kept, positions kept),
      written by a read-only tool the owner runs, committed like the
      price-notes corpus; the listing state and the render run on it in
      the suite, with a time guard where the audit view was quadratic.
   3. *Scenario tests through the spawned binary* — set, show, render,
      clear as one story, with the races: two writers on one row, a
      stale `--if-revision`, a clear under a render.
   4. *The store finding from the price-notes run* — a daemon exit
      leaves the facts file's WAL uncheckpointed; pin the checkpoint on
      stop.
   5. *The site as the oracle* (Q11; the owner's, by hand, any time) —
      the seller-account search for this account and Standard against
      `acq price list --effective game`: the one complete statement of
      what is listed, so the listing state's counts are read against
      it rather than against themselves.
   Done when each item has its pin or its reading, and the closed
   record (step 3 of the session-close skill) can be cut.

**Done criterion:** the currency table committed; the buyout value
strict and pinned by id; the parser passing the fixture; the listing
state legible with the raw note beside the parse; the owner's first
rows set by hand from an empty file; a page rendered, pasted and read correct on the trade site
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
| Pricing | plan 3 outside review | open | (1) **Medium** — "tab name" and `metadata.public` are ambiguous for a nested item: C69 reads "the tab name", C70 keeps inheritance to the manual side, and the store holds substashes and folder children with their own name, metadata and parent (`snapshot.rs`). A read-only look at the owner's facts (2026-09-06, `mode=ro` through the existing WAL): 141 nested tabs of 402 — 77 folder children (ordinary tabs: own name, `colour`, no `public` set, none priced) and 64 substashes under 18 map and unique parents, whose names are the game's own (`1 (Remove-only)`, ` (Remove-only)`), metadata `items` and `map` only, **never `public`**, none priced; no priced parent has children. So the rule was structural for the owner's data but unobserved on the site. Owner, 2026-09-06, verbatim: "I agree with ruling it. Folders never carry price, but substash parents can." — C80, Provisional until the hand experiment in "Observations still open". (2) **Medium** — C69's *Pinned:* reads as if the `c69_` tests pin the whole entry; they pin the parse only. The qualified wording does not fit the 800-byte gate without a trim (the owner's, as before). | (1) C80, the observation; (2) C69's *Pinned:* after the owner's trim |
| plan 6 outside review | `9cfdef5a` | Five findings, all accepted (the owner ran the review 2026-09-07). (1) **High** — the render subtracted timestamps and added one to a recorded index unchecked: a corrupt `last_seen` panicked in debug and wrapped in release, against C47. Ages saturate as the planner's do; `Stash<n>` is a checked add over a non-negative index, and a negative or ceiling index is the `invalid_index` cell. (2) **Medium** — the table failed open: every manual priced kind posted (any non-negotiable kind as `~price`) and every realm but poe2 counted as one the site lists. Kinds and realms are matched by name — `exact`/`negotiable`, `pc`/`xbox`/`sony` (T4) — and any other word is a blocked cell (`unruled_kind`, `realm_unlisted`); a typed effective kind on the listing would make the check the compiler's and waits for a listing-schema step. (3) **Medium** — an uncovered item past the window was called stale with the current plan's request count as its remedy, though that plan does not fetch its container; a snapshot or compile failure was swallowed. `stale` is over covered containers only; uncovered items past the window are `stale_uncovered`, said beside the coverage line; the compile failure rides as `refresh_problem` and the stale line prints it. (4) **Medium** — `--expand` searched the whole left-out list per line for a field the line held (quadratic over 26k items): the line takes the `LeftOut`. (5) **Medium** — the league-less check ran before the effective outcome, so an unpriced or hand-skipped item on such a character was blocked instead of off the page or omitted, against the module's own ordering: the check follows the classification. | (1), (2) `c47_corrupt_facts_and_unruled_words_are_cells_never_panics`; (3) `c72_…` (the split), `c53_shop_render_…` (the remedies); (4) `left_out_line`'s signature; (5) `c74_every_item_…` (two league-less items) |
| plan 7 outside review | `8ea70c56` | Three findings, all accepted (the owner ran the review 2026-09-09). (1) **High** — the ground truth still says the dialog strips a note's suffix (T17) and leaves Q2, Q10, Q11 open, against parser v2 and the two readings: the closeout blocker under the authority rules; the claims are drafted paste-ready for the master-side PR (`brainstorming-notes/16`). (2) **Medium** — `site-join.py` read a "Price with Note" row as the parser's difference before the tab's `public`, so a note-priced row from a non-public tab would have hidden exactly Q11's condition; a second branch was unreachable. The not-listed branch now reads `public`, then `skip`, nothing else; `--self-test` pins the order over synthetic rows; the docstring no longer says v2 refuses a suffix. Both captures unchanged (535 and 866 agree). (3) **Medium** — the record described finished work as open (PR #227, the C68 question, step 7; question 5; three observations the readings answered); C80's module doc said provisional; the `listing-report` example claimed `--json` prints the whole report where the CLI prints a view; the record said the render blocks a game-priced socketed item where `shop.rs` omits it as game-listed first. All corrected. **Found on the way** (the gate, once in five runs): `price_story`'s writers race refused round 1 with "exists with content but no schema stamp" — the creation race's second window: the loser's `fresh` is false because the winner's file already exists, and `user_version` is still 0 because the winner's stamp is not yet committed; the refusal meant for a foreign file fires before the immediate transaction that would have serialized them. **Closed before the cut** (owner: "a real first-write race, not merely a flaky test […] The fix should pin both boundaries"): `Annotations::init` reorders a first open — connection settings, a read-only version gate that refuses only a stampless file *with tables* (the file untouched), then the immediate transaction, where a loser waits out the winner's create under the busy timeout and a foreign writer that landed tables meanwhile is refused with nothing written, and only then the journal-mode switch. Two tests hold the winner by hand between file creation and stamp commit: the loser proceeds; a foreign writer that commits without a stamp is refused and the file is byte for byte as it left it, no WAL beside it. Both seen failing on the old open path (spliced onto it) and passing on the new. | (2) `tools/site-join.py --self-test`; (3) the record, `listing.rs` C80 doc, `listing-report.rs` doc; the race: `c35_a_losing_first_opener_waits_for_the_winners_stamp_and_proceeds`, `c35_a_foreign_writer_landing_under_the_lock_is_refused_and_left_untouched` (`annotations.rs`), the price story green five times over |
| Pricing | plan 6 reading 2, second pass | `1c85cdcb`, `c7d8c840`; T24 `635e583c` master-side, `7ef30761` here; T7, T12, T21 upgraded master-side | **pass** — the Allflame page (27 items: 20 inherited from a tab row, two item rows in the folder's child tab, three in a top-level tab, one in the public priced tab, one on a character) pasted, previewed and posted; every link resolved and the site lists every item; the no-price item is listed with no price under the empty title (T21); the hand price in the public `~price 222 divine` tab is listed at the hand price (T12 confirmed); the character link resolved as `I_EXIST` (T7's case question). Found on the first paste: `Stash<n>` past a folder was one too high — fixed to the website's rank (T24). The report's cells on the real page: 39 the game lists, 1 skipped by hand, 1 socketed, 5 reading-1 rows naming nothing in Allflame | the owner's verdict verbatim below; T7, T12, T21, T24 |
| Pricing | plan 6 reading 2, first pass | `423d39aa`; T20–T22 `9d51fbf7` master-side, `bc596b48` here | the owner's page shape — one spoiler per price titled with the price (the C++ prefix's leading space, T15), links run together, one page spoiler `Shop Post n of N (k items)`, newlines only between spoiler tags (T22) — with the page title reserved at its widest before the cut and a group closed and reopened across one; `hand_no_price` a posting row (the link alone under an empty title; T21); `substash` blocked on an observation, not a question (T20); `Posted.title` replaces `price` (schema 3); the `c74_` page tests rewritten, `tests/shop_json.rs`, `CLI-REFERENCE.md` |
| plan 6 outside review, round 2 | `6794c0e4` | The reviewer's re-read accepted four of the five fixes and found the fifth half-done: `stale` held covered items only, but `oldest_seconds` still measured every posted item, uncovered ones included, and the CLI's stale line presented that age as the covered set's — the `c72_` test pinned the mismatch (covered stale items 3,800 s old; 4,900 asserted, from the uncovered one). The age is now the oldest of the stale set, computed where the set is built, and the field is `oldest_stale_seconds` — a rename, so shop render schema **2** and the fixture regenerated; schema 1 never reached a consumer. | `c72_…` (3,800, the stale set's), `c53_shop_render_…` (the CLI sentence), `shop-render-schema-2.json` (the schema-1 fixture renamed and regenerated) |
| plan 1 currency table | `10ef2374` | `currency-v1.toml` v1: 39 active rows (tag = emit = the game's word, display from the owner's dialog reading, a `game:` entry each), 4 legacy aliases, 3 retired rows; `currency.rs` (the loader as the reviewer's checklist); `acq reference currency`; the `c68_` tests |
| Pricing | plan 3 game-side parser | `ee7d7bd7` | `game_side.rs`: `read(source, text, table)` over a note or a tab name — `exact`, `negotiable`, `skip`, `invalid` naming why, or none (C69); a leading `~` opens a price note; a tab name tolerates trailing text and refuses a ratio (T11), a note holds the grammar exactly (T10) and takes a ratio (T2); the word resolves through the table to a tag (C68); `NOTE_PARSER_VERSION`; the fixture reader shared with the currency tests (`price_notes_fixture.rs`); the `c69_` tests |
| Pricing | plan 4 listing state | `c340b487`, `25f99e4f` | the store's `PricingSnapshot` (`Store::pricing_snapshot`: the refresh snapshot's tabs and characters, every live item at them with its `note` verbatim, every `buyout` row raw; the readers shared with `refresh_snapshot`); `listing.rs`: `resolve(&snapshot)` → a `ListingReport` (schema 1) with one `Listing` per tab, substash, character and item — the manual side by specificity (C70: item, substash, tab, folder; item, character), the game side through the tab C80 names (note first, then the tab name, both texts verbatim beside both readings, `public`), the relation with a sentence naming both sides, per-listing basis, parser and table versions, the row accounting (applied, other realm, unmatched, unreadable) and the counts; `acq price status | show | list` under C53's three views (`price_cmd.rs`); `PriceTarget: FromStr`, `Buyout: Deserialize` (the strict parse), `Amount` as text in JSON; the `c69_`, `c70_`, `c80_` and `c53_price_` tests |
| Pricing | plan 4b C81 build | `9db1c99a` | `ignore` → `skip` (C67); the game side a statement only for a public tab (C81, T1) — a note or name that reads as a price or `skip` elsewhere is `residue`, shown, never a side; an unreadable note has no effect and the tab applies (T18); `Effective` on every listing (the more specific statement, the game's on a tie, by chain position; `side`, `from`, `why`); `Counts.by_effective`, `residue`; `acq price` says who decides (`status`'s first line, the `wins` column, `show`'s `effective:` line); the recorded C69 text in `listing.rs` and `game_side.rs` follows the registry; the `c81_` test |
| Pricing | plan 2 annotations v3, typed value | `3d2902cd` | annotations v3 by stepwise `ALTER` (`written_via`, `actor`; C65); `IntentValue` + `check_value` in the store crate (version gate, per-kind strict parse, exact round-trip for a current-schema value, then CAS; C66) with `SyncPolicy` moved onto it unchanged; `Provenance` required by `put`/`delete` and threaded through `put_sync_policy`, the CLI (`cli`) and MCP (`mcp`); `list(scope, kind)`; `AnnotationError::Busy`; `price.rs`: `PriceTarget` (realm-bearing tab and substash keys), `Amount`, `Buyout` v1 (C67); the `c65_`, `c66_`, `c67_` tests |
| Pricing | plan 5 set, clear | `5fc19304` | `price.rs`: `set_buyout` / `clear_buyout`, the one write path for every frontend (single-row CAS through the typed door, the prior row returned as a `PriceWrite` — C78's clause until receipts; a new price never names a retired tag — C67's writer's rule; the blind "replace whatever is stored" a frontend policy, as with the sync policy); `acq price set <target> <type> [<amount> <currency>]` and `acq price clear <target>`, `--if-revision` as `acq policy set` has it, the receipt under C53 (what it is now, what it was, the `set` words that put the prior back; `--json` the `PriceWrite`); the `c78_`, `c67_…retired…`, `c35_…stale…` tests, the CLI's grammar and receipt tests, and `price_json.rs`'s process pin |
| Pricing | plan 5 reading 1 | — | the owner's rows set by hand from an empty intent file, read back; verdict verbatim in "What validation reading 1 taught"; rigorous testing deferred to after step 6 |
| Pricing | plan 6 shop render | `13f50f9d` | `shop.rs`: `render(&ListingReport, &RenderOptions)` — one policy-table cell per item read from the effective outcome (side and kind together) then the address; the two posting rows (a stash item at a listed tab by `Stash<index+1>`, a character item by its slot — T7, T8, T13, T15), omission for what the game lists or skips (T11, C81) and a hand skip, off-page for nothing-applies, and every unobserved case blocked and counted naming its question (Q3 substash, Q5 no_price, Q6 ratio, a retired tag, socketed, an unlisted tab, no slot, poe2, unresolved, league unknown, over the page size); `~price`/`~b/o <amount> <emit>` on the line after each link, grouped by price, pages cut under `--size` (default 50,000) with the template counted and labelled n of N; the C72 report over the posted items (coverage by `Selection::covers_tab`, staleness by the window with the `RefreshPlan`'s request count, positions seen before the listing); `acq shop render` under C53 (`--page N` for the clipboard, `--template FILE`, `--expand` the whole table); the snapshot's `inventoryId` and `Subject`'s `x`, `y`, `inventory_id`, `index` (additive); the `c74_`, `c72_`, `c53_shop_render_` tests, `shop-render-schema-1.json`, `tests/shop_json.rs` |
| Pricing | plan 7.1 property tests | `2e60be6b`, `53421bd0` | proptest 1.11 as `acquisition-plan`'s dev-dependency, the workspace's first property-testing crate. `price.rs`: any amount round-trips through its canonical text — a fixed point of the parse, the shortest spelling, the JSON both ways; any spelling of the decimal grammar parses to the amount it names, zero refused; any lot pair kept unreduced; any text parses or is refused naming it (C47). `game_side.rs`: any text under either source reads and never panics, trailing whitespace changes nothing, no `~` is none, a price reading is its own grammar re-read from the text and re-written; a well-formed price reads as written, a ratio invalid in a tab name (T11), text after the word tolerated by a tab name and refused by a note; the generator mixes the grammar's pieces, their neighbours, the game's suffixes and arbitrary strings. `shop.rs`: the page cutter extracted from `render` as `cut_pages` (pure over the candidates, the template and the size; behaviour unchanged, the exact-JSON fixture holds), and for any entries, template and size every candidate is posted or set aside, never both; every page within the size with the template around it, numbered n of N; every link on its page exactly once; groups sorted across the run and contiguous on a page. Each property broken deliberately and seen to fail (the canonical text padded; the trim dropped; the page's fixed cost dropped; the sort dropped). Found on the way in: the render fixture the test read was named for schema 2 and held schema 3, while the schema-3 file held the stale document nothing read — renamed, the test pointed at it (`2e60be6b`). | `c67_any_…`, `c47_any_…` (`price.rs`); `c47_c69_any_…`, `c69_a_well_formed_…` (`game_side.rs`); `c74_any_entries_…` (`shop.rs`); `shop-render-schema-3.json` |
| Pricing | plan 7.2 fixture writer | `0f749018` | `examples/redact-pricing.rs` in `acquisition-plan`: one (realm, league)'s `pricing_snapshot` from the owner's store, opened as every `acq` read opens it, written to stdout as the `PricingSnapshot` JSON the suite reads back; every name and id hashed under a per-run secret kept nowhere (ids keep their length; the `name` and `id` inside a character's verbatim listing entry and fetched envelope too — a first dry-run leaked them; a buyout key through the target grammar), notes, tab names, positions, timestamps, type lines, tab metadata and buyout values kept; a summary by count on stderr. Dry-run on the owner's Standard league (scratchpad only, deleted): 402 tabs, 41 characters, 1,977 items, 16 rows, 1.0 MB; every reference resolves within the file; parses back, resolves in 8 ms, renders in 1 ms, round-trips exactly. 11 of the 16 rows name items outside Standard — the report's `unmatched`, real data. The owner ran it, read the summary and the file, and committed it beside the corpus (`ab2b88f9`): `reference/pricing-snapshot-2026-09-07.json`, pc/Standard — 402 tabs (16 folders, 64 substashes, 16 named with a price), 41 characters, 1,977 items (50 with a note), 16 rows. The suite's tests over it are the next row. | the example; the fixture |
| Pricing | plan 7.2 fixture tests | `48d542eb` | `real_scale_fixture.rs` in `acquisition-plan` and `acquisition-cli` (test-only loaders; the CLI's tiles the items). `listing.rs`: the league resolves whole — one listing per fact, every location and chain reference a subject of the report, every count summed against the items and read against the file: 50 notes, all in the one public tab with a row on it, are 46 exact, 2 negotiable, 1 skip and the dialog's one residue; the 16 `~` names are 14 prices (1 public) and the two lot ratios a tab name cannot carry (T19); the 16 rows are 5 applied, 11 unmatched (10 items of another league, 1 tab), none unreadable. The owner's five rows land by C70/C80/C81: the tab row covers the test tab's 80 items (78 inherited, 2 own), the 49 game statements decide over it, the substash row its 5 under a non-public parent that holds nothing, the character's `skip` its 3; `list`'s default is those 88 in four containers; the report re-reads exactly and every container's `show` holds its two sets. `shop.rs`: every item in one cell — 48 game prices, 1 `~skip`, 3 hand skips omitted; 5 substash items blocked; 1,889 nothing; 31 posted on one page under three titles, each link once; T24 with the site as the oracle — the snapshot's listed tabs are the website's list of the day by name and type, and every posted link numbers its tab as the site does (`Stash57`); C72 at the snapshot's clock: the test tab four days stale under an hour's window, the one uncovered container under a policy naming none of it. Time guards where a debug build measured milliseconds — cliffs, not budgets: resolve and render under 2 s; `render_text(expand)` under 2 s at 63k items tiled, where the review's quadratic search measured 3.9 s at half that scale; `list --effective none`, `show`, `status` expanded under 5 s at 32k. | `c69_at_real_scale_every_fact_has_one_listing_and_the_counts_are_the_files`, `c70_at_real_scale_the_owners_rows_cover_what_the_rules_say`, `c53_at_real_scale_the_report_round_trips_and_every_container_shows_its_two_sets` (`listing.rs`); `c74_at_real_scale_every_item_lands_in_one_cell_and_the_page_is_the_owners_test_tab`, `t24_at_real_scale_every_posted_link_numbers_its_tab_as_the_website_does`, `c72_at_real_scale_the_freshness_lines_name_the_owners_test_tab` (`shop.rs`); `c53_the_expanded_render_is_linear_over_the_owners_league_tiled` (`shop_cmd.rs`); `c53_the_expanded_list_show_and_status_are_linear_over_the_owners_league_tiled` (`price_cmd.rs`) |
| Pricing | plan 7.3 story and races | `1eaef2f4` | `tests/price_story.rs` (acquisition-cli): set, show, render, clear as one story through the spawned binary, then the races as properties under every interleaving, run in rounds with the processes started together — two blind writers on one row never clobber and the receipts chain from the row before the round to the row `show` reads after it (C35, C78); a stale `--if-revision` on `set` and `clear` conflicts naming the current revision, `{"error":…}` with exit 1, nothing changed (C35, C11); a clear under a render always lands and the render is one function of one read, never a torn page (C64, C74). **Found:** the seed leaves no intent file, so round 1 races its creation, and about one run in six a first-ever writer was refused "busy: another writer held it past 5 s" in under a second — `PRAGMA journal_mode=WAL` on a fresh file takes the write lock from inside the read transaction the pragma opened, a path SQLite's busy handler does not cover, so the second creator is answered `database is locked` at once. **Fixed:** the three opens (facts, intent, queue) go through `ensure_wal` — an already-WAL file is left alone, the creation switch retries up to the busy timeout; 20 runs after: no refusal, one to four genuine conflicts a run, every chain intact. A first theory (the busy timeout installed too late) was wrong — rusqlite installs it at open — and was reverted. **A second window** in the same race (the loser refused as a foreign file while the winner's stamp was uncommitted) surfaced in the 2026-09-09 gate and is closed in the review row below. | `c74_set_show_render_clear_…`, `c35_two_blind_writers_…`, `c35_a_stale_revision_…`, `c74_a_clear_under_a_render_…`; `ensure_wal` (`acquisition-store/src/lib.rs`) |
| Pricing | plan 7.4 checkpoint on stop | `1eaef2f4` | the daemon leaves by `process::exit`, so its connections never closed and the WALs stayed beside the facts file and `daemon.db` (the price-notes run: 1.1 MB the census refused). `Store::checkpoint` and `JobDb::checkpoint` (`PRAGMA wal_checkpoint(TRUNCATE)`); the daemon's two exits — stop request, idle watchdog — go through one `exit_process` that checkpoints the open store and the queue, logs the page counts, removes the socket, exits. `tests/daemon_stop_checkpoint.rs`: a listing landed through a real daemon over the mock, both WALs holding pages before the stop (so an empty one after is the checkpoint's doing), the socket gone, both WALs empty; seen failing first at 107 KB. `tools/census.py`'s guard and `tools/notes-check.py`'s "stop the daemon first" stand, and now mean what they say. | `daemon_stop_checkpoints_the_facts_file_and_the_queue`; `exit_process` (`daemon.rs`) |
| Pricing | plan 7.5 the site as the oracle | `df248a17` | `tools/site-listings.py`: the owner's saved pages (rendered DOM, eight parts, `runs/site/`) → one table by item id, deduped across parts, each part's "Showing N results" line kept, the price label (Exact, Asking, No Price Set, and a fourth, "Price with Note:"), amount, the site's currency word, the note verbatim, the age, and the channel — a stash row links the profile, a forum row its thread. `examples/listing-report.rs`: the `ListingReport` `acq price list --json` renders, unredacted, to stdout (the join key is the id), kept under `runs/`. `tools/site-join.py`: the join under a reason list fixed before it ran; two reasons it found (`socketed`, `stackable`) were added with the run, and the doc says so. The pages of 2026-09-08: 551 item-search rows, none shared between parts, 547 stash and 4 forum, all verified, one account, and a ninth page, the bulk exchange, 8 offers (read the same evening, `c5b7e314`); the store the same day (the run ledger's refresh): 20,941 items, 757 the state expects on the site. **Q11 answered:** every stash row is from a tab whose `metadata.public` is true; the 13 non-public priced tabs (1,121 items) put nothing there — C81's premise holds, and note 15's 8888 finding was the flag set after the 09-07 listing (the 09-08 facts hold it public). 535 agree, 20,180 absent on both; 15 differ; 9 site-only, the forum rows; 207 state-only; one row no reason explains. **No difference changes what a page does today** — the render omits what the game lists at whatever price, socketed or not, blocks a hand-priced socketed item (T13), and posts nothing the site shows — with one narrow candidate the exchange page raised (point 2): a manual row on an exchange-eligible stack in a ratio-named public tab would post a price the exchange contradicts. Each is an observation with its trigger or a claim for the owner ("What the site taught"). | `reference/site-listings-2026-09-08.json` (the table; the pages stay in `runs/`); the tools; the join's output is reproducible from the two |
| Pricing | plan 7.5 second reading: the four experiments | `87dab5bb`, `895a5433` | The owner changed four tabs in game (2026-09-08 evening), the driver refetched them and the unique tab's 19 substashes (run ledger 2026-09-09: 27 sends after a re-login; then one listing GET for the unique tab's rename), and the site was captured again: 13 item-search parts and the exchange page, 890 rows, `reference/site-listings-2026-09-09.json`. The normalizer learned the exchange's compact layout (one `per-have` block per offer; a stack priced by note and by tab offered twice; the offer shown per unit, so the join compares rates); the join learned that a "Price with Note" row agrees under parser v2, and that the exchange keys one row per item type per account, every public stack's offer under it. Results, "What the site taught" 12–16: C80 confirmed on all three counts; a note beats a valid tab name (Q2); the suffix read again (`~price 666 chaos tested` → "Price with Note: 666"); 63 items in six tabs missing from the capture (the owner's paging, not the site: both names checked list) and one "character" item the site lists (moved to a stash tab after our last character fetch) — both closed by the owner the same day. 866 agree, 14 differ (the eight words, the five fractions, `facetors`), 10 site-only (4 forum, 5 forum stacks on the exchange, the character item), 275 state-only (95 ratio-tab, 111 socketed, 6 stacks, 63 open). | the tools; the table; `tools/site-join.py --rows` over `runs/site/` |
| Pricing | plan 7.5 follow-ups: parser v2, the exchange list | `43e281a0` | `game_side.rs` **v2**: text after the word is tolerated by a note as by a tab name — the game stores a note's suffix whole and its dialog displays only the parsed part, the API serves the whole, the site reads the price out of it ("Price with Note"); the owner confirmed all three in game and on the site with `~price 777 chaos testing` and a fresh `~price 666 chaos tested` (2026-09-08). One branch fewer; the one rule the two sources still differ in is the ratio (T2, T11). `NOTE_PARSER_VERSION` 2 (a reading of the same text changed); the two committed report fixtures carry it. The exchange's item groups (Q10), from the owner's saved page with 15 of 22 groups expanded — 750 ids and display names — committed as a proposal beside the currency table; no code reads it until the parked table's trigger fires. | `c69_text_after_the_word_is_tolerated_by_a_tab_name_and_a_note_alike`, `c69_a_well_formed_price_reads_as_written` (`game_side.rs`); `reference/exchange-items-2026-09-08.json` |
| Pricing | plan 7 gate | `04993ca5` | **Found by the bare gate at session close:** the store's C35 export race test failed one run in twelve — on the pre-session store code too — with both exports refused. An export's partial file was named by pid and the clock's nanoseconds; two threads within one tick (microseconds on macOS) shared the name, one `VACUUM INTO` failed on the other's file and its cleanup removed that file before it was published. A per-process counter joins the name; thirty runs after, none failed. | `simultaneous_exports_to_one_destination_publish_exactly_one` (`annotations.rs`) |

## Findings

One row per review round; the finding, then the property or test that
holds it now.

| Round | Commit | Findings | Held by |
| --- | --- | --- | --- |
| 2a clause audit | `8aa7a507` | (1) C41's caller-asserted freshness read ("fails with the exact `RefreshPlan` it would take") has no consumer and no code — held by nothing; its first consumer is C72's report at the render step. (2) "The daemon never reads the store" was described as graph-enforced, but the daemon crate links the store crate to *write* facts (C28); what the graph can enforce is daemon ∌ planner and daemon never names the intent API — now a check. (3) C44's "fact drift does not refuse" was stated in the doc and held by the shape of `check_spendable`, pinned by nothing. (4) The uncited ids were the ones whose pins existed under descriptive names; the fix was citation, not tests — 30 tests now name their id. | (1) `decisions/plans.md` C41 *Pinned:* says so; (2) `tools/docs-check.sh` check 4; (3) `c44_fact_drift_never_refuses`; (4) doc comments on the tests, `docs-check`'s uncited report |
| 2b annotations review, fixed now | `3e234fa5` | (1) `synchronous=NORMAL` under WAL keeps the file consistent but lets the last commits before a power loss roll back — on the one file with no server to refetch from. (2) `export` wrote `dest` directly and never fsynced it: an interrupted export left a partial file that looked like a backup and, under never-overwrite, blocked every retry. | (1) FULL; `c35_the_intent_file_is_fully_synchronous`; a single-row put measured 87 µs on this machine. (2) `<dest>.partial` → `quick_check` → fsync → link; an existing `dest` refuses before anything is written; `export_is_a_consistent_snapshot_and_never_overwrites` |
| 2b annotations review, constraints on the intent step | — | (3) Tombstone generations are invisible: `get`/`list` hide a tombstoned row and `Conflict { current: None }` hides its revision, and `put(None)` conflates never-existed (revision 0) with tombstoned. (4) One IMMEDIATE transaction per `put` and no batch: all-or-none for a plan of hundreds is impossible through the API (10k single puts in 449 ms under NORMAL — atomicity, not cost). (5) The migration shape `0 \| 1 => CREATE TABLE IF NOT EXISTS` cannot add a column; v3 needs stepwise `ALTER TABLE … ADD COLUMN … NOT NULL DEFAULT 'unknown_legacy'` inside the same IMMEDIATE transaction; a v3 file is already refused by a v2 build. (6) The `tab` key is realm-less and a substash is the caller's `parent/id` convention; C67's targets carry realm and the owner's file holds zero tab-scoped rows — the realm-bearing key must be defined before the first row lands (P3). (7) `list` filters by scope only; pricing reads want one `kind` across four scopes; at 10k rows `list` takes 35 ms, `get` 27 µs, export 7 ms — a kind filter is needed, an index is not. (8) A writer blocked past the 5 s busy timeout surfaces as `Db(…)`, indistinguishable by kind from any other SQLite error, though a driver must tell "retry later" from "re-read and retry". (9) Receipt growth: an import of this owner's file is ~1.4k mutations, ~300 KB per receipt at ~200 B each; a `receipts` + `receipt_mutations` layout indexed by target keeps "since *T*" a query. (10) `open_for` creates the file and stamps the uuid on read paths — a read that writes; harmless, noted. **Re-priced 2026-09-04:** (5), (6), (7), (8) and a reduced (3) — a `clear` then `set` on one target must work through the tombstone — are paid by plan step 2; (4) and (9) are parked with C71/C78. **Paid 2026-09-05** (plan step 2): (3) reduced, `c67_clear_then_set_on_one_target_works_through_the_tombstone`; (5) the stepwise `ALTER` was built, then replaced by the floor (finding below) — `c65_a_file_below_the_floor_is_refused_never_migrated`; (6) `c67_the_target_key_carries_the_realm_and_round_trips`; (7) `list_filters_by_scope_and_by_kind`; (8) `a_writer_held_past_the_busy_timeout_is_busy_not_db`. | (4), (9): C71/C78's park; the rest: the tests named |
| 2c census | `cdfd02cc` | (1) A `file:` URI truncates at `#`, so the owner's `userstore-GERWARIC#7694.db` opened as an empty file; a stray zero-byte `userstore-GERWARIC` beside it, dated 2026-08-13, is the same trap met by someone else. (2) A `mode=ro` open of a WAL file created `-shm`/`-wal` in the owner's data directory. | `tools/census.py`: percent-encoded `immutable=1` open, refusal of an uncheckpointed WAL; the side files removed |
| 2d currency source | `51a33751` | No official GGG data export for the currency vocabulary is known, and the trade site's static endpoint is rejected as a tooling source (packet §1(d)); v1's only cited source was the C++ tables plus the census. **Superseded 2026-09-04:** the game's own client is the source for `emit` (the price-notes run) and the owner's reading of the dialog for display names (both in the fixture). | `currency-v1.toml` header; `SURFACES.md`; plan step 1 |
| 2e forum reading and price-notes run | `0d9fee64` | (1) C67's mechanism reduced `a/b` and compared rationals, but the trade site's ratio is `wanted/lot` with the denominator a lot size (`2/35` is not `1/17.5`; `3/1` is not `3`) — the amount is a pair or a two-place decimal, ruled. (2) The game writes `exalted`, `chrome`, `jewellers`, `fusing` where the C++ table emitted `exa`, `chrom`, `jew`, `fuse`; `chisel`, `coin`, `silver` are no longer in the game; 23 words are missing — the table's `emit` column was folklore. (3) The game side has four outcomes, not two: the dialog writes `~skip ` for "Do not index" and leaves `~price  chaos` (empty amount) after an invalid entry; a ratio in a tab name unlists the whole tab (owner, in game). (4) `/character` carries no `forum_note` for a forum-listed item: the forum is write-only from our side. (5) Every stash item's `inventoryId` is the literal `Stash1`; a link code's `Stash<n>` must come from the tab's index; socketed items have no position. (6) The forum resolves a `linkItem` into `[item post= index=]` at post time and the site emits `realm=` today — the C++ code had it, the wiki and Procurement do not. (7) A daemon exit leaves the facts file's WAL uncheckpointed (1.1 MB after this run), so `census.py`'s guard refuses the file; whether the stop path skips the final checkpoint is a store/daemon look before the import trigger ever fires. (8) The journal's default path is `<socket dir>/acquisition-playground.ggg.sends.jsonl`, not `<socket>.ggg…`; the owner copied the daemon log by mistake. | (1) C67; (2) C68, the fixture file, plan step 1; (3) C69; (4) `PRICING-SLICE.md` State, C69; (5), (6) C74's policy rows, the render step; (7) open — a finding for the store; (8) `LIVE-TESTING.md` ledger row, the live-run skill's wording |
| plan 2 annotations v3, typed value | `3d2902cd` | (1) The exact round-trip (C66) tightens what the sync policy accepts: `tabs: []` beside a facet that names work, or `characters: null`, were accepted and normalized on parse; now refused at write *and at read* naming the path. Read-only check of the owner's intent file (immutable open, WAL empty): one row, a canonical v3 policy at revision 9 — nothing stored is affected. (2) With the write door typed the store crate owns no real kind, so its own tests declare loose kinds (`test_kinds`); a raw untyped put no longer exists in the API, which is the point. (3) `null` and absent are different JSON; the difference finder reports "absent" rather than conflating them, since `characters: null` is exactly the case. (4) A tombstone keeps its last value text under the row; step 5's "returns the prior value" must say a tombstoned value is not a prior (a `set` over a tombstone has no prior), noted there. (5) `acq policy show --json` and the MCP `sync_policy` tool now carry `written_via` (and `actor` when claimed) on the row — additive. | (1) `c66_a_current_schema_policy_must_be_canonical_and_older_ones_upgrade`; (2) `annotations.rs`, `test_kinds`; (3) `the_first_difference_names_the_path_and_both_sides`; (4) step 5; (5) `c65_every_write_carries_its_provenance` |
| plan 2 owner review: the floor | `0550e6e0` | The build migrated v1/v2 files by stepwise `ALTER … DEFAULT 'unknown_legacy'` and reserved that marker so no writer could claim it — a concept that existed only because development builds wrote rows without provenance (one file, one policy row). Owner, 2026-09-05: "Prior versions only exist in development and are already obsolete" — v3 is the **floor**: a file below it is refused naming the file and the fix, never migrated, never touched; the marker, the ladder and two migration tests are gone, and C65 lost the sentence that carried them (room for its *Pinned:* pointer). The cost is one `acq policy set` by the owner; the deadline was now, before step 5's price rows make an old file worth migrating. A later column is still a stepwise `ALTER` from the floor up, as the facts store does. | `c65_a_file_below_the_floor_is_refused_never_migrated`; C65 |
| plan 2 outside review | `46b14308`, `b960e364`, and this commit | Four findings, all accepted. (1) **High** — `Amount` held hundredths and refused a third digit, but the step-3 fixture carries the game's own `999.123` and `999.1234` (T10) which C69 must read as prices; a second amount type for observed notes would have made step 4's comparison a conversion. Owner: adopt four places (his reading of the trade site: it takes four); one bound everywhere, ten-thousandths, no version bump (widening is compatible), C67 amended. (2) **Medium** — export's "never overwrite" was racy: a shared `<dest>.partial` and a `rename` fallback that replaces. Per-invocation partial, `hard_link` publication (EEXIST refused), `rename` only where hard links are unsupported (the documented residual). (3) **Medium** — an existing unstamped file with content was initialised over as empty intent; now refused and left byte-identical, a zero-byte file created over (nothing to preserve; the C++ app has left one). (4) **Medium** — the plan said "unchanged" while the C66 round-trip had tightened what a v3 policy accepts (`tabs: []`, `characters: null`), and the planner doc, the decisions intro ("no pricing code exists yet") and four pre-v2 comments were stale. Accepted as a development-only break, like the floor: the only stored v3 policy is canonical (checked); no new policy version, the documents say what the code does. Not taken: a typed `PriceTarget` write wrapper (step 5's first line, as the reviewer allowed). | (1) `c67_the_amount_is_a_four_place_decimal_or_an_unreduced_lot_pair`, C67; (2) `simultaneous_exports_to_one_destination_publish_exactly_one`; (3) `an_unstamped_file_with_content_is_refused_and_an_empty_one_is_created_over`; (4) this file's plan, `lib.rs` (`SYNC_POLICY_VERSION`), `decisions/pricing.md` intro, `annotations.rs` |
| plan 4 listing state | `c340b487`, `25f99e4f` | (1) The store had no read of the items at a location, and the bodies are the wrong grain for a league (26k rows): the pricing snapshot reads thirteen columns and `json_extract(json, '$.note')`, under the same membership predicate as the refresh snapshot's tabs and characters, so an item never cites a location the snapshot lacks; the resolver still reports one that did rather than drop it. (2) SQLite orders NULL first, so socketed gems (no position) led their holders; ordered after them explicitly. (3) `Buyout` had no `Deserialize` and the report is a contract that must re-read: implemented as the strict parse, so a report round-trips exactly (pinned), and `Amount` serializes as its canonical text, never a number. (4) A row that cannot be read (a newer value schema, a malformed address) never yields to a less specific row: the walk stops, the listing says why, the report lists the row once — the C++ app's fall-through is exactly what let a broken note clear a price. (5) C70's "the way a policy id does" was read with C37: a folder's row covers the tabs it groups (the folder itself is never fetched and never read as a price, C80). (6) "Priced tabs" counts the tabs a user can name — top-level and folder children — never substashes (their name is the game's) or folders. (7) `ignore` beside a game price is reported as `conflict` with C69's sentence in the `why`; whether that word is right is an observation below. (8) Under C53 the group header says the container's context once (its reading and `public`), and a substash is labelled under its parent's name, since the parent's is the name read. | (1) `the_pricing_snapshot_carries_notes_and_buyout_rows_verbatim_and_live_items_only`; (2) the same; (3) `rows_are_accounted_for_and_the_report_round_trips`; (4) `c70_the_manual_side_resolves_by_specificity`; (5) the same; (6), (7) `c69_two_sides_resolve_independently_and_the_relation_names_both`; (8) `c53_price_list_groups_by_container_and_lists_ten_or_fewer` |
| plan 4 outside review | `06197271`, `4f2e1de6` | Five findings, all accepted (the owner ran the review 2026-09-06; C81's unbuilt behaviour was not counted). (1) **High** — the pricing snapshot reused the planner's character read, which carries a league-less character under every league so every plan can report it; in pricing that became membership, and the resolver dropped the `None`. Kept rather than excluded (excluding would report its rows as naming nothing): the character and its items carry `league_unknown`, counted, said in `status` and `show`; the owner has no such character, the flag exists for the store's rule. (2) **Medium** — `list --in tab/xbox/X` without `--realm`, a disagreeing `--realm`, or a typo gave "none match": one realm helper shared with `show` reconciles the address with `--realm`, and a container not on record refuses naming the facts. (3) **Medium** — `items_in` was the physical set only, while C70 says a folder's or parent tab's row covers its children: every listing now carries its chain, `items_covered_by` is the covered set, `show` prints both, `list` takes `--in` (physical) and `--covered-by` (C70). (4) **Medium** — `list --json` and `show --json` omitted the container records the text used (names, `public`, readings, a substash's parent), so text was not a function of JSON (C53): `ListView` and `ShowView` are the documents, carrying the header, the selection, every container on the selected items' chains and both item sets, and the renderers read nothing else; a process test parses them from the spawned binary. (5) **Medium** — the zero-item `status` path returned before the rows: it now reports the rows and picks its next action from them (C35). | (1) `a_league_less_characters_items_are_flagged_in_every_leagues_report`, the snapshot's doc; (2) `the_realm_comes_from_the_address_and_a_disagreement_refuses`, `tests/price_json.rs`; (3) `c70_a_targets_coverage_is_every_listing_whose_chain_holds_it`; (4) `render_list`/`render_show` signatures, `tests/price_json.rs`; (5) `c35_an_empty_league_still_reports_its_rows` |
| plan 4b outside review | `d034713b` | Four findings, all accepted (the owner ran the review 2026-09-06 after the C81 build). (1) **High** — the walk stopped at an unreadable row but the effective price never learned of it, so a lower-level game price became "the game decides; no row applies" — the C++ fall-through by another door. The unreadable row's target now reaches the effective resolver: where it could beat the game's statement (a more specific level, or no statement) the effective price is `unresolved`, naming the row; where the game's statement is at least as specific the game decides as it would against any row there. `status` says "N unresolved", the `wins` column `?`. (2) **Medium** — after C81 `by_game_reading` had an unreachable `invalid` bucket and `game_priced_not_public` could never count, while `status` read the unreachable bucket and so hid invalid notes: `by_game_statement` (exact, negotiable, skip, none) and `game_priced` replace them, `status` reads `invalid_notes`. (3) **Medium** — `show`'s item sets: an item's view listed itself under "covered below", and a parent tab holding items directly and in substashes showed only one set: the sets are a container's only, and both are listed under their own line. (4) **Medium** — the active plan step 2 and the `listing.rs` "As built" still said `ignore`, and the C69 test's doc said an invalid note is the game side: corrected; the census's `[ignore]` rows stay as history. | (1) `c81_…` (an unreadable item row over a public tab price and an unreadable substash row over the parent's are unresolved; an unreadable folder row under a tab price is not), `c53_price_status_…`; (2) the `Counts` doc, `c69_…`; (3) `c81_…` (an item's empty sets), `the_raw_note_sits_beside_the_parse_in_show` (the mixed parent); (4) this file's plan, `listing.rs` |
| plan 4b outside review 2 | `aad86746` | Two findings, both accepted (the owner, 2026-09-06). (1) **Medium** — the count rename replaced keys under schema 1, against C53's additive rule: the schema is **2** (1 never left development; the const's doc says why), and the shape is pinned by a committed serialized fixture the resolved test snapshot must match exactly — a same-type round trip cannot catch a rename; `ACQ_UPDATE_FIXTURES=1` regenerates it for an additive change, anything else is a bump. (2) **Medium** — the relation sentence still said "no row applies" beside an unreadable row, and an unresolved item with relation `none` was hidden from the default list while `status` counted it: the sentence names the row, the default list includes every unresolved item, `--effective game|manual|none|unresolved` selects by side, the breakdown counts unresolved apart from unlisted, and C81 records the unknown-row precedence (763 bytes after dropping the worked example, which lives in `listing.rs`). | (1) `the_report_json_matches_the_committed_fixture`, `reference/listing-report-schema-2.json`; (2) `c69_…` (the sentence), `c81_…` (the default list, the filter), C81 |
| plan 4b outside review 3 | `eeb4220a` | Two findings, both accepted (the owner, 2026-09-06). (1) **Medium** — the default suppression of relation `none` ran before the `--effective` filter, so `--effective none` could select nothing: the suppression runs only when neither selector is given; all four sides pinned, and the help and the observation say so. (2) **Medium** — the render handoff said "reads `effective.side`", but `none` and `unresolved` both carry no side: the handoff now names the complete outcome — game omitted and counted, manual posted, none off the page, unresolved blocked and counted. | (1) `c81_…` (each side selects on its own; their sum is the item count); (2) this file's observations |
| plan 5 outside review | `0233b18e` | Four findings, all accepted (the owner, 2026-09-07). (1) **High** — the blind `set`/`clear` read only the current revision, so an older binary could replace or delete a newer build's value that the listing state reports as unreadable, and the receipt's "undo" was a value this build cannot write: the blind read now parses the row through the value's own check and refuses one it cannot read, naming the revision and the reviewed path; `--if-revision` stays the deliberate way past it, and the receipt then shows the JSON and promises no undo. (2) **Medium** — the retired-tag rule lived in `set_buyout` while `Annotations::put::<Buyout>` (public, linked by every frontend) did not run it — a convention, not a door: `IntentValue::check_write` is the writer's rule, run by `put` after the parse and never by a read (`ValueError::RefusedForWrite`); `Buyout::check_write` refuses a retired tag; `set_buyout`'s own copy is gone. (3) **Medium** — the printed undo omitted `--if-revision`, so it could overwrite a later edit, and the store's doc said a stale writer "always conflicts" though `put(None)` lands over a tombstone of any generation: the replacement undo names the revision just written; the clear's undo is a create and says so; the claims are narrowed in `put`'s and `set_buyout`'s docs (generations stay with C71/C78, 2b constraint (3)). (4) **Medium** — C73 still said the two legacy rows would be set by hand, the census conclusion repeated it, and the registry indexes (`CONTEXT.md`, this file's preamble) omitted C80 and C81: corrected. | (1) `price_set_and_clear_refuse_a_row_this_build_cannot_read_unless_the_revision_is_named` (`price_json.rs`), `the_write_receipt_says_what_was_and_how_to_put_it_back`; (2) `c66_a_writer_s_rule_refuses_a_new_write_and_never_a_read` (store), `c67_a_new_price_never_names_a_retired_tag_though_a_stored_row_may` (the door refuses; a row stored before the retirement reads); (3) the receipt test; `annotations.rs` `put`, `price.rs` `set_buyout` docs; (4) C73, the census, `CONTEXT.md` |
| plan 4b outside review 4 | `bff8528d` | One finding, accepted (the owner, 2026-09-06). **Medium** — the render handoff said every hand-decided item is posted, but `side_word` folds the manual kinds together: a manual `skip` means leave it out, `no_price` waits on Q5, and a manual price still passes the policy table. The handoff names the outcome by `side` and `kind` together; `Effective`'s doc says so beside the type. | this file's observations; `listing.rs` |
| plan 1 currency table | `10ef2374` | (1) The 2d draft's tags were the C++ app's while the game writes other words for four of them; with zero intent rows on record, the game's word became the tag and the C++ tag an alias, so `tag == emit` on every active row today and a legacy `exa` still resolves. (2) The draft's alias lists (`alts`, `alteration`, `jewelers`, …) were the C++ parser's guesses, not observations — dropped under C68's "nothing more". **Superseded 2026-09-05:** the owner checked shorthand by hand on the trade site and the forum and added it row by row (`browser:` evidence; C68 amended to "each cited"; the count is not pinned, the loader checks the citation). (3) Whether a table is well formed is decided by the loader, not by reading: one word to one row, a `game:` entry and a realm per row, the version stamp; a bad edit fails the tests, never a price. | (1), (2) the file's header; (3) `c68_the_loader_refuses_what_a_reviewer_would` |

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
neither. So the manual desired state was **two rows** (one character
item at 2222 `jew`, one tab ignored) — ruled residue and dropped on
2026-09-06 (question 1 below), so step 5's reading starts from an
empty file; the other 1353 item rows and 17 location rows are
derivations the facts will reproduce once the sale tabs are fetched. `~c/o`: 0 rows.
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

## What validation reading 1 taught (2026-09-07)

The owner set a few rows by hand from an empty intent file — items,
a map substash and a character among the targets — and read them back
through `acq price show | list`. No live run: nothing contacted GGG.
Verdict, verbatim: "I have played around with a few price sets and
listings. It looks good to me, but this is a pretty informal check."
On rigor: "After step 6 I want to implement some more rigorous
testing, but not yet" — agreed, since the render is the consumer that
gives the rows a use and reading 2 is where the surprises will be.
What the reading clarified: a row on a substash or a character is
intent the game cannot express, and the render turns it into one link
per covered item (C70, C74); the substash link code stays a blocked
cell until the owner's hand experiment at reading 2. The two reading-1
questions below (a target the facts do not hold; alias resolution)
were not put to the owner and stay open.

## What validation reading 2 taught (2026-09-07)

**Second pass, the verdict, verbatim:** "That worked. All 27 items
appeared and show up as verified by the website. Horror Spur is empty
under the title. Dread Dome is 99 chaos. I can't see which item came
from I_EXIST, but all 27 items are linked." Read as: every link on the
renumbered page resolved and every item is listed (the character item
among them, so `I_EXIST` as the listing spells it is accepted, T7); a
no-price item under an empty spoiler title is listed with no price
(T21); a hand price beats the public tab's name on the site (T12,
confirmed). Not said, and not pressed: whether the coverage, stale and
positions lines changed what the owner did — the page was rendered
minutes after the refresh, so none of the three had anything to say.
The done criterion's "read correct on the trade site in the owner's
words, with every omission counted" is met.

**First pass:**

The owner pasted a page into the shop thread and read the result on
the trade site; the render's page was reshaped from what came back.
Verbatim: "Q3: the web stash view doesn't allow me to select items
from map or unique stashes at all"; "Q5: items linked to the forum
without a price annotation are listed with 'No Price Set' on the trade
site."; and the shape: "Items with the exact same price should be
listed together; all prices should be wrapped in a spoiler tag;
newlines only between spoiler tags; each page should be wrapped in a
spoiler", with the stored post as the example (T22). What changed: the
substash cell is blocked on T20 instead of Q3; the no-price cell posts
(T21); the page is nested spoilers (schema 3). Asked and answered the
same day, verbatim: "the spoiler text is used as the price for every
item within that spoiler block" (T22 upgraded, `8c312138`) — so the
spoiler title is where the indexer reads the price, as the C++ app had
it (T15). The second pass's first paste (Allflame, after the refresh
in the run ledger) answered Q1 the hard way: the first link, `Stash18`
for an item in the tab inside the owner's folder, did not resolve; the
website's own button wrote `Stash17`, and the owner's saved copy of the
website's tab list (now `reference/website-tabs-2026-09-07.json`) showed
the rule — the site numbers the tabs it lists from 0 with folders and
substashes absent, and the link is that number plus one (T24). The
render now ranks the listed tabs the site's way and writes the
attributes in the site's order (`1c85cdcb`). Still to read on the
reshaped, renumbered page: the no-price item under an empty title, a
hand price against the public tab's name on the site (T12), the
character link's name case (`I_EXIST` as listed, `I_Exist` as the site
wrote it, T7), and the owner's verdict, with whether the coverage,
stale and positions lines changed what they did.

## What the site taught (step 7, item 5, 2026-09-08)

The seller-account search for this account in Standard, saved by the
owner in eight parts and joined by item id against the listing state of
the same day (the ledger row above; `tools/site-join.py --rows` lists
every row below). The test the brief set (`brainstorming-notes/15`): a
difference earns a rule only if it flips a render verdict for some
item. None did. What follows is each reason with its verdict, the
trigger that would make it more, and the claim the owner may author
master-side (`docs/design/trade-ground-truth.md`; never from here).

1. **`metadata.public` is the whole condition** (Q11). 547 stash rows,
   every one from one of the 13 tabs the facts hold public; the 13
   non-public priced tabs — `~price 30 chaos (C)`, the twelve
   `(Remove-only)` names, 1,121 items — put nothing on the site, nor
   did any of the 114 other residue notes. C81 stands on it. Claim: the
   seller-account search shows a stash item only from a public premium
   tab; a priced name on a non-public tab reaches nothing [RUN, 09-08].
2. **A ratio in a public tab's name lists nothing on the item search
   and offers the tab's exchange-eligible stacks on the bulk exchange**
   (T11 corrected, T2 confirmed for a tab name): the two public tabs
   `~b/o 5000/2 chaos` (56 items) and `~price 1000/2 chaos` (70) show
   two item-search rows, both items with their own `~price 1999 chaos`
   note (the note beats an invalid name), and the exchange page offers
   the first tab's Infused Engineer's Orbs and Tailoring Orb as "2 for
   5000 chaos" — `wanted/lot` read as T2 says, under `~b/o` too (half
   of Q6). Its two Vials are offered nowhere: not exchange-eligible
   (Q10's list). Our state calls all 122 "public, the tab applies, no
   statement". For the 120 that are not stacks the render is right
   either way (nothing applies, or a row posts what the site does not
   show). For a stack the site offers, a manual row would post a forum
   price against a standing exchange offer — the contradiction C74
   forbids, for a case the owner has not created. Without Q10's list
   the safe render rule is a blocked cell: a stack in a ratio-named
   public tab is "the exchange may list this", counted, never posted.
   Trigger: a manual row on such a stack. Claims: the two surfaces of a
   ratio name; `~b/o` with a ratio accepted in a tab name.
3. **A socketed item is not listed on its own**: 107 gems in sockets of
   items in public tabs, none on the site — 32 in `3.15 Bane
   Pathfinder`, 26 in `3.10 ED/C Trickster`, 24 in the 8888 tab (where
   our state says each is listed at 8888 chaos), 25 in `~price 1000/2 chaos`.
   The render omits them as game-listed before the socketed cell is
   reached (the cell is for a hand-priced one, T13); the listing state
   counts them as the game's. Trigger: the same count consumer. Claim: the site
   indexes a socketed item with its host, never as a listing.
4. **A currency-class stack is not in the item search; a priced one is
   on the bulk exchange**: eight stacks the state expects are absent
   from the search — Scrolls of Wisdom (one carrying `~b/o 1.5 divine`,
   the test tab's second `~b/o`), a Runegraft, Valdo's Puzzle Box, two
   Vials, a Tailoring Orb, an Infused Engineer's Orb, and an Omen (frame
   type 5, no stack size; the join reads it `unlisted_since` since the
   report carries no frame type). The exchange page (read the same
   evening) offers the noted scroll at 1.5 divine with **stock 40** —
   its 21 plus the 8888 tab's unpriced stack of 19: T3 confirmed, stock
   counts every public stack of the type, priced or not — and the two
   ratio-tab stacks (point 2); the five unpriced stacks in unpriced
   tabs and the two Vials are offered nowhere. Trigger: the owner prices
   currency. Claims: the search omits stacks; the exchange offers the
   priced ones; the stock count.
5. **The site reads a price out of a note with trailing text**: `~price
   777 chaos testing` on Memory Vault in the 8888 tab shows "Price with
   Note: 777 chaos" — a fourth label beside Exact, Asking and No Price
   Set — where our parser reads the note `invalid` and lets the tab's
   8888 apply. The effective price is wrong by 8,111 chaos; the page is
   not, since the game lists it either way. The fix is one branch in
   `game_side.rs` (a note tolerates a suffix as a tab name does — the
   C++ regex did) and one test flipped
   (`c69_a_tab_name_tolerates_trailing_text_and_a_note_does_not`); it
   touches C69's mechanism, and T17 stays true of the dialog. The
   owner looked in game the same evening: the dialog shows `~price 777
   chaos`, the API and the site hold the whole text, and a fresh
   `~price 666 chaos tested` behaved the same — the dialog displays
   the parsed part and stores the whole. **Built as parser v2** on the
   owner's approval (question 6; the ledger row below). Claim: T17
   rewritten as a display fact; the label and the reading. The same
   row answers Q2 for one case: the tab's price is valid (8888) and the
   site shows the note's (777) — a note beats a valid tab name, the
   order C69 took from the C++ code; the clean-note case is the
   experiment below.
6. **`facetors` is a word the site does not read**: `~price 999
   facetors`, written by the game's own dialog (the price-notes corpus),
   lists as "No Price Set". T16's loose matching has a hole where the
   game's word and the site's differ wholly. The page is unchanged (the
   game lists it, at no price). Owner, 2026-09-08: a bug, reported to
   GGG, unlikely to change before the next release, and a currency
   nobody prices in — ignored; the table's row stays as it is.
7. **Eight words the site spells differently and resolves**:
   `excep-ember`, `grand-ember`, `greater-ember`, `lesser-ember`,
   `excep-echor`, `grand-echor`, `greater-echor`, `lesser-echor` list at
   the right currency under the site's ids (`exceptional-eldritch-ember`,
   … `-ichor`). T16 widened by eight; nothing to change — acquisition
   writes the game's words and the site reads them.
8. **The site drops the fraction on chaos**: `999.1234`, `999.123`,
   `999.12`, `999.1` show "999"; `1.4 divine` keeps its fraction. A
   display fact; the amount is the seller's (C67). Claim, once a second
   currency is seen either way.
9. **The forum rows**: four items in non-public remove-only tabs, from
   two threads, all "Asking Price" (the owner's hand posts of the
   reading-2 shape), "listed last month" and "2 months ago" though the
   posts are days old — the site dates a forum listing by something
   other than the post. The exchange page shows the forum's other face:
   five stacks of the same non-public tab (four Runegrafts and the
   Wombgift, which is on both pages) offered at 4321 blessed each. T8
   and T12 at nine rows; the manual side holds no row for them, as
   expected of hand posts. No claim yet.
10. **One row no reason explains**: Lethal Pride (a Timeless Jewel,
    unique, item level 86) in the 8888 tab — not socketed, not a stack,
    the tab public and 32 of its neighbours listed — has no row on the
    site. The owner searched the site for it by hand the same evening:
    listed nowhere. Left as it is.
12. **C80 holds on the site, all three ways** (2026-09-09). `Uniques 1`
    made public and named `~price 4554 chaos`: 326 of its 370 uniques,
    in 19 substashes, listed at 4554 chaos — a substash reads its
    parent's name and flag. Folder `3.19` named `~price 3333 chaos`
    with `3.19 Cursebot` public and unpriced inside it: 14 of 18 listed
    as "No Price Set" — a folder's name is never a price. `3.19 Helix
    Raider` named `~price 2222 chaos` and public inside that folder:
    9 of 9 at 2222 — a folder child reads itself. C80's provisional
    clause is closed in the registry. Claims: the three.
13. **A note beats a valid tab name** (Q2): Memory Vault, re-noted
    `~price 666 chaos tested` in the 8888 tab, lists as "Price with
    Note: 666 chaos"; parser v2 reads 666 and the join agrees. The
    clean-note case (no suffix) was not run; the site's reading of a
    suffixed note is the same mechanism, so Q2 is answered for the
    note-wins direction. Also seen: the remove-only unique tab renamed
    by mistake to `~price 4444 chaos (Remove-only)` cannot be public
    and lists nothing — one more residue tab.
14. **The exchange's row is per item type per account.** The scroll
    row carries both offers, 1.5 divine (the test tab's noted stack)
    and 8888 chaos (the 8888 tab's unpriced stack, at the tab's
    price), under the test-tab stack's id; the ratio tab's stacks show
    per unit (2500 for 1). Claim: the grouping.
15. **63 items in six tabs are absent from the capture and no site
    rule explains them**: 40 of the priced unique tab's 370, 9 of the
    8888 tab's (all listed the day before, Lethal Pride among them), 4
    of Cursebot's, 4 of MARKET's, 3 of Bane Pathfinder's, 3 of Unique
    Rings' — none socketed, none stacks, three of the tabs untouched
    since the day before, when the same tabs were complete. The one
    reading that fits all six is the capture: a search paged in
    thirteen parts while the site's order shifted underneath drops
    rows between parts. **Closed 2026-09-09**, the owner: "The indigon
    and ungil's harmony show up on the trade site, but it's possible my
    download was done wrong since it was heavily manual with lots of
    button pushes to keep track of." The union is the hole; the site
    lists them; no rule. A capture's coverage is a number the join
    prints, never a site fact — the first day's, complete in every
    public tab, stays the count that pins the domain.
16. **One character item on the site**: Rune Gorget, on a character in
    our facts of the day before with the note `~price 1 facetors`,
    shows as "No Price Set", listed three hours before the capture,
    through the stash channel. **Closed 2026-09-09**, the owner: "I
    changed the price on the rune gorget as part of my bug report on
    the facetor pricing, which included moving it out of the
    character's inventory to a stash tab." Our characters were a day
    old; the site was right. The move-since case C72 reports, seen
    once, from the other side.
11. **Coverage**: the item search's union is 551 rows against the
    owner's "560 matched" read off the form on 09-07; the saved DOM
    carries no matched count, and the 8888 tab was set public between
    the two days. The arithmetic closes: 547 stash rows = 532 agree + 15
    differ; the exchange's 8 offers = 3 agree + 5 forum; 757 expected =
    547 + 3 + 207 state-only; 207 = 93 ratio-tab items + 107 socketed +
    5 stacks + 2 (the Omen, Lethal Pride).

## Observations still open

Agent observations that became neither a ruling nor a finding; each is
data for the step that touches it. The hand experiments are the
owner's, one each, when the render step needs them; the forum preview
shows the item picture, so a wrong link is visible before posting.

- Still blocked-and-counted in the render's policy table until
  observed: what the indexer does with a forum ratio on a non-bulk
  item (Q6; `~b/o a/b` is accepted in a tab name, "What the site
  taught" 2), and a game `skip` against a manual price (Q7).
- The game-side parser's real corpus is the test tab (fixture) plus the
  userstore's 120 notes; the facts hold the sale tabs listed, not
  fetched, until the policy covers them. Two of its rules were the
  parser's own and are now observed (2026-09-06): an unreadable `~`
  note has no effect and the tab applies (T18 — the parser still reads
  it `invalid`, the listing state treats `invalid` and none alike);
  a note's suffix is tolerated as a tab name's is (parser v2,
  2026-09-08: the game stores it, the dialog displays the parsed
  part, the site reads through it — T17 to be rewritten as a display
  fact). Still the parser's own: `~skip` as a tab name reads `skip`. The indexer's loose word matching (T16) is not
  modelled; a hand-typed alias reads `invalid`, shown verbatim.
- C74's omission rule is built (`shop.rs`, the policy table): the
  render reads `side` and `kind` together, and each blocked cell names
  the question or claim that holds it. The page's shape is the owner's
  (T22, 2026-09-07); two choices inside it are still the render's own:
  the group order (`~price`, then `~b/o`, then no price) and the empty
  spoiler title for a no-price item (the C++ app's, T15) — whether the
  site lists that item as "No Price Set" under an *empty* title, as it
  did under none (T21), was reading 2's second pass: it does (Horror
  Spur, "What validation reading 2 taught").
- "Moved or reindexed since the render's basis" (C72) has no stored
  basis to compare with ("what did I last post" is parked, v1 reposts
  whole pages), so the render reports the one it has: a posted stash
  item whose fetch predates the stash listing its `Stash<n>` comes
  from — the link's two halves observed at different times. Whether
  that line ever changes what the owner does is reading 2's evidence.
- The store's `tabs.idx` falls back to the listing position when an
  entry carries no `index` (T13 says every one of 402 did), so the
  render cannot tell a real index from the fallback; a tab with `idx`
  null was fetched directly and never listed, and its items are the
  `tab_unlisted` cell. Since T24 the index only orders the tabs — the
  rank among the listed non-folder tabs is what a link carries — so the
  fallback would matter only if it reordered them.
- T24 held at scale the same day: the owner's Standard list (322
  tabs, 16 folders interleaved, 64 substashes and 274 remove-only tabs)
  ranks every listed tab at its site number plus one; both lists sit
  beside the API's listings in
  `crates/acquisition-plan/reference/website-tabs-2026-09-07.json`,
  pinned by `t24_stash_numbers_match_the_websites_own_list`.
- `priced_tabs` counts names that read as a price whether or not the
  tab is public (the owner's 13 remove-only tabs are the case), beside
  `priced_tabs_public`; whether `status` should lead with the residue
  count for such a stash is a reading-1 question.
- The 0.18 userstore's 120 notes and 17 tilde tab names are not in the
  fixture (the census read shapes, never texts); a later census that
  writes them there, verbatim, would widen the parser's pin from the one
  test tab to the owner's real corpus.
- The C++ userstore stored `last_update` as text in an INTEGER column
  (Qt bound a `QDateTime`); nothing to fix, noted.
- `acq price list` leaves relation `none` out when neither `--relation`
  nor `--effective` is given, except unresolved items, always listed
  (the owner's data holds 26k items and ~1.4k listed); `--relation none`
  or `--effective none` asks for the rest. Whether "unlisted" should be
  visible by default is a reading-1 question.
- `set` lands on any well-formed target, whether or not the facts hold
  it: intent is league-less (C67), never gated by facts (C64), and a
  price on a tab the policy has not fetched yet is legitimate (C72
  reports it). A typo'd id therefore lands too, and `status` counts it
  under "name nothing in these facts". Whether `set` should say so at
  write time (a note, never a refusal) is a reading-1 question.
- `set`'s type words are the C67 vocabulary plus the game's (`price`,
  `b/o`, `~price`, `~b/o`, `~skip`); the amount and currency are read by
  the value's own parse, so the CLI has no second grammar and an alias
  is refused naming the tag rather than resolved. Whether the owner
  wants the alias resolved for them is a reading-1 question.

## Questions for the owner

1. ~~Are the two manual rows (a character item at 2222 `jew`; one tab
   ignored) real intent to set by hand at step 5, or test residue to
   drop?~~ **Answered 2026-09-06**, owner verbatim: "Let's call them
   residue to drop. I'd rather start fresh. The oddball pricing tells me
   I was using it to debug something." The 0.18 userstore's manual rows
   are not carried; step 5's reading starts from an empty intent file.
   (The C++ combo's `[Inherit]` was the clear; the ignored tab's row
   still carries 4321 `blessed` underneath.)
2. C73 parked as "a 0.18 user asks": pricing is niche, but the 0.18
   import is a product question for other users, not only yours. Park
   stands unless you say otherwise.
3. ~~Reading 2, second pass.~~ **Answered 2026-09-07**, verbatim in
   "What validation reading 2 taught": all 27 items linked and listed;
   Horror Spur empty under the title; Dread Dome at 99 chaos.
5. ~~Close the slice once PR #227 merges, or keep it open through the
   rigorous test pass?~~ **Answered by the course taken**: PR #227
   merged 2026-09-07; the record stayed open through step 7 (items 1–5,
   the last read twice, 2026-09-07..09); the owner's outside review of
   2026-09-09 named the closeout blocker — the site facts authored
   master-side — and two consistency items, fixed. The cut follows.
   Still owed: your verdict on the page, verbatim, with whether the
   coverage, stale and positions lines changed what you did.
6. ~~The site reads `~price 777 chaos testing` as 777 chaos ("What the
   site taught", 5); amend C69's mechanism so a note tolerates trailing
   text as a tab name does, or park it?~~ **Answered 2026-09-08**,
   owner: "I approve the parser and list changes" — parser v2 built,
   the exchange list committed as a proposal. C81's *Evidence:* pointer
   left off on the agent's recommendation (the ruling cites T1 and T11;
   the evidence is the ledger row).
4. ~~C68's "enumerable through every surface" against the parked MCP
   step.~~ **Answered 2026-09-07**, owner: "agree to narrow" — C68 now
   reads "enumerable by every surface built" (795 bytes), the code's
   copies follow; the MCP step stays parked on its trigger.
