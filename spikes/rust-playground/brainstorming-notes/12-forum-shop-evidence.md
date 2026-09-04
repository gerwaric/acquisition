# 12 — Forum shop evidence, read before step 3 (2026-09-03)

Deliberation, disposable (`AGENTS.md`, routing) — **but not yet deletable**: E1–E6, E10 and E18–E22 are facts about GGG awaiting `T<n>` claims master-side, and the sweep's verdicts moved to their homes on 2026-09-04 (`decisions/pricing.md`, `PRICING-SLICE.md`, `SURFACES.md`); delete this note once the claims exist. The owner paused the
slice to bring in what the trade site and the forum actually do, so
the framework harvested at step 1 could be checked against it before
step 3 builds on it. This note is the evidence, item by item with its
source, then what each item does to the rulings, then the recommended
path. Nothing here is a ruling; facts about GGG become `T<n>` claims
master-side, and the owner's in-game observations are marked as such.

## Sources, and how each was read

| Source | Kind | How obtained | Date |
| --- | --- | --- | --- |
| `pathofexile.com/trade/about` | official GGG page, not the API reference | the owner saved the page in a browser, logged in as `GERWARIC#7694`; the agent read the saved copy in `pricing-info/` (a `browser` read under C79 — the About text sits in a Vue `x-template`, so a plain text strip loses it) | saved 2026-09-03 19:21 local |
| PoE Wiki, *Guide: Listing items for sale using the Path of Exile trade forum* | third party, CC BY-NC 3.0; "last updated March 7, 2022", page edited 2024-12-01 | same, `pricing-info/` | saved 2026-09-03 |
| `pathofexile.com/developer/docs/reference` | the sanctioned API reference | fetched by the agent (a sanctioned surface) | 2026-09-03 |
| Procurement, `github.com/Procurement-PoE/Procurement` | third-party tool, Artistic-2.0; last release 1.29.2 on 2022-12-22, no commits since | files read through the GitHub API | 2026-09-03 |
| Acquisition C++ on `master` (`src/shop.cpp`, `src/itemlocation.cpp`, `src/buyout.cpp`, `docs/user/forum-shop.md`, `docs/user/pricing.md`) | the project's own prior reading | `git show` | — |
| the owner's in-game observations (this conversation) | owner, verbatim below | — | 2026-09-03 |
| the spike's facts for `GERWARIC#7694` | read-only `immutable=1` open, WAL empty | — | listing of 2026-09-03 11:36 UTC |

## Evidence items (E1–E17; candidates for `T<n>` claims where the source is GGG)

**From the trade site's About page (official).**

- **E1.** "This website allows you to search through items listed for
  sale in Shop Forums as well as those listed using Public Premium
  Stash Tabs in-game." Two listing channels, no third.
- **E2.** Bulk Item Exchange: "Certain items may be listed for bulk sale
  (listed below). Your eligible items will be grouped together under
  one price, with the total amount of items presented as 'stock'. In
  order to list exchange entries, either in-game or on the forum, use
  the ratio format on the item you wish to offer for the type of item
  you would like in return."
- **E3.** The two worked examples: a Chaos Orb stack of 10 with
  `~price 3/1 alch` "will list your intent to buy 3 Orbs of Alchemy
  for 1 Chaos Orb"; an Orb of Transmutation stack of 20 with
  `~price 2/35 chaos` "will list your intent to buy 2 Orbs of Chaos
  for 35 Orbs of Transmutation. Though this is technically equivalent
  to selling 17.5 Orbs of Transmutation per Chaos Orb, pricing this
  way is more appropriate for bulk sets of items." So the grammar is
  `~price <wanted>/<lot of this item> <currency wanted>`, and **the
  denominator is a lot size, not a divisor**: `2/35` is deliberately
  not `1/17.5`, and by the same token `3/1` is not `3`.
- **E4.** "Stock includes all of the items of that type that are
  publicly listed (priced or not)."
- **E5.** The eligible-item list ("Item Tags – *group*") is rendered at
  runtime from `exchangeOptions`; it is **not in the saved page**. A
  human can expand the groups in the browser and record them; the
  tool source behind it is the static-data endpoint already rejected
  under C79.
- **E6.** The site's realm list is `pc`, `xbox`, `sony` (PoE 1 only);
  its league list includes Allflame, Ruthless and hardcore variants.

**From the wiki guide (third party, 2022).**

- **E7.** The indexer reads "the Public stash tab API, as well as ...
  the forums (particularly the trading subforums)"; "a few times every
  minute"; it "has a limit to how many threads it searches", hence
  bumping. Threads live in per-league shop subforums ("Standard League
  - Shops"); "the title of the post does not matter".
- **E8.** The link code, obtained by clicking an item in the website's
  stash view: `[linkItem location="Stash2" league="Standard" x="5" y="0"]`,
  followed by the price text, e.g. `~b/o 3 chaos`. The forum "has a
  system to link items from your game stash to your posts, which helps
  verify if the item is still available".
- **E9.** "This procedure has been automated by various inventory
  management tools." (The link goes to the wiki's application list.)

**From the API reference (sanctioned).**

- **E10.** `Item` carries both `note` (?string, "user-generated text")
  and **`forum_note`** (?string, "user-generated text"). `Item` objects
  come back from `/stash`, `/character` and `/public-stash-tabs`.
  `StashTab` has `index` and `metadata`; `PublicStashChange` has
  `public`, `accountName`, `stash` (the tab name).

**From Procurement and the C++ app (prior readings of the same folklore).**

- **E11.** Both emit the same two link shapes: stash
  `[linkItem location="Stash<n>" league="<L>" x= y=]`, character
  `[linkItem location="<inventoryId>" character="<name>" x= y=]`.
  The C++ app adds `realm="<r>"`; Procurement and the wiki do not.
  Neither is evidence of what the forum *requires*.
- **E12.** Price text after the code: `~b/o`, `~price`, `~c/o`, one per
  line (Procurement) or as a `[spoiler="~b/o 5 chaos"]` header
  grouping items (C++). The C++ post limit is 50,000 characters, one
  `[items]` template token, and game-sourced prices were **never
  posted** (`Buyout::IsPostable` excludes `source == game`).
- **E13.** Procurement's currency map has 16 abbreviations, a strict
  subset of the C++ 19 (no `coin`, `mirror`, `silver`); its tab
  buyouts are keyed by tab **name**.

**From the facts (this account, listing of 2026-09-03 11:36 UTC).**

- **E14.** The 12 tabs with `metadata.public` and the 17 tabs whose
  name parses as `~price …` are **disjoint** — every in-game tab price
  the owner has is on a non-public (remove-only) tab. Under E1 the
  trade site cannot see any of them from the stash; a forum post is
  their only channel. This is the C++ user doc's claim ("including
  items in remove-only tabs and character inventories, which the site
  does not index from your stash directly") seen in the owner's own
  data.
- **E15.** Every stash item's `inventoryId` is the literal `Stash1`,
  whatever its tab or substash (792 of 792). The `Stash<n>` in a link
  code cannot come from the item; it has to be derived from the tab's
  `index` (the C++ app uses `index + 1`). Character items carry the
  slot (`MainInventory`, `Weapon2`, `PassiveJewels`, …); socketed items
  (24 stash, 531 character) have neither position nor `inventoryId`.
- **E16.** `forum_note` appears on zero stored items; `note` appears on
  zero stored items. The owner's `ACQUISITION-PRICE-TEST` tab is not in
  the facts — it postdates the last listing.
- **E17.** The owner's observations, in game, 2026-09-03 (verbatim):
  "only individual items can be listed in forum shops, not entire
  tabs"; "only bulk-tradeable items can use the 'X/Y' format for
  price"; "items priced in forums take precedence over the tab if the
  tab has an in-game price"; "tab prices can only be set in-game by
  renaming the tab". And: a tab named `ACQUISITION-PRICE-TEST` prices
  one item in each currency the in-game dialog offers, plus "several
  other oddball pricing notes".

**From the owner's second message (2026-09-03, verbatim where quoted).**

- **E18.** The website's link button today emits
  `[linkItem realm="pc" location="BodyArmour" character="I_Exist" x="0" y="0"]`
  — so `realm=` is current (the C++ app was right; the wiki and
  Procurement are stale). Once the post is saved, the edit view shows
  `[item post="26816146" index="1"]`: the forum **resolves the link
  into a stored item bound to the post and an ordinal**. Our rendered
  text is therefore never what the forum holds; "did anything change?"
  can only be answered against our own previous output (the C++ hash),
  and whether the forum re-resolves an item that later moves is a
  matrix cell, not an inference.
- **E19.** That item, on character `I_EXIST` in Allflame, "shows up on
  the official site as a trade listing". A character-inventory item is
  listable through the forum, confirmed on this account. The character
  is listed in the facts (11:30 UTC) and **never fetched** — one GET
  `/character/I_EXIST` is the `forum_note` probe, already set up.
- **E20.** "The in-game tab prices may be invisible, but every item in
  a priced tab shows up in the trade listing unless the tab pricing is
  invalid. It looks like tab prices do not support the 'X/Y' syntax."
  So a public tab with a valid name price lists each item at that
  price, and a **ratio in a tab name invalidates the tab's pricing for
  every item in it**. (E14 still stands for the non-public remove-only
  tabs unless the trade site's seller-account filter shows otherwise —
  that search, run in a browser, is the oracle for the whole listing
  derivation.)

**From the price-notes run (2026-09-04 00:51 UTC, ledger row; `runs/2026-09-04-price-notes/notes-check.txt`).**

- **E21.** The in-game price dialog's vocabulary, as GGG's client wrote
  it into `note` on the owner's `ACQUISITION-PRICE-TEST` tab (a public
  premium tab with no price in its name; 80 items, 50 notes), 39
  distinct words, verbatim: `chaos`, `chance`, `offer`, `facetors`,
  `divine`, `annul`, `offer-dedication`, `engineers`, `alch`, `aug`,
  `offer-gift`, `infused-engineers-orb`, `exalted`, `regret`,
  `offer-tribute`, `alt`, `scour`, `lesser-ember`, `mirror`,
  `transmute`, `greater-ember`, `chrome`, `wisdom`, `grand-ember`,
  `blessed`, `portal`, `excep-ember`, `fusing`, `scrap`, `lesser-echor`,
  `jewellers`, `whetstone`, `greater-echor`, `regal`, `gcp`,
  `grand-echor`, `vaal`, `bauble`, `excep-echor`. Against the C++ 19:
  twelve identical (`chaos`, `chance`, `divine`, `alch`, `alt`, `scour`,
  `regret`, `mirror`, `blessed`, `regal`, `gcp`, `vaal`); four spelled
  differently (`exa`→`exalted`, `chrom`→`chrome`, `jew`→`jewellers`,
  `fuse`→`fusing`); three the dialog does not offer (`chisel`, `coin`,
  `silver`); 23 new. The `echor` spelling is the game's, to be checked
  against the trade site's ids, not corrected. Amount and shape edges
  in the same tab, verbatim: `~b/o 999 chaos` (negotiable), `~b/o 1.5
  divine`, `~price 1.4 divine`, `~price 999.1 chaos`, `~price 999.12
  chaos`, `~price 999.123 chaos`, `~price 999.1234 chaos` (twice),
  `~price 12345 chaos`, `~price  chaos` (empty amount, two spaces),
  `~skip ` (trailing space). The owner's word on them (2026-09-04):
  the empty amount is what the game leaves after an invalid entry,
  such as a ratio on a non-bulk item; `~skip` is the game's "Do not
  index" choice; the five-digit amount is the game's; prices should be
  rounded to two places in Acquisition even though the game accepts
  more; `chisel`, `coin` and `silver` are no longer in the game; forum
  posts have been seen with `mir` for mirror, so the indexer matches
  loosely, and we do not model that. No ratio note is present (the
  dialog does not offer one on incubators, E17). Thirty items carry no
  note.
- **E22.** `/character/I_EXIST` returned the body armour (Foulborn Skin
  of the Lords, `BodyArmour` (0,0)) with **no `forum_note`** and no
  `note`, though the owner's forum post links and prices it and the
  trade site lists it. Nothing else in the store carries `forum_note`
  either. So the character endpoint does not report forum listings;
  whether the public-stash stream does is moot for us (we never read
  it). **The forum is write-only from our side**, and C69's game side
  stays two sources: item note, then tab name.

## What each item does to the rulings

- **C68 (currency table): `emit` now has its source** (E21). The
  renderer should write what GGG's own client writes; the C++ tags
  become aliases where they differ (`exa`, `chrom`, `jew`, `fuse`), the
  three absent tags keep their rows (never removed, C68) with the
  evidence that the dialog no longer offers them, and 23 rows are
  added with `game:2026-09-04 note in ACQUISITION-PRICE-TEST` as their
  evidence. Display names for the new rows still need a source (the
  trade site's Item Tags list in a browser, or the owner naming them).
- **C69 (listing state): the game side is note then tab name, final**
  (E22); a forum listing is intent we hold, never a fact we observe.
  Step 5 builds on two sources. "Did my post index?" is answered on
  the trade site by the owner, and the seller-account search remains
  the oracle.
- **C67 (amount): the game emits decimals to four places and empty
  amounts** (E21). The canonical text must accept up to four
  fractional digits at least (whether the dialog truncates a fifth is
  the owner's to say); an empty amount is a parse failure the listing
  state reports, not a value.
- **C69 (game side): a tab-name price parses to valid or invalid, and
  invalid means no game listing for the tab's items** (E20) — the same
  shape C69 already gives an unparseable note. **C67:** a ratio is a
  legal manual value on any target (the forum posts per item, C70
  expands), but the *game* accepts it on items only; the render's
  policy for a ratio on a non-bulk item stays a matrix cell.
- **C74 (render): the link code carries `realm=`** (E18); the forum's
  post-time resolution (E18) means the render's "unchanged since last
  paste" is a hash of its own output, recorded as the render's basis,
  never a comparison with the forum.


- **C67 (the `buyout` value): the ratio is a pair, not a rational.**
  The packet's mechanism says canonical text is "`a/b` reduced with
  `b > 1`" and "equality is on the reduced rational". E3 contradicts
  both: `2/35` must stay `2/35`, and `3/1` must not collapse to `3`.
  So `amount` needs two shapes — a scalar (the decimal canonical text)
  and a **lot ratio** of two positive integers kept as written — with
  structural equality. This is the one correction that must land
  before Buyout v1 freezes (step 4). It also answers `PRICING-SLICE.md`
  question 1: the three `~price a/b chaos` notes are meaningful and are
  kept as pairs (22 chaos per lot of 10, 55 per 600, 10 per 80).
- **C68 (reference data): two tables, and a better source for one
  column.** (a) The currency table's `emit` column has a source
  stronger than any browser read: what GGG's own client writes into
  `note` when the owner uses the in-game dialog. The census already
  showed it writes `exalted`, never `exa`. One refresh that lists and
  fetches `ACQUISITION-PRICE-TEST` yields the emitted word for every
  dropdown currency at once, plus any currency the C++ list lacks
  (additive rows). `aliases` stay the parser's business and still come
  from claims. (b) The bulk-exchange item list (E2, E5) is a second
  reference table if the system ever gates ratios by item class. Its
  only source is a browser read of the expanded groups. Recommend
  parking it (trigger: the owner prices a non-currency item by ratio)
  and not gating the value: a ratio on any target is accepted; what
  the exchange does with a ratio on an ineligible item is a matrix
  cell.
- **C69 (listing = two resolutions and a relation): a possible third
  game-side source.** E10 puts `forum_note` in the same schema as
  `note`. If `/stash` or `/character` returns it for an item the owner
  has linked in a forum post, then a forum listing is an observable
  fact through the sanctioned API: the game side becomes
  `forum_note` → `note` → tab name, and "did my post index?" is
  answered from facts, never from the forum. If it does not, the field
  is public-stash-stream only and the forum stays write-only from our
  side. Either way the design holds; which world we are in decides
  what step 5 builds. One post by hand, one refresh, one grep.
- **C69 / C74 (the publication policy): E17's third observation is the
  first evidence about a relation.** "Forum beats tab" says a manual
  price posted on an item whose tab carries an in-game price is
  effective, not a no-op. That is a candidate row of the policy table
  (relation `conflict`, game side = tab name → include, forum governs).
  It needs a `T<n>` claim from the matrix before it is ruled; note it
  is the opposite direction from the C++ lock C69 rejected. Forum
  versus item *note* is unknown.
- **C70 / C74: tab-scoped intent never reaches the forum as a tab.**
  E17's first and fourth observations plus E8: the forum knows only
  items, and a tab-wide game price is a tab-name fact set in game. So a
  tab-scoped manual row is purely the coverage device C70 already
  describes, expanded by the render into per-item link codes; there is
  no "tab price" in the rendered page. This simplifies the render.
- **C74 (render) / C72 (freshness report): forum identity is
  positional.** E8, E11, E15: a link is `(league, tab index + 1, x, y)`
  or `(character, slot, x, y)`, so it breaks when an item moves, a tab
  is reordered, or a league merges. The store already has what the
  report needs — `item_events` (moves), `tabs.idx`, `removed_at`, the
  response basis — so the render must carry its basis and the report
  gains a sharper line: "moved or reindexed since the render basis".
  Eligibility reasons E15 adds: socketed items (no position), items in
  substashes (how the forum addresses a child of a map or unique stash
  is unknown — a matrix cell), and any item whose tab has no `index`.
  The `realm` attribute is unverified (C++ only).
- **C74 (policy): the game-only relation is the one that matters for
  this owner.** E12 and E14 together: the C++ app never posted
  game-sourced prices, and every one of the owner's game-priced tabs is
  invisible to the trade site. So whether the render includes
  `game-only` items from non-public tabs decides whether it covers two
  items or about thirteen hundred. That is the owner's call and the
  first policy row worth ruling; until then it is `blocked-unruled`,
  counted.
- **C73 (import): unchanged.** Game rows stay non-actions; the import
  is not how those prices reach the forum, the render policy is.
- **C79 / `SURFACES.md`: the register gains rows.** The About page as a
  dated browser read (the saved copy is the observation); the wiki and
  Procurement as third-party corroboration, never authority; the
  developer docs are sanctioned and need no row.
- **Publishing (parked, `CONTEXT.md`): confirmed as its own boundary.**
  E7 adds bumping and the thread limit to what publishing would own;
  nothing here moves it forward.

## What the evidence does not settle (matrix cells, some now cheap)

1. Does `/stash` (or `/character`) return `forum_note` for a
   forum-listed item? — one hand post, one refresh, one grep. **Do
   before step 5.**
2. The word GGG's client writes for each dropdown currency, and the
   full dropdown list. — refresh `ACQUISITION-PRICE-TEST`. **Do before
   step 4.**
3. Forum price versus item note (E17 covers only forum versus tab).
4. A ratio on an item outside the bulk list: ignored, listed, or
   grouped? And `~b/o a/b` versus `~price a/b`.
5. The link code for a substash item; whether `realm=` is needed or
   harmful; the post size limit; the `Stash<n>` numbering under folders
   (E15: is `n` the tab's `index + 1` when folders occupy indices?).
6. The bulk-exchange item list itself (E5), if ever gated.

## Recommendation: no rewind; four amendments and two observations first

The framework survives this reading because it was built to: game
pricing is a derivation over facts (C64), consumer rules come from
claims (C69, C74), the render is a pure function (C74), publishing is
outside (C79, parked). What changes is specific:

1. **Amend C67 before step 4**: `amount` is a scalar or a lot ratio;
   the ratio is an unreduced integer pair; equality is structural.
   (Answers question 1 of the slice record.)
2. **Add the render's basis and the positional-staleness line to C72's
   report** (a sentence on C74; the mechanism to the code at step 8).
3. **Record E17 as owner observations** in the slice record, marked as
   such, each pointing at the matrix cell that will make it a claim.
4. **Register the About page read and the corroborating sources** in
   `SURFACES.md`; open the master-side `T<n>` file with E1–E6 and E10
   when the owner next authors claims.

And re-order two observations ahead of code, both human-run: the
refresh that brings `ACQUISITION-PRICE-TEST` into the facts (a live
run under the procedure, nothing new to build), and the `forum_note`
probe (one junk item linked by hand in a disposable thread, then that
tab refreshed). Step 3 (annotations v3, typed intent) does not depend
on either and can proceed; step 4's freeze waits for the first, step
5's listing state for the second.

## Sweep of the pricing plan against the trimmed scope (2026-09-04)

Every ruling, parked item, finding and question, checked against the
plan as trimmed on 2026-09-04 (render-first, plans and import parked,
copy-and-paste publishing). "Holds" needs no edit; "amend" names the
edit; "park" names the trigger.

### The rulings

| Id | Verdict | What changes |
| --- | --- | --- |
| C64 | holds | — |
| C65 | amend | `written_via` and `actor` land in v3; `applied_plan` waits for receipts (a nullable column added by the stepwise `ALTER` the 2b review already designed) and is not required by the write API |
| C66 | holds | the per-kind trait stays small: one new kind |
| C67 | amend | `amount` is a decimal with at most two fractional digits, or a lot ratio of two unreduced positive integers; equality is structural; the write API refuses more digits rather than rounding; the game side keeps a note verbatim and compares at two places. Type names: consider `negotiable` / `exact` (the game's words) over `buyout` / `fixed` now that no import needs the C++ vocabulary — owner's call |
| C68 | amend | `emit` = the word GGG's client writes (E21); aliases = that word plus the legacy C++ tag, nothing else (no modelling of the indexer's matching); a row's `game:` evidence kind added to the table header; retired currencies (`chisel`, `coin`, `silver`) keep their rows, marked not offered in game; display names for the 23 new rows still need a source |
| C69 | amend | the game side resolves to price (exact or negotiable), `skip` (do-not-index), `invalid` (the empty-amount residue, or a ratio in a tab name), or none; note before tab name; a game `skip` against a manual price is a relation the render's table must rule (blocked until then) |
| C70 | holds | — |
| C71 | park | trigger: a batch a human cannot review row by row (an agent writer, or a document of many rows). Its one clause that survives moves to the write API: a cited currency tag must resolve at write |
| C72 | holds, smaller | the report is the render's: priced locations outside coverage, priced facts older than the window, and items moved or reindexed since the render basis |
| C73 | park | the owner's two manual rows are set by hand; trigger: a 0.18 user with manual rows asks for them carried over (a product question, not the owner's) |
| C74 | amend | output is stdout (the pipe is the copy path); pages labelled *n* of *N*; the policy table's opening rows: manual price → post; unpriced (`no_price`) → post without a tag; game-only → behind one flag, counted either way; conflict where the game side is a tab name → post, forum governs (E17, pending its claim); game `skip`, a ratio on a non-bulk item, a substash item → blocked, counted; socketed → omitted, counted; character items → posted (E19); `realm=` emitted (E18); price written on the line after the link (the documented form, E8) rather than only in a spoiler title; the post size a parameter defaulting to 50,000 |
| C75 | holds, test deferred | the pattern-9 test (what two plan-bearing consumers share) cannot run while `PricePlan` is parked; the module stays in `acquisition-plan` |
| C78 | park | with C71; `price set` prints the prior value so a single write can be undone by hand |
| C79 | holds | register rows to add: the About page as a dated browser read (the saved copy), the wiki and Procurement as corroboration; the developer docs need no row |
| C76, C77 | unaffected | — |

### The parked items

- **Fired, delete on the C67 commit:** the amount grammar's edge.
- **Reword the trigger (they named receipts or step 9):** row-granularity history → "a since-question at the readings"; one change cursor → "an agent consumer reading facts and intent together"; the read economy → "the first MCP consumer"; universal plan grammar (`plans.md`) → "a second plan-bearing consumer exists".
- **Add to what publishing owns:** bumping and the thread limit (E7).
- **Unchanged:** third-party feeds, user-scoped intent (the template comes from a file; the default is `[items]`), batch by query, `current_offer`, PoE2 currencies, coverage advice, `aging`, currency totals, the explicit-selection door, name→id resolution, policy merge.
- **New:** the bulk-exchange item list as a second reference table → trigger: the owner prices a non-currency item by ratio.

### The 2b constraints, re-priced

Still paid by the intent step: (5) the stepwise `ALTER` migration; (6) the realm-bearing tab and substash key, defined before the first row; (7) a kind filter on `list`; (8) the busy-timeout error kind; (3) reduced to "a `clear` then `set` on the same target must work through the tombstone". Parked with the plan: (4) the batch, (9) receipt growth. Noted: (10).

### Findings and questions in the slice record

- Question 1 answered: the three ratio notes are lot pairs, kept.
  Question 2 open but moot for the build (by hand either way).
  Question 3 superseded: the game's own vocabulary is the source for
  `emit`; a browser read is still needed for display names.
- New finding for the table: a daemon exit leaves the facts file's WAL
  uncheckpointed (1.1 MB after the price-notes run), so `census.py`'s
  guard refuses it; `notes-check.py` reads through it. Whether the
  stop path skips the final checkpoint is a store/daemon look before
  anything else reads that file under the same guard.

### Cracks — things with no durable home yet

1. **The E21 corpus lives in a gitignored run directory and a
   disposable note.** It must land as a committed parser fixture (note
   strings and expected outcomes, no ids) and as the currency table's
   evidence before this note is deleted.
2. **No `T<n>` file exists yet.** E1–E6, E10 and E18–E22 are claims
   about GGG; someone authors them master-side before the note goes.
3. **The done criterion in the packet is the old slice's.** The
   trimmed one: the currency table committed; the buyout value strict
   and pinned; the parser passing the fixture; the listing state
   legible with the raw note beside the parse; the owner's two rows set
   by hand; a page rendered, pasted, and read correct on the trade
   site in the owner's words with every omission counted; claims
   authored; documents at budget.
4. **The owner's real sale tabs are listed, never fetched, and only
   Standard is listed.** A real render needs the 13 Standard tilde tabs
   in coverage (a policy edit or the ad-hoc door), and Hardcore and
   Allflame listed for their four. C72's report will say so; the
   sequencing is the owner's.
5. **Two matrix cells decide whether the first paste is trustworthy:**
   `Stash<n>` numbering when folders occupy indices (E15), and item note
   against tab name in game (rename the test tab to a price and watch
   the site). Both are one hand experiment each; the forum preview
   shows the item picture, so a wrong index is visible before posting.
6. **`LIVE-TESTING.md` is at 91% of budget**; route at session close.
