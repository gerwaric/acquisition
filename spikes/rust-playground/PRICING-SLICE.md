# The pricing slice — closed record

The pricing slice (render-first pricing: the currency table, the typed
buyout value, the game-side parser, the listing state, the single-row
write, `acq shop render`, and the test pass over all of it) was built
between 2026-09-03 and 2026-09-09 and is closed. This file is its
permanent short record in the mold of `REFRESH-SLICE.md`: what landed,
in which commits, what the reviews found, and what the census, the
readings and the site taught. The open record's full text — the packet,
the re-scoped plan, the census tables and the five narrative sections —
lives in git: the file at `6203e984` holds all of it.

The rulings survive in `decisions/pricing.md` (C64–C75, C78, C80, C81),
`decisions/plans.md` (C76, C77) and `CONTEXT.md` (C79); the properties
are pinned by the tests named below; live runs are rows in
`LIVE-TESTING.md`'s run ledger; facts about the trade site are the
`T<n>` claims in `docs/design/trade-ground-truth.md`, authored
master-side and cherry-picked here (T1–T15 from the forum reading,
PR #224; T20–T24 from the page readings, PR #227; T25–T34 with the
rewritten T17 and the amended T11 from the site readings, PR #228, as
`6203e984`), and N46 is a claim in `network-ground-truth.md`. Nothing
here is a second authority.

The site-as-the-oracle readings (step 7, item 5) validated the listing
state twice against the trade site — the owner's saved search pages
joined by item id against the store of the same day, 551 rows on
2026-09-08 and 890 on 2026-09-09 — and **no render verdict changed**
either time.

## Final state

- Tip at closure: the cut commit. Annotations **v3**, facts **v7**,
  sync policy **v3**, plan schema **6**, currency table **v1**, buyout
  value **v1**, game-side note parser **v2**, listing report schema
  **2**, shop render schema **3**; gate green.
- The pricing code: `acquisition-plan`'s `currency.rs`, `price.rs`,
  `game_side.rs`, `listing.rs`, `shop.rs`, the store's `snapshot.rs`,
  `acq reference currency`, `acq price status|show|list|set|clear`,
  `acq shop render`; the read-only tools `tools/census.py`,
  `notes-check.py`, `site-listings.py`, `site-join.py`.
- Reading a store or intent change before reviewing one: the findings
  table below is the checklist, and `REFRESH-SLICE.md`'s beside it.

## Step ledger

| Slice | Step | Commits | What landed |
| --- | --- | --- | --- |
| Pricing | 1 harvest | `ecdb9d3d`, `889c941d`, `0e4a4d9f` | the accepted packet into the registry (C64–C79); parks routed to `decisions/<area>.md` |
| Pricing | 2a clause audit | `8aa7a507` | C35–C44 and C52 cited from the tests that hold them; the dependency direction (C34, C39, C41) a `docs-check` check |
| Pricing | 2b annotations review | `3e234fa5` | FULL sync, an atomic checked export, eight constraints for the intent step |
| Pricing | 2c census | `cdfd02cc` | `tools/census.py` (immutable open, WAL guard); the census below |
| Pricing | 2d currency source | `51a33751` | `SURFACES.md` and its register under C79; `reference/currency-v1.toml`, 19 cited rows |
| Pricing | 2e forum reading, price-notes run, re-scope | `0d9fee64` | T1–T15; the price-notes run (ledger row 2026-09-04) and its corpus as a fixture; `tools/notes-check.py`; C65, C67, C68, C69, C72, C74 amended, C71, C73, C78 parked; the render-first plan |
| Pricing | plan 3 outside review | — | "tab name" and `metadata.public` ambiguous for a nested item — ruled C80, owner verbatim: "I agree with ruling it. Folders never carry price, but substash parents can."; C69's *Pinned:* overclaims. Held by C80, confirmed on the site (T30); C69's *Pinned:* after the owner's trim |
| Pricing | plan 6 outside review | `9cfdef5a` | five, all accepted: unchecked arithmetic against C47; a policy table failing open; staleness over uncovered containers; a quadratic `--expand`; the league-less check ahead of the classification. Held by `c47_corrupt_facts_and_unruled_words_are_cells_never_panics`, `c72_…`, `c53_shop_render_…`, `left_out_line`'s signature, `c74_every_item_…` |
| Pricing | plan 7 outside review | `8ea70c56` | three, all accepted: the ground truth still said what parser v2 disproved — the closeout blocker; `site-join.py` read "Price with Note" before the tab's `public`; the record called finished work open. The first-write race, found on the way and closed on the owner's word, verbatim: "a real first-write race, not merely a flaky test […] The fix should pin both boundaries". Held by `tools/site-join.py --self-test`, `c35_a_losing_first_opener_waits_for_the_winners_stamp_and_proceeds`, `c35_a_foreign_writer_landing_under_the_lock_is_refused_and_left_untouched` |
| Pricing | plan 6 reading 2, second pass | `1c85cdcb`, `c7d8c840`; T24 `635e583c` master-side, `7ef30761` here; T7, T12, T21 upgraded master-side | **pass** — 27 items posted, every link resolved and listed; `Stash<n>` past a folder was one too high, fixed to the website's rank (T24); verdict verbatim below |
| Pricing | plan 6 reading 2, first pass | `423d39aa`; T20–T22 `9d51fbf7` master-side, `bc596b48` here | the owner's page shape (T22, T15, T21, T20), `Posted.title` for `price` (schema 3); the `c74_` page tests rewritten, `tests/shop_json.rs`, `CLI-REFERENCE.md` |
| Pricing | plan 6 outside review, round 2 | `6794c0e4` | the fifth fix half-done: `oldest_seconds` measured every posted item; now `oldest_stale_seconds` over the stale set — a rename, schema 2, fixture regenerated. Held by `c72_…` (3,800 s), `c53_shop_render_…`, `shop-render-schema-2.json` |
| Pricing | plan 1 currency table | `10ef2374` | `currency-v1.toml` v1: 39 active rows (tag = the game's word, a `game:` entry each), 4 aliases, 3 retired; `currency.rs`; `acq reference currency`; the `c68_` tests |
| Pricing | plan 3 game-side parser | `ee7d7bd7` | `game_side.rs`'s `read(source, text, table)` — exact, negotiable, `skip`, `invalid`, none (C69, C68, T2, T10, T11); `NOTE_PARSER_VERSION`; `price_notes_fixture.rs`; the `c69_` tests |
| Pricing | plan 4 listing state | `c340b487`, `25f99e4f` | `Store::pricing_snapshot` (`snapshot.rs`); `listing.rs`'s `resolve` → `ListingReport` schema 1 (C69, C70, C80); `acq price status`, `show`, `list` under C53 (`price_cmd.rs`); the `c69_`, `c70_`, `c80_`, `c53_price_` tests |
| Pricing | plan 4b C81 build | `9db1c99a` | `ignore` → `skip` (C67); a game statement only from a public tab, the rest `residue` (C81, T1, T18); `Effective`, `Counts.by_effective`; the `c81_` test |
| Pricing | plan 2 annotations v3, typed value | `3d2902cd` | annotations v3 by stepwise `ALTER` (C65); `IntentValue` + `check_value` carrying `SyncPolicy` (C66); `Provenance` on `put`/`delete`; `list(scope, kind)`; `AnnotationError::Busy`; `price.rs`'s `PriceTarget`, `Amount`, `Buyout` v1 (C67); the `c65_`, `c66_`, `c67_` tests |
| Pricing | plan 5 set, clear | `5fc19304` | `price.rs`'s `set_buyout` and `clear_buyout`, the one write path for every frontend (C67, C78); `acq price set` and `clear`, `--if-revision`, the C53 receipt; the `c78_`, `c67_…retired…`, `c35_…stale…` tests, `price_json.rs`'s process pin |
| Pricing | plan 5 reading 1 | — | the owner's rows set by hand from an empty intent file, read back; verdict verbatim below |
| Pricing | plan 6 shop render | `13f50f9d` | `shop.rs`'s `render`: two posting cells, the rest blocked or omitted and counted (C74, C81, T7, T8, T11, T13, T15), pages cut under `--size`, the C72 report; `acq shop render` (`shop_cmd.rs`); the `c74_`, `c72_`, `c53_shop_render_` tests, `shop-render-schema-1.json`, `tests/shop_json.rs` |
| Pricing | plan 7.1 property tests | `2e60be6b`, `53421bd0` | proptest 1.11, the workspace's first: the amount's canonical text a fixed point, the parser total (C47), `cut_pages` pure; every property broken deliberately and seen to fail. `c67_any_…`, `c47_any_…` (`price.rs`), `c47_c69_any_…`, `c69_a_well_formed_…` (`game_side.rs`), `c74_any_entries_…` (`shop.rs`), `shop-render-schema-3.json` |
| Pricing | plan 7.2 fixture writer | `0f749018`, the owner's run `ab2b88f9` | `examples/redact-pricing.rs`: a league's `pricing_snapshot` with names and ids hashed under a per-run secret, notes, tab names, positions and values kept; the owner ran it and committed `reference/pricing-snapshot-2026-09-07.json` (402 tabs, 41 characters, 1,977 items, 16 rows) |
| Pricing | plan 7.2 fixture tests | `48d542eb` | `real_scale_fixture.rs` in both crates: the league resolves whole, every count the file's, the owner's five rows landing by C70/C80/C81, every item in one cell, every link numbered as the site numbers it, time guards as cliffs. `c69_at_real_scale_every_fact_has_one_listing_and_the_counts_are_the_files`, `c70_at_real_scale_the_owners_rows_cover_what_the_rules_say`, `c53_at_real_scale_the_report_round_trips_and_every_container_shows_its_two_sets` (`listing.rs`); `c74_at_real_scale_every_item_lands_in_one_cell_and_the_page_is_the_owners_test_tab`, `t24_at_real_scale_every_posted_link_numbers_its_tab_as_the_website_does`, `c72_at_real_scale_the_freshness_lines_name_the_owners_test_tab` (`shop.rs`); `c53_the_expanded_render_is_linear_over_the_owners_league_tiled` (`shop_cmd.rs`); `c53_the_expanded_list_show_and_status_are_linear_over_the_owners_league_tiled` (`price_cmd.rs`) |
| Pricing | plan 7.3 story and races | `1eaef2f4` | `tests/price_story.rs`: the story through the spawned binary, the races as properties (C11, C35, C64, C74, C78). Found and fixed: `PRAGMA journal_mode=WAL` on a fresh file takes the write lock from inside its own read transaction, refusing a first-ever writer one run in six — the three opens go through `ensure_wal` (`acquisition-store/src/lib.rs`). `c74_set_show_render_clear_…`, `c35_two_blind_writers_…`, `c35_a_stale_revision_…`, `c74_a_clear_under_a_render_…` |
| Pricing | plan 7.4 checkpoint on stop | `1eaef2f4` | the daemon left by `process::exit`, its WALs beside the facts file (1.1 MB the census refused): `Store::checkpoint`, `JobDb::checkpoint`, one `exit_process` (`daemon.rs`); `daemon_stop_checkpoints_the_facts_file_and_the_queue` |
| Pricing | plan 7.5 the site as the oracle | `df248a17` | `tools/site-listings.py`, `examples/listing-report.rs` (unredacted, under `runs/`), `tools/site-join.py`; the 2026-09-08 capture: 551 rows, **Q11 answered**, 535 agree, 15 differ, no difference changes a page; `reference/site-listings-2026-09-08.json` |
| Pricing | plan 7.5 second reading: the four experiments | `87dab5bb`, `895a5433` | the owner's four in-game experiments refetched (run ledger 2026-09-09), the site captured again: 890 rows, 866 agree, 14 differ, 10 site-only, 275 state-only; C80 confirmed, Q2 answered; `reference/site-listings-2026-09-09.json`, `tools/site-join.py --rows` |
| Pricing | plan 7.5 follow-ups: parser v2, the exchange list | `43e281a0` | `game_side.rs` **v2** — a note tolerates text after the word (T2, T11); `NOTE_PARSER_VERSION` 2, both report fixtures bumped; the exchange's item groups as a proposal (Q10). `c69_text_after_the_word_is_tolerated_by_a_tab_name_and_a_note_alike`, `c69_a_well_formed_price_reads_as_written`; `reference/exchange-items-2026-09-08.json` |
| Pricing | plan 7 gate | `04993ca5` | the C35 export race, one run in twelve on pre-session code: two threads within one clock tick shared a partial file's name; a per-process counter joins it; `simultaneous_exports_to_one_destination_publish_exactly_one` (`annotations.rs`) |

## Findings

One row per review round; the finding, then the property or test that
holds it now.

| Round | Commit | Findings | Held by |
| --- | --- | --- | --- |
| 2a clause audit | `8aa7a507` | four citation gaps: C41's freshness read had no consumer (1); "the daemon never reads the store" is not what the graph can enforce (2); C44's fact drift pinned by nothing (3); 30 ids uncited (4) | (1) `decisions/plans.md` C41 *Pinned:*; (2) `tools/docs-check.sh` check 4; (3) `c44_fact_drift_never_refuses`; (4) the tests' doc comments, `docs-check`'s uncited report |
| 2b annotations review, fixed now | `3e234fa5` | `synchronous=NORMAL` can roll back the last commits on the one file no server can refetch (1); `export` wrote `dest` directly, unfsynced (2) | (1) FULL, `c35_the_intent_file_is_fully_synchronous`; (2) `<dest>.partial` → `quick_check` → fsync → link, `export_is_a_consistent_snapshot_and_never_overwrites` |
| 2b annotations review, constraints on the intent step | — | ten on the intent API: invisible tombstone generations (3), no batch (4), a migration shape that cannot add a column (5), a realm-less `tab` key (6), no kind filter (7), a blocked writer indistinguishable from any error (8), receipt growth (9), a read that writes (10); paid by plan step 2 | (4), (9): C71/C78's park; (3) `c67_clear_then_set_on_one_target_works_through_the_tombstone`; (5) `c65_a_file_below_the_floor_is_refused_never_migrated`; (6) `c67_the_target_key_carries_the_realm_and_round_trips`; (7) `list_filters_by_scope_and_by_kind`; (8) `a_writer_held_past_the_busy_timeout_is_busy_not_db` |
| 2c census | `cdfd02cc` | a `file:` URI truncates at the `#` in the C++ filename (1); a `mode=ro` open of a WAL file wrote beside the owner's database (2) | `tools/census.py`: percent-encoded `immutable=1` open, refusal of an uncheckpointed WAL; the side files removed |
| 2d currency source | `51a33751` | no GGG export for the vocabulary, the trade site rejected as a tooling source, so v1 cited only the C++ tables. Superseded 2026-09-04 by the game's client and the owner's dialog reading | `currency-v1.toml` header; `SURFACES.md`; plan step 1 |
| 2e forum reading and price-notes run | `0d9fee64` | eight: the ratio is `wanted/lot` (1); the C++ `emit` column was folklore (2); four game-side outcomes (3); no `forum_note` on `/character` (4); every `inventoryId` is `Stash1` (5); `linkItem` resolved at post time (6); a daemon exit leaves the WAL uncheckpointed (7); the journal's default path (8) | (1) C67; (2) C68, the fixture, plan step 1; (3) C69; (4) C69, T6 (the forum bullet below); (5), (6) C74's policy rows; (7) closed by plan 7.4, `daemon_stop_checkpoints_the_facts_file_and_the_queue`; (8) `LIVE-TESTING.md`'s ledger row, the live-run skill |
| plan 2 annotations v3, typed value | `3d2902cd` | five: the round-trip tightened what a v3 policy accepts, at write and read (1); the store crate owns no real kind (2); `null` and absent stay different (3); a tombstoned value is not a prior (4); provenance on the policy row (5) | (1) `c66_a_current_schema_policy_must_be_canonical_and_older_ones_upgrade`; (2) `test_kinds` (`annotations.rs`); (3) `the_first_difference_names_the_path_and_both_sides`; (4) plan step 5; (5) `c65_every_write_carries_its_provenance` |
| plan 2 owner review: the floor | `0550e6e0` | the migration ladder existed only for development files. Owner, 2026-09-05, verbatim: "Prior versions only exist in development and are already obsolete" — v3 is the floor, a file below it refused and never touched | `c65_a_file_below_the_floor_is_refused_never_migrated`; C65 |
| plan 2 outside review | `46b14308`, `b960e364`, and this commit | four: `Amount` refused the decimals the game's own notes carry (1); "never overwrite" was racy (2); an unstamped file with content was initialised over (3); plan and documents stale (4) | (1) `c67_the_amount_is_a_four_place_decimal_or_an_unreduced_lot_pair`, C67; (2) `simultaneous_exports_to_one_destination_publish_exactly_one`; (3) `an_unstamped_file_with_content_is_refused_and_an_empty_one_is_created_over`; (4) `lib.rs` (`SYNC_POLICY_VERSION`), `decisions/pricing.md` intro, `annotations.rs` |
| plan 4 listing state | `c340b487`, `25f99e4f` | eight: no read at a league's item grain (1); NULL positions led their holders (2); the report must re-read (3); no fall-through past an unreadable row — the C++ bug (4); a folder's row covers the tabs it groups (5); "priced tabs" are the nameable ones (6); `ignore` beside a game price is `conflict` (7); C53's group header (8) | (1), (2) `the_pricing_snapshot_carries_notes_and_buyout_rows_verbatim_and_live_items_only`; (3) `rows_are_accounted_for_and_the_report_round_trips`; (4), (5) `c70_the_manual_side_resolves_by_specificity`; (6), (7) `c69_two_sides_resolve_independently_and_the_relation_names_both`; (8) `c53_price_list_groups_by_container_and_lists_ten_or_fewer` |
| plan 4 outside review | `06197271`, `4f2e1de6` | five (owner-run 2026-09-06): a league-less character read as membership (1); `--in` without or against `--realm` (2); `items_in` was the physical set only (3); the JSON omitted the records the text used (4); a zero-item `status` returned before the rows (5) | (1) `a_league_less_characters_items_are_flagged_in_every_leagues_report`; (2) `the_realm_comes_from_the_address_and_a_disagreement_refuses`, `tests/price_json.rs`; (3) `c70_a_targets_coverage_is_every_listing_whose_chain_holds_it`; (4) `render_list`/`render_show` signatures, `tests/price_json.rs`; (5) `c35_an_empty_league_still_reports_its_rows` |
| plan 4b outside review | `d034713b` | four (owner, 2026-09-06): an unreadable row never reached the effective price — the fall-through by another door (1); an unreachable bucket hid invalid notes (2); `show` listed an item under itself (3); `ignore` still in the docs (4) | (1) `c81_…`, `c53_price_status_…`; (2) the `Counts` doc, `c69_…`; (3) `c81_…`, `the_raw_note_sits_beside_the_parse_in_show`; (4) `listing.rs` |
| plan 4b outside review 2 | `aad86746` | two (owner, 2026-09-06): a count rename replaced keys under schema 1, against C53's additive rule (1); the relation sentence and the default list hid unresolved items (2) | (1) `the_report_json_matches_the_committed_fixture`, `reference/listing-report-schema-2.json`; (2) `c69_…`, `c81_…`, C81 |
| plan 4b outside review 3 | `eeb4220a` | two (owner, 2026-09-06): the default suppression ran before `--effective`, so `--effective none` could select nothing (1); the handoff named a side where `none` and `unresolved` carry none (2) | (1) `c81_…` (each side selects on its own); (2) `listing.rs` (the render handoff's doc) |
| plan 5 outside review | `0233b18e` | four (owner, 2026-09-07): a blind write could replace a value this build cannot read (1); the retired-tag rule was a convention, not a door (2); the undo omitted `--if-revision` (3); C73 and the registry indexes stale (4) | (1) `price_set_and_clear_refuse_a_row_this_build_cannot_read_unless_the_revision_is_named` (`price_json.rs`), `the_write_receipt_says_what_was_and_how_to_put_it_back`; (2) `c66_a_writer_s_rule_refuses_a_new_write_and_never_a_read`, `c67_a_new_price_never_names_a_retired_tag_though_a_stored_row_may`; (3) the receipt test, `annotations.rs` and `price.rs` docs; (4) C73, `CONTEXT.md` |
| plan 4b outside review 4 | `bff8528d` | one (owner, 2026-09-06): the handoff said every hand-decided item is posted, though `side_word` folds the manual kinds together | `listing.rs` (`Effective`'s doc, beside the type) |
| plan 1 currency table | `10ef2374` | the draft's tags were the C++ app's where the game writes other words (1), its alias lists the parser's guesses (2) — superseded 2026-09-05 by the owner's row-by-row checking (`browser:` evidence, C68 amended); well-formedness is the loader's decision (3) | (1), (2) the file's header; (3) `c68_the_loader_refuses_what_a_reviewer_would` |

## What the census, the readings and the site taught

**The census of the owner's real data (step 2c, 2026-09-03).**

- 1,231 of the userstore's 1,355 `item_buyouts` rows are one tab-name
  price materialized onto each item — C64's rejected shape, as data.
  The two rows that were real manual intent were ruled residue, so step
  5 started from an empty intent file; owner, verbatim: "Let's call them
  residue to drop. I'd rather start fresh. The oddball pricing tells me
  I was using it to debug something."
- The C++ table's `emit` column was folklore (the game writes
  `exalted`, `chrome`, `jewellers`, `fusing`; 23 words were missing):
  C68 and `currency-v1.toml` v1, from the game's own dialog.
- Lot ratios are in the owner's own notes: C67's unreduced pair.
- None of the 13 priced tabs was public and none of the 12 public tabs
  priced — the premise C81 rules on, and why the forum is the channel.
- Every stash item's `inventoryId` is `Stash1`, a socketed item has no
  position (T13): a link's `Stash<n>` comes from the tab, later T24.
- Two traps became tool guards, not rulings: the `#` in the C++
  filename truncates a `file:` URI, and `mode=ro` on a WAL file writes
  beside the owner's database (`tools/census.py`, the 2c findings row).

**The forum reading and the price-notes run (2026-09-03/04).**

- The forum is write-only from our side: no `forum_note` on
  `/character` (T6), so a listing there is intent we hold, never a fact
  we observe — C69's game side is the note, then the tab name.
- The game side has four outcomes, not two: C69.
- The forum resolves a `linkItem` at post time and the site emits
  `realm=` (T7): C74's link codes.
- The owner's in-game observations became T11 and T12; the reading
  re-scoped the slice and parked C71, C73, C78 (`0d9fee64`).

**Validation reading 1 (2026-09-07)** — rows set by hand from an empty
intent file and read back; nothing contacted GGG.

- Verdict, verbatim: "I have played around with a few price sets and
  listings. It looks good to me, but this is a pretty informal check."
  On rigor, verbatim: "After step 6 I want to implement some more
  rigorous testing, but not yet" — which became plan step 7.
- A row on a substash or a character is intent the game cannot express;
  the render turns it into one link per covered item (C70, C74).
- Three questions it raised were never put to the owner: observations.

**Validation reading 2 (2026-09-07)** — a rendered page posted to the
shop thread and read back on the trade site.

- Second pass, the verdict, verbatim: "That worked. All 27 items
  appeared and show up as verified by the website. Horror Spur is empty
  under the title. Dread Dome is 99 chaos. I can't see which item came
  from I_EXIST, but all 27 items are linked." Every link resolved, the
  character item among them (T7); a no-price item is listed under an
  empty title (T21); a hand price beats the public tab's name (T12).
  Step 6's done criterion, in the owner's words.
- First pass, verbatim: "Q3: the web stash view doesn't allow me to
  select items from map or unique stashes at all" (T20, so the substash
  cell is blocked on an observation, not a question); "Q5: items linked
  to the forum without a price annotation are listed with 'No Price
  Set' on the trade site." (T21, so that cell posts); and the shape:
  "Items with the exact same price should be listed together; all
  prices should be wrapped in a spoiler tag; newlines only between
  spoiler tags; each page should be wrapped in a spoiler" (T22, shop
  render schema 3).
- Where the indexer reads the price, verbatim: "the spoiler text is
  used as the price for every item within that spoiler block" (T22
  upgraded, `8c312138`) — the title, as the C++ app had it (T15).
- The first paste's unresolved link gave the site's numbering rule:
  listed tabs ranked from 0, folders and substashes absent, plus one
  (T24, `1c85cdcb`; `t24_stash_numbers_match_the_websites_own_list`).

**The site as the oracle (step 7, item 5; 2026-09-08, and 2026-09-09
after four in-game experiments).** A difference earned a rule only if
it flipped a render verdict for some item. None did.

1. `metadata.public` is the whole condition — 547 stash rows all from
   public tabs, the 13 non-public priced tabs nothing: Q11 answered,
   C81's premise holds — T25.
2. A ratio in a public tab's name lists nothing on the item search and
   offers the tab's stacks on the bulk exchange — T11 amended, T2 and
   T26; Q10's list is T33; a manual row on such a stack stays a blocked
   cell, an observation with its trigger.
3. A socketed item is never listed on its own (107 gems, none) — T26;
   the render omits them as game-listed first.
4. Currency-class stacks are absent from the item search and the priced
   ones offered on the exchange, whose stock counts every public stack
   — T26, T27 (T3 confirmed).
5. The site reads a price out of a note with trailing text, labelled
   "Price with Note" — T28, T17 rewritten as a display fact, and
   **parser v2** on the owner's word, verbatim: "I approve the parser
   and list changes".
6. `facetors`, written by the game's own dialog, lists as "No Price
   Set" — T32; owner, 2026-09-08: a bug, reported to GGG, a currency
   nobody prices in, so the table's row stays as it is.
7. Eight eldritch shorthands the site spells differently it resolves —
   T16 widened by eight, T32; nothing to change.
8. The site drops a chaos amount's fraction and keeps divine's — T31,
   a display fact; the amount is the seller's (C67).
9. The forum rows: four items, two threads, dated by something other
   than the post, five stacks of the same tabs on the exchange — T34.
10. One row no reason explains (Lethal Pride, listed nowhere when the
    owner searched by hand) — observation, left as it is.
11. Coverage is the join's arithmetic, not a site fact: 547 = 532 agree
    + 15 differ; 757 expected = 547 + 3 + 207 state-only; 207 = 93
    ratio-tab + 107 socketed + 5 stacks + 2 — observation.
12. C80 holds on the site all three ways — a substash reads its
    parent's name and flag, a folder's name is never a price, a folder
    child reads itself: **C80 confirmed**, its provisional clause
    closed in the registry — T30.
13. A note beats a valid tab name — Q2 answered in the note's favour,
    the order C69 took from the C++ code — T29.
14. The exchange keys one row per item type per account, shown per
    unit — T27.
15. 63 items in six tabs absent from the second capture, no site rule
    explaining them. **Closed by the owner**, verbatim: "The indigon
    and ungil's harmony show up on the trade site, but it's possible my
    download was done wrong since it was heavily manual with lots of
    button pushes to keep track of." A capture's coverage is the join's
    number, never a site fact.
16. One character item on the site through the stash channel. **Closed
    by the owner**, verbatim: "I changed the price on the rune gorget
    as part of my bug report on the facetor pricing, which included
    moving it out of the character's inventory to a stash tab." Our
    characters were a day old; the site was right — C72's move-since
    case, from the other side.

## Observations still open

Agent observations that became neither a ruling nor a finding; each is
data for the next slice that touches it.

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
  part, the site reads through it — T17, rewritten as a display
  fact). Still the parser's own: `~skip` as a tab name reads `skip`. The indexer's loose word matching (T16) is not
  modelled; a hand-typed alias reads `invalid`, shown verbatim.
- "Moved or reindexed since the render's basis" (C72) has no stored
  basis to compare with ("what did I last post" is parked, v1 reposts
  whole pages), so the render reports the one it has: a posted stash
  item whose fetch predates the stash listing its `Stash<n>` comes
  from — the link's two halves observed at different times. Whether
  that line ever changes what the owner does is still unread: at
  reading 2 the page was rendered minutes after the refresh, so it had
  nothing to say.
- The store's `tabs.idx` falls back to the listing position when an
  entry carries no `index` (T13 says every one of 402 did), so the
  render cannot tell a real index from the fallback; a tab with `idx`
  null was fetched directly and never listed, and its items are the
  `tab_unlisted` cell. Since T24 the index only orders the tabs — the
  rank among the listed non-folder tabs is what a link carries — so the
  fallback would matter only if it reordered them.
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

## Owner questions still open

- **2.** C73 parked as "a 0.18 user asks": pricing is niche, but the
  0.18 import is a product question for other users, not only yours.
  Park stands unless you say otherwise.
- **5.** Still owed from the page reading: your verdict on the page,
  verbatim, with whether the coverage, stale and positions lines
  changed what you did.
- **C69's *Pinned:***. It reads "the `c69_` tests", which pin the parse
  only; the qualified wording needs your trim to fit the 800-byte gate
  (plan 3's outside review, finding (2)).

## Process used

Build / owner review / fix, one step at a time, with the rulings
written in `decisions/pricing.md` before code and the plan re-scoped
once, on the owner's framing, after the first forum reading. The
outside reviews did the catching, sixteen findings over seven rounds:
an amount that refused the decimals the game itself writes, a
league-less character read as membership, an unreadable row that let a
game price through — the C++ fall-through by another door, a blind
write that could delete what this build cannot read, a policy table
that failed open, staleness measured over containers the plan does not
cover, and at the cut a ground truth still saying what parser v2 had
disproved (the closeout blocker) beside a join that read "Price with
Note" before the tab's `public`, hiding the very condition it was
there to test. The two live readings of the site as the validating
consumer (P2) bought what no pin could: reading 2 reshaped the page
and renumbered every link; the site as the oracle answered Q2, Q10 and
Q11, closed C80's provisional clause and found the note suffix the
parser had refused — without flipping a single render verdict. What
was not worth its cost: this record, open, grew to 90 KB, every review
round and reading written into one file each session then read past.
The routing rule in `AGENTS.md` (a finding is a row here, a ruling is
a line there, the narrative is the commit) is its lesson, for the
second time.
