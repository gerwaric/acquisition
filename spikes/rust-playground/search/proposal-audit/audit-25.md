# Audit — note 25 (`brainstorming-notes/25-search-proposal-astra.md`), run at `a47fa1e5`

Measurement only. Every row points at a line: a quoted section of the
proposal, an `S<n>` opened in `search/DIGEST.md`, a registry entry opened
by its `C<n>`, or a numbered cold-start call. Section references are the
proposal's own headings (`§1–2`, `§3`, `§4.1`…`§4.7`, `§5`, `§6`, `§7`,
`§8`, `§9`); "the page" means `§1–2. The model and grammar — one page`.

Sizes measured: the file is 38,801 bytes; the page (`§1–2`) is 61 lines /
3,087 bytes. Cited claim ids: 60 distinct `S<n>`, every one present in the
digest; registry ids named: C12, C46, C53, C64, C68, C69, C79.

## 1. Findings, most consequential first

| id | check | claim | evidence | what would settle it |
| --- | --- | --- | --- | --- |
| F1 | 5, 1 | The predicate form the design offers for stance 3's everyday question — a named total — can evaluate to `unknown` rather than to an answer, and the stated recovery is that the user writes per-template line tests, i.e. the finer machinery. | §4.2: "Unknown applicable lines that cannot be ruled out as contributors make a semantic total incomplete, not a deceptively exact lower number. The result exposes its known subtotal and uncovered lines as evidence; the predicate on the total remains unknown." §6: "If the named total has unknown contributors, the answer is a bounded candidate set plus a stated gap; more tool calls do not magically turn unknown into false. The reader can then supply a more exact line test." The design marks this itself: §4.2 "This conservative completeness policy is the design's largest practical uncertainty"; §9 "The one cut I most want reversed is a worked coverage trial of resistance totals on real line families". Note 22 stance 3: the everyday question "is one or more precise requirements among others that may be broader, and the model makes that the easy case, never the advanced one". OQ1 and AQ2 are both written on named totals. | The coverage trial §9 names: on the 36,139-item corpus, the count of line families a resistance total cannot rule out as contributors, per item class — i.e. how often the OQ1/AQ2 predicate is `unknown` rather than true or false. |
| F2 | 1, 2 | OQ2's query asks a one-slot template for the line whose text the claim it cites carries with two numbers; under the design's own normalization that line is a two-slot family. | §5 OQ2: `line(template = "#% increased Reservation Efficiency of Skills" and n1 >= 10 and n1 <= 20)`, with "S165 supplies this particular criterion". S165's text: `"10-20% increased Reservation Efficiency of Skills"`. §4.1: "replaces each signed decimal number with `#`, recording its signed value in ordered `n1`, `n2`, … slots" and "Recognized range separators are not negative signs: `10-20` has values 10 and 20" — which gives `#-#% increased Reservation Efficiency of Skills`, two slots, not `n1` bounded 10–20. | The verbatim line as an item body carries it. No cited claim supplies it: S165 is the owner describing a variant, and no digest claim measures a range separator inside an observed `explicitMods` line. |
| F3 | 3 | The OQ1 cold-start count omits the invocation that locates the verb and the read that supplies the realm token. | §6 "**Two search reads, or three terminal invocations counting `--help`.**" against §6 call 1, `acq items search describe --realm pc --league Standard --terms 'ring,fire resistance,dexterity'`. §6 says "The tool description itself shows the Query v1 skeleton … The CLI help gives the same information" without naming the help level, and nothing in §6 says help enumerates realm or league tokens; S181 puts the realm vocabulary on other store surfaces (a further read), and §4.6 makes a wrong token an error that lists choices (S199), i.e. one retry. My count: 4 invocations / 2 search reads at best; 5 if the realm vocabulary is read. First divergence: call 2 (see §5 of this audit). | The design naming which help level carries the skeleton, and whether that help lists the realm and league tokens (so `pc` needs no read). |
| F4 | 2 | S70 is cited for a base-to-**class** table; the claim measures a base-to-**base** join. | §3: "PoE1 classes use a reviewed, bundled base-to-class table; unmatched bases remain unclassed (S70)." S70: "35,011 join an export base by `baseType` (98.2 %), 243 transfigured gems by stripping ` of …`, 397 do not". The class half of the join is S69 ("65 of the site's 68 leaf category ids generate from the export's classes … 103 classes") and S93 ("the class join rests on repoe F3"); neither is cited. | Citing S69/S93 (or the export's class column) for the class half and keeping S70 for the coverage half. |
| F5 | 2 | S50–S51 are cited for crafted/fractured arriving as flags on a private body; S51 measures a **fetched** (trade) line and S50 does not enumerate the flags a private body carries. | §4.1: "crafted/fractured are flags when that is how the body carries them (S50–S51)". S50 is private items carrying `{description, flags?}` objects, "never with a hash; enchant, utility, veiled, crucible arrays stay strings". S51 is the trade `fetch` shape `{description, domain, hash, mods: […], flags?}`, in which "crafted (12) and fractured (4) lines sit in `explicitMods`". §4.1 hedges ("when that is how the body carries them"); §5 OQ1 then uses `flag.fractured = true` without the hedge. | A census of `flags` values on private explicit lines (item-facts F3's 118,633 object lines), which no digest claim reports. |
| F6 | 2, 5 | The number that carries the engine choice was measured on a projection narrower than the one specified. | §3 cites S151 ("38 ms at 36,139 and 1,142 ms at 1.08 million for a narrower projection: these are evidence, **not this design's performance promise**. Full fields, reference rules and intent will cost more"). The projection §3 specifies carries full identity, canonical location with tab ancestry and socket parent, the body by reference, typed item fields, line occurrences and numeric slots, socket groups, "other observed collections", extraction diagnostics and freshness evidence. §4.7 lists "cold/reload latency is unproven" as an additional limit. S151's 18 columns plus line rows are the measured object. | A load measurement of the specified projection at 36,139 (and, for the claim about a million, at 1,084,170). |
| F7 | 2, 5 | The short-lived frontend path is the case S152 names as unable to use the scan; the design keeps it and states the cost rather than reconciling it. | §3: "MCP/GUI hold arrays for their process lifetime; CLI exits and pays the projection load again … At a million items CLI cold reads may take seconds." S152: "A process that does not outlive one query cannot use the scan: at 1,084,170 it pays 5,884 ms of load to answer a 4.5 ms question, where SQLite's 77 ms median wins by two orders of magnitude." The design's CLI substitutes S151's projection load (1,142 ms, narrower projection) for S150's 5,884 ms JSON parse in the same position; §3's phrase is "queried through the same Rust evaluator in short-lived ones". | The CLI-path measurement of F6, set against S147's SQLite per-query numbers at the same scale. |
| F8 | 4 | Eleven concepts the text uses are not named in the inventory. | `fact["API.path"]` raw-path accessor (§3, with its own type/coverage discovery); `refine` as an adapter operation (§5 AQ2: "`refine` is the adapter's local operation of appending a predicate"); the line family `(realm, normalized template, source, decoded flags)` (§4.1); collections other than `lines` and their fields (`socket_groups`, `red`, `blue` — §3, §5 OQ3); "Optional reviewed slot labels" (§4.1); "semantic aliases … with candidates and provenance" (§4.1); the listing tri-partite resolution and the `priced` alias (§4.6); the socket parent / "parent chain" (§3, §4.6); the bundled reviewed reference extract and its version (§3, §4.6 "reference and derivation versions"); `if(P,E,E)` (grammar block); `snapshot_busy` and the snapshot-establishment retry (§3). | A row each, or a sentence attaching each to an existing row — §7 claims the inventory "includes costs hidden from the everyday text syntax". |
| F9 | 4 | Six inventory entries, and half of two more, are not exercised by any of the twelve; their justifications rest on the trade floor, on S27/S68 and on note 22's floor, not on the acceptance set. | Unexercised by the twelve: `at_least` (entry 7 — no acceptance query uses it), the arithmetic half of entry 8 (`+ - * /` appears in no acceptance query; §4.5 uses it for trade weights), `summarize` (entry 9 — AQ1 uses `facets`), basis and continuation (entry 13), saved query and semantic version (entry 14), translation remainder (entry 15, which §7 itself calls "Peripheral to the core"). Half-unexercised: entry 4's repeated-occurrence half (no acceptance query needs two occurrences; §4.1's `count(lines where …) >= 2` appears in no cell) and entry 5's `n2` (no acceptance query reads a second slot). | Nothing: stance 1 makes the twelve a test and not a specification, and §7 already concedes at-least-N "could expand to Boolean combinations". Recorded as a measurement of the deletion test's reach. |
| F10 | 1, 6 | The appendix uses five notations the grammar block does not define. | `$name` placeholders (§5 OQ3, OQ6, OQ7, AQ3, AQ4) — §5 says only "Examples supply missing user parameters explicitly", while `$announced_value_test` (OQ7) and `$previous_predicate` (AQ3) substitute whole predicates, not parameters. `Q1.refine(…)` (AQ2) — no production, defined in the cell. A list literal: `class in [Staff, Warstaff]` (OQ4) and `template in $announced_templates` (OQ7) against `E := field | literal | …`. A boolean as the right side of `=`: `flag.fractured = true` (§5 OQ1, §4.2 "Boolean flags are `= true`, `= false`") against `P := true | false` and `E := … literal`. Ordering by a `select` alias: `max(…) as life … order life desc` (AQ5) against `order E asc|desc`. | Productions added to the grammar block, or the convention stated on the page. |
| F11 | 6 | A second implementer could not build the same search from the page; fifteen definitions must be fetched from later sections. | See §7 of this audit for the list, each with the section it sits in. The page states the categories ("An item has ordinary fields and repeated evidence"; "Discovery lists fields, collections, enums and templates before rows") and leaves the deciding rules — normalization, tri-state semantics, the field catalogue, the select targets — to §3, §4.1, §4.2 and §4.6. | Nothing measurable: the brief asks for the list, and it is given. |
| F12 | 5 | Stance 2's floor — "Every question it [the trade site] can ask of an item, a stash search can ask" — is asserted, not demonstrated. | §3: "Every documented item field is reachable, including those without a convenience name, through a published `fact["API.path"]` accessor and collection navigation." §9: "I left out an exhaustive field/recipe catalogue". The target quantity is S42: "59 read a field a private item carries or derives, 7 the market, 27 unsettled"; no section maps those 59 onto fields or recipes. S13 bounds the reachable set differently again (85 of 105 top-level reference fields observed). | The catalogue against S42's `gap.csv`: each of the 59 as a convenience field, a recipe, a raw path, or a listed gap. |
| F13 | 2 | S200 is cited for "never their mean"; S200 states a schema-documentation requirement. | §4.1: "`Adds 17 to 30` has two slots, never their mean (S68, S200)". S200: "If a lines table is ever exposed (C48 stands), its template convention and the location names are in the schema, or the miss is silent". The mean is S68's ("let's keep min and max separate"), against S25/S47. The design does satisfy S200, elsewhere: §3's published contract lists "line normalization v1" and the `lines`, `line_values` views, and §4.6 puts the location name beside its id. | Moving the S200 cite to §3's published-contract sentence. |
| F14 | 2 | S49 measures the trade site's pseudo total on listed items and is used to fix a local recipe's contributor set. | §4.2: "`total.fire_resistance` sums qualifying fire, all-elemental and relevant hybrid contributions once per line occurrence (S49)". S49: "At `pseudo_total_fire_resistance` min 80 every total equalled the sum of the item's displayed fire, all-elemental and fire-and-X lines, explicit and crafted; a 46–48 mod shown `+60%` counted 60". Note 22's lane table makes trade-query evidence of "the grammar players already know, and the boundary", "not evidence of … a stash", and its gravity row says "Take its shape of asking; leave its contents". The design describes recipes as its own "small reviewed recipes over this same evidence" with declared coverage, and §3 states the blanket "citations identify the evidence motivating them, not a claim that a track mandated this design". | The same coverage trial as F1, run on private lines rather than on the site's totals. |
| F15 | 1 | AQ1's query answers per **location**; the acceptance reading asks per **tab**, and the cell routes the difference to a second query. | Digest AQ1: "how many items per tab, per league, per rarity?" · "three small tables of counts, no rows". §5 AQ1: `in account where true facets [location, league, rarity]`, with "location includes characters, with tab-only counts available as a location-kind refinement". The grammar has no production restricting a facet key (`facets [E, …]`), so the refinement is a scope or predicate change, i.e. a second submission. | Whether a facet key may be restricted inside one query, or whether the reading is satisfied by a location facet whose keys carry kind (§4.2: "location keys include kind and full identity"). |
| F16 | 2 | Two floor items the design satisfies are named in prose without their registry ids. | §7: "Removing it violates the settled floor" for the published SQL view — the entry is C48 ("The SQLite schema is internal; raw SQL is not a surface … No cached search service"), unnamed here and nowhere in the file (the file names C12, C46, C53, C64, C68, C69, C79 only). §3: "The daemon merely calls the fact mutation surface; it gains no search, fact-reading or intent-reading behavior" — the boundary is C2 and C34 ("Facts mutate only through the store crate's ingest surface … derivations are always reproducible from declared inputs … no annotation reads"), also unnamed. Both statements are consistent with the entries as written. | Adding the two ids. |

## 2. The twelve questions against the page

Marks: **holds** = every construct the query needs is defined on the page;
**holds off-page** = it needs something defined only in a later section,
named in the last column; **gap, listed**; none is *claimed, does not
hold* and none is *missing*. All twelve are written out, in §5, as a
twelve-row table. No gap is closed by SQL anywhere in the file.

Page tokens available: `in`, `where`, `select … as`, `summarize … by`,
`facets`, `order asc|desc`, `inspect`, `true`, `false`, `and`, `or`,
`not`, `at_least`, `any`, `present`, `unknown`, `count`, `sum`, `min`,
`max`, `if`, the eight `op`s, `line(P) := any(lines, P)`, `lines`,
`page_size`; field names introduced by the page's own example: `base`,
`template`, `n1`, `armour`, `required.level`; concepts named in the
page's prose: source, flags, socket groups, "other collections", named
totals as derived fields, scope as "a document naming provider, account,
realm, league and live/removed membership".

| Q | the design's query, verbatim | mark | the page text that decides it, and what is off-page |
| --- | --- | --- | --- |
| OQ1 | `in S where rarity = rare and class = Ring and (total.fire_resistance >= 40 or total.dexterity >= 30) select identity, location, total.fire_resistance, total.dexterity, evidence` — and the refinement `base = "Spiked Gloves" and line(template = "#% chance to Suppress Spell Damage" and flag.fractured = true and n1 >= 14)` | holds off-page | On page: `Q := in S [where P] [select E [as name], …]`, `P … E op E | P and P | P or P | (P)`, "Strings are quoted; enums are unquoted", "Named totals are ordinary derived fields with inspectable formulas, not a second class of query", `line(P) := any(lines, P)`, "several constraints inside it refer to the **same occurrence**", and `template`/`n1`/`base` from the page's example. Off-page: `rarity`, `class` and the class derivation (§3 "PoE1 classes use a reviewed, bundled base-to-class table"); the `total.*` recipes and their unknown rule (§4.2); `identity` and `evidence` as select targets (§4.6 "Default rows carry full usable identity …"; "bounded positive witnesses"); the `flag.*` namespace (§4.1) and booleans as comparison operands (§4.2 "Boolean flags are `= true`"). |
| OQ2 | `in S where name = "Ashes of the Stars" and line(template = "#% increased Reservation Efficiency of Skills" and n1 >= 10 and n1 <= 20) select identity, location, evidence` | holds off-page | On page: the same productions, plus "Strings are quoted". Off-page: normalization v1 decides the template and the slot count (§4.1) — see F2; `name` (discovery); `evidence` (§4.6). The cell's own gap ("the word 'legacy' alone is not a criterion") is the register's S178 row, not a model gap. |
| OQ3 | `in S where base = "Spiked Gloves" and line(template = $interaction_line and n1 >= $minimum) and any(socket_groups, red >= 2 and blue >= 1) select identity, location, evidence` | holds off-page | On page: `P … any(collection, P)`, and prose "An item has ordinary fields and repeated evidence: displayed lines, socket groups, and other collections". Off-page: `socket_groups` as the collection's name and `red`/`blue` as its fields (§3 "socket/link/colour facts"); `$…` placeholders (§5's preamble "Examples supply missing user parameters explicitly") — F10. |
| OQ4 | "**Gap in the question as asked:** neither league of origin nor the remembered offer is a stored fact." Candidate query only: `in S where class in [Staff, Warstaff] and line(source = crucible) select identity, location, evidence` | gap, listed | The page carries `op := … in` and the prose "A line keeps its source, flags, words and separate numbers"; the gap itself is the register's S177 row, which the digest's own AQ/OQ reading concedes ("league of origin is not a field (S177)"). Off-page: the list literal `[Staff, Warstaff]` (F10); `class` (§3). The cell does not close the gap with SQL or with an inference: "No 'Crucible item' inference is silently substituted for origin." |
| OQ5 | `in S where equipable = true and required.level >= 1 and required.level <= 28 select identity, location, required.level order required.level asc` | holds off-page | On page: `[order E asc|desc, …]` and `required.level` from the page's example. Off-page: `equipable` (§3 "`equipable` follows that taxonomy and shares its coverage"); the absent-vs-unknown rule the cell's gap relies on (§4.2 "A missing known optional field is absent, an unparseable/unsupported one unknown"). |
| OQ6 | "**Gap:** market valuation is outside search." Prerequisite query: `in S where class = BodyArmour and line(template = $legacy_explode_line and $legacy_value_test) select identity, location, evidence` | gap, listed | The digest's own reading places the price outside search ("a price, outside the stash search, after finding the item and knowing it is the legacy version"), and §8 refuses market valuation (S180). The identification half uses only page productions plus the `$…` convention (F10) and `class` (§3). |
| OQ7 | `in S where line(template in $announced_templates and $announced_value_test) select identity, location, evidence` | holds off-page | On page: `line(P)`, `op := … in`. Off-page: `$announced_value_test` substitutes a predicate, not a parameter (F10); the template list comes from discovery (§4.3). The cell's gap is S52's register row, not a model gap. |
| AQ1 | `in account where true facets [location, league, rarity]` | holds off-page | On page: `[facets [E, …]]`, `P := true`, and "`facets` adds independent count tables" — which decides the cell's "three independent count tables, not a Cartesian grouping". Off-page: the bare `account` scope token's span (defined in the cell: "`account` explicitly spans its realms and leagues"), the page saying only that scope "abbreviates a document naming provider, account, realm, league and live/removed membership; no implicit 'last league'"; `rarity`; the facet denominator rule (§4.2 "Their denominators are all true matches, before pagination"). See F15 on tab versus location. |
| AQ2 | `Q1.refine(total.elemental_resistance >= 60)` | holds off-page | The page's decisive line is "Refinement adds a condition to that question" plus "One typed query tree is the contract"; `refine` itself is not a production and is defined in the cell ("the adapter's local operation of appending a predicate to Q1's returned query document and resubmitting it"). Off-page: the recipe `total.elemental_resistance` and its chaos exclusion (§4.2, and the cell). The submitted object is one query document, so the "one query" mark stands on the model even though the notation is not on the page. |
| AQ3 | `in S where $previous_predicate inspect $item_id` | holds off-page | On page: `[inspect id]` and "`inspect` explains one id instead of listing matches". Off-page: the output the cell promises — "that item's predicate tree with true/false/unknown leaves, raw supporting lines and scope membership" — is §4.6's answer envelope and §4.2's tri-state atoms; `$previous_predicate` is a predicate substitution (F10). |
| AQ4 | `in account where id = $full_id select identity, location, evidence` | holds off-page | On page: scope includes "live/removed membership", which decides the cell's "Default live-only scope reports a matching removed id as excluded when inspected". Off-page: `id` and `identity` (§4.6 "Every answer carries resolved scope …"; "Default rows carry full usable identity"); the `account` token (AQ1 cell). |
| AQ5 | `in account where line(template = "# to maximum Life" and n1 >= 90) select identity, location, evidence, max(lines where template = "# to maximum Life", n1) as life order life desc` | holds off-page | On page: `E := … max(collection where P, E)`, `select E [as name]`, `order E asc|desc`, `line(P) := any(lines, P)`, `template`, `n1`. Off-page: ordering by a `select` alias is not stated on the page (F10); the count the question asks for is the envelope's (§4.6 "exact matched/false/unknown item counts"), and "spans every recognized mod array" rests on §4.7's coverage limit ("a new unknown array remains queryable through its raw path and marks line coverage incomplete"), against AQ5's "across every mod array". |

## 3. The citations

Every `S<n>` the file cites, opened in the digest. Ranges are expanded.
60 ids cited, all present in the digest, none invented, and §9 states
"no undigested finding is used" — consistent with the cite list, which
contains no README finding id.

| `S<n>` | what the design uses it for | verdict |
| --- | --- | --- |
| S9 | "PoE2 property 109 is a class source" (§3) | holds — S9: "poe2 items carry it as `properties[].type == 109`". Lane: item-facts, what an item is. |
| S10 | map tier parsed from the base name; "absence of the old property is not tier zero" (§3) | holds — S10: name/typeLine/baseType all read `Map (Tier N)`; "none of 7,273 carries a `Map Tier` or any `Tier`-named property". |
| S12 | veiled placeholder has no value (§4.1); limits row (§4.7) | holds — register: "a veiled line is shown as the placeholder it is; a value query never matches it". |
| S14 | limits row `map.area: unknown — not supplied` (§4.7) | holds — register: "a query for one gets an unknown, never the icon's art". |
| S26 | named alternatives for displayed / base / quality-normalised defence (§3) | holds — S26 is the site's `extended.ar/ev/es` against the property with quality removed and 20 % applied. Lane: cpp-search, a rule that was relied on. |
| S27 | repeated lines survive; `count(lines where …) >= 2` for two occurrences (§4.1) | holds — S27: "905 lose 914 numbered lines to a duplicate template (last wins)". |
| S28 | same (§4.1) | holds — owner: "Ignored mods and duplicates are both design bugs in the c++." |
| S37 | measure sorting and allocation before adding indexes (§3) | holds — S37: "5,562 ms, 93 % the sort; materialized keys cut a sort to 130 ms". Lane: cpp-search, what instant cost. |
| S41 | "the digest names eight types but does not fully specify all eight" (§4.5) | holds — S41 names the eight and quotes only the `if` tip; the kill list confirms the other tips were cut for budget. |
| S44 | local/global twins observationally indistinguishable (§4.1) | holds — S44: "Text does not identify a key … 93 entries have a `(Local)` twin". |
| S49 | the `total.fire_resistance` contributor set (§4.2) | holds with a lane note — F14: S49 is the site's pseudo total on listed items, used to define a local recipe. |
| S50 | crafted/fractured as flags on the body; "Explicit" as the physical array (§4.1) | partial — F5: S50 gives `{description, flags?}` and names four arrays that stay strings; it does not enumerate flag values. |
| S51 | same (§4.1) | partial — F5: S51 is the trade `fetch` line shape, not a private body. |
| S52 | generating mods unknown (§4.7, §5 OQ7) | holds — S52: "Only the trade `fetch`'s `mods` list attributes a line … a private item has none". |
| S53 | derived class with coverage, or unclassed (§4.7) | holds — register: "a class the search names is a derivation it owns, and an item it cannot class is shown unclassed". |
| S56 | decode a self-contained query URL locally (§4.5) | holds — S56: the search id "is the `query` object gzipped and base64url-encoded". |
| S61 | trade numbers map through reviewed reference data (§4.5) | holds — S61: 7,170 of 11,257 translation entries carry a trade id; 61.7 % of the capture. |
| S62 | same (§4.5) | holds — per-category coverage, including the zero categories. |
| S63 | same (§4.5); supports "No hash reconstruction is needed to search" (§4.1) | holds — S63: the `Stats.dat` hash "is absent from the export … and no function of the id string". |
| S64 | same (§4.5) | holds — S64: 6,958 stat ids recovered by the bijection. |
| S65 | same (§4.5) | loose — S65 classes Path of Building's 29,288 (mod, number) rows; it bears on the reference data's completeness ("423 of 5,489 distinct numbers (7.7 %) are not in the capture") rather than on a local predicate mapping. In lane (repoe as the spine), weakest member of the cited range. |
| S66 | "Several possible hidden identities are not solved by arbitrarily choosing the first or OR-ing everything" (§4.5) | holds — S66: "a stat id with two single-stat numbers has at most one right … the hash picks one for 90, neither for 32". |
| S67 | same (§4.5); limits row (§4.7) | holds — register: "shown with both candidates, never one guessed"; the design's row blocks translation as equivalent. |
| S68 | two numbers are two slots, never their mean (§4.1) | holds — owner: "let's keep min and max separate." |
| S70 | "a reviewed, bundled base-to-class table; unmatched bases remain unclassed" (§3) | partial — F4: S70 is the base-to-export-base join; the class half is S69/S93. The unmatched-remains-unclassed half holds (397 non-joining bases). |
| S72 | twins need the item's context and a reviewed rule (§4.1) | holds — S72: the 150 ambiguous templates are local/global twins "split by the export's `is_local` per id or the mod table by item class". |
| S90 | displayed against base defence (§3) | holds — S90: `BaseArmour` etc. "read base values before mods where C++ … and trade … read the total". |
| S107 | unknown line shown and text-searchable, no guessed alias (§4.1, §4.7) | holds — S107: "Unknown lines are kept, never guessed … no fuzzy match on mod text exists anywhere". |
| S110 | a renamed line becomes a new family (§4.1) | holds — S110: "2436 times a trade id survived while the text it displays changed". |
| S111 | unbound saved selector; "built-in recipe dependencies fail validation loudly" (§4.1, §4.7) | holds — S111: loud for the 210 hard-coded refs, silent for the other ~9000. |
| S131 | character league and location stay authoritative joins, not copied stamps (§3) | holds — S131: a league-filtered search "can answer from a stamp the last listing has already moved". |
| S141 | no speed promise from cloned corpora (§8) | holds — S141: the ×10 and ×30 corpora "are clones with fresh ids, so selectivity per query is identical at every scale". |
| S142 | same (§8) | holds as the measurement conditions (M4, rustc 1.94.1, warm = median of 7). |
| S144 | query time was cheap (§3) | holds — 2.6 ms for twelve at 36,139; 4.47 ms warm median at 1,084,170. |
| S145 | same (§3) | holds — per-million cost flat across scales. |
| S146 | same (§3) | holds — SQLite slower on eleven of twelve. |
| S147 | same (§3) | holds — five of twelve past 100 ms on SQLite at 1,084,170. |
| S148 | same (§3) | holds — the one shape SQLite wins, and that at 36,139 it "wins nothing". |
| S149 | same (§3) | holds — `count(*)` within 4 % of the row-delivering form. |
| S150 | "loading and decoding was expensive" (§3) | holds — 5.4 µs per item; 202 ms at 36,139, 5,884 ms at 1,084,170. |
| S151 | the projection load, explicitly labelled "not this design's performance promise" (§3) | holds, and the label is accurate — F6 records the width difference. |
| S152 | part of the engine range (§3) | holds as a claim; F7 records that the design's CLI path is the case S152 describes. |
| S153 | no search service; corpus-holding process is frontend-side (§3, §4.4) | holds, with the lane respected by disclaimer — note 22 says engine-bench is "not evidence of who holds the corpus — that is a design question", and §3 calls the engine choice "my judgment informed by S144–S153", §4.4 "The benchmark justifies avoiding repeated body decoding, not a particular index or a guarantee about reload latency". |
| S155 | "The core's item reach does not wait for trade identifiers" (§4.5) | holds — S155: 32.6 % of mod lines carry no trade stat id. |
| S157 | measure before adding indexes (§3) | holds — S157: 764,694,528 bytes and a 5,687 ms build at 1,084,170. |
| S158 | `not present(required.level) or required.level < 80` does not grant level zero (§4.2) | holds — S158: 35,956 of 36,139 match "because a requirement the item lacks reads as 0". |
| S164 | refuses tracking what the game can generate (§8) | holds — owner: "keeping track of what is possible to create in the game is a complex task beyond the scope of acquisition". |
| S165 | supplies OQ2's criterion (§5) | holds as the criterion's source; F2 is about the template shape, not the cite. |
| S169 | Boolean composition across fields (§4.2) | holds — the owner's `(Armour > 1000) OR (Required Level < 80)` and "only for stat modifiers, not for any of the other search fields". |
| S172 | same (§4.2) | holds — R2 verbatim. |
| S174 | "'Instant' is an acceptance measurement on the implementation" (§4.3) | holds — R4: "The mod vocabulary served instantly, categorised by kind, ranked sensibly — a read-model property". |
| S177 | limits row: origin unknown; first-seen and current league named as different facts (§4.7) | holds — register: "a derivation … is offered as a derivation, never as the answer". |
| S178 | limits row: legacy is the user's criterion (§4.7); §8 refusal | holds — R8 and the register agree. |
| S180 | refuses market valuation, item movement, shop publishing (§8) | holds — non-goals: "acting on the stash; direct forum updates; a pricing engine". |
| S181 | discovery "does not require a sample item" (§4.3) | holds — S181: "the first row pulled (call 5, 3.5 KB, a gem, `rarity: null`) is the only schema". |
| S183 | bodies opt-in (§4.6) | holds — S183: 50 default rows are 91.5 KB compact / 149 KB pretty. |
| S190 | vocabulary readable before any row (§4.3) | holds — R9 verbatim. |
| S193 | decision view by default, caller names fields (§4.6) | holds — R12, "prevents F3: 31.8 KB for twenty gloves", matching §4.6's "not the price of obtaining twenty candidates". |
| S199 | unknown scope token is a structured error listing valid choices (§4.6) | holds — R18: "an unknown league lists the known ones". |
| S200 | cited for "never their mean" (§4.1) | miss — F13: S200 is the schema-documentation requirement; satisfied elsewhere (§3's published contract). |

**Choices resting on no claim.** §3 states a blanket label: "All
behavioral rules here are proposed choices; citations identify the
evidence motivating them, not a claim that a track mandated this
design"; §3 also labels the engine choice "my judgment", §6 labels the
agent-benefit claim "my design judgment, not a finding measured by the
tracks", and §5 labels the appendix "specifications, not executed
searches". Under that blanket I found no rule presented as a
measurement. Two uncited rules are load-bearing and worth naming for the
author, both covered by the blanket: the range-separator rule (§4.1
"`10-20` has values 10 and 20, while `-10 to -5` has -10 and -5" — no
digest claim measures a range separator in an observed line; it decides
F2), and the elemental-total convention (§4.2 "intentionally counts an
all-elemental contribution three times").

## 4. Floor, stances, limits

One row per item checked. "No breach found" is a result.

| item checked | where it is decided | result |
| --- | --- | --- |
| a gap routed to SQL (stance 6, the acceptance test) | §3 "It may use the familiar SQL language fully, but gains no additional facts"; "Query evaluation and SQL obtain values from the same derivation"; every appendix cell is a query or a named gap | no breach — no cell routes a question to SQL; the three surviving gaps (origin, legacy knowledge, market value) are register rows and §8 refusals |
| a reading that could be stale (floor: "cannot be stale, by construction and not by a refresh") | §3 "A projection version mismatch is rebuilt before use or derived directly from the current snapshot for that read; there is no period where old derived values can answer as current"; "resident arrays are reusable only if their facts revision and derivation version match that snapshot exactly"; "The GUI may finish an already-started read of revision R, labelled R, but may not reuse it to answer a new query against R+1"; §4.6 "Every answer carries resolved scope, canonical query, snapshot basis" | no breach — the one reading delivered after its basis moves is labelled with that basis, which is what C48's *why* ("stale results mistaken for current truth") turns on |
| a cached search service (C48: "No cached search service") | §3 "No search service"; §8 "an always-on corpus service" refused; §7 "No query handles, sessions, macros, user-defined executable functions, semantic mod-id spine, query planner DSL or search daemon" | no breach; the persisted projection is a derivation reproducible from `items.json` (C34: "derivations are always reproducible from declared inputs"), the shape S135 classes as "a derived table" |
| the daemon's blindness (C2, C34) | §3 "Each fact mutation updates affected item projections **in the same store transaction**"; "The daemon merely calls the fact mutation surface; it gains no search, fact-reading or intent-reading behavior" | no breach found against C34 as written ("Facts mutate only through the store crate's ingest surface … no annotation reads"); F16 notes the ids are unnamed, and §3 flags the one structural cost itself ("Existing effective-listing resolution must be reused, not copied. Its dependency location may need adjustment … without a store-to-plan cycle") |
| two surfaces, adapters, one semantics (C12, C46) | §3 "Search remains within the store read boundary, with shared semantics in Rust (C12, C46) … not permission for a third frontend channel"; §4.2 "there is one semantic tree"; §8 refuses "A second semantic implementation in MCP, the CLI, SQL projection or the eventual webview" | no breach — C12's "a frontend that wants a third channel is a protocol or store change" is quoted in effect |
| output levels (C53) | §4.6 "JSON is authoritative; text follows C53, listing ten or fewer entities and otherwise counting with a continuation action" | holds against C53's "Ten or fewer entities are listed, more counted" |
| annotations and saved intent (C35, the floor) | §3 "A saved search is the query document as user-scoped intent through the store, not retained results" | no breach |
| governed surfaces (C79, C68) | §4.5 "Never fetch a short URL or contact the site"; §3 "No runtime reference fetch"; "Reference inputs follow C68/C79: bundled, versioned, reviewed extracts, with human review before adoption. The local-source permissions in `SURFACES.md` are not permission for automatic updates"; §9 "not a request to relax C79 or fetch new material during this run" | no breach — C68 ("versioned, reviewed, committed table … shipped inside the binary") and C79 ("tooling fetches one only from an official export or with explicit permission") are both satisfied as written |
| realm above league; PoE2 as an axis | §4.1 the family is `(realm, normalized template, source, decoded flags)`; §4.2 "League keys include realm"; §4.4 "keeps PoE2 as realm-scoped vocabulary and extraction rules" | no breach |
| game knowledge the app is asked to hold (stance 4) | §3 the bundled base-to-class table and `equipable`; §4.1 "semantic aliases are optional metadata, with candidates and provenance, never replacements for the raw family"; §4.2 named recipes; §8 refuses "An app-maintained account of what the game can currently generate, automatic legacy classification" | no breach established — the class derivation is sanctioned by the register's S53 row; recipes fall under stance 2's "A computed stat the app can derive from lines is in reach". The closest item is the twin alias, which needs a reviewed rule (S72's `is_local`) the API does not carry; it is optional, provenance-carrying and under C68 |
| stance 3: an everyday question needing the advanced machinery | §4.2 and §6, quoted in F1 | **breach recorded — F1** |
| stance 2's reach floor | §3 and §9, quoted in F12 | **asserted, not demonstrated — F12** |
| stance 2's vocabulary ceiling ("leave its contents") | §4.1 "The family describes observations, **not a hidden mod identity**"; §4.5 trade ids only as a translation with a remainder | no breach; F14 is the one place a site behaviour becomes a local definition |
| stance 1 (not tuned to one user's questions) | §5 "Examples supply missing user parameters explicitly; they do not claim those are the owner's actual build requirements"; OQ1 "this illustrates the notation, not a claim that this base/roll exists" | no breach |
| the limits register, all nine rows | §4.7's table: S12, S14, S52, S53, S67, S107, S111, S177, S178 | none dropped, none contradicted; each row's wording matches the register's (checked one by one in §3 of this audit) |
| edge-case gravity (each edge a register row unless a `main` claim depends on it) | veiled → §4.7 row; twice-numbered → §4.7 S67 row and §4.5; the unreachable collisions → §4.5 "not solved by arbitrarily choosing the first or OR-ing everything"; `[Tag|Display]` → §4.1 normalization, resting on S4 (`main`) | no breach |
| engine gravity ("added with a number, not an instinct") | §3's numbers and disclaimers | no breach in form; F6 and F7 record which number is doing the work |
| derivation gravity ("A grammar that is a WHERE clause in new spelling was designed from the table") | the grammar block's clause set — `where`, `select … as`, `summarize … by`, `order … asc|desc` — corresponds one to one to SQL's WHERE / SELECT-AS / GROUP BY / ORDER BY, and §3 publishes SQL views of the same read model | no breach established: the model's subjects (occurrence, slot, family, `any`/`count` over collections) are not the store's columns (S128, S129), and §3 comes after §1–2. Recorded as a measurement, not a finding |
| change gravity ("A layer that exists to survive change is a concept the inventory must justify") | §7 justifies "Saved query and semantic version"; the reference-version layer is not an inventory row | recorded in F8 |

## 5. The cold start, call by call

The stranger's condition: tool descriptions and help only, no row seen,
and only what the design says those contain — §6: "The tool description
itself shows the Query v1 skeleton, basic Boolean operators, default
scope resolution, decision fields, and the discovery call … The CLI help
gives the same information." §6 also excludes two things from its count:
"ordinary user clarification is identified separately" and "They assume
an already selected, populated account. If several accounts need
selection, that is one additional account read."

**OQ1, terminal.** Design's count: "**Two search reads, or three terminal
invocations counting `--help`.**"

| # | call | what it yields | design's numbering |
| --- | --- | --- | --- |
| 0 | user clarification: "Standard, a rare ring, 40 fire resistance or 30 dexterity" | the slot and the deficits, which §6 says "no truthful search can invent" | explicitly outside the count |
| 1 | `acq --help` | that `items search` and `items search describe` exist | the design counts one `--help`; it does not say which level carries the skeleton |
| 2 | `acq items search --help` | the Query v1 skeleton, Boolean operators, default scope resolution, decision fields, the discovery call | **first divergence** — the design's three-invocation count has room for one help, not two |
| 3 | (conditional) a realm/league vocabulary read | the token `pc` used in the design's own call 1. §6 nowhere says help enumerates realm tokens; S181 puts the realm vocabulary on other store surfaces; §4.6 makes a wrong token an error listing choices (S199), so a guess costs a retry of call 4 instead | not counted |
| 4 | `acq items search describe --realm pc --league Standard --terms 'ring,fire resistance,dexterity'` | "the Ring enum predicate, the rare enum, both recipe predicates with definitions/coverage, and a complete example of combining them" | design's call 1 |
| 5 | the OQ1 query | the decision view with location, requested totals, matching lines and completeness | design's call 2 |

My count: **4 invocations / 2 search reads** if one help carries the
skeleton and lists the realm tokens; **5 invocations / 3 reads** as the
walk is written. Design's count: 3 invocations / 2 search reads. First
parting: call 2. Recorded as F3. Not counted by either of us: the audit
read §6 calls "a deliberate third read", and the further query §6 offers
when a total's contributors are unknown ("The reader can then supply a
more exact line test") — the F1 path.

**AQ5, MCP.** Design's count: "**Two calls, no sample-body detour.**"

| # | call | what it yields | design's numbering |
| --- | --- | --- | --- |
| 0 | the tool list, in context | "Use `search_describe` to get typed fields and line predicates; `search_items` accepts Query v1; line numbers are signed, positional, and never averaged." | not a call, correctly |
| 1 | `search_describe`, scope `account`, terms `maximum Life` | "`# to maximum Life`, its one numeric slot, source/flag alternatives, a verbatim example, and the exact predicate fragment … the aggregate/order expression shape for that slot" | design's call 1 |
| 2 | `search_items` with the AQ5 query object | compact rows, exact total, descending max matching line value | design's call 2 |

My count: **2 calls** — agrees, with no divergence. Two assumptions
carry it, both asserted in §6 rather than shown: that the tool
description enumerates the scope tokens (the bare `account` whose span
AQ1's cell defines), and that its field names and facets shape suffice
for AQ1 in one call ("AQ1 would be one call from the tool description
alone: its field names and the facets shape appear there"). §6 states
the validation rule itself: "Descriptions must be held to these concrete
cold-start walks in validation, rather than declaring victory because a
trained agent eventually succeeds."

## 6. The concepts

Inventory: 16 entries (§7). Tested entry by entry against the twelve.

| entry | needed by the twelve? | the question that needs it, or the reason it is not needed |
| --- | --- | --- |
| Query document | yes | AQ2 (`Q1.refine(…)` appends to "Q1's returned query document"); AQ3 re-submits `$previous_predicate` |
| Explicit scope | yes | OQ1–OQ7 run `in S` (pc/Standard); AQ1, AQ4, AQ5 run `in account` — two different scopes are needed in the same twelve |
| Typed field, including derived field | yes | OQ5's numeric `required.level` bracket; AQ1's `rarity` facet; OQ1's recipes |
| Repeated occurrence and same-occurrence condition | half | same-occurrence: AQ5 (`template` and `n1` on one line), OQ1's refinement, OQ3. Repeated occurrence (never collapsed): no acceptance query needs two occurrences; §4.1's `count(lines where …) >= 2` appears in no cell — F9 |
| Template plus ordered captures | half | template plus one capture: OQ2, OQ7, AQ5. A second capture (`n2`): unused by the twelve — F9 |
| Source and flags | marginal | used only in OQ1's illustrative refinement (`flag.fractured = true`) and OQ4's candidate query (`source = crucible`); OQ7 says "a source/flag restriction is added only if the announcement requires it". Retained on S50/S171 and judgment, not on the twelve |
| Boolean composition, including at-least-N | half | `and`/`or`/`not`/parentheses: OQ1, OQ3, OQ5. `at_least`: no acceptance query — and §7 concedes it "could expand to Boolean combinations" — F9 |
| Scalar arithmetic and aggregation | half | aggregation: AQ5's `max(lines where …, n1)`, AQ1's counts. Arithmetic (`+ - * /`, `if`): no acceptance query; §4.5 needs it for trade weights — F9 |
| Selection, order and summaries | most | `select`: ten of twelve. `order`: OQ5, AQ5. `facets`: AQ1. `summarize`: no acceptance query — F9 |
| Unknown/absent with reasons | yes | OQ5's gap (absent requirement is not zero), AQ3's true/false/unknown leaves, OQ1's incomplete totals |
| Vocabulary descriptors with predicate fragments | yes | call 1 of both cold starts |
| Evidence/inspection | yes | AQ3 (`inspect`); `evidence` is selected in nine cells |
| Basis and continuation | no | no acceptance query paginates or re-continues; motivated by S192/S183 and §4.6 — F9 |
| Saved query and semantic version | no | no acceptance query; motivated by note 22's floor ("a saved search, if one exists, is intent") — F9 |
| Translation remainder | no | no acceptance query; §7 itself: "Peripheral to the core, but indispensable if the bridge ships" — F9 |
| Published SQL view | no | no acceptance query; retained by the floor, and §7's reason ("Removing it violates the settled floor") is accurate against C48 and note 22's floor — F9, F16 |

**Concepts the text uses that the inventory does not name** (F8): the
raw-path accessor `fact["API.path"]`; `refine`; the line family
`(realm, normalized template, source, decoded flags)`; collections other
than `lines` and their fields (`socket_groups`, `red`, `blue`);
"Optional reviewed slot labels"; "semantic aliases … with candidates and
provenance"; the listing tri-partite resolution and the `priced` alias;
the socket parent and "parent chain"; the bundled reviewed reference
extract and its version; `if(P,E,E)`; `snapshot_busy` and the
snapshot-establishment retry. §7's own framing — "This inventory
includes costs hidden from the everyday text syntax" — is the line
these are measured against.

## 7. The one page

**Does it fit?** As presented, §1–2 is one section of 61 lines / 3,087
bytes, headed "one page", containing two code blocks (the 12-line
grammar and a 4-line example). By size it is one page.

**Could a second implementer build the same search from that page
alone?** No. Fifteen things must be fetched from later sections (F11):

1. Normalization v1 — what a template is, how numbers become `#`, slot
   order and signs, range separators, the `[Tag|Display]` unwrap (§4.1).
2. Tri-state semantics — `false and unknown`, `not unknown`, `at_least`
   under unknowns, absent versus unknown (§4.2).
3. Aggregate semantics on empty and unknown collections, and division by
   zero (§4.2).
4. Sort semantics for absent and unknown, and the identity tie-break
   (§4.2).
5. `contains` and `=` semantics, and which field they read (§4.2).
6. The field catalogue: `rarity`, `class`, `name`, `id`, `equipable`,
   `stack.count`, defence alternatives, map tier, displayed DPS (§3).
7. The `flag.*` namespace, and booleans as comparison operands
   (§4.1, §4.2).
8. Collection names and their inner fields — `socket_groups`, `red`,
   `blue`, "other observed collections" (§3, §5 OQ3).
9. The raw-path accessor `fact["API.path"]` and what it reads (§3).
10. Named recipes: definition, coverage, and the rule that makes the
    predicate unknown (§4.2).
11. `select` targets `identity`, `location`, `evidence` (§4.6).
12. The scope document's fields and tokens, including the bare `account`
    (§3's projection description, §5 AQ1).
13. The answer envelope — counts, truncation, basis, coverage warnings —
    which AQ1, AQ5 and AQ3 need to be answers at all (§4.6).
14. `page_size`, continuation and `basis_changed`, which the page names
    without defining (§4.6).
15. The JSON encoding and its shorthand (`all`/`any`/`not`/`at_least`,
    the `line` object desugaring) (§4.2).

Three of the page's own productions are also incomplete against the
appendix: list literals, boolean literals in `E`, and an alias as an
`order` expression (F10). One page statement does carry a mechanism the
later sections only elaborate: `line(P) := any(lines, P)` plus "several
constraints inside it refer to the **same occurrence**" is enough to get
same-occurrence binding right without §4.1.

## 8. The output shape (note 22's nine parts and the closing line)

| part | present, partial or absent | where, and what decides it |
| --- | --- | --- |
| 1. The model on one page | present | §1–2, headed "The model and grammar — one page"; 61 lines. Parts 1 and 2 are merged into one section |
| 2. The grammar in as few lines as it takes | present | the 12-line `Q/P/E/A/op/line` block, plus five lines of precedence and clause semantics |
| 3. The derivation the model needs, after the model and never before it | present, and in order | §3, after §1–2; carries what it holds, the transaction rule, the snapshot protocol and the "**Published contract:**" paragraph |
| 4. An answer to each question above | present | §4.1–§4.7, one subsection per question, in note 22's order. §4.4 is five lines and defers to §3 ("Section 3 is the commitment"); the restart cost it owes is given qualitatively ("CLI cold reads may take seconds") and §9 lists measured timings as left out |
| 5. The appendix: every acceptance question in the model's notation, each marked one query or a listed gap, discovery reads and refinements apart | present | §5's twelve-row table; marks in bold in each cell; the separation is stated in the closing paragraph ("Discovery and refinement are separate from these acceptance queries") |
| 6. The cold start: owner Q1 and one agent scenario, from tool descriptions and help only | present | §6, OQ1 (terminal) and AQ5 (MCP), each with a count |
| 7. The concept inventory | present | §7, 16 rows plus the deletion answer ("**it is the first concept I would delete if forced**") |
| 8. What the design refuses | present | §8, five bullets plus three accepted consequences |
| 9. The decisions left to the owner | present | §9: the boundary rulings acceptance would need, the game knowledge only the owner supplies, whether to fund the trade bridge, and whether conservative totals are tolerable |
| closing line: what it left out, and the one cut to reverse | present | §9's last paragraph: the six omissions, then "**The one cut I most want reversed is a worked coverage trial of resistance totals on real line families**" |

Note 22's size guide (about 16 KB, "never a target"): the file is 38,801
bytes.
