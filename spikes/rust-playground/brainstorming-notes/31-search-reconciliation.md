# 31 — The search reconciliation (stage 4, step 5)

Provenance: Fable (`claude-fable-5-1`), a fresh session, under note 23's
"Stage 4: how it runs", step 5. Read: note 23, note 22,
`search/DIGEST.md`, notes 24 and 25 with their `## Repairs` tables, both
second-round audits (`search/proposal-audit/`, at `cb2a6fdb`), notes 26
and 27, note 32, and the commit stories of `cb2a6fdb` and `548ecc4f`.
Reached to verify a claim: item-facts F2 and F3 with
`data/field-census.csv`, trade-query F2 with `data/gap.csv`,
engine-bench F1, and the two parks in `decisions/store.md`.

**Design A is note 25 (Astra's); design B is note 24 (Fable's).** I am
Fable, so by note 23's guardrail a recommendation of mine that lands on
B is the weak direction and one that lands on A the strong one. Each
recommendation below is marked **→A**, **→B**, **→both** or **→new**, so
the owner and Astra's check can discount accordingly. Every row is a
recommendation to the owner, never a ruling.

**The owner's seat (step 4) has not been run.** It is optional and is
worth the most before the reviews are read: ten minutes asking OQ1 of
`search/designs/design-a.md` §1–2 and `design-b.md` §1–2. The reviews'
verdicts start at "What the users said"; stop above it to keep the seat
clean.

## How the evidence is counted

Three kinds, never added together (note 23, guardrails).

- **Audit findings** count by the line they point at. Second round: 18
  on B, 20 on A; after round 2 of the repair, B has 4 left unanswered on
  purpose (F1 freshness, F4 the define syntax, F9 the page, F18 the mean
  inside `sum`) and A has 3 (F5 the reach floor, F6 the everyday gap,
  F20 the size) — "what the reconciliation and the ruling weigh, not
  slips to repair" (`548ecc4f`). Neither author disputed a finding in
  either round, so there is no disputed finding to settle by the line.
- **Agreement** counts by its evidence. Where A and B agree while
  standing on the same text, the row says which text, and that is one
  piece of evidence about the text.
- **Testimony** is one seat's experience. The two reviewers split on the
  whole: Fable would work in B, Astra in A. Each chose its own model's
  design, so both verdicts are the weak kind and they cancel; neither is
  used below. What is strong is what each reviewer said *against* that
  grain, and there is a good deal of it:

| Said by | For the other model's design |
| --- | --- |
| Fable (note 26) | A's same-occurrence binding ("B has no way to say either exactly"); A's summaries over the set (B cannot total a stack: "not in the design's own gap list"); A's don't-know-is-not-no, "for what could not be read"; A's `where (old) and P`; A's explicit scope, "a chore in use and is right here" for PoE2; A's versioned saved document |
| Astra (note 27) | B's discovery while searching ("its strongest idea for me"); B's per-term resolutions and counts; B at a terminal ("I would lean B"); B's user names, as an idea it did not exercise; its doubt of A's conservative totals ("I am least sure…") |

Both reviewers are agents; no human has used either design.

## What the users said (both seats, in one place)

- **The sum rule cost trust in B.** Astra, AQ5: "I would ordinarily read
  this request as a line with at least 90, and B's own 20 + 75 example
  would qualify here… This was the largest trust cost I encountered."
  The wand range (low ≥ 12, high ≤ 28 on one line) is unanswerable in B.
  The same rule was welcome on the gloves ("I want the contribution from
  the gloves").
- **Unknown cost work in A.** Fable: three of eight questions ran into
  unknown; "absent by nature" and "could not decode" share one counter;
  "A counter that is nonzero on every ordinary query is one I will stop
  reading by the second day." Astra on `not present(…)`: "I would not
  describe this predicate as effortless." A's own appendix OQ5 omits the
  absence branch its §4.2 teaches.
- **The absence clause cost both.** OQ5 needs `-has:reqlevel` (B) or
  `not present(required.level)` (A); both reviewers nearly omitted it in
  both designs. In B the answer's per-term "items lacking it" teaches
  it; in A the unknown count does, less pointedly.
- **Aggregation felt like programming in A** (Fable F6: `max` sorts
  life-less rings last with no score, `sum` was right; Astra AQ5: "a
  small database exercise") **and its absence was a wall in B** (Fable
  F2: stacks of currency cannot be totalled; Astra Q5: `--count
  league,class` returns two tables and no error).
- **B's modes are silent.** `#` or not inside quotes; a bare word that
  is secretly a name; `:` a substring except on `colors`; `or` looser
  than adjacency, so appending a term to OQ3's text narrows one branch.
  A has no modes; its cost is length and a describe call before most
  queries.
- **Both reviewers wrote text, in both designs.** Fable on A's MCP JSON:
  "As an agent I would rather send the text"; Astra "used text notation
  to expose meaning".
- **Neither reviewer reached for SQL or felt the engine.**

## The decision table

The index; each row's content follows under its number. "Agree" is about
the designs; "evidence" says whether that agreement is independent.

| # | Question | Agree? | Decided by | Recommendation |
| --- | --- | --- | --- | --- |
| 1 | A line's identity | mostly; one text | S3, S4, S17, S68, S71, S107, S111 | (realm, kind, template), sign in the number, numbers as ordered slots **→both** |
| 2 | The query model and its one grammar | tree yes; surface no | S169, S172, S195, stance 5 | one tree; B's term surface carried as text on every seat, the tree accepted and always returned **→B, with →A parts** |
| 2a | A line's number: one line or the item's sum | **no** | S27, S28, S68, both reviews | one occurrence; a sum is asked for by name **→A** |
| 2b | Absent, unknown, false | **no** | S158, S107, both reviews | two-valued with a per-term lacking count, plus a could-not-read count **→B, with →A's conscience** |
| 2c | Named totals: a sum that answers, or one that may decline | **no** | S29, S49, S111, A's own F6 | a sum that answers, its definition and contributions shown **→B** |
| 2d | Summaries over the set | **no** | S170, S191, S189 | counts may cross and may carry a summed column **→A's reach, →new spelling** |
| 2e | Scope | **no** | the realm ruling, S199 | terms, never a default that spans realms **→A's rule in →B's spelling** |
| 3 | The vocabulary read | the need yes (one text); the call no | S174, S190, S181 | the counts view under a query, each row carrying the term that selects it **→B, with →A's fragment** |
| 4 | Who holds the corpus; the derivation and its contract | the holder and the contract yes; persistence no | S135, S150–S153, S157, the engine-gravity warning | not persisted first, behind the one contract; persistence when a number asks **→B**; the floor question is the owner's |
| 5 | The trade boundary | yes; one text | S41, S47, S56, S67, S106, S155 | B's node mapping inside A's remainder report; the site's average lives only in the translation **→both** |
| 6 | What a result carries | yes | S176, S192, S193, S196–S198, C53 | B's one answer shape, A's continuation bound to the revision **→both** |
| 7 | Limits and non-goals | yes; inherited | the register | inherited whole by both; three additions below **→both** |
| 8 | Sockets: a collection, or flat fields | **no** | S16, S22, S58 | colours are asked within a link group; no general collection **→A's reach, →new spelling** |
| 9 | A grouping above class | **no** | S42, S54, S69 | the trade category's prefix, carried by the class table **→new** |
| 10 | A raw-path accessor | **no** | stance 6, S127, S135 | none; a missing field is a field to add **→B** |
| 11 | Saved queries, user names, tags | the home yes; the form no | note 22's floor, the `user.db` park | out of the first slice; the read model first **→neither** |
| 12 | What the MCP carries | **no** | S195, both reviews | the text; the tree accepted **→B**, against R14 — the owner's |

### 1. A line's identity

- **A:** an occurrence is (item, source array, ordinal); its family is
  (realm, normalized template, source, flags). Sign belongs to the
  ordered slots `n1`, `n2`; markup unwrapped; a rename is a new family
  and an unbound saved template is a named diagnostic; twins stay one
  family unless a reviewed rule resolves them.
- **B:** identity is (kind, template); kind is the array plus the
  `crafted` and `fractured` flags; sign folded into the number; markup
  to its display half; a comparison holds when every number on the line
  does, `*` dropping one; twins are one line; an exact template that
  resolves to nothing shows the templates sharing its words.
- **Agreement, and its evidence.** Both: displayed text is the identity,
  no stat-id spine, no hash, the mod behind a line unknown, unknown shown
  and never fuzzy-matched, the same sign convention out of the three the
  digest lists. Most of this is one text: note 22's convergence signals
  ("the text is the moving part") and its RePoE gravity warning push it,
  and S52, S107, S111 are register rows both inherit. Independent: both
  chose the same sign convention unprompted, and both put the stat id at
  the trade boundary only.
- **A fact both designs left open, settled by the track's own data.**
  A lists private crafted/fractured decoding as unverified (its F5 gap);
  B offers `fractured:"…"` flatly; both reviewers saw the designs
  disagree and could not settle it. item-facts F3 (not in the digest)
  counts the flags on private `explicitMods` lines: `crafted` 1,041,
  `fractured` 289, `mutated` 81, none on `implicitMods`. B stands; A's
  gap closes. New claim S17 below.
- **Differences that remain.** Realm in the family (A) or a field beside
  it (B): with B's default scope a template and a total span two games
  (note 26, "two steps out"). Numbers: A's positional slots against B's
  every-number rule with `*`.
- **Recommendation →both.** Identity is (realm, kind, template); kind is
  the source array with the flags the line carries, as the data has it;
  sign in the number; numbers are ordered slots, and B's `*` is a
  spelling of "this slot unconstrained" (S68). Realm in the family is
  A's and is the realm ruling applied.

### 2. The query model and its one grammar

- **A:** `in S where P select … summarize … facets … order … inspect`;
  P is Boolean over fields, `line(…)`, `any(collection, P)`,
  `at_least`, arithmetic, `if`, aggregates. Text for the CLI, JSON for
  the MCP, one tree.
- **B:** terms side by side; `-`, `or`, parentheses, `has:`, `is:`,
  `thing:text`, comparisons, ranges, `holds(…)`, `sum(…)`; views
  (`--count`, `show`, `--against`) on one operation. One tree
  (`FromStr + Display + Serialize`), the string on every seat.
- **Agreement:** one typed tree, every surface an encoding of it;
  and/or/not/at-least-N across every field. The composition half rests
  on one text (stance 2, S169, S172). Both reached "the answer returns
  the canonical query" without being handed it.
- **The audits:** both model pages fail to stand alone (B F9, A F7 and
  its own §4.7); of the twelve, B holds 6 on the page and 5 off it, A 1
  and 8. Neither has a claimed query that fails.
- **Note 22's derivation-gravity warning** reads: "A grammar that is a
  WHERE clause in new spelling was designed from the table." A's grammar
  has `where`, `select`, `order` and `summarize … by`. Neither audit
  raised it; I name it as my reading, in the weak direction.
- **Recommendation →B for the surface, with →A parts.** The term surface
  is the one both reviewers could type cold, and Astra leans to it at a
  terminal. It takes from A: refinement defined as `(old) new`, with the
  parentheses, since B's own OQ3 text breaks under a bare append;
  same-occurrence binding (2a); and no silent modes — the three the
  reviews met (bare word as name, `#` inside quotes, `colors`) each want
  one visible rule or an error. That list is the first thing the build
  plan should test, by the mistake walk note 26 wished it had run.

### 2a. One line, or the item's sum

- **A:** a condition inside `line(…)` binds one occurrence; a sum is
  written `sum(lines where …, n1)`. **B:** an item's value for a
  template is the sum of its lines of that template; §9.1 hands it to
  the owner; its own inventory marks it deletable.
- **Deciding claims:** S27–S28 say only that a repeated line must not be
  lost, and both designs keep it. S68 keeps a line's two numbers apart.
  Nothing in the digest says which a player means by "life ≥ 90".
- **Users:** the strongest single piece of testimony in either review
  bears here, and both reviewers — Fable against its own model's design —
  name occurrence binding as what must survive.
- **First order:** a sum can be built from occurrences by a word; an
  occurrence cannot be recovered from a sum. **Second order:** with one
  occurrence as the default, the item with implicit 20 and explicit 75
  is missed by `"# to maximum Life">=90`, silently, for the user who
  meant the item. The answer's terms block is where that shows: beside
  "items matching", "items whose lines together reach it and no one line
  does". That count is proposed here, not in either design.
- **Recommendation →A.** A quoted template under a comparison means one
  line. The item's total is `sum("…")` or a named total. The owner's
  reading decides; B asked for exactly that.

### 2b. Absent, unknown, false

- **A:** three-valued throughout; absent and undecodable both make a
  comparison unknown; the envelope counts matched, false, unknown.
  **B:** absent is false, and each term reports how many items lacked
  it; nothing is ever "could not read" (note 26: "There is no 'could not
  read this' anywhere").
- **Agreement:** a missing value is never zero (S158) — one text.
- **Users:** above. The two seats converge on a form neither design
  has: note 26's "Three numbers per term, not two. Matched, lacked,
  could-not-read", the third almost always zero "which is what would
  make it read when it is not."
- **Recommendation →B with →A's conscience.** Evaluation is two-valued;
  absent is false and counted per term; a body or array the deriver
  could not read is a third per-term count and a scope line, never a
  match and never silent. Three-valued Boolean algebra leaves the model.

### 2c. Named totals

- **A:** a recipe whose predicate stays unknown while a possibly
  contributing line is unclassified — declared by A as an
  everyday-search gap against stance 3 (its F6, unanswered by design),
  and the cut A most wants reversed is the coverage trial that would
  size it. Astra doubts it in use. No section says what rules a line
  out as a contributor (note 26, F1).
- **B:** the 35 pseudomods (S29) as sums written in the grammar,
  definitions printed, each template listed with its count. Its declared
  gap: a moved template and one the stash lacks look the same, so S111's
  loud failure is not met (its F6).
- **A's S111 row** says recipe dependencies "fail validation loudly" and
  gives no mechanism; a check against the corpus cannot tell moved from
  lacking either. Neither design solves this.
- **Recommendation →B.** A total is a sum that answers, its definition
  readable and its contributions on the row. The moved-template gap
  stays a listed limit. A's trial is still worth running, reframed: how
  many lines *containing* "Resistance" does `total-res` not count, on the
  real corpus — a measurement of B's totals, one script over the census.

### 2d. Summaries over the set

- **A:** `summarize count(), sum(stack.count) by base`, and crossed
  groups. **B:** counts by one name per table, never crossed, no sum;
  cross-tabs are a listed gap, the stack total is an unlisted one.
- **Deciding claims:** the owner, S170: "knowing counts and total of
  different currencies, equipable items, and other non-equipable items…
  might be useful"; S191 (R10); S189, the agent's own `GROUP BY 1, 2`.
- **Recommendation →A's reach, →new spelling.** The counts view takes
  several names as one crossed table and an optional summed thing beside
  the count (note 26's third idea). No general aggregation language:
  A names collection aggregation as its largest learning cost and its
  first deletion.

### 2e. Scope

- **A:** explicit, "no implicit 'last league'". **B:** place is terms;
  the default is every realm and league, live items.
- **Recommendation →A's rule in →B's spelling.** Place stays terms. A
  query that names no realm, over a store holding more than one, is an
  error that lists them (S199); within a realm the default is every
  league, live. The answer's scope block says what was searched.

### 3. The vocabulary read

- **A:** `search_describe`, a separate call taking several terms at
  once, returning descriptors, value sets, coverage and ready-to-use
  predicate fragments. **B:** `--count line [text]` under any query —
  "the same call as search" — and a closed list of twenty-nine fields
  printed in the help.
- **Agreement:** R4 and R9 are one read. The digest says so itself
  (S174, S190: "the owner's dropdown is the agent's discovery"), so this
  is one text. What each added is its own.
- **Users:** Astra names B's scoped discovery as B's strongest idea;
  Fable names A's fragments as "the feature that makes this design
  usable at all". Each is the strong direction.
- **The audits' cold starts:** B's OQ1 in 3 calls (the auditor: 4 to the
  appendix's own query), AQ5 in 1; A's OQ1 in 5 terminal invocations
  (auditor: 6), AQ5 in 2 (auditor: 3). The counts are not like for like
  (A counts help invocations), and note 26 observes both assume a right
  first query.
- **Recommendation →B with →A's fragment.** The vocabulary is the counts
  view under the query in hand, several texts in one call; each row
  carries the exact term that selects it. The field list, the kinds, the
  flags and the totals' names are printed in the help — the audit's F9
  list is what the help must hold.

### 4. Who holds the corpus; the derivation and its contract

- **A:** a store-owned projection maintained in the fact transaction,
  with a facts revision; frontends hold arrays valid only at a matching
  revision; the CLI loads per command. Its own gaps: the projection's
  load is unmeasured at either scale, and the CLI's cold response is
  unestablished. **B:** nothing persisted; a library crate derives from
  bodies on first ask and compares the store's change counter before
  every answer; persistence is "the same contract moved", triggered by a
  measured CLI ask over 500 ms. Its own gaps: the body read is
  unmeasured, and a keystroke after a store change pays the whole
  re-derive — under a refresh, the common case.
- **Agreement:** no search service; long-lived frontends hold the
  corpus; a revision the store advances, checked before an answer;
  the SQL contract is the derived relations with the template convention
  and place names in the schema (S200), never the facts schema. Both
  need a counter the store does not have. The bench claims are shared
  text; the no-daemon conclusion is forced by C2 and C34.
- **Deciding claims:** S150–S152 (the load, not the query); S135 and the
  store park "Search-at-scale … Trigger: a real consumer with a measured
  latency or duplication case"; note 22: "An index, an engine or a
  persisted anything is added with a number, not an instinct."
- **B's F1, the one floor question left open.** The floor says the
  search "cannot be stale, by construction and not by a refresh". The
  auditor marked B's per-ask counter comparison a conditional breach and
  A no breach. But A's held arrays use the same comparison; A differs
  only in that the persisted copy is written in the fact transaction. A
  comparison before every answer, against a counter advanced in the
  transaction that changed the facts, has no timer, no TTL and no
  message to miss. I read it as construction; if it is not, both designs
  breach for every held array. The owner's line.
- **One thing the audit passed that I would look at:** A's projection
  runs the extractor inside every fact mutation, so derivation code runs
  in the daemon's process on the write path. A says the daemon "gains no
  search, fact-reading or intent-reading behavior"; whether that holds
  C34's spirit is a boundary question, and it is S135's third and
  costliest class of change.
- **Recommendation →B.** Build the contract both agree on, unpersisted,
  and measure the two numbers neither design has: the streaming body
  read, and the re-derive under an active refresh. The second is the
  trigger most likely to fire, and when it does, A's transactional
  projection is the design to move to. Where the crate lives (a library
  the frontends link, or behind the store API as the park words it) is
  the owner's boundary.

### 5. The trade boundary

- **Agreement:** a URL decodes with no network (S56); `and`, `not`,
  `count` → at-least-N, `weight` → a weighted sum, `if` → an
  absence-or-comparison; market filters named and dropped; a line with
  no id cannot cross (S155); building it is new scope. All of it stands
  on the same claims and note 22's triage (c): one text.
- **Differences:** A wraps translation in a per-clause report (exact,
  ambiguous, unsupported, listing-only) and a nonempty remainder blocks
  running the result as equivalent; several ids are OR-ed only where the
  mapping proves the union. B emits several ids as a `count` min 1
  (S106, Awakened's practice) and shows S67's undecided pair as both
  candidates — which is the register's wording; A's row is narrower.
- **B's F18, left open:** inside `sum` a two-number line gives its mean,
  against the owner's S68. The site's average (S47) is a fact about the
  site, so it belongs in the translation of a trade query and nowhere
  in the core — which is what A does ("Averaged trade range bounds
  translate to arithmetic over both slots").
- **A second-order point neither design draws:** the site's `ar`, `ev`,
  `es` are quality-normalised (S26) and both designs' `armour` is the
  displayed property, so a translated defence bound is inexact and
  belongs in the remainder until a normalised field exists. It is
  derivable from lines, so stance 2 puts it in reach.
- **Recommendation →both.** B's node-for-node mapping as the content,
  A's remainder report as the form; S67 shown with both candidates; the
  mean confined to the translation. `sum` over a two-number line in the
  core is an error that names the slots.

### 6. What a result carries

- **Agreement:** a compact row — id, name, base, rarity, place by name,
  the lines the query touched; the body only by `show`; C53's levels;
  total, returned and how to continue (S192); price joined read-only;
  legacy never a field; every printed id accepted back (S198). The R-
  lines are shared text; the shapes are close beyond them.
- **Differences:** B's per-term block against A's per-answer envelope
  and `inspect` (the same thing as `show --against`); A binds a
  continuation to its basis and returns `basis_changed`; A distinguishes
  manual, observed and effective listing state (C64, C69) where B has
  `priced price`.
- **Recommendation →both.** B's one answer shape with the terms block
  (2b's three counts); a continuation is refused across a revision
  change, with the query to restart (A). `priced` means the effective
  resolution the pricing area already defines; the other two are fields
  to add when a question needs them.

### 7. Limits and non-goals

Both inherit the register whole and refuse the same things: fuzzy
matching, valuation, judging legacy, fetching, acting on the stash,
recovering the mod behind a line. B's two declared exceptions stand as
listed limits: S111 for shipped totals (2c), and twins summed on one
item — which 2a removes, since occurrences are no longer summed.
Additions this reconciliation would list: the could-not-read count is
the only form "unknown" takes; a defence bound from the site is inexact
(row 5); base defences need game data and stay the user's (stance 4).

### 8. Sockets

- **A:** `any(socket_groups, red >= 2 and blue >= 1)` — colours within
  one link. **B:** `links`, `sockets`, and `colors:rrg` as a multiset
  over the whole item. Both marked the body's socket data as unverified
  (A a coverage gap, B "an assumption").
- **Settled by the track's data:** `field-census.csv` has `sockets` on
  5,922 items (16.4 %), `sockets[].group` on 5,918, `sColour` and `attr`
  on 5,894, and `stackSize` on 2,749 (7.6 %). Both gaps close. New claim
  S16.
- **The floor:** trade-query `gap.csv` lists two socket filters of one
  shape, `sockets` and `links` ("Link Groups"), each min/max with
  colours; that the second counts colours inside a link group is my
  knowledge of the site, not the capture's ("computation not in the
  capture"). If so, B's multiset is below stance 2's floor, as note 26
  found by use. New claim S58.
- **Recommendation →A's reach, →new spelling.** Two colour fields, one
  over the item and one within the largest link group; no general
  `any(collection, P)`. The one other collection A names is lines, which
  the term already binds (2a).

### 9. A grouping above class

B ships none and lists OQ5 as a gap needing a user's name (its F7, F10);
A has `equipable` from "that taxonomy", undefined. The digest already
holds a grouping players know: the trade `category` ids are dotted
(`accessory.ring`, S54), 65 of the 68 leaves generate from the export's
classes (S69), and `category` is the largest of the 27 filters B leaves
unmapped (S42, its F11). **Recommendation →new:** the class table
carries the trade category id, so `category:armour` and
`category:accessory` are substrings of a value the player has typed
before. It closes most of F11 and OQ5 without a user name, and adds no
concept.

### 10. A raw-path accessor

A publishes `fact["API.path"]` over the retained body, decoded per
query at an unmeasured cost (its F10), "an escape from incomplete
naming". B has no such door: what displays is a line, the rest is a
field or `show`. Stance 6 makes reach a property of the derivation, and
S135's second class (a derived column, one `rebuild`) is the cheap way
to add a field. **Recommendation →B:** no accessor. Fable's review met
its cost in use: `fact["corrupted"] = false` is unknown on nearly every
item where the named flag is false.

### 11. Saved queries, user names, tags

Both put a saved search in user-scoped intent through the store — note
22's floor, one text. A saves a versioned document that refuses to
reinterpret itself; B saves a name usable as a term, and would most
like its cut tags back. B's define syntax does not exist (F4), its
inventory calls user names the first deletion, and a name silently
changes what a bare word means (note 26). Astra likes names as an idea
it did not exercise. Every form is a write and fires the `user.db`
park. **Recommendation →neither, yet:** the first slice is the read
model; row 9 removes OQ5's dependence on a name. When the park fires,
A's version rule and B's composability are both wanted, and the bare-
word collision is solved first.

### 12. What the MCP carries

A: the tree as JSON, of which the page shows two lines (note 26:
"a promise"). B: the string, a declared departure from R14 (S195: "a
query object, not a text argument"), "on judgment, not measurement".
Both reviewers wrote text. **Recommendation →B:** the text on every
seat, the tree accepted where a client holds one, and returned in every
answer — which keeps R14's intent (a refinement is the previous query
plus one term, parenthesised) if not its letter. It departs from a
recorded requirement, so it is the owner's.

## Audited clean and a labour to use, or the reverse

- **A's three-valued logic:** no audit breach — the auditor lists it as
  a choice resting on no claim — and it was the largest cost in use, for
  both reviewers, on the design's own everyday questions.
- **A's same-occurrence binding and B's sum rule:** the audit found B's
  sum rule deletable against the twelve and otherwise clean; use found
  it the largest trust cost. The audit tested A's binding only as "AQ5
  needs it"; use found it the thing most worth keeping.
- **B's modes:** the auditor caught one (F2, `:` on a closed value set)
  and repair fixed it; use found more — the bare word that is a name,
  `#` inside quotes, `or` under an appended term, `--count a,b` not
  crossing — none an audit finding, because each is stated on the page
  and is still a surprise.
- **B's F9 and A's F7** (the page does not stand alone) mattered in use
  only through names: both reviewers guessed field and total names, and
  B's printed field list is why Fable "never made a discovery call for a
  field".
- **Size.** A is 48,881 bytes against the 16 KB guide, B 26,405 with its
  repair tables (44,271 and 20,740 as the reviewers read them). Note 26
  counts a bolded "Gap:" thirteen times in A and reports having to
  decide "whether a sentence offered a feature or withdrew one". I read
  A's length as repair sediment more than design: two audit rounds asked
  it to declare, and it did. The model page itself is 3,351 bytes,
  smaller than B's 6,422.

## The withheld checklist

| Seed | Reached unprompted by | Note |
| --- | --- | --- |
| The query as a value | both | A: the query document, saved, versioned, handed between consumers, a trade URL in and out. B: `FromStr + Display + Serialize`, the string handed between a human and an agent. R14 (S195) hands "a query object" to both; the answer returning its canonical query is each design's own. Neither cites C38 as the mold |
| The vocabulary is a read | both, and handed | S174 and S190 say it; evidence about the digest. B's "same call as search" and A's fragments are their own |
| Identity is the template plus its kind; the stat id at the boundary | both | pushed by note 22's RePoE warning and S155, so partly handed; the sign convention and the boundary placement coincide unprompted |
| Query by example | **neither**, and neither reviewer missed it | proposed here: the one-item view can print the item as a query — one term per line, `kind:"template">=value` — for the user to edit. A rendering, no new grammar; it needs 2a, since an item's lines are occurrences. Parked behind the first seat that asks |
| The model's semantics as views | A in part; B the reverse | A: "Query evaluation and SQL obtain values from the same derivation". B keeps totals out of SQL ("a total is a query anyone can read"), so a SQL caller must rebuild them and can disagree. Proposed here: the shipped totals are one reference table (template, weight), and both the evaluator and a SQL view are generated from it |
| Every tool that lasted asks in the item's own terms | B explicitly; A in its model, not its grammar | B: "An item is what it displays…", the stash box. A: "Identity belongs to observable evidence", under a `where` clause. Stance 2 hands "the corpus's own lines are the vocabulary" to both |

## The digest's changes

**Added** (this commit; verified against the track data; ids from each
track's free range, never reused):

- S16, item-facts — sockets, their groups and colours, and stack size,
  with coverage. Cited undigested by A; an assumption in B.
- S17, item-facts — the flags private explicit lines carry. Reached by
  this reconciliation to settle row 1.
- S57, trade-query — the `weight` and `weight2` tips. Cited undigested
  by B.
- S58, trade-query — the two socket filters. Reached here for row 8.
- S159, engine-bench — the worst query per scale. Cited undigested by
  B. The number is the data file's (13.3 at ×10; the digest's
  contradictions section already records F1's table saying 13.1).

**Pruning — candidates only, for the owner.** Of 165 claims the two
proposals cite 92: 42 both, 22 B alone, 28 A alone, 73 neither. Uncited
is weak evidence of uselessness: the brief did not ask for a citation
per design move, note 22 carries several owner claims into the stances
(S162, S163, S166 are uncited and shaped both designs), both designs
meet R11 (S192) without citing it, and stage 6 reads the digest as its
tests. By track, the uncited fall heaviest on item-filter (13 of 15) and
prior-art (10 of 15): two whole lanes the designs did not need. So:

- *Kill-list candidates:* item-filter S82–S86, S91, S92, S94, S95;
  prior-art S101–S103, S108, S109, S112–S115; repoe S74–S76; store
  S124–S126. These describe a source's mechanism, which no design
  question turned on.
- *Keep though uncited:* every owner and requirement claim (S161–S170,
  S175, S179, S188–S192, S196, S197); S87–S89 (the vocabulary table
  note 22 points at); S104, S143, S154, S156 (they decide row 1 and row
  4 if reopened); S43, S45, S46, S48, S54, S55 (the trade build); S1,
  S5 (the corpus's size).
- *Unjudged, and they stay:* the other 19 (S7, S8, S11, S15, S21, S24,
  S30–S32, S39, S73, S81, S122, S123, S129, S130, S136, S182, S185) —
  facts about the item and the store that the build will meet whether or
  not a proposal cited them.

## What stage 5 needs from the owner

Each answerable in a line; my recommendation in brackets.

1. Does `"# to maximum Life">=90` mean one line, or the item's lines
   together? [one line; the sum by name — 2a]
2. Is a total a sum that always answers, with its definition and
   contributions shown, or one that declines when it may be incomplete?
   [answers — 2c]
3. Absent is false and counted per term, with a separate could-not-read
   count — or three-valued logic throughout? [the former — 2b]
4. Is a revision comparison before every answer "by construction", or a
   refresh? [construction — row 4]
5. Persist the projection at ingest now, or build unpersisted and move
   when a measured number asks? [unpersisted first — row 4]
6. Does the search crate sit behind the store API, or is it a library
   the frontends link over store reads? [yours; it is a C1 edge and
   moves a store park]
7. Does the MCP take the query as text, departing from R14's letter?
   [yes, the tree also accepted — row 12]
8. With two realms in a store, is a realm-less query an error that lists
   them? [yes — 2e]
9. Are saved queries, user names and tags out of the first slice?
   [yes — row 11]
10. Do the trade site's link filters count colours within one link
    group? (Your knowledge; it decides row 8.)
11. Do the digest's kill-list candidates go? [the listed ones, yes]
12. Will you take the owner's seat before ruling?

**The owner's answer (2026-09-17), to all twelve at once:** press
ahead according to plan; his input enters when the design is close to a
surface he can use, "even if it means cycling back on the design"
(verbatim in note 23, decision 15). So the bracketed recommendations go
to stage 5 as provisional, each with that revisit trigger. Question 10
stays open: it is game knowledge, not design.

## What I left out, and am least sure of

I did not re-audit either design, open the first-round audits, or test
any recommendation against the corpus. The recommendations take more of
B's surface and more of A's semantics; I cannot tell from inside how
much of the first half is my model recognising its own idiom, and the
rows where that matters most are 2, 3 and 12. Rows 2a, 2d, 8 and the
realm rule go against that grain and are the ones I hold most firmly.
The three things proposed here that neither design holds — the
"together reach it" count (2a), the category grouping (row 9), the two
colour fields (row 8) — have been checked by no audit and used by no
reviewer.

## Astra's check — stage 4, step 6

Astra, from Codex, 2026-09-17; checked the reconciliation at
`2ad2986d` under note 23, step 6. Read both repaired proposals and
their repair tables, both second-round audits, both reviews, the
digest, notes 22, 23, 29 and 30, and the cited standing decisions.
Verified the new claims and taxonomy against the committed track
evidence named below. This is a check of the reconciliation, not a new
seat, a third proposal, or an owner ruling. A is my model's proposal;
my earlier review and this check do not create another independent
voice for it.

**Verdict: the recommendations are substantially traceable, but the
table is not ready to be copied unchanged into rulings.** The useful
direction survives: B's compact terms and scoped discovery, explicit
occurrence semantics, and an answer that explains its interpretation.
The joins between those parts leave several questions unanswered, and
the socket and category recommendations claim reach their stated
mechanisms do not preserve. Stage 5 can proceed under decision 15,
carrying the corrections and open choices below. The owner's later
seat can judge usability; it cannot substitute for settling these
meaning differences.

### Row-by-row trace

Section references in the A/B columns are to notes 25/24, respectively.
“Supported” means a reasoned recommendation from the cited material,
not a measured or accepted design. K1–K7 below name the qualifications.

| Row | A | B | Check of evidence and recommendation |
| --- | --- | --- | --- |
| 1 | §4.1 | §§1–2, §4.1 | Supported by S3, S4, S17, S68, S71 and the inherited limits. S17 establishes the observed private flags; it does not make B's overriding kinds identical to A's source-plus-flags. Preserve both coordinates when defining `explicit` and `fractured`, rather than silently choosing one bucket rule. |
| 2 | §§1–2, §4.2 | §§1–2 | One tree and cross-field composition follow from stance 2, S169/S172 and both designs. B's surface is a defensible preference, qualified by K1 and the evidence accounting below; it is not a consequence of those claims. |
| 2a | §§4.1–2, §5 AQ5 | §1, §9.1 | S27–S28 support retaining occurrences, S68 separate slots. Neither prescribes the default. Both reviews support the proposed default; K1 is still needed to preserve the precision for which it was chosen. |
| 2b | §4.2, §4.6 | §1 | Known absence as false plus per-term diagnostics is a motivated new combination. S158 motivates not substituting zero; S107 motivates showing unreadable evidence. Neither decides its negation semantics. K2 prevents the proposed combination from contradicting itself. |
| 2c | §4.2, §4.7 | §§3, 4.1 | S29/S49 and both reviews support useful, inspectable recipe sums. The S111 exception is correctly disclosed. A missing mechanism in A does not prove every possible validation has B's corpus-count limitation; distinguish reference-definition validation from detecting a renamed line in the stash. |
| 2d | §4.2 | §1, §5 gaps | Crossed counts and stack sums follow from S170/S189 and the reviewers' concrete tasks. S191 and AQ1 also require independent facets. Keep both; K3. |
| 2e | §§1–2, §5 scope/AQ1/AQ5 | §4.2 | C58 supports realm-qualified meaning; S199 supports an informative error, not a mandatory single-realm query. A explicitly permits account-wide scope across realms. The new default is reasonable, but label it new and retain an explicit all-realms scope. |
| 3 | §4.3 | §1, §4.3, §6 | Supported: S174/S190 state the shared read need; S181 supplies the cold-start failure. Both reviews contribute concrete reasons to combine scoped counts with insertion-ready fragments. No comparative latency or call-count victory is established. |
| 4 | §3, §4.4 | §3, §9.4 | An unpersisted first measurement follows the engine warning and S135/S150–S153. It remains a provisional implementation choice. A counter alone is not a snapshot contract, and the later move to A is a candidate, not a measured conclusion; K4. |
| 5 | §4.5 | §4.5, §2 | The remainder report, local URL decoding and separate core slots are supported by S56/S67/S68/S155. B's node mapping cannot be accepted as complete arithmetic semantics merely by wrapping it in A's report; K5. S26 correctly exposes the defence translation gap. |
| 6 | §4.6, §3 | §1, §4.6 | Supported by S176/S192–S193/S196–S198 and C53. Reuse C64/C69's effective resolution for `priced`; bind continuation to all semantic inputs, including intent and reference versions, not only a facts counter. K4 supplies the basis. |
| 7 | §§4.7, 8 | §8, §5 gaps | The inherited register and non-goals are traceable, with the S111 exception already noted. Occurrence binding removes accidental summation of twins; it does not identify their hidden local/global meanings (S52/S72). The added unreadability limit needs K2, and unresolved reach must remain a gap. |
| 8 | §3, §5 OQ3, §7 | §3, §5 OQ3 | S16 establishes available group/colour data and S58 records the captured filters. Neither supports “largest” as equivalent to A's existential group. The recommendation loses A's reach; K6. |
| 9 | §3, §5 OQ5 | §3, §5 OQ5 | S42/S54/S69 support a familiar derived category, not a prefix-only hierarchy or a complete equipment selector. The idea is useful and new, but the claimed closure is overstated; K7. |
| 10 | §3, §7 | §§1, 3 | Supported as a simplicity choice. S127 supplies cost motivation, S135 a route to new fields; neither proves named fields already cover the floor. Carry A's acknowledged S42 catalogue gap forward when deleting its escape hatch. |
| 11 | §3, §9 | §§1–2, §7.3, §9.3 | Deferral is supported as a first-slice choice by B's deletion test and the store park. Note 22 makes the home conditional on a saved query existing. Deferring the write does not require deferring versioned query values passed between clients. |
| 12 | §§1–2, §4.2 | §2 | Text plus an accepted/returned tree is compatible with the proposed synthesis, but S195 does not supply evidence for preferring text on MCP. The reviews did not compare complete text and JSON interfaces; retain this as a provisional adapter choice, not a second vote for row 2. |

### Corrections needed when harvesting the rows

**K1 — One-occurrence atoms do not bind two atoms to one occurrence
(rows 1, 2, 2a).** Row 2a fixes AQ5's 20 + 75 surprise. It does not
yet fix note 27's wand: one low bound and one high bound must hold on
the same line. With hypothetical occurrences `[15,40]` and `[5,20]`,
separate existential tests of low ≥ 12 and high ≤ 28 both pass;
neither occurrence satisfies the requested range. A's single
`line(... n1 >= 12 and n2 <= 28)` expresses that distinction; B's two
starred terms do not. The combined model needs a shared occurrence
binder, or an explicitly equivalent compound term, before claiming
A's reach. It also needs a scalar rule for sorting a template that
now denotes several occurrences: A's AQ5 explicitly uses `max`; B's
sort previously inherited its sum. The proposed “together reach it”
counter is useful only for a defined scalar case; it is not a meaning
for arbitrary multi-slot comparisons. These are unresolved semantics,
not reasons to reverse the one-line default.

**K2 — “Could not read” cannot be false under ordinary negation and
also “never a match” (rows 2b, 6, 7).** If an unreadable life array makes
the atom `life >= 90` false, `-(life >= 90)` becomes true. Likewise,
treating an unreadable requirement as absent makes `-has:reqlevel`
admit it in the leveling query. A separate counter does not change
either result. Stage 5 must choose and state how undecidable evidence
affects compound queries: for example, exclude affected candidates
under a defined validity rule, or retain an internal indeterminate
outcome while simplifying its presentation. The former also needs a
rule for `true OR unreadable`; excluding it gives up a known positive
witness. Neither choice is fully specified by the recommendation or
settled by the reviews. Known absence can still have B's simple rule
without equating it to decode failure. Nor are matched/lacked/unreadable
three exhaustive buckets: a present, readable value below a threshold
belongs to none of them. Name the denominator and the ordinary failed
comparison in the answer contract.

**K3 — Add crossed counts without replacing independent facets
(row 2d).** AQ1 explicitly asks for three small tables, and both
proposals provide them. Redefining every multi-name counts request as
one crossed table loses that answer unless another form remains.
Specify the two shapes distinctly. The new summed column also needs
an empty/missing/unreadable rule; note 26's B/F6 already found the
missing sum rule. This is a small contract addition, but calling it a
view does not eliminate that cost. Keep the withheld seed's shared
recipe definitions for SQL and the evaluator; the recipe includes
source/flag and slot selection where relevant, not merely a template
string and weight detached from row 1's identity.

**K4 — Freshness is a snapshot property, independent of persistence
(rows 4, 6).** A §3 explicitly opens a fresh snapshot, matches resident
arrays to that snapshot, and establishes overlapping facts/intent
snapshots. B §3 names a counter and a streaming read but does not bind
those reads together. A counter read followed by a multi-read rebuild
could otherwise label mixed revisions as one. An unpersisted design
can satisfy the floor: derive from one consistent basis and validate
reuse against that basis, including joined location metadata, intent
and reference/derivation versions. A write after that basis is chosen
does not retrospectively invalidate a correctly labelled in-flight
answer; both designs need the same rule. Persistence alone would not
provide it either.

The claim that A “differs only” in writing the persisted copy therefore
overlooks its explicit snapshot contract. Conversely, running a
store-owned extractor inside ingest does not itself give the daemon a
fact-read API: C28/C29 already distinguish the daemon's `record` call
from the store's extraction work, and C34 allows reproducible
derivations. No boundary violation is established by sharing a process.
Keep crate placement as an explicit C1/C12 decision. Measure reload
and cold-query costs before choosing the next mechanism: S152 also
supports direct SQLite queries for short-lived consumers, not only
loading A's projection into a scan. Row 4's two proposed measurements
justify an experiment, not a preselected successor for every failure.

**K5 — Translation needs bounded counts and guarded contributions
(row 5).** S41's pointer, trade-query F2, gives `count` both group min
and max. “Count → at-least-N” covers its lower bound only; B's numeric
`holds` can express both, as can a suitable Boolean tree. S57 expressly
distinguishes `weight` from `weight2` by how per-stat requirements gate
contributions. A bare `sum` of values does not specify those guards.
A §4.5 leaves unverified groups unsupported; preserve that condition
instead of presenting every weighted group as a completed node map.
The row also confines two-slot arithmetic to translation while deleting
general arithmetic from the public surface. If translation emits a
query tree, its averaging/guarded expressions must have a representation
in the published model, or the affected clause stays in the remainder.
The report makes a missing mapping honest; it does not supply one.

**K6 — Any qualifying group is not the largest group (row 8).** A
query for two linked blue sockets should match hypothetical groups
`RRR` and `BB` under A's proposed semantics. A field describing only
the largest group sees `RRR` and misses it. Equal-sized groups require
a choice too; taking each colour's maximum independently can invent
a combination no group contains. Keep group membership and evaluate
the requested constraints together on one qualifying group, even if
the adapter exposes a compact dedicated term rather than general
collection syntax. This counterexample concerns the proposal's own
semantics and needs no assumption about the live trade site. S58's
exact site computation remains unverified, as row 8 correctly says.
S16 closes the absence of group/colour evidence, not universal coverage:
5,922 items have `sockets`, 5,918 have a group, 5,894 colour/attribute
fields; the same census records 24 with `sockets[].type`. Preserve
those different shapes rather than declaring every socket decoded.

**K7 — Dotted category ids are not a complete prefix hierarchy
(row 9).** RePoE F3 explicitly calls the 14 parents *unions*. Its
`class-to-trade-category.csv` assigns Staff to `weapon.basestaff` and
Warstaff to `weapon.warstaff`; `weapon.staff` is a parent in the
associated `base-taxonomy.py`. Neither leaf contains that parent id.
Thus `armour` and `accessory` work as the row's examples without proving
the general prefix recommendation. S69 also says seven leaves need
base-name rules; a single class-to-id value does not cover them.
Use the reviewed membership/base rules, or label the narrower feature
honestly. Finally, OQ5 still needs a definition of which categories
count as equipment; category names make that easier but do not provide
the missing definition. One mapped category filter does not close
most of the other 26 unsettled filters merely because its option list
is large. Keep the remaining reach catalogue open.

### Evidence accounting, seeds and digest

The reconciliation generally obeys note 23's counting rule: shared
requirements are identified as shared text, and the two overall choices
are discounted. There is no demonstrated two-vote majority. Several
phrases nevertheless give evidence more force than it has:

- **A recommendation's destination is not another seat.** Fable's
  reconciliation choosing A is not automatically strong evidence in
  the same sense as its earlier cross-preference *during use*. The
  arrows disclose provenance; the quoted experience or counterexample
  does the evidentiary work. Similarly, two own-design preferences are
  weak, conflicting testimony, not measured quantities that “cancel”.
- **Rows 2 and 12 reuse the same testimony.** Note 27 preferred B at a
  terminal for quick remembered-item searches, and A for precise saved
  gear selection. “Astra leans to it at a terminal” loses that condition.
  Both reviewers wrote A text too. Note 27 expressly used text to expose
  meaning; A did not provide a full JSON contract. This supports making
  text available and testing it, not a demonstrated MCP preference
  between equally specified encodings. Note 26's `where` criticism also
  cannot establish that the design was derived from a table: familiar
  syntax alone is not evidence of its design process.
- **Unprompted does not mean independent of the digest.** S71 and the
  S200/S25 contrast already offered the sign convention; S155 and note
  22's RePoE warning pushed ids toward the trade boundary. The proposals
  independently selected those options, but did not independently
  discover their supporting facts. The withheld-checklist row is more
  careful about this than row 1's “unprompted” claim. The seed audit
  otherwise holds: query values in both; vocabulary partly handed;
  query-by-example in neither; A closer to shared SQL semantics; the
  item's-own-terms thesis visible in both. Returning a canonical query
  is a genuine common design addition, not a separate fact measurement.
- **The concept comparison remains incomplete.** Note 22 asks for one.
  A's inventory has 16 entries; B's 11 omit costs its second audit lists
  separately, so subtracting those totals would be misleading. The
  recommendation removes A's general collection/arithmetic machinery
  and B's user names, but retains occurrence binding, line summation,
  adds two summary shapes, and must resolve group binding and decode
  failure. Those concepts survive regardless of their shorter spelling.
  The combined model needs that inventory and a mistake walk; neither
  existing review validates the combination.

The five new `S` ids have traceable homes: S16/S17 in item-facts F2/F3
and `field-census.csv`, S57/S58 in trade-query F2 and `gap.csv`, S159 in
engine-bench F1/Numbers and `results.csv`. The private flag counts close
A's stated evidence gap for the observed flags. The socket qualification
is K6. These additions do not require another research run.

The citation arithmetic also reproduces **when its denominator is
stated**: before the five additions, 165 claims; in the proposal bodies
excluding repair tables, 92 cited (42 both, 22 B only, 28 A only), 73
uncited. The current digest has 170, with 78 uncited under that same
rule. Repair tables contain superseded citations and should stay out
of this pruning count. The proposed kill candidates are not yet a
pruning: they remain candidates, as this note correctly labels them.
Recheck them against the combined model and stage 6's acceptance tests
before removal, rather than treating noncitation as disproof of need.

No author dispute was found in either current repair table; “no
disputed finding” is accurate. Unanswered findings and newly exposed
gaps remain a different matter. In particular, the absence of a complete
mapping of the trade site's item-reading filters survives the
reconciliation, and a clean audit is not proof that it disappeared.
The three proposals called
unchecked at the end of Fable's section now have this logical check:
the together-count needs a bounded meaning (K1), the colour fields need
group binding (K6), and category needs membership rules (K7). None has
been exercised by a user or against the corpus. Decision 15 permits
progress with those facts visible; it supplies no new game evidence
and no additional endorsement of a specific semantic choice.
