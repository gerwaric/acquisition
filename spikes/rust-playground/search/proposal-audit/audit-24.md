# Audit — `search/designs/design-b.md`, at `ed209d47`

Run under `search/proposal-audit/BRIEF.md`. A measurement, not a review.
Section numbers and quotations are the design's own; a number is its
number. "The page" means sections 1–2 (the model and its grammar),
which is the object of checks 1 and 6.

Marks on the twelve: **holds** 6, **holds off-page** 5, **gap, listed**
1, **claimed, does not hold** 0, **missing** 0. No gap is routed to SQL.
Findings: 18.

## 1. Findings

| id | check | claim | evidence | what would settle it |
| --- | --- | --- | --- | --- |
| F1 | 5 | The design's freshness is a per-ask revalidation against a store read the digest does not record, where the floor requires non-staleness "by construction and not by a refresh". | §3: "Every ask first compares the store's change counter and re-derives on a mismatch"; "It cannot be stale because it does not outlive the comparison — **given the counter**"; "Needs from the store: one streaming read of live bodies with coordinates, and the change counter. Neither is on the read surface the digest records (S121, S132, S133)". Note 22, the settled floor: "it cannot be stale, by construction and not by a refresh". | An owner ruling on whether a counter comparison per ask is "by construction"; or the store acquiring the counter, making the conditional unconditional. |
| F2 | 1, 2 | Two incompatible rules govern `field:value` on an enum-valued field, and three of the twelve queries plus cold-start call 1 depend on which holds. | §3: "`colors:rrg` matches as a multiset — **the one field where `:` is not a substring**" (so `rarity:`/`class:` are substrings); §4.3: "a wrong enum value errors with the legal ones (S199)"; §6 call 1: "validates `rarity`/`class` spellings (an error lists the legal values)". Under the substring rule there is no wrong value and no error; `class:staff` (OQ4) matches every class whose name contains "staff". | One rule for enum-valued fields, stated on the page. |
| F3 | 3 | Cold-start call 1's stated return — the shipped totals — is defined in no other section, and without it the stranger has no route to the names OQ1's appendix query uses. | §6 call 1: "returns the resist lines **and the shipped totals that cover them**"; against §1 ("ranked by how many carry each, with the value range") and §4.3 ("`--count line [text]` under any query: kind, template, items, value range, ranked by count") — lines only, no totals. §6 also concedes the description names none: "two if the description spends the space to list the 35 totals by name". | Either §1/§4.3 stating that the vocabulary read returns the totals covering the matched lines, or the cold-start count raised. |
| F4 | 1, 4 | OQ5's one-query form rests on a user-defined name, and no section of the design gives the syntax for defining a name; §9 leaves shipping user names to the owner at all. | Appendix OQ5: "one query only once the user has named `gear` as the `or` of the classes they mean"; §1: "the *user's* own"; §2: "defining a name that exists is an error" (the only mention of defining); §9.3: "Land it, or ship fields and totals only." | A define form in the grammar, or OQ5 rewritten in fields and totals alone. |
| F5 | 5 | The design claims S12 inherited whole while its own rules give a veiled placeholder a matchable number. | §1: "Properties (`Armour: #`), requirements (`Level: #`), gem lines and **every mod array** are all lines"; a line is "a *kind*, a *template* and its *numbers*". §8: "Inherited whole: S12". Register row S12: "a veiled line is shown as the placeholder it is; **a value query never matches it**." S12: `veiledMods` is a `Prefix#`/`Suffix#` placeholder re-rolled per response. | The design naming the veiled array's treatment (a kind whose numbers are not values, or an exception in §8). |
| F6 | 5 | S111's register row asks that a name the code relies on fail loudly; the design's shipped totals are such names and its stated response is a listing, not a failure. | §4.1: "A shipped total lists which of its templates occur in the corpus." Register row S111: "a name the code itself relies on fails loudly." | The design saying what happens when a shipped total's template count reaches zero. |
| F7 | 2 | The bare-word rule is cited to prior-art's stash-search rows, which note 22's source table names as what prior-art is not evidence of. | §2: "(S113–S115 show the habit, not the game's matching rule)". Note 22, the ten sources: prior-art is "Not evidence of … a corpus, a saved query, **a stash search**". S115's examples are one tool's shipped defaults, quoted multi-token strings (`"Pack Size: +3"`), not a bare word. The choice itself is labelled judgment ("what I judge a player means"); the miss is the citation. | Dropping the citation and leaving the rule as judgment, or a claim in lane. |
| F8 | 2 | `sockets`, `colors` and `stack` rest on no claim and are not labelled judgment. | §3: "Computed from the body's sockets: `sockets`, the count; `links` … `colors`, the socket letters"; "The body (S2): `name base rarity ilvl stack id`". S2's 13 always-present fields name no socket array and no stack; S22 covers `links` only; S188 records colours as unanswerable on the seat projection. | A claim (or a README id marked `undigested`) that the body carries the socket array with colours and a stack size. |
| F9 | 6 | The page is 109 lines / 1,086 words / 6,422 bytes, and things its own appendix queries need sit outside it, so a second implementer cannot build the same search from it. | Outside §§1–2: the totals' names and definitions (§3), `colors`' multiset rule (§3), all derivations of the 29 fields (§3), the class table and `class:unknown` (§3), the default scope (§4.2), the id-prefix rule (§4.6). §2 also defers two vocabularies to a document the design does not contain: "Kinds … and flags … are closed lists in the help." | Printing the kinds and flags lists and the totals' names, or the page absorbing them. |
| F10 | 2 | "the model ships no grouping above class (S69)" attributes a design choice to a claim that does not state it. | Appendix OQ5. S69 records 103 export classes and that 65 of the site's 68 leaf category ids generate from them; S42 records `category` with "83 option ids" and S54 the owner's own `accessory.ring` — a grouping above the leaf that the digest does record. | Restating the parenthesis as the design's choice, or citing the claim that forecloses a grouping. |
| F11 | 2 | `edps` adopts a formula S22 records with a defect, without naming the defect. | §3: "Computed from property lines: `pdps edps dps`, APS × the mean of the range, quality not applied (S22)". S22: "eDPS **ignores the damage type tag**". | The design stating what `edps` sums, or listing the tag as a limit it adds. |
| F12 | 4 | Concept 3's deletability line is contradicted by appendix OQ5. | §7.3: "*The user-defined part is what I would delete first*: the model stands without it." Appendix OQ5 is the one of the twelve whose single-query form is a user name (F4). | Either OQ5 without `gear`, or the concept line naming the cost. |
| F13 | 2 | `note` is listed as read-only intent while the digest's only record of a `note` is the item body's own field. | §3: "Intent, read-only: `priced price note`". S2: "`note` 0.5 %" among the body's fields. | The design saying which `note` the field is, or carrying both. |
| F14 | 4, 6 | "count" names two constructs in one grammar, and count-by accepts the second. | §1: "*Counts* group the set by any name"; "count-by all take the same things a term does". §2: "`count(q, q, …)` how many of these hold". §6 passes it as `count:"line"`. So `--count count(a,b)` is legal by §1's rule. | One of the two renamed. |
| F15 | 1 | AQ2's offered base includes a query whose refinement is not the shape AQ2 wants. | Appendix AQ2: "R: `<AQ1's or OQ1's text> total-res>=60`". AQ1's text is the empty query with `--count tab,league,rarity`; digest AQ2 wants "a shorter list, the same shape". | Naming OQ1's text alone. |
| F16 | 6, 7 | Two sections rest on an audit the design neither names nor dates. | §5: "Found by the audit:"; §6: "**Three calls**, the audit's count". | A citation, or the sentences rewritten as the design's own. |
| F17 | 2 | The absence of a store change counter is inferred; none of the three claims cited enumerates the read surface or mentions a counter. | §3: "Neither is on the read surface the digest records (S121, S132, S133)". S121 lists four item predicates and `Store::item`; S132 the per-item stamps; S133 the aggregates and `events_since`. S134 ("What a search consumer can do today") is the digest's nearest enumeration and is cited elsewhere in the same paragraph, for another number. | Citing the claim that records the surface, or marking the absence as inference. |
| F18 | 2, 5 | `sum` contradicts an owner claim on judgment, and §9 routes only the other sum judgment to the owner. | §2: "a `sum` needs one number, so there the line contributes its mean, as on the site (S47) — a judgment, and against S68's grain". S68 (owner): "let's keep min and max separate." §9.1 hands over "**The sum rule**" — §1's across-lines sum — not this one. | §9.1 covering both, or `sum` over a two-number line defined without the mean. |

## 2. The twelve questions against the page

Marks per the brief. "Off-page" names the section that supplies the
missing definition.

| Q | the design's query, verbatim | mark | the page text that decides it |
| --- | --- | --- | --- |
| OQ1 | `rarity:rare class:ring (total-res>=60 or total-str>=30 or total-dex>=30)` · D: `rarity:rare class:ring --count line resist` · refined: `class:gloves fractured:"suppress"` | holds off-page (§3) | §1 names `rarity`, `class` in the closed field list and defines totals only by concept and one instance: "*totals* shipped as reviewed reference data and written in this grammar (`total-fire-res = sum(…)`, **listed in section 3**)". `total-res`, `total-str`, `total-dex` are in §3's list, not on the page. §2 gives `thing:text`, `thing cmp number`, `( query )`, `or`, and `kind:"text"` for the refined case (`fractured` is a kind). `--count line resist` is §1's own example. F2 applies to `rarity:rare`/`class:ring`. |
| OQ2 | `name:"ashes of the stars" "reservation efficiency of skills":10..20` | holds | §2: `thing:text contains`; `thing := "text"` — "any displayed text: name, base, every line"; `thing:a..b inclusive, either side blank`; "without `#`, it is a case-insensitive substring, and under a comparison it means 'any template it resolves to' — the answer lists them"; "A line with several numbers satisfies a comparison when every number does". §1's terms block carries the per-term counts the appendix's "no" answer uses. |
| OQ3 | `base:"titan gauntlets" colors:rrg "# to maximum Life">=80 or name:"kaom's roots"` | holds off-page (§3) | §1's field list carries `base`, `colors`, `name`; §2 gives the substring, the template-with-`#`, the comparison and "`or` binds looser than adjacency". `colors:rrg`'s meaning is defined only in §3: "which `colors:rrg` matches as a multiset — the one field where `:` is not a substring". On the page alone, `colors:rrg` is a substring over a socket string. |
| OQ4 | `class:staff has:crucible` | holds | §2's grammar line prints the construct and this exact example: "`has:name | has:kind it is there        explode  has:crucible  -has:reqlevel`". `class` is in §1's field list. (F2: whether `class:staff` is one class or every class containing "staff" is decided off-page.) |
| OQ5 | `gear (reqlevel<=30 or -has:reqlevel) --count tab` then rows | holds off-page (§1's user namespace; §9.3) | Every construct is on the page — `reqlevel` in the field list, `cmp`, `or`, `( )`, `-has:`, and §1's "*Counts* group the set by any name". The referent is not: `gear` is a name in §1's third namespace ("the *user's* own"), and the design's own line concedes it — "one query only once the user has named `gear`". No section gives a define syntax (F4) and §9.3 may not ship the namespace at all. |
| OQ6 | find: `explode class:"body armour"`; worth: refused (S180) | gap, listed | §2: a bare word "searching all displayed text"; `thing:text`. The second half is declared: appendix — "worth: refused (S180). **Listed gap, by design**"; §8 refuses "worth (S180)". Not closed by SQL. The digest's own reading of OQ6 puts the price "outside the stash search". |
| OQ7 | `"<words of the announced mod>" --count tab`, then rows; all realms, tabs and characters by default | holds off-page (§4.2) | §2's `"text"` = any displayed text, substring; §1's counts and rows. The answer wanted is "a list across all tabs and characters", which rests on the default the page does not state; §4.2 does: "default scope is every realm and league, live items". |
| AQ1 | `--count tab,league,rarity` | holds | §1: "*Counts* group the set by any name — or several, one table each, never crossed"; §2 line 1: "none is every item in scope" (the empty query). `tab`, `league`, `rarity` are all in §1's field list. The comma as the separator appears only in the appendix; the construct ("or several") is on the page. |
| AQ2 | R: `<AQ1's or OQ1's text> total-res>=60` | holds off-page (§3) | §2: "a refinement is still the previous query plus one term". The term is a total, named only in §3 (as OQ1). F15 applies to the AQ1 base. |
| AQ3 | the terms block and the touched lines on every row; an empty answer carries scope and per-term counts; why-not is `show <id> --against '<query>'` | holds | §1's answer shape prints `terms  per term: what it resolved to, items matching it alone, items lacking it` and `scope`; "*Rows* carry … the lines the query touched with their values — matched-on and the decision view are the same thing"; "with `--against '<query>'` its terms block carries the item's own value for each term, which is the why-not". |
| AQ4 | `show <id>`, or `id:<printed id>` inside any query | holds | §1: "*One item* is the set `id:<x>` shown whole, spelled `show <x>`"; `id` is in the field list; §2's `thing:text` makes `id:` a substring, and a full printed id as a substring selects itself. The appendix's further claim ("a prefix is a substring") is not exactness: §4.6's "any printed id or unique prefix, S198" carries the uniqueness the page does not, and no section says what `show` does when a prefix matches several. |
| AQ5 | `"# to maximum Life">=90 --sort "# to maximum Life" --desc` | holds | §2: `"text"` containing `#` "names one whole template"; `thing cmp number`, whose printed example is this query's first clause. §1: "Sort, extra columns and count-by all take the same things a term does (`--sort thing`, ascending; `--desc` reverses)"; `total matches` supplies the count; §1's "the sum of its lines of that template across the kinds in scope" supplies "every kind summed"; §1's sign folding supplies "typed as `+#` it canonicalises to the same template". |

## 3. The citations

Every `S<n>` the design prints, opened in the digest. 66 rows; 67 ids
(S113–S115 is one range citation). Lane per note 22's source table.

| S | what the design uses it for | verdict |
| --- | --- | --- |
| S2 | what else a body holds is a field or `show`; the body-sourced fields `name base rarity ilvl stack id` | partial: S2's 13 always-present fields give `name`, `baseType`, `ilvl`, `id` and record `rarity` at 70.2 %; no stack (F8) |
| S3 | kind = the array plus flags | holds (`{description, flags}` objects) |
| S4 | markup reduced to its display half | holds as evidence the markup exists in display text; the reduction rule is S71's |
| S6 | "S6 says only that properties defeat a field list" | holds (579 property names over 66 type ids) |
| S9 | PoE2 carries the class | holds (`properties[].type == 109`) |
| S10 | `tier` from a base reading `Map (Tier N)` | holds |
| S12 | inherited limit | cited; contradicted in mechanism (F5) |
| S13 | what else a body holds | holds (85 of 105 top-level fields observed) |
| S14 | map areas refused; inherited | holds |
| S22 | `links` = longest run in one group; `pdps edps dps` = APS × mean of range, quality not applied | holds for `links`, `pdps`; partial for `edps` (F11) |
| S23 | no map carries a tier property | holds (0 of 7,273) |
| S25 | one of the three template conventions | holds (the contradictions row "S200 against S25") |
| S26 | refuses quality-normalised defences | holds |
| S27 | dropping a repeated template is the bug | holds (905 items, 914 lines, last wins) |
| S28 | same | holds (owner, verbatim) |
| S29 | the sum rule's grain; the 35 shipped totals | holds (35 pseudomods over 117 templates, N× counting) |
| S41 | `if` → `(-has:x or x>=n)`; `count` stands on the trade mapping | holds (the `if` tip verbatim) |
| S42 | 59 item filters → fields; 7 market dropped; 27 unsettled unmapped | holds, exactly |
| S47 | the site averages a two-number line; the mean inside `sum` | holds |
| S49 | pseudomods sum displayed lines on the site | holds |
| S52 | the mod behind a line is not known; refuses tiers and ranges; inherited | holds |
| S53 | base → class is needed; inherited | holds (no class field on a private item) |
| S56 | "A URL needs no network" | holds (the id is the gzipped query) |
| S67 | both candidates shown as undecided; inherited | holds (32 ids, the site cannot decide) |
| S68 | min and max kept apart; named as contradicted by the mean inside `sum` | holds; the contradiction is declared (F18) |
| S69 | the class vocabulary is 103 classes; PoE1 base → class; "no grouping above class" | holds for the 103 and the join; miss for the grouping (F10) |
| S70 | the base → class join | holds (98.2 %, transfigured by suffix, `Map (Tier N)` a base) |
| S71 | markup reduced to its display half | holds (`[Tag|Display]` → display, sign before a placeholder dropped) |
| S72 | local/global twins are one line; "those sources tell them apart by the item's class" | partial: S72's first route is the export's `is_local` per id, its second the mod table by item class |
| S90 | refuses base defences | holds |
| S105 | twins are one line, told apart by category | holds (`resolve.test: ["ARMOUR", null]`) |
| S106 | outward, several ids emit a `count` min 1 | holds |
| S107 | never applies near templates; inherited | holds (unknown kept, never guessed, no fuzzy match) |
| S111 | the moved-text path; inherited | holds as the claim; the register row's loud failure is not met (F6) |
| S113–S115 | the bare-word habit | out of lane (F7); S113 is the clipboard/Ctrl+F mechanism, S114 the game's box, S115 ten shipped defaults |
| S121 | the counter and the streaming read are not on the read surface | holds as a record of the surface; the counter's absence is inference (F17) |
| S128 | ingest coordinates `realm league tab character container` | holds (6 ingest facts; the design renames location_kind/location_id and carries `socketed_in` as "place names its parent") |
| S131 | the league ambiguity resolved to the current-truth join | holds |
| S132 | the stamps `first-seen last-seen removed`; the read surface | holds |
| S133 | the read surface | holds (aggregates and `events_since` only, no grouping) |
| S134 | 2,335 ms is today's sorted `search("")` | holds (full scan, temp sort, one parse per item) |
| S135 | persisting the projection is the third class; the counter may be a schema cost | holds |
| S144 | "0.2 ms on average" | holds (2.6 ms for twelve at 36,139; the average is the design's arithmetic and is stated as one) |
| S146 | the bench's indexed 18-column schema | holds |
| S150 | 202 ms of parse at 36,139 | holds |
| S151 | 5.2× cheaper load | holds |
| S152 | a one-shot process is the wrong seat at a million | holds for the scan half; S152's SQLite comparison assumes a persisted schema this design does not have |
| S155 | a line with no trade id cannot cross | holds (32.6 %) |
| S157 | 162 ms as an upper bound on building the two tables | holds; the read as an upper bound is labelled judgment |
| S158 | absent must not read as 0 | holds (35,956 of 36,139 matched) |
| S171 | kind is part of identity | holds; R1's "and, where a mod is named, the mod" is declined under S52 |
| S173 | `-is:x` no, silence any | holds |
| S174 | R4 and R9 are one read | holds |
| S176 | Place | holds |
| S177 | crucible lines as a derivation of origin; refused as a fact; inherited | holds |
| S178 | legacy is the user's; OQ2's values | holds |
| S180 | worth refused | holds (non-goals: a pricing engine) |
| S183 | 2,335 ms for 22,721 items | holds (the CLI figure, correctly attributed) |
| S186 | the bare word fixes the silent miss | holds (`items search Explode` empty against 32 items) |
| S187 | "S187 measured 477 items" | holds (477 over the projection's 36,139) |
| S190 | R9, the vocabulary before rows | holds |
| S195 | the departure from R14 | holds; declared as judgment |
| S198 | any printed id or unique prefix | holds |
| S199 | a wrong enum value errors with the legal ones | holds as the claim; contradicted in mechanism (F2) |
| S200 | the template convention and place names in the schema; the three conventions | holds |

Non-`S` reaches, both marked: engine-bench F1's worst column (1.27 ms)
and trade-query F2's `weight` tip, each labelled `undigested` per note
29. Decisions cited: C1 (verified, `CONTEXT.md` — the dependency check
and C12's two surfaces as edges), C12, C46, C48 (verified — "The SQLite
schema is internal; raw SQL is not a surface"; the design's contract is
two derived tables, not the facts schema), C53, C79. The store park the
design quotes twice is verbatim: `decisions/store.md` — "Search-at-scale
(FTS at ingest, a search crate) → behind the store API (C48)."

**Choices resting on no claim and not labelled judgment:** `sockets`,
`colors`, `stack` (F8); `note` as intent (F13); the default scope
("every realm and league, live items", §4.2); the contract "versioned by
one integer" (§3); "never crossed" for multi-name counts (§1); `or`
binding looser than adjacency (§2). The design does label as judgment:
the sum rule, the mean inside `sum`, the bare word, the 162 ms upper
bound, the `weight` → `sum` reading, the query-as-string departure from
R14, and the 500 ms / ≈ 90 k trigger ("arithmetic, not a measurement").

## 4. The cold start, counted

The stranger holds exactly what §6 says: "the grammar block above, the
field list, the tool's arguments by name (`query count text sort desc
columns expand next`), and '`count: line` lists the lines your items
carry.'" So: §2's grammar block with its twelve inline examples, the 29
field names, eight argument names, one sentence. Not held: §1's prose,
§3's totals and derivations, the kinds list, the flags list, the class
vocabulary, the rarity values, §4.2's default scope.

**OQ1** ("Do I have a rare that I can use to flesh out the resistances
or attribute requirements for a build I'm testing?")

| # | call | what the design says it returns | what the stranger has after it |
| --- | --- | --- | --- |
| 1 | `search {query:"rarity:rare class:ring", count:"line", text:"resist"}` | "validates `rarity`/`class` spellings (an error lists the legal values) and returns the resist lines and the shipped totals that cover them" | Per §4.3, the resist lines only: kind, template, items, value range. The totals are not in this read's definition (F3). Per §3's substring rule there is also no spelling validation to be had (F2). The `text` argument's meaning is a name in the list with no stated semantics. |
| 2 | the same read with `text:"strength"` | "because attributes are totals and the description names none" | the attribute templates |
| 3 | the OQ1 query with `sort` | the answer | — |

My count: **4** to reach the appendix's OQ1 query — calls 1 and 2 give
templates, not totals, so a third call (help, or a read whose definition
§1/§4.3 would have to carry) is needed before `total-res>=60` can be
typed, and the query is call 4. **3** to reach a correct answer in a
different query: the templates from calls 1–2 compose as
`sum("#% to Fire Resistance", …)>=60`, which §2 defines. The design's
count: **3** ("two if the description spends the space to list the 35
totals by name"). First parting: **call 1**, on its stated return.

**AQ5** ("+# to maximum Life at 90 or more")

| # | call | note |
| --- | --- | --- |
| 1 | `search {query:"\"# to maximum Life\">=90", sort:"# to maximum Life", desc:true}` | The query clause is printed verbatim in the grammar block the stranger holds. `sort`'s value type is not stated in the enumerated description (§1 states it, and §1 is not in the description); passing a term's thing is a guess that the arg name supports. |

My count: **1**. The design's count: **1**. No parting. The design's
added claim — that a miss "shows the near templates — so one call is
also the call that tells it that it was wrong" — is §4.1's behaviour and
is not in the stranger's description; it costs nothing here because the
question carried the template.

## 5. The concepts

### Deletable, tested against the twelve

| # | concept | can the twelve still be asked without it | line |
| --- | --- | --- | --- |
| 1 | Item | no — every one of the twelve returns items or counts of items | §7.1 |
| 2 | Line (kind, template, numbers) | no — OQ1, OQ2, OQ3, OQ4, OQ7, AQ5 each name a line | §7.2 |
| 3 | Name | no for the field half (OQ1, OQ5, AQ1, AQ4); **the user half is not deletable either** — OQ5's one-query form is a user name, against the entry's "the model stands without it" (F12, F4) | §7.3 |
| 4 | Term | no — every query is terms | §7.4 |
| 5 | Absent is false | askable without it; OQ5's answer changes (its `reqlevel<=30` would admit every item lacking the requirement, S158's failure) | §7.5 |
| 6 | Value is the sum | **yes** — confirmed: no appendix line needs it (AQ5's comparison works per line; OQ1's totals are explicit `sum`), as the entry itself says, and §9.1 hands it to the owner | §7.6 |
| 7 | Composition | and/or/not needed (OQ1, OQ3, OQ5); `sum` needed only inside the totals' definitions — "`sum` is AQ2" is indirect, AQ2's text carries `total-res`, not `sum`; **`count(…)` deletable**, "used by none of the twelve", as the entry says | §7.7 |
| 8 | Place | no — OQ4, OQ5, OQ7, AQ1 | §7.8 |
| 9 | Answer | no — AQ3 is this concept | §7.9 |
| 10 | View | no — AQ1 (counts), AQ3/AQ4 (one item) | §7.10 |
| 11 | The two-table contract | **yes** — confirmed: no appendix line is SQL; "the floor's SQL surface is its whole justification" | §7.11 |

### Used by the text, not in the inventory

| concept | line |
| --- | --- |
| Canonicalisation (one function over corpus and query; sign folded into the number; markup to its display half) | §1: "the query's text is canonicalised by the same function, so no convention can cause a miss" |
| Flag (the item's booleans, `is:`) — distinct from kind | §2: "flags (the item's booleans) are closed lists in the help"; `is:corrupted` |
| Scope, and its default | §1's `scope` row; §4.2: "default scope is every realm and league, live items"; §1: "across the kinds in scope" |
| Freshness / fetch age | §1: "oldest fetch"; §4.6: "freshness is the location's fetch age" |
| Revision, and the change counter | §3: "the answer names the revision read" |
| Intent, joined read-only | §3: "Intent, read-only: `priced price note`"; §4.6 |
| Reference data (the class table; later the stat-id table) — the inventory names totals only | §3: "Two reference tables, committed and reviewed like the currency table" |
| The trade correspondence (node-for-node mapping, undecided ids shown, `count` min 1) | §4.5 |
| The number rules (`*` drops a number; every number must satisfy; the mean under `sum`) | §2 |
| Pagination (`next`: returned, total, the continuing argument) | §1's answer shape; `next` in §6's argument list |
| Sort and columns | §1: "Sort, extra columns and count-by"; §6's `sort desc columns` |

## 6. The floor, the stances, the limits

One row per item checked; "no breach" is a result.

| item checked | verdict | line |
| --- | --- | --- |
| A gap routed to SQL (stance 6; note 22's acceptance test) | no breach — all six declared gaps (OQ5's grouping, OQ6's worth, the 27 trade filters, cross-item, quantity-to-quantity, cross-tabs) are listed, none routed | §5's two gap paragraphs; §3: "Both languages read these and only these, so reach is one property (stance 6)" |
| A reading that could be stale (floor: "by construction and not by a refresh") | **breach, conditional** — F1 | §3 |
| Ditto, mid-answer ingest | **breach, declared** — "an ingest landing mid-answer is the next ask's mismatch", so that answer is served from the superseded derivation | §3 |
| C48 ("No cached search service") | no breach as written — the derivation is per-process and revalidated, and the SQL contract "is these two tables", never the facts schema | §3; C48 verified in `CONTEXT.md` |
| Game knowledge the app is asked to hold (stance 4, S164) | no breach — the class derivation is what register row S53 sanctions ("a class the search names is a derivation it owns, and an item it cannot class is shown unclassed"); the design ships it unfetched (C79) and routes the access method to the owner | §3: "Anything unjoined is `class:unknown`, counted in scope"; §9.2 |
| Ditto, the 35 shipped totals | no breach — sums of the corpus's own displayed lines, which stance 2 puts in reach, "printed with its definition" and readable in the grammar | §1, §3 |
| Ditto, legacy | no breach — OQ2 and OQ6 leave the variant and its values to the user (S178) | §5 |
| An everyday question needing the advanced machinery (stance 3) | **breach, declared** — OQ5 needs a user-defined name or an explicit `or` of classes plus an absence clause: "The `or` is where an ordinary question feels like programming" | §5, OQ5 |
| Ditto, the stance-3 shape itself (a precise value among broader requirements) | no breach — `class:gloves fractured:"suppress"` and `class:ring "#% to Fire Resistance">=40 rarity:rare` are one line of the grammar's easy case | §5's refined OQ1; §2 |
| Register row S12 (a value query never matches a veiled line) | **breach** — F5 | §1, §8 |
| Register row S14 (map areas) | no breach — refused by name | §8 |
| Register row S52 (which mods made a line) | no breach — refused by name; §4.1 restates it | §8, §4.1 |
| Register row S53 (class as an owned derivation; unclassed shown) | no breach | §3 |
| Register row S67 (both candidates, never guessed) | no breach — "both candidates, shown as undecided, never one guessed" | §4.5 |
| Register row S107 (unknown shown, never fuzzy) | no breach — fuzzy matching refused; near templates shown, "never applies them" | §8, §4.1 |
| Register row S111 (the moved text; a relied-on name fails loudly) | **breach, partial** — F6 | §4.1 |
| Register row S177 (league of origin as a derivation) | no breach — OQ4 "offered as one, never the answer" | §5, §8 |
| Register row S178 (legacy is the user's) | no breach | §8 |
| Stance 2's floor of reach (every question the site can ask of an item) | no breach — the 7 market filters read a listing, and the 27 unsettled are the ones S42 says "no private field gives"; both named, not silently dropped | §4.5 |
| Stance 2's ceiling of vocabulary (the corpus's own lines) | no breach — "The vocabulary of lines is the corpus's own (stance 2)" | §1 |
| Stance 1 (not tuned to one user's questions) | no breach — the shipped totals are S29's 35, a tool's rule, and `gear` is the user's, not shipped | §3, §5 |
| An owner claim contradicted (S68) | **contradiction, declared** — F18 | §2 |
| C53's three levels | no breach — rows as the decision view, `--expand` as the audit view, the tree in JSON | §1 |
| C12 / C46 (two surfaces, adapters) | no breach — "a library crate frontends link, reading only through the store API"; the C1 edge routed to the owner | §3, §9.4 |
| C35 / saved searches as intent | no breach — a saved *result* refused, "a name saves a query, so it cannot be stale"; the `user.db` park named | §8, §9.3 |
| Realm above league | no breach — `realm` a field, default scope every realm, `class` covers PoE2 via S9 | §3, §4.2 |
| Note 22's kill-list rule (`undigested`) | no breach — both reaches marked | §3, §4.5 |
| R11 / S192 (a truncated answer says total, returned, how to continue) | no breach — the `next` block, uncited | §1 |
| The 16 KB guide ("never a target") | not a breach; the file is 20,032 bytes | — |

## 7. The one page

| measure | result |
| --- | --- |
| §§1–2 as presented | 109 lines, 1,086 words, 6,422 bytes (lines 3–111) |
| Fits one page | no, on either measure — over two printed pages at 1,086 words, and over two terminal screens at 109 lines |
| A second implementer builds the same search from it | no |

What they would have to fetch, by section (F9):

- **§3** — the totals' names and definitions (OQ1, AQ2 need them);
  `colors:rrg`'s multiset rule, which the page's `thing:text` would read
  as a substring (OQ3); the derivations of `armour evasion es reqlevel
  pdps edps dps tier links sockets colors class`, i.e. twelve of the
  twenty-nine fields; the class table and `class:unknown`; the two-table
  contract and the revision comparison the answer's scope block reports.
- **§4.1** — that identity is (kind, template), and the whole moved-text
  behaviour.
- **§4.2** — the default scope (OQ7).
- **§4.5** — the trade correspondence.
- **§4.6** — the place coordinate's parts, freshness as fetch age, and
  the id prefix rule (AQ4).
- **§6** — the argument names (`query count text sort desc columns expand
  next`); the page names `--count`, `--sort`, `--desc`, `--expand`,
  `--against` and `show` but not `text`, `columns` or `next`.
- **Nowhere in the design** — the kinds list and the flags list, both
  deferred to "closed lists in the help"; the class vocabulary; the
  rarity values; the syntax for defining a name (F4).

## 8. The output shape

Note 22's nine parts and the closing line.

| # | part | verdict | where |
| --- | --- | --- | --- |
| 1 | The model on one page | present; over one page (§7 of this audit) | §1 |
| 2 | The grammar in as few lines as it takes | present — one block, 17 lines, plus 20 lines of rules after it | §2 |
| 3 | The derivation the model needs (what it carries, its contract), after the model | present, and after — two relations, the contract "these two tables with the template convention and place names in the schema itself (S200), versioned by one integer", persistence declined with a numeric trigger | §3 |
| 4 | An answer to each of the seven questions | present, all seven numbered; three answer by pointer (Q2 "sections 1–2", Q4 "section 3", Q7 "section 8") | §4 |
| 5 | The appendix: every acceptance question, one query or a listed gap, discovery and refinements apart | present — twelve lines, D on OQ1, R on AQ2, plus two gap paragraphs | §5 |
| 6 | The cold start: owner Q1 and one agent scenario, counted | present — OQ1 in 3, AQ5 in 1, with the stranger's description enumerated | §6 |
| 7 | The concept inventory, one line each on why the model cannot do without it | present — 11 entries; two entries (6, 11) state that the model can do without them | §7 |
| 8 | What the design refuses | present — fourteen refusals, nine inherited limits, two added | §8 |
| 9 | The decisions left to the owner | present — four | §9 |
| closing | What was left out; the one cut to reverse | present — five things left out, the least-sure item, and one cut named ("tags") | "Left out, and the cut I would reverse" |
