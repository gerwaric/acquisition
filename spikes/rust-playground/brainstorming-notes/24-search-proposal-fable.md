# 24 — Search proposal (Fable)

Provenance: Fable (`claude-fable-5-1`), run under the stage-3 brief,
note 29 at `e5d270bc`. Read: the brief, `search/DIGEST.md`, note 22, the
standing reading, C53 and the two store parks by id, and cpp-search F3
to verify S27. The memory files on item search were not opened.

## 1. The model on one page

**An item is what it displays, where it is, and what you said about
it.** What it displays is a set of **lines**. A line is a *kind*, a
*template* and its *numbers*: `explicit · # to maximum Life · [92]`. The
template is the displayed English text with every number replaced by `#`
— markup reduced to its display half (S4), a sign folded into the number
(S71, S200) — and the query's text is canonicalised by the same function,
so no convention can cause a miss. Properties (`Armour: #`),
requirements (`Level: #`), gem lines and every mod array are all lines;
nothing an item displays is outside them (S6: 579 property names no
field list could hold). The vocabulary of lines is the corpus's own
(stance 2): a line is never unknown to the search, because the search
never needed a name for it that the item did not bring.

A **name** is a bare word for something askable. Three owners, one
namespace: *fields* the model builds in (a closed list of about thirty,
printed in the help: `name base class rarity ilvl armour evasion es
links sockets colors tier stack reqlevel pdps edps dps realm league tab
character id priced price note first-seen …`); *totals* shipped as
reviewed reference data and written in this grammar
(`total-fire-res = sum(…)`), so what counts is readable; and the
*user's* own, which is where game knowledge the API does not carry
enters (stance 4) and where what an agent learned about a stash outlives
its session.

A **term** is a thing and, optionally, a comparison. The thing is a
name, a line, or a quantity built from them. **Absent is never a
number**: a comparison on something the item lacks is false, and the
answer counts how many items lacked it. An item's value for a line is
the **sum of its lines of that template** across the kinds in scope —
implicit 20 + explicit 75 is 95 life, each contribution shown — because
that is what the item gives (S27, S28, S48, S73); naming a kind narrows
it.

**One operation: ask.** A query selects a set of items; the answer is
one shape whatever was asked:

```
query    the text as understood, canonical (and its tree, in JSON)
scope    items searched; realms, leagues; locations never fetched; oldest fetch
terms    per term: what it resolved to, items matching it alone, items lacking it
total    matches
rows | counts | one item        the three views of the same set
next     returned, total, and the argument that continues
```

*Rows* carry short id, name, base, rarity, place by name, and the lines
the query touched with their values — matched-on and the decision view
are the same thing. *Counts* group the set by any name, or by `line`:
the vocabulary read, facets, and autocomplete are **the same call as
search**, scoped by the same query (`class:ring --count line resist` is
"the resist lines my rings have", ranked by how many carry each, with
the value range). Sort, extra columns and count-by all take the same
things a term does. There is no second discovery API to learn.

## 2. The grammar

```
query  := term term …                 side by side is AND
term   := -term | ( query ) | term or term
        | thing                       it is there        has:crucible  is:corrupted
        | thing:text                  contains           name:kaom  class:ring  tab:"dump 3"
        | thing cmp number            = > >= < <=        ilvl>=84  "# to maximum Life">=90
        | thing:a..b                  inclusive, either side blank
thing  := name                        bare word: a field, a total, a user's name
        | "text"                      any displayed text: name, base, every line
        | kind:"text"                 lines of one kind  fractured:"spell suppress"
        | count(q, q, …)              how many of these hold      count(a, b, c)>=2
        | sum(thing, 2*thing, …)      weighted total of values
"text" containing #  names one whole template; without #, it is a
case-insensitive substring, and under a comparison it means "any
template it resolves to" — the answer lists them.
```

`or` binds looser than adjacency. `is:` takes the flags; `-is:x` is
"no", silence is "any" (S173). A line with several numbers satisfies a
comparison when every number does; `*` for a number drops it from the
comparison (`"Adds * to # Cold Damage">=40`); in a `sum` it contributes
its mean (S47, S68). A bare word — `explode` — is what the game's own
stash box does (S113–S115), and fixes S186 as the default. The text is
the rendering; the model is the tree (`Query: FromStr + Display +
Serialize`), bijective with it. A GUI holds the tree as rows; the CLI
and the MCP both pass the string, because a refinement is then the
previous string plus one term (S195), a human and an agent can hand each
other a query, and it is far shorter than its JSON — a judgment, not a
measurement.

## 3. The derivation

Two relations and nothing else: `items` (one row: the fields, the full
place coordinate with names, intent joined read-only, first/last seen,
removed) and `lines` (item, kind, template, verbatim text, numbers).
Both languages read these and only these, so reach is one property
(stance 6); the SQL contract is these two tables with the template
convention and place names in the schema itself (S200), versioned by one
integer.

It is **not persisted**. A process derives it from the store's bodies
on first ask — 202 ms at the real 36,139 (S150) — and holds it; every
ask first compares the store's change counter and re-derives on a
mismatch, and the answer names the revision read. It cannot be stale
because it does not outlive the comparison. A scan answers in 0.2 ms on
average (S144; worst 1.27 ms, engine-bench F1, `undigested`), so the
GUI's keystroke budget is met by the holder it already is, and the MCP
server likewise. The CLI pays the load per command; `sql`
builds the same two tables in memory (+162 ms, S157). Persisting the
projection at ingest (S135's third class; 5.2× cheaper load, S151) is
the same contract moved, and its trigger is a number: a real corpus
whose CLI ask exceeds 500 ms (≈ 90 k items). At a million a one-shot
process is the wrong seat under either (S152); that is a limit this
design adds. Needs from the store: one streaming read of live bodies
with coordinates, and the change counter. Home: a library crate
frontends link, reading only through the store API (C12, C46).

Two reference tables, committed and reviewed like the currency table,
never fetched at runtime (C79): base → class for PoE1 (S53, S69, S70;
PoE2 carries it, S9), and the shipped totals (seeded from the 35
pseudomods, S29). Anything unjoined is `class:unknown`, counted in
scope.

## 4. The seven questions

1. **Identity** is (kind, template); kind is the array plus flags (S3,
   S171). Text identity has no unnameable line. Twins (S44, S72) are one
   line here and are split by `class:`; they matter only at the trade
   boundary. The mod behind a line is not known (S52). When text moves,
   the corpus holds the old wording until refetched and the new after;
   a substring finds both and the terms block shows both; an exact
   template that resolves to nothing says so and *shows* the templates
   sharing its words — never applies them (S107, S111). A shipped total
   lists which of its templates occur in the corpus.
2. **Query model and grammar**: sections 1–2. Place is terms like any
   other; default scope is every realm and league, live items.
3. **Vocabulary** is `--count line [text]` under any query: kind,
   template, items, value range, ranked by count — R4 and R9 are one
   read (S174, S190). Field value sets are `--count rarity` and so on;
   a wrong enum value errors with the legal ones (S199).
4. **Corpus**: section 3.
5. **Trade boundary**: a trade query maps node for node — `and`,
   `not`, `count`, `weight` → `sum`, `if` → `(-has:x or x>=n)`, the 59
   item-reading filters → fields (S41, S42); the 7 market filters are
   dropped and named. A URL needs no network (S56). Stat id ↔ (kind,
   template) is a third reference table; an id it lacks becomes a term
   that matches nothing and says why; outward, several ids for one text
   emit a `count` min 1 (S106), a line with no id (S155) cannot cross
   and is named. The model guarantees the correspondence; building it
   is new scope (c).
6. **A result carries** section 1's row; place is realm, league,
   tab or character by name and id, container, position; freshness is
   the location's fetch age; the body only through `show <id>`
   (any printed id or unique prefix, S198). Intent joined: price and
   note. Legacy is not a field (S178).
7. **Limits and non-goals**: section 8.

## 5. Appendix — the acceptance set

Discovery reads (D) and refinements (R) apart; every line is one query.

- OQ1 · `rarity:rare class:ring count(total-ele-res>=60, strength>=30, dexterity>=30)>=1` · D: `rarity:rare class:ring --count line resist`. The refined case: `class:gloves fractured:"suppress"`.
- OQ2 · `name:"ashes of the stars" "reservation efficiency of skills":10..20` — the variant and its values are the user's (S178); "no" is an empty answer whose terms block shows, say, `name:` matching 3 and the line 0.
- OQ3 · `base:"titan gauntlets" colors:rrg "# to maximum Life">=80 or name:"kaom's roots"` — whatever the interaction names.
- OQ4 · `class:staff has:crucible` · one query under S177: crucible lines are a derivation of origin, offered as one, never the answer.
- OQ5 · `class:equipment (reqlevel<=30 or -has:reqlevel) --count tab` then rows · the `or` is where an ordinary question feels like programming; I kept one absence rule over a special case.
- OQ6 · find: `explode class:"body armour"`; worth: refused (S180). Listed gap, by design.
- OQ7 · `"<words of the announced mod>" --count tab`, then rows; all realms, tabs and characters by default.
- AQ1 · `--count tab,league,rarity` — three tables, no rows, empty query.
- AQ2 · R: `<AQ1's or OQ1's text> total-res>=60`.
- AQ3 · the terms block and the touched lines on every row; an empty answer carries scope and per-term counts, so the term that emptied it is visible; why-not is `show <id> --against '<query>'`, each term marked with the item's own value.
- AQ4 · `show <id>`, or `id:<prefix>` inside any query.
- AQ5 · `"+# to maximum Life">=90 --sort -"+# to maximum Life"` — count and sorted list, every kind summed.

Gaps found writing it: no question across two items (a chest *and* the
gems in it — a socketed gem's place names its parent, no more); no
comparison between two quantities; no cross-tabulated counts; OQ5's
absence clause.

## 6. The cold start

The stranger has the tool description: the grammar block above, the
field list, and "`count: line` lists the lines your items carry."

- OQ1: call 1 `search {query:"rarity:rare class:ring", count:"line", text:"resist"}` — validates `rarity`/`class` spellings (an error lists the legal values) and returns the resist lines and the shipped totals that cover them. Call 2 the OQ1 query with `sort`. **Two calls**; one if the stranger trusts a total named in the description.
- AQ5: the question carries the template. **One call**; the terms block says what the text resolved to (one template, its line and item counts; S187 measured 477 items) or, on a miss, shows the near templates — so one call is also the call that tells it that it was wrong.

## 7. Concept inventory

1. **Item** — the unit of every answer.
2. **Line** (kind, template, numbers) — without it the vocabulary is a field list someone maintains against the game.
3. **Name** — fields need it; totals and the user's names reuse it. *The user-defined part is what I would delete first*: the model stands without it. It stays because it is the only place stance 4's knowledge and an agent's learning can accumulate.
4. **Term, absent-is-false, value-is-the-sum** — the one evaluation rule.
5. **Composition**: and, or, not, `count`, `sum` — the site's reach (stance 2); `count` and `sum` are quantities, so they add no comparison syntax.
6. **Place** — every answer's "where" (S176), and terms like any other.
7. **Answer** — one shape with its terms block; trust in an empty result is this concept.
8. **View** (rows, counts, one item) — what makes discovery, facets and vocabulary not be separate APIs.
9. **The two-table contract** — what SQL and the model both stand on.

## 8. What the design refuses

Fuzzy or guessed matching; regex; ranking by relevance; arithmetic
beyond `sum`; mod names, tiers, roll ranges (S52); map areas (S14);
league of origin as a fact (S177); judging legacy (S178); worth (S180);
acting on the stash; fetching anything (C79); base or
quality-normalised defences (S26, S90 — `armour` is the displayed
property); a saved *result* — a name saves a query, so it cannot be
stale; a persisted index without the number in section 3. Inherited
whole: S12, S14, S52, S53, S67, S107, S111, S177, S178. Added: the
million-item one-shot CLI; S131's league is resolved to the store's
current-truth join, league-less characters shown as such.

## 9. Decisions left to the owner

1. **The sum rule.** I commit to it; whether "life ≥ 90" means the
   item's total or one line is a player's reading, and yours decides.
2. **Access method for the reference tables** (class; later trade ids)
   — Q8 and C79 are yours.
3. **User names fire the `user.db` park** (saved searches are its
   listed trigger). Land it, or ship fields and totals only.
4. **C1**: a search crate is a new edge in the dependency check.

## Left out, and the cut I would reverse

Left out: regex and wildcards beyond `*`; cross-tabs; a trade
importer's build; quantity-to-quantity comparison; PoE2 specifics
beyond "lines and realm already cover it". Least sure: a substring
under a numeric comparison fanning out to many templates — the terms
block makes it visible, not impossible. **The cut I would most want
reversed is tags**: an answer written back as intent (`tag these 14
leveling`), readable as `tag:leveling` by the next ask. It is the
truest form of accretion — the stash gets more understood with every
session, for the human as much as the agent — and I cut it only because
a write does not belong in a read model's first proposal.
