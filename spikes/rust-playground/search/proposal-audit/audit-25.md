# Audit — `search/designs/design-a.md`, at `ed209d47`

Run under `search/proposal-audit/BRIEF.md`. Read: the brief,
`search/DIGEST.md`, `brainstorming-notes/22-search-framing.md`,
note 29 (the proposal brief, at `e5d270bc`), the design, and
`decisions/pricing.md` rows C64, C68, C69 to verify three cited rulings.
"The page" means §1–2 (lines 3–66, 3,351 bytes, 64 lines).

## 1. Findings

| id | check | claim | evidence | what would settle it |
| --- | --- | --- | --- | --- |
| F1 | 1, 6 | `evidence` is a select target in 8 of the 12 appendix queries and is defined in no section of the design. | §5 OQ1, OQ2, OQ3, OQ4, OQ6, OQ7, AQ4, AQ5 all end `select … evidence`; the page defines only "`select` chooses item fields"; §3's projection list and §4.6's "Default rows carry … bounded positive witnesses" name no field `evidence`; §4.7 lists "select targets" among the definitions the page lacks without saying which section holds them. | A named list of select targets, `evidence` among them with its expansion, or its replacement by §4.6's witness field names. |
| F2 | 2, 1 | OQ3's socket condition and the `socket_groups` collection rest on no `S<n>` and carry no judgment label. | §5 OQ3 `any(socket_groups, red >= 2 and blue >= 1)`; §7 row 4 "Collections also include `socket_groups` with `red`/`blue` fields"; §3 "socket/link/colour facts". The digest carries no claim that socket, group or colour data is present in the corpus: S2's census names 13 always-present fields and four shares, none of them sockets; S188 records the seat projection "cannot answer sockets, links or colours"; S22 is the C++ link derivation; S194 (R13) names links and colours as a requirement only. | A census claim (or an item-facts finding reached by id and marked `undigested`, note 29) giving socket/group/colour presence and coverage. |
| F3 | 3 | The OQ1 cold start's call 4 uses three flags the design does not say help documents, so a stranger needs one more read; my count is 6 against the design's 5. | §6 "Help documents `--scope account` and `--fields` for this use of the existing scoped discovery operation" against call 4, `acq items search describe --realm pc --league Standard --terms 'ring,fire resistance,dexterity'`. `--realm`, `--league` and `--terms` appear in no stated help or description content. | Either a stated help surface that documents those flags, or a walk that spends the extra `describe --help` call. |
| F4 | 3 | AQ5's two-call walk needs an expression production that neither described source carries; my count is 3, parting at call 2. | §6 call 1 says `search_describe` "also gives the aggregate/order expression shape for that slot"; §4.3's list of what that call returns (descriptors, value sets, coverage counts, predicate fragments, field types and units, template/source/flag alternatives, slot examples, recipe definitions, supported/unknown counts, location names) does not include it, and the tool description carries "the Query v1 skeleton, basic Boolean operators". AQ5's query needs `max(lines where template = …, n1) as life order life desc`. | §4.3 extended to return the aggregate/order shape, or a third call for the expression grammar. |
| F5 | 5 | The stance-2 reach floor is not demonstrated; the design states this. | §3 "**Reach gap:** no catalogue maps S42's 59 private-item filters to these fields or recipes; raw access alone does not demonstrate that floor"; repeated in §4.7 "the complete S42 filter mapping [is] unverified". Note 22 stance 2: "Every question it [the trade site] can ask of an item, a stash search can ask." | The catalogue mapping S42's 59 filters to fields, recipes or raw paths. |
| F6 | 5 | An everyday question of stance 3's kind reaches the answer only through a recipe whose predicate can be unknown, or through the concept §7 names as the largest learning cost; the design states the gap. | §4.2 "This conservative completeness policy leaves an **everyday-search gap**: OQ1/AQ2 may not yield the requested answer. No private-corpus coverage trial establishes how often this happens. Writing finer line tests does not repair the promised easy case."; §5 OQ1 and AQ2 gap cells; §7 "The largest learning cost is collection aggregation; **it is the first concept I would delete if forced**". | The coverage trial §9 names as the one cut it most wants reversed. |
| F7 | 6 | §4.7's list of what the page cannot specify is itself incomplete by at least six items a second implementer must fetch. | §4.7 names fifteen ("Normalization, three-valued logic, aggregate/empty/error rules, ordering, text comparison, field catalogue, decoded flags, collection schemas, raw paths, recipe coverage, select targets, scope tokens, answer envelope, continuation, and JSON encoding"). Not named: the discovery call and its returns (§4.3), the published SQL view contract (§3, required by note 22's floor), snapshot/basis establishment and `snapshot_busy` (§3), the projection's contents (§3), semantic aliases and reviewed slot labels (§4.1), the limits-as-output table (§4.7). | One list of the definitions the page defers, with the section holding each. |
| F8 | 2 | The template and sign convention is chosen without citing a claim, against a contrast the digest flags. | §4.1 "replaces each signed decimal number with `#`, recording its signed value in ordered `n1`, `n2`, … slots. Thus `+90 to maximum Life` has template `# to maximum Life`, `n1=90`" and "Normalization v1 unwraps `[Tag|Display]` to Display", both uncited; line 210 "Do not delete punctuation" sits beside consuming the leading `+` into the value, so `+#%` and `#%` lines share one template. The digest's contradictions section: "S200 against S25, contrast: … three conventions for one line"; S71 is the precedent for dropping a sign before a placeholder; S4 is the `[Tag|Display]` observation. | Citing S4, S25, S71, S200 and stating whether the sign belongs to the template or the slot. |
| F9 | 1, 5 | While any mod array is unrecognized, a non-matching item cannot be `false` for a line predicate, only unknown; the limits name the coverage gap and not this consequence. | §4.2 "Collection existence is false only when the relevant collection is known complete and no occurrence qualifies" against §4.7 "a new unknown array remains queryable through its raw path and marks line coverage incomplete until extraction understands it"; §6 promises "exact matched/false/unknown item counts". Bears on AQ5's "exact number of matching items" and OQ7's list. | A stated rule for when `lines` counts as known complete. |
| F10 | 2, 5 | The derivation's stated benefit does not cover the raw accessor, and no limit or measurement gap names that. | §3 "Persisting the small facts used by search removes repeated JSON decoding" against "That accessor means the retained API body, never a SQLite column" (`fact["API.path"]`); S127 (each returned row costs one full body parse) and S150 (5.4 µs per item, 202 ms at 36,139) are the costs, cited nowhere in §3's gap list, which names only the full-projection load and CLI cold response. | A limit row or measurement for a query whose predicate reads a raw path. |
| F11 | 7, 3 | §6 contradicts itself on what the MCP tool description contains. | §6 opening: the description "shows the Query v1 skeleton, basic Boolean operators, default scope resolution, decision fields, and the discovery call"; §6 close: "AQ1's broader location facets would be one call from the tool description alone: its field names and the facets shape appear there." | One list of the description's contents. |
| F12 | 1 | OQ3's prose does not describe its own query. | §5 OQ3: `any(socket_groups, red >= 2 and blue >= 1)` followed by "All three colours must be in one group" — the query constrains two colours and three sockets. | Either a third colour term in the query or the prose corrected to the sockets it names. |
| F13 | 2 | §8's refusal of build simulation cites three claims, none of which names it. | §8 "An app-maintained account of what the game can currently generate, automatic legacy classification, build simulation or market valuation (S164, S178, S180)". S164 is game-generation knowledge, S178 legacy as the user's knowledge, S180 the three non-goals (acting on the stash, direct forum updates, a pricing engine). §3's blanket judgment sentence covers §3, not §8. | Marking build simulation as judgment, or a claim that carries it. |
| F14 | 1 | AQ1's listed gap rests on a reason narrower than the digest's reading of the question. | §5 AQ1: "Facets share one predicate; restricting it to tabs also changes the league/rarity denominators … the location table is not claimed as the tab table." The digest's AQ1 reading is "three small tables of counts, no rows"; the location facet's rows already carry a count per tab, alongside character rows. | Whether AQ1's reading accepts a location table that also carries character rows. |
| F15 | 4 | §7's disclosure of what the twelve do not exercise omits several inventory parts and grammar forms that appear in no table query. | §7 "The twelve do not exercise at-least-N, general arithmetic, `summarize`, continuation, saved-query versions, translation remainders or SQL, nor do they require repeated occurrences or a second numeric slot". Also unexercised by §5's table: `if(P,E,E)`, `present(E)`, `unknown(E)`, `contains`, `count(collection where P)`, `sum`/`min`, the `fact["API.path"]` accessor, the manual/observed/effective listing fields and the `priced` alias, socket parent chains, semantic aliases, reviewed slot labels; `source` and flags appear only in OQ1's self-disclaimed illustration and OQ4's candidate-only query. | Extending the disclosure sentence to the forms the appendix does not reach. |
| F16 | 4 | The concept §7 names as first to delete is used by AQ5's one query, and the deletion note does not record that. | §7 "it is the first concept I would delete if forced, but then weighted questions and arbitrary line combinations become application features or listed gaps" against §5 AQ5 `max(lines where template = "# to maximum Life", n1) as life order life desc`. | Naming AQ5's sorted list as the cost of that deletion, or an order expression that avoids the aggregate. |
| F17 | 4 | Two concepts the text uses are not named in the inventory. | Observation age and per-item live/removed membership as answer content: §3 "freshness/membership evidence", §6 "observation age", §5 AQ4 "Default live-only scope reports a matching removed id as excluded when inspected" — row 2 names live/removed only as a scope element, row 13 only the snapshot basis. Request cancellation with a visible loading state: §3 "A rebuild is a visible loading state. Cancel obsolete keystroke requests", §4.3 "GUI autocomplete calls this same read, with cancellation". | Inventory rows for both, or their attachment to an existing row's line. |
| F18 | 2, 3 | S181 is cited for a claim it half contradicts, and the claim's supporting measurement is not cited. | §6 "Neither supplies the account's realm/league vocabulary: that needs a discovery read (the cold-start need is S181)". S181: "From the tool list and `acq --help` alone the store surfaces yield the corpus size, tabs with counts, characters with counts, **the realm vocabulary** and the currency table". The league half is S184's ("`store_status` reports `leagues: 0`"), uncited. | Citing S184 for leagues, or restating the claim as one about help text rather than reads. |
| F19 | 1 | OQ7's query is scoped to one league while its reading asks for the account. | §5 preamble: "`S` … is the selected account/provider's live items, in realm `pc`, league `Standard`"; OQ7's query uses `in S`; the digest's OQ7 reading: "a list across all tabs and characters, item and tab, to decide keep or sell" for a modifier "going away except on Standard". AQ1, AQ4 and AQ5 use `in account`. | Whether OQ7 is meant at account scope. |
| F20 | 7 | The file is 44,295 bytes against note 22's guide. | Note 22: "The rest is as long as honesty takes — about 16 KB is a guide, never a target". The file is 2.8× that; §1–2 is 3,351 bytes of it. | Nothing to settle; recorded as a measurement. |

## 2. The twelve questions against the page

Marks: **holds** (every construct on the page), **holds off-page** (needs
something defined only elsewhere in the design), **gap, listed**,
**claimed, does not hold**, **missing**.

| Question | The design's query, verbatim | Mark | The page text that decides it |
| --- | --- | --- | --- |
| OQ1 | `in S where rarity = rare and class = Ring and (total.fire_resistance >= 40 or total.dexterity >= 30) select identity, location, total.fire_resistance, total.dexterity, evidence` | holds off-page | `Q := in S [where P] [select E [as name], ...]`; `P := … E op E \| P and P \| P or P \| (P)`; `op := … >=`; "enums are unquoted"; "Named totals are ordinary derived fields with inspectable formulas". Off-page: `rarity`, `class` and their value sets (§3, §4.3), the two recipes (§4.2), `identity`/`location` (§4.6). `evidence` is in no section (F1). The design's own cell adds a gap on unclassified contributors (F6). |
| OQ2 | `in S where name = "Ashes of the Stars" and line(template = $legacy_template and $legacy_value_test) select identity, location, evidence` | holds off-page | "`$name` marks an example substitution (value or predicate), not executable syntax"; `line(P) := any(lines, P)`; "Literals include numbers, quoted strings and booleans". Off-page: `name`, `template`, `n1`, `n2` (§4.1); `evidence` (F1). The design's gap cell is the user-supplied criterion, which S178's register row requires of any model. |
| OQ3 | `in S where base = "Spiked Gloves" and line(template = $interaction_line and n1 >= $minimum) and any(socket_groups, red >= 2 and blue >= 1) select identity, location, evidence` | holds off-page | `P := … \| any(collection, P)`; the page's own example uses `base` and `n1`. Off-page: `socket_groups`, `red`, `blue` (§7 row 4, §3), which rest on no claim (F2); `evidence` (F1). Prose and query disagree on colours (F12). |
| OQ4 | `in S where class in [Staff, Warstaff] and line(source = crucible) select identity, location, evidence` — marked "**candidate query only**" | gap, listed | The page carries no origin field and no notation for one; `op := … in`, `E := … [E, ...]` make the candidate query legal. The gap is S177's register row ("league of origin is not a fact the search holds"), and §5 states "No 'Crucible item' inference is silently substituted for origin". Not closed by SQL. |
| OQ5 | `in S where equipable = true and required.level >= 1 and required.level <= 28 select identity, location, required.level order required.level asc` | holds off-page | `[order E asc\|desc, ...]`; "Literals include numbers, quoted strings and booleans"; the page example uses `required.level < 80`. Off-page: `equipable` (§3, "follows that taxonomy and shares its coverage"). The absent-requirement rule is §4.2. |
| OQ6 | `in S where class = BodyArmour and line(template = $legacy_explode_line and $legacy_value_test) select identity, location, evidence` — marked "**one query for the prerequisite identification**, not the requested price" | gap, listed | The page has no price or valuation construct; the gap is stated ("market valuation is outside search") and matches S180's non-goal and the digest's own OQ6 reading ("a price, outside the stash search"). Not closed by SQL. |
| OQ7 | `in S where line(template in $announced_templates and $announced_value_test) select identity, location, evidence` | holds off-page | `op := … in`; `E := … [E, ...]`; `$name` substitution; `line(P) := any(lines, P)`. Off-page: `template` (§4.1); `evidence` (F1). The cell's gap is S52's register row. Scope differs from the question's reading (F19). |
| AQ1 | `in account where true facets [location, league, rarity]` — marked "**Gap in the exact three-table answer**" | gap, listed | `Q := … [facets [E, ...]]` with no per-facet predicate slot, and `P` applies once, at `where`: "`facets` adds independent count tables". Repeated in §4.7: "AQ1 cannot restrict just its tab facet in one query." Not closed by SQL. The gap's stated reason is narrower than the digest's reading (F14). |
| AQ2 | `Q1.refine(total.elemental_resistance >= 60)` | holds off-page | "`Q.refine(P)` denotes resubmitting Q with `where (old) and P`"; "Named totals are ordinary derived fields". Off-page: the elemental recipe (§4.2, "sums fire, cold and lightning, and intentionally counts an all-elemental contribution three times"). The design's own cell carries the coverage gap (F6). |
| AQ3 | `in S where $previous_predicate inspect $item_id` | holds | `Q := in S [where P] … [inspect id]`; "`inspect` explains one id instead of listing matches"; "`$name` marks an example substitution (value or predicate)". Every construct is on the page; the contents of the reply (predicate tree with true/false/unknown leaves) are §4.6 and §4.2. |
| AQ4 | `in account where id = $full_id select identity, location, evidence` | holds off-page | `Q := in S [where P] [select …]`; `op := =`; "Scope notation abbreviates a document naming provider, account, realm, league and live/removed membership". Off-page: the bare `account` token (§5, "`account` explicitly spans its realms and leagues"), `id` (§4.1's "item identity"), `identity`/`location` (§4.6), `evidence` (F1). |
| AQ5 | `in account where line(template = "# to maximum Life" and n1 >= 90) select identity, location, evidence, max(lines where template = "# to maximum Life", n1) as life order life desc` | holds off-page | `E := … \| sum\|min\|max(collection where P, E)`; `select E [as name]`; `[order E asc\|desc, ...]`; "In `order`, a select alias resolves to its expression"; `line(P) := any(lines, P)`. Off-page: `template`, `n1` (§4.1); `evidence` (F1). The count is the envelope's (§4.6). |

Counts: **holds 1** (AQ3); **holds off-page 8** (OQ1, OQ2, OQ3, OQ5, OQ7,
AQ2, AQ4, AQ5); **gap, listed 3** (OQ4, OQ6, AQ1); **claimed, does not
hold 0**; **missing 0**. No appendix query uses a form outside the page's
productions, and no gap is closed by SQL.

## 3. The citations

63 claims opened: the 53 ids written in the file plus the members of its
five ranges (S144–S153, S61–S67, S110–S111, S27–S28, S141–S142). "Lane"
is note 22's source table.

| `S<n>` | Used for | Verdict |
| --- | --- | --- |
| S9 | PoE2 property 109 as a class source | holds; in lane (item-facts: what an item is) |
| S10 | Map tier parsed from the base name; absence is not tier zero | holds — S10 gives both the base-name tier and the absent property; the "reads 0" behaviour it is contrasted with is S21/S23's, which the design does not claim |
| S12 | A veiled placeholder has no value; limits row | holds |
| S14 | `map.area: unknown`; limits row | holds |
| S26 | A quality-normalised defence as a distinct named alternative | holds |
| S27 | Duplicate line occurrences must survive | holds; in lane (cpp-search: rules relied on, what instant cost) |
| S28 | The owner calling ignored mods and duplicates bugs | holds |
| S37 | Measure sorting and allocation before adding indexes | holds |
| S41 | Eight trade group types named, not all specified | holds; the digest's kill list confirms the tips were cut |
| S42 | 59 private-item filters as the unmapped reach floor | holds, exactly; the use is stance 2's own |
| S44 | Local/global twins not observationally distinguishable | holds |
| S49 | The fire-resistance recipe's motivation, explicitly not its validation | holds; the design states the limit of the lane ("evidence about trade fetch only", "neither private contributor coverage nor equivalence with the site") |
| S50 | Optional flags on private lines, values not established | holds |
| S51 | Crafted/fractured decoding as trade-fetch evidence only | holds |
| S52 | Generating mods unknown; OQ7's gap; limits row | holds |
| S53 | Class not a field; derived-or-unclassed limits row | holds |
| S56 | A self-contained query URL decodes locally | holds |
| S61 | Trade number → stat id mapping through reviewed reference data | holds |
| S62 | (range member) per-category trade-stat coverage | holds |
| S63 | (range member) the `Stats.dat` hash absent from the export | holds |
| S64 | (range member) single-stat numbers recovered by hash | holds |
| S65 | (range member) the classification of PoB rows | holds |
| S66 | (range member) the two deterministic detectors | holds |
| S67 | Undecided twice-numbered ids; translation blocked as equivalent; limits row | holds; the design's row is narrower than the register's "shown with both candidates", which §4.5 supplies ("not solved by arbitrarily choosing the first or OR-ing everything") |
| S68 | Two numbers kept in separate slots, never their mean | holds |
| S69 | The export's classes behind a bundled base-to-class table | holds with a note: S69 is class → trade category; the base → class join is S93's pointer to repoe F3, which the design cites beside it |
| S70 | Base-join coverage; unmatched bases unclassed | holds |
| S72 | The reviewed rule that resolves twins (`is_local`, item class) | holds |
| S90 | Base against total defence as distinct named fields | holds |
| S93 | No class vocabulary on the filter page; the join rests on repoe | holds |
| S107 | Unknown lines shown, candidates never substituted in evaluation | holds |
| S110 | Text moves under stable ids; a rename makes a new family | holds |
| S111 | Loud failure for code-relied names, unbound selectors otherwise | holds; both halves used |
| S131 | League and location as authoritative joins, not copied stamps | holds, exactly |
| S141 | No speed promise from cloned corpora | holds |
| S142 | (range member) the bench's single hardware and method | holds |
| S144 | (range) engine choice informed by the bench, labelled judgment | holds; the label answers the lane's "not evidence of who holds the corpus" |
| S145 | (range member) flat per-million scan cost | holds |
| S146 | (range member) SQLite slower on eleven of twelve | holds |
| S147 | SQLite's per-query times as the comparison this design does not meet | holds |
| S148 | (range member) SQLite's one winning shape | holds |
| S149 | (range member) the gap is the engine's own work | holds |
| S150 | (range member) 5.4 µs per item to parse | holds |
| S151 | 38 ms / 1,142 ms for a *narrower* projection, not a promise | holds; the qualifier is accurate |
| S152 | SQLite faster for a process that does not outlive one query | holds; the design keeps S152's load (the JSON parse) apart from S151's |
| S153 | The corpus-holding process is frontend-side and does not exist | holds |
| S155 | Item reach must not wait on trade identifiers | holds |
| S157 | The derived schema's build and file cost before adding indexes | holds |
| S158 | A missing requirement must not read as zero | holds, exactly |
| S164 | Game-generation knowledge refused | holds |
| S165 | Names the legacy variant but gives no verbatim item line | holds; the design's reading of what S165 does not establish is accurate |
| S169 | Composition across every field, in the owner's own words | holds |
| S172 | R2, boolean composition across fields | holds |
| S174 | "Instant" as an acceptance measurement, not a borrowed assertion | holds |
| S177 | Origin not a fact; first-seen and current league distinct | holds |
| S178 | Legacy is the user's knowledge; limits row; §8 refusal | holds |
| S180 | Market valuation refused | holds for valuation (S180's "a pricing engine") |
| S181 | Discovery needs no sample item; the cold-start need | first use holds; second use half-contradicted — S181 lists the realm vocabulary among what existing surfaces yield (F18) |
| S183 | Bodies opt-in, not the price of twenty candidates | holds |
| S190 | R9, schema and vocabularies readable before any row | holds |
| S193 | R12, a decision view by default | holds |
| S199 | An unknown scope token is a structured error | holds |
| S200 | Template conventions and location names documented in the SQL views | holds |

Misses: F13 (build simulation carried by none of S164/S178/S180), F18
(S181's realm half). No source is used outside its lane; the three lanes
most at risk are handled explicitly — engine-bench's ownership question
is labelled judgment (§3), trade-query's S49 is labelled unvalidated on
private lines (§4.2), and repoe is bundled under C68/C79 rather than
fetched (§3, §8).

Choices resting on no claim and not labelled judgment (§3's blanket
sentence, "All behavioral rules here are proposed choices", is inside §3):

| Choice | Line | Nearest available claim |
| --- | --- | --- |
| `socket_groups`, `red`/`blue`, OQ3's linked-colour condition | §3, §5 OQ3, §7 row 4 | S22, S188, S194 (F2) |
| The template/sign convention and `[Tag|Display]` unwrapping | §4.1 | S4, S25, S71, S200 (F8) |
| The three-valued truth table and at-least-N unknown rule | §4.2 | none in the digest; the register's "an unknown shown, never guessed, is the model" and S107 are the nearest |
| Discovery's ranking rule (exact, then token, then frequency) | §4.3 | S168 ("lists autocomplete options that makes sense in an order that makes sence") |
| `contains` as case-insensitive substring over a named display field, with no match against an icon URL | §4.2 | S24, S121, S182 (the substring behaviour), S129 (icon in no column) |
| The attribute recipes, "the analogous published recipe" | §4.2 | S49 covers fire resistance only |
| Refusal of build simulation | §8 | none of the three cited (F13) |

## 4. The cold start, counted

Walked as the stranger the design describes: tool descriptions and help
only, no row seen, and only the contents the design states those carry —
§6's "The tool description itself shows the Query v1 skeleton, basic
Boolean operators, default scope resolution, decision fields, and the
discovery call", "`acq --help` locates `items search`; `acq items search
--help` gives that skeleton and names the discovery command", "Help
documents `--scope account` and `--fields`", the AQ5 tool-list sentence,
and §4.3's list of what `search_describe` returns. §6 assumes a selected,
populated account; I keep that assumption.

**OQ1, terminal.** The user supplies "Standard, a rare ring, 40 fire
resistance or 30 dexterity" (§6 states the question itself cannot).

| # | Call | Why it is needed |
| --- | --- | --- |
| 1 | `acq --help` | locates `items search` (§6) |
| 2 | `acq items search --help` | the skeleton, the discovery command's name, `--scope`, `--fields` (§6) |
| 3 | `acq items search describe --help` | call 4 uses `--realm`, `--league` and `--terms`; §6 says help documents `--scope account` and `--fields` only, so the flag set is not available at call 2 (F3) |
| 4 | `acq items search describe --scope account --fields realm,league` | resolves `pc`, verifies `Standard`; §6 says neither help surface supplies them |
| 5 | `acq items search describe --realm pc --league Standard --terms 'ring,fire resistance,dexterity'` | the Ring enum predicate, the recipe predicates and their coverage (§6 call 4). §4.3 returns descriptors for the terms given; `rare` is not among the three terms the design passes, so the rarity value set arrives only if the response is unfiltered or `rare` is added to the terms — free either way |
| 6 | the OQ1 query | §6 call 5. As written in §5 it selects `evidence`, which no described help or discovery output names (F1); the default decision view avoids that |

My count **6**; the design's count **5** ("two help calls and three search
reads"); they part at **call 3**, i.e. at the design's call 4. Two notes
in the design's favour: §4.3's "A first call can ask for `ring`, `fire
resistance`, and `dexterity` at once" also permits merging its calls 3 and
4 into one, which would make its walk four; and per §3 each of these
reads is a separate process that "pays the projection load again", a load
the design records as unmeasured ("**Measurement gap**", "**CLI gap**").

**AQ5, MCP.** Tool list: "Use `search_describe` to get typed fields and
line predicates; `search_items` accepts Query v1; line numbers are
signed, positional, and never averaged."

| # | Call | Why it is needed |
| --- | --- | --- |
| 1 | `search_describe`, scope `account`, terms `maximum Life` | returns `# to maximum Life`, its one slot, source/flag alternatives, a verbatim example and the predicate fragment (§4.3, §6). The scope token `account` must come from "default scope resolution" in the description; §5's gloss of `account` is not a described source |
| 2 | a read for the expression grammar (help, or the schema help §3 gives SQL) | AQ5's query needs `max(lines where template = …, n1) as life order life desc`; §4.3's return list does not include an aggregate/order shape and the description carries "basic Boolean operators" (F4) |
| 3 | `search_items` with the AQ5 query object | the count comes from the envelope (§4.6) |

My count **3**; the design's count **2**; they part at **call 2**. Without
the sort, calls 1 and 2 of the design's walk do answer "a count and a
list", so the parting is the sorted half of AQ5's reading.

## 5. The concepts

Tested entry by entry against the twelve as §5 writes them.

| # | Inventory entry | Can the twelve still be asked without it? |
| --- | --- | --- |
| 1 | Query document | No — AQ2 is `Q1.refine(…)`, and AQ2's reading is "the previous query plus one predicate" |
| 2 | Explicit scope | No — OQ1–OQ7 run in `S` (pc/Standard, live) and AQ1/AQ4/AQ5 in `account`; one set needs both |
| 3 | Typed field, including derived field | No for the row (OQ1's `rarity`/`class`/totals, OQ5's `equipable`/`required.level`). Yes for three of its parts: the `fact["API.path"]` accessor, the manual/observed/effective listing fields with the `priced` alias, and socket parents appear in no table query |
| 4 | Repeated occurrence and same-occurrence condition | No for same-occurrence — AQ5 needs `template` and `n1` on one line, as does OQ1's refinement. Yes for repeated occurrences: `count(lines where …) >= 2` appears only in §4.1 (the design discloses this) |
| 5 | Template plus ordered captures | No — AQ5 and OQ2. The second slot appears only in OQ2's self-disclaimed illustration (disclosed) |
| 6 | Source and flags | **Yes** — used only in OQ1's illustration, which the design disclaims ("illustrates the notation, not a claim that this base/roll exists"), and in OQ4's candidate-only query. Not disclosed (F15) |
| 7 | Boolean composition, including at-least-N | No for boolean (OQ1's `or`, OQ3's conjunction). Yes for at-least-N (disclosed) |
| 8 | Scalar arithmetic and aggregation | No for aggregation — AQ5's order expression (F16). Yes for general arithmetic (disclosed) |
| 9 | Selection, order and summaries | No — `select` in nine queries, `order` in OQ5 and AQ5, `facets` in AQ1. Yes for `summarize` (disclosed) |
| 10 | Unknown/absent with reasons | No — AQ3 returns true/false/unknown leaves; OQ5's absent requirement |
| 11 | Vocabulary descriptors with predicate fragments | Yes of the twelve as queries (§5 separates discovery); no of the cold start, which spends three of its six calls on it, and of S181/S190 |
| 12 | Evidence/inspection | No — AQ3's `inspect` |
| 13 | Basis and continuation | Yes of the twelve; every answer carries the basis by §4.6, and continuation is disclosed as unexercised |
| 14 | Saved query and semantic version | Yes (disclosed) |
| 15 | Translation remainder | Yes — no trade question is among the twelve (disclosed) |
| 16 | Published SQL view | Yes (disclosed); held by note 22's floor, not by the twelve |

Concepts the text uses that the inventory does not name:

| Concept | Line |
| --- | --- |
| Observation age, freshness and per-item live/removed membership as answer content | §3 "freshness/membership evidence"; §6 "observation age"; §5 AQ4's removed-id rule (row 2 names live/removed as a scope element; row 13 the snapshot basis) |
| Request cancellation and the visible loading/rebuild state | §3 "A rebuild is a visible loading state. Cancel obsolete keystroke requests"; §4.3 "with cancellation" |
| The `false` item count as a third counter beside matched and unknown | §4.6 "exact matched/false/unknown item counts" (marginal: row 10) |
| Extraction diagnostics as projection content | §3 "extraction diagnostics"; §4.1 "a diagnostic" (marginal: row 10) |
| `page_size` and transport options | §1–2 "`page_size` and continuation are transport options" (marginal: row 13) |

## 6. The floor, the stances, the limits

Every item checked, breach or not.

| Item | Result | Line |
| --- | --- | --- |
| Five invariants; the daemon owns GGG traffic and never reads facts (C2, C34) | no breach | §3 "The daemon merely calls the fact mutation surface; it gains no search, fact-reading or intent-reading behavior (C2, C34)" |
| A frontend consumes two surfaces (C12) | no breach | §3 "Search remains within the store read boundary, with shared semantics in Rust (C12, C46)"; §8 refuses "a third frontend channel" |
| Shared semantics in Rust, an adapter per frontend (C46) | no breach | §1–2 "These are encodings, not competing evaluators"; §8 refuses "A second semantic implementation in MCP, the CLI, SQL projection or the eventual webview" |
| SQL a surface on CLI and MCP alike, read-only, over a published contract that is never the facts schema (C48) | no breach | §3 "**Published contract:** … SQL views of precisely this read model"; "Internal layout stays private"; "no `ATTACH`, file functions, writes or facts-schema escape"; "available on both CLI and MCP as required" |
| Whatever the search reads cannot be stale by construction (C34, C48) | no breach | §3 "there is no period where old derived values can answer as current"; "resident arrays are reusable only if their facts revision and derivation version match that snapshot exactly"; §8 refuses "a warm array reused without a current basis check" |
| Annotations only through the store; a saved search is intent (C35) | no breach | §3 "A saved search is the query document as user-scoped intent through the store, not retained results" (C35 is not cited by id) |
| Three output levels, JSON the contract (C53) | no breach | §4.6 "JSON is authoritative; text follows C53, listing ten or fewer entities and otherwise counting with a continuation action" |
| Governed surfaces stay governed (C79) | no breach | §3 "Reference inputs follow C68/C79: bundled, versioned, reviewed extracts"; "The local-source permissions in `SURFACES.md` are not permission for automatic updates"; §3 "No runtime reference fetch"; §8 refuses "Runtime access to governed sources". C68 and C69 were opened and say what is claimed of them |
| Realm a coordinate above league; PoE2 an axis | no breach | §4.1 family is "(realm, normalized template, source, decoded flags)"; §4.2 "League keys include realm"; §4.4 "keeps PoE2 as realm-scoped vocabulary and extraction rules" |
| Stance 1 — flexible, not tuned to one user's questions | no breach | §5 "Examples supply missing user parameters explicitly; they do not claim those are the owner's actual build requirements" |
| Stance 2 — the trade site's reach as the floor | **breach, self-declared** | §3 "**Reach gap:** no catalogue maps S42's 59 private-item filters to these fields or recipes; raw access alone does not demonstrate that floor" (F5). Composition is met: `and`, `or`, `not`, `at_least` across every field |
| Stance 3 — the everyday question is the easy case | **breach, self-declared, for the named-total route** | §4.2 "leaves an **everyday-search gap**: OQ1/AQ2 may not yield the requested answer … Writing finer line tests does not repair the promised easy case"; the alternative route is collection aggregation, §7's "largest learning cost" (F6). The precise-modifier form itself is short: the page example `line(template = "# to maximum Life" and n1 >= 90)` with field predicates beside it |
| Stance 4 — game knowledge is the user's input | no breach | §8 refuses "An app-maintained account of what the game can currently generate, automatic legacy classification"; OQ2, OQ6, OQ7 take the criterion from the user. The class taxonomy and recipes are derivations the register sanctions (S53's row, stance 2's "A computed stat the app can derive from lines is in reach") |
| Stance 5 — idiom in the adapter; the test is the counted cold start | no breach found; the cost is recorded | §1–2 "Text is a proposed notation, not existing CLI"; the MCP literal shape is given in §4.2; the cold start is counted in §6 and re-counted in §4 above; the inventory is 16 rows |
| Stance 6 — a gap is listed, never routed to SQL | no breach | OQ4, OQ6 and AQ1's gaps are listed in §5 and §4.7 and none mentions SQL; §3 "It may use the familiar SQL language fully, but gains no additional facts"; "Query evaluation and SQL obtain values from the same derivation" |
| A reading that could be stale | no breach | §3 "Character league and location metadata remain authoritative joins rather than copied stamps (S131)"; the GUI's in-flight read is "labelled R" and may not answer against R+1 |
| S12 | inherited, consistent | §4.7 "Veiled placeholder shown; no numeric value match" |
| S14 | inherited, consistent | "`map.area: unknown — not supplied`; no icon inference" |
| S52 | inherited, consistent | "Displayed line and value known; generating mods unknown" |
| S53 | inherited, consistent | "Derived class with source/coverage, or unclassed" |
| S67 | inherited, narrower than the register's "shown with both candidates", which §4.5 supplies | "All unresolved trade candidates, translation blocked as equivalent" |
| S107 | inherited, consistent | "Unrecognized semantic line still shown and text-searchable; no guessed alias"; §4.1 "Offer lexical candidates for a human to inspect, never substitute them in evaluation"; §4.3 "Discovery is not fuzzy query execution" |
| S111 | inherited, consistent, both halves | "Changed wording becomes an unbound saved selector or new family; built-in recipe dependencies fail validation loudly" |
| S177 | inherited, consistent | "Origin unknown; first-seen/current league explicitly named as different facts"; §5 OQ4 |
| S178 | inherited, consistent | "Legacy criterion supplied by user; no claim about present game availability" |

All nine register rows are present; none is dropped or contradicted. The
design adds its own limits paragraph (§4.7), which names the unmeasured
projection load, CLI cold response, unverified private flags, the S42
mapping, the OQ1/AQ2 total gap and AQ1's two-query answer.

## 7. The one page

- §1–2 is lines 3–66: 64 lines, 3,351 bytes, two fenced blocks (13 lines
  of grammar, 4 of example).
- Could a second implementer build the same search from that page alone?
  No, by the design's own statement: "This page is a model synopsis, not
  a standalone implementation contract (the missing definitions are a gap
  recorded in §4.7)", and §4.7 "**One-page contract gap:** §1–2 alone
  cannot specify an interoperable implementation."
- What they would have to fetch, and from where:

| Needed | Section | In §4.7's list? |
| --- | --- | --- |
| Normalization v1, slots, range separators | §4.1 | yes |
| Three-valued logic and the at-least-N rule | §4.2 | yes |
| Aggregate, empty-collection, division and error rules | §4.2 | yes |
| Ordering of absent/unknown, the identity tie-break | §4.2 | yes |
| `contains` and `=` text comparison | §4.2 | yes |
| The field catalogue and its enum value sets | §3, §4.3 | yes |
| Decoded flags and the line family | §4.1 | yes |
| Collection schemas (`lines`, `socket_groups`, others) | §3, §7 | yes |
| Raw `fact["API.path"]` paths | §3 | yes |
| Named recipes and their coverage | §4.2 | yes |
| `select` targets (`identity`, `location`, `evidence`) | §4.6 for two, nowhere for `evidence` | yes, but unresolved (F1) |
| Scope tokens (`S`, `account`) | §5 | yes |
| The answer envelope and its counts | §4.6 | yes |
| Continuation, `basis_changed`, `page_size` | §3, §4.6 | yes |
| The MCP JSON encoding | §4.2 | yes |
| The discovery call, its arguments and its returns | §4.3 | **no** |
| The published SQL view contract | §3 | **no** |
| Snapshot/basis establishment, `snapshot_busy`, retry | §3 | **no** |
| The projection's contents and versioning | §3 | **no** |
| Semantic aliases and reviewed slot labels | §4.1 | **no** |
| The limits-as-output table | §4.7 | **no** |

Nothing in §5's twelve queries uses a form outside the page's
productions, so what the page lacks is definitions, not grammar.

## 8. The output shape

Note 22's nine parts and the closing line.

| Part | State | Where |
| --- | --- | --- |
| 1. The model on one page | present | §1–2, 3,351 bytes; a synopsis by its own statement, and not a standalone contract (§7 above) |
| 2. The grammar in as few lines as it takes | present | the 12-line production block in §1–2, plus precedence, literal and alias rules |
| 3. The derivation the model needs, after the model | present, in order | §3, with the projection's contents, the published contract, versioning and the snapshot rules |
| 4. An answer to each of the seven questions | present | §4.1 identity, §4.2 query model, §4.3 vocabulary read, §4.4 ownership and restart (deferring to §3 for the contract), §4.5 trade boundary, §4.6 what a result carries, §4.7 non-goals and limits |
| 5. The appendix, each question one query or a listed gap, discovery and refinement apart | present | §5's twelve rows; the closing paragraph separates them ("Discovery and refinement are separate from these acceptance queries") |
| 6. The cold start, OQ1 and one agent scenario, phase-one stranger | present | §6, OQ1 (5 calls claimed) and AQ5 (2 claimed); my counts are 6 and 3 (F3, F4) |
| 7. The concept inventory | present | §7, 16 rows, each with a deletion attempt |
| 8. What the design refuses | present | §8, five bullets and three accepted consequences |
| 9. The decisions left to the owner | present | §9 — the boundary rulings, the legacy/build knowledge, and whether to fund the trade bridge now |
| Closing line: what was left out; the one cut to reverse | present | §9 "I left out an exhaustive field/recipe catalogue, detailed SQL DDL, parser error productions, GUI layouts, complete trade-group translations, and measured cold/reload timings"; "**The one cut I most want reversed is a worked coverage trial of resistance totals on real line families**" |

Two measurements beside the table: the file is 44,295 bytes against note
22's "about 16 KB is a guide, never a target" (F20), and it carries no
provenance line of the kind note 29 asks for — the file's first line is
its title. That line is not one of the nine parts, and the file is
presented as `design-a.md` rather than under an author's name.
