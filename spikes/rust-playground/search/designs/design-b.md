# Design B

## 1. The model on one page

**An item is what it displays, where it is, and what you said about
it.** What it displays is a set of **lines**. A line is a *kind*, a
*template* and its *numbers*: `explicit · # to maximum Life · [92]`. The
template is the displayed English text with every number replaced by `#`
— markup reduced to its display half (S4, S71), a sign folded into the
number, so `+#` and `#` are one template (my choice among the three
conventions the digest's contradictions list under S200 against S25) —
and the query's text is canonicalised by the same function, so no
convention can cause a miss. Properties (`Armour: #`), requirements
(`Level: #`), gem lines and every mod array are all lines — a veiled
placeholder a line with no numbers (S12); "displays"
means those arrays by this design's definition, not by a measurement
(S6 says only that properties defeat a field list), and what else a
body holds (S2, S13) is a field, a flag, or seen through `show`. The
vocabulary of lines is the corpus's own
(stance 2): a line is never unknown to the search, because the search
never needed a name for it that the item did not bring.

A **name** is a bare word for something askable. Three owners, one
namespace: *fields* the model builds in (a closed list of twenty-nine,
printed in the help, each derived in section 3: `name base class rarity
ilvl armour evasion es links sockets colors tier stack reqlevel pdps
edps dps realm league tab character container id priced price note
first-seen last-seen removed`); *totals* shipped as reviewed reference
data and written in this grammar (`total-fire-res = sum(…)`, listed in
section 3), so what counts is readable; and the
*user's* own, which is where game knowledge the API does not carry
enters (stance 4) and where what an agent learned about a stash outlives
its session.

A **term** is a thing and, optionally, a comparison. The thing is a
name, a line, or a quantity built from them. **Absent is never a
number**: a comparison on something the item lacks is false, and the
answer counts how many items lacked it. An item's value for a line is
the **sum of its lines of that template** across the kinds in scope —
implicit 20 + explicit 75 is 95 life, each contribution shown — because
that is what the item gives: a judgment (section 9), in the grain of
the pseudomods, which sum displayed lines on the site and in the C++
app (S29, S49), and of S27–S28, where dropping a repeated template is
the bug; naming a kind narrows it.

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
are the same thing; `--expand` is C53's audit view, each row with every
line and its full place. *Counts* group the set by any name — or
several, one table each, never crossed — or by `line`:
the vocabulary read, facets, and autocomplete are **the same call as
search**, scoped by the same query (`class:ring --count line resist` is
"the resist lines my rings have", ranked by how many carry each, with
the value range). *One item* is the set `id:<x>` shown whole, spelled
`show <x>`; with `--against '<query>'` its terms block carries the
item's own value for each term, which is the why-not. Sort, extra
columns and count-by all take the same things a term does (`--sort
thing`, ascending; `--desc` reverses). There is no second operation and
no second discovery API to learn.

## 2. The grammar

```
query  := term term …                 side by side is AND; none is every item in scope
term   := -term | ( query ) | term or term
        | thing | has:name | has:kind it is there        explode  has:crucible  -has:reqlevel
        | is:flag                     the flag is set    is:corrupted
        | thing:text                  contains           name:kaom  class:ring  tab:"dump 3"
        | thing cmp number            = > >= < <=        ilvl>=84  "# to maximum Life">=90
        | thing:a..b                  inclusive, either side blank
thing  := name                        bare word: a field, a total, a user's name
        | "text"                      any displayed text: name, base, every line
        | kind:"text"                 lines of one kind  fractured:"spell suppress"
        | holds(q, q, …)              how many of these hold      holds(a, b, c)>=2
        | sum(thing, 2*thing, …)      weighted total of values
"text" containing #  names one whole template; without #, it is a
case-insensitive substring, and under a comparison it means "any
template it resolves to" — the answer lists them.
```

`or` binds looser than adjacency. Kinds (`property`, `requirement`,
each mod array, and the `crafted` and `fractured` flags that override
it) and flags (the item's booleans) are closed lists in the help;
`-is:x` is "no", silence is "any" (S173). On a field with a closed
value set (`rarity`, `class`) `:` is a substring of the legal values:
the terms block lists those it resolved to, and text none contains is
an error listing them (S199). A bare word is a name when one
exists and text otherwise; quotes force text, and defining a name that
exists is an error. A line with several numbers satisfies a comparison
when every number does — min and max kept apart (S68), against the
site's average (S47); `*` for a number drops it from the comparison
(`"Adds * to # Cold Damage">=40`); a `sum` needs one number, so there
the line contributes its mean, as on the site (S47) — a judgment, and
against S68's grain. A bare word — `explode` — searching all displayed
text is what I judge a player means by typing one into a stash box; it fixes
S186 as the default. The text is the rendering; the model is the tree
(`Query: FromStr + Display + Serialize`), bijective with it. A GUI holds
the tree as rows; the CLI and the MCP both pass the string. That departs
from R14's "a query object, not a text argument" (S195), on judgment,
not measurement: a refinement is still the previous query plus one
term, a human and an agent can hand each other a query, the string is
far shorter than its JSON, and the tree comes back in every answer.

## 3. The derivation

Two relations carry the corpus: `items` (one row: the fields, the full
place coordinate with names, intent joined read-only, first/last seen,
removed) and `lines` (item, kind, template, verbatim text, numbers).
Both languages read these and only these, so reach is one property
(stance 6); the SQL contract is these two tables with the template
convention and place names in the schema itself (S200), versioned by one
integer.

It is **not persisted**. A process derives it from the store's bodies
on first ask and holds it: 202 ms of parse at the real 36,139 (S150),
plus the read of the bodies out of the store, which is unmeasured —
S183's 2,335 ms for 22,721 items is today's sorted `search("")` (S134),
not the streaming read asked for below. Every ask first compares the
store's change counter and re-derives on a mismatch, and the answer
names the revision read; an ingest landing mid-answer is the next ask's
mismatch. It cannot be stale because it does not outlive the
comparison — given the counter. A scan answers in 0.2 ms on average
(S144; worst 1.27 ms, engine-bench F1, `undigested`), so between store
changes the GUI's keystroke budget is met by the holder it already is,
and the MCP server likewise; a keystroke after a change pays the whole
re-derive, and under an active refresh that is the common case — a gap,
its rate unmeasured. The CLI pays the load per command; `sql` builds the
same two tables in memory (S157's 162 ms built the bench's indexed
18-column schema, S146, to a file; I read it as an upper bound, a
judgment). Persisting the projection at ingest (S135's third class;
5.2× cheaper load, S151) is the same contract moved, and its trigger is
a number: a real corpus whose CLI ask exceeds 500 ms (≈ 90 k items by
the parse alone — arithmetic, not a measurement, and sooner with the
read). At a million a one-shot process is the wrong seat under either
(S152); that is a limit this design adds. Needs from the store: one
streaming read of live bodies with coordinates, and the change counter.
Neither appears in what the digest records of the read surface (S121,
S133, S134) — an inference from silence, no claim enumerating it;
both are requests, and whether the counter is a new read or a schema
cost (S135) is not known here. Home: a library crate frontends link,
reading only through the store API (C12, C46) — where the store area
parked a search crate "behind the store API (C48)"; whether this moves
that park is the owner's (section 9).

Two reference tables, committed and reviewed like the currency table,
never fetched at runtime (C79): base → class for PoE1 (S53, S69, S70;
PoE2 carries it, S9), and the shipped totals: the 35 pseudomods (S29),
each under its own name shortened — `total-res`, `total-ele-res`,
`total-fire-res`, `total-str`, `total-dex` … — and printed with its
definition. The class vocabulary is the export's 103 classes (S69) and
nothing above them. Anything unjoined is `class:unknown`, counted in
scope. The reference tables stand beside the contract, as inputs: class
reaches both languages as an `items` column, and a total is a query
anyone can read, so SQL's reach is unchanged.

The fields, by source. The body (S2): `name base rarity ilvl id
note`. Property and requirement lines under a short name: `armour
evasion es reqlevel`. Computed from the body's sockets: `sockets`, the
count; `links`, the longest run in one group (S22); `colors`, the
socket letters, which `colors:rrg` matches as a multiset — the one
field where `:` is not a substring. That the body carries socket
colours, and a stack size for `stack`, is on no claim (S22 implies only
the sockets' `group`) — an assumption. Computed from property lines:
`pdps edps dps`, APS × the mean of the range, quality not applied;
`edps` takes every range on the elemental line, the damage type tag
ignored (S22). `tier`, the N of a base reading `Map (Tier N)`, which no
property carries (S10, S23). `class`, the table above. Ingest
coordinates (S128): `realm league tab character container`. The store's
stamps (S132): `first-seen last-seen removed`. Intent, read-only:
`priced price`. An item's attributes and resistances are totals, not
fields.

## 4. The seven questions

1. **Identity** is (kind, template); kind is the array plus flags (S3,
   S171). Text identity has no unnameable line. Local/global twins (S72,
   S105) are one line here. Those sources tell them apart by the item's
   class, so a `class:` term separates twins on different classes; the
   design carries no `is_local` and no mod table, no claim says two
   twins cannot sit on one item, and there the sum rule would add them
   — a gap. The mod behind a line is not known (S52). When text moves,
   the corpus holds the old wording until refetched and the new after;
   a substring finds both and the terms block shows both; an exact
   template that resolves to nothing says so and *shows* the templates
   sharing its words — never applies them (S107, S111). A shipped total
   lists each of its templates with its count, zeros included; a
   moved template and one the stash lacks look the same there, so
   S111's loud failure for a relied-on name is not met — a gap.
2. **Query model and grammar**: sections 1–2. Place is terms like any
   other; default scope is every realm and league, live items.
3. **Vocabulary** is `--count line [text]` under any query: kind,
   template, items, value range, ranked by count — R4 and R9 are one
   read (S174, S190). Field value sets are `--count rarity` and so on.
4. **Corpus**: section 3.
5. **Trade boundary**: a trade query's stat groups map node for node —
   `and`, `not`, `count` → `holds`, `weight` → `sum` (my reading of its tip,
   trade-query F2, `undigested`), `if` → `(-has:x or x>=n)` (S41) — and
   the 59 item-reading filters → fields (S42); the 7 market filters are
   dropped and named. The 27 unsettled filters (S42) — `category` and
   its 83 option ids the largest — have no mapping here: named as
   unmapped on the way in, a gap. A URL needs no network (S56). Stat
   id ↔ (kind, template) is a third reference table; an id it lacks
   becomes a term that matches nothing and says why; outward, several
   ids for one text emit a `count` min 1 (S106), and so does a line
   among S67's 32 undecided ids — both candidates, shown as undecided,
   never one guessed; a line with no id (S155) cannot cross and is
   named. The model guarantees the correspondence; building it
   is new scope (c).
6. **A result carries** section 1's row; place is realm, league,
   tab or character by name and id, container, position; freshness is
   the location's fetch age; the body only through `show <id>`
   (any printed id or unique prefix, S198). Intent joined: price.
   Legacy is not a field (S178).
7. **Limits and non-goals**: section 8.

## 5. Appendix — the acceptance set

Discovery reads (D) and refinements (R) apart; every line is one query.

- OQ1 · `rarity:rare class:ring (total-res>=60 or total-str>=30 or total-dex>=30)` · D: `rarity:rare class:ring --count line resist`. The refined case: `class:gloves fractured:"suppress"`.
- OQ2 · `name:"ashes of the stars" "reservation efficiency of skills":10..20` — the variant and its values are the user's (S178); "no" is an empty answer whose terms block shows, say, `name:` matching 3 and the line 0.
- OQ3 · `base:"titan gauntlets" colors:rrg "# to maximum Life">=80 or name:"kaom's roots"` — whatever the interaction names.
- OQ4 · `class:staff has:crucible` · one query under S177: crucible lines are a derivation of origin, offered as one, never the answer.
- OQ5 · `gear (reqlevel<=30 or -has:reqlevel) --count tab` then rows · one query only once the user has named `gear` as the `or` of the classes they mean: the model ships no grouping above class — my choice, not S69's — a listed gap. The `or` is where an ordinary question feels like programming; I kept one absence rule over a special case.
- OQ6 · find: `explode class:"body armour"`; worth: refused (S180). Listed gap, by design.
- OQ7 · `"<words of the announced mod>" --count tab`, then rows; all realms, tabs and characters by default.
- AQ1 · `--count tab,league,rarity` — three tables, no rows, empty query.
- AQ2 · R: `<OQ1's text> total-res>=60`.
- AQ3 · the terms block and the touched lines on every row; an empty answer carries scope and per-term counts, so the term that emptied it is visible; why-not is `show <id> --against '<query>'`, each term marked with the item's own value.
- AQ4 · `show <id>`, or `id:<printed id>` inside any query — a prefix is a substring, so `:` means what it always means.
- AQ5 · `"# to maximum Life">=90 --sort "# to maximum Life" --desc` — count and sorted list, every kind summed; typed as `+#` it canonicalises to the same template.

Gaps found writing it: no question across two items (a chest *and* the
gems in it — a socketed gem's place names its parent, no more); no
comparison between two quantities; no cross-tabulated counts; OQ5's
absence clause. Also: no grouping above class (OQ5);
local/global twins on one item are summed (section 4.1); the 27
unsettled trade filters have no mapping (section 4.5); a keystroke
after a store change pays the whole re-derive (section 3); a shipped
total's moved template does not fail loudly (section 4.1).

## 6. The cold start

The stranger has the tool description: the grammar block above, the
field list, the tool's arguments by name (`query count text sort desc
columns expand next`), and "`count: line` lists the lines your items
carry."

- OQ1: call 1 `search {query:"rarity:rare class:ring", count:"line", text:"resist"}` — validates `rarity`/`class` spellings (an error lists the legal values) and returns the resist lines. Call 2 the same read with `text:"strength"`. Call 3 the OQ1 query with `sort`, its totals written as `sum(…)` over the templates the reads returned, because the description names no total. **Three calls**; two if the description spends the space to list the 35 totals by name.
- AQ5: the question carries the template. **One call**; the terms block says what the text resolved to (one template, its line and item counts; S187 measured 477 items) or, on a miss, shows the near templates — so one call is also the call that tells it that it was wrong.

## 7. Concept inventory

1. **Item** — the unit of every answer.
2. **Line** (kind, template, numbers) — without it the vocabulary is a field list someone maintains against the game.
3. **Name** — fields need it; totals and the user's names reuse it. *The user-defined part is what I would delete first*: the model stands without it, and OQ5 then types its classes out. It stays because it is the only place stance 4's knowledge and an agent's learning can accumulate.
4. **Term** — a thing and an optional comparison; every question is made of these.
5. **Absent is false** — without it a missing requirement reads as 0 (S158); OQ5's `-has:` clause exists for it.
6. **Value is the sum** — what a number means when a template occurs twice; the twelve can be asked without it, which is why section 9 hands it to the owner.
7. **Composition**: and, or, not, `holds`, `sum` — `sum` is AQ2; `holds` is used by none of the twelve and stands on stance 2's "at-least-N-of" and the trade mapping's `count` (S41, S106), not on a question. Both are quantities, so they add no comparison syntax.
8. **Place** — every answer's "where" (S176), and terms like any other.
9. **Answer** — one shape with its terms block; trust in an empty result is this concept.
10. **View** (rows, counts, one item) — what makes discovery, facets and vocabulary not be separate APIs; `show` and `--against` are the one-item view, not a second operation.
11. **The two-table contract** — what SQL and the model both stand on. None of the twelve needs it; the floor's SQL surface is its whole justification.

## 8. What the design refuses

Fuzzy or guessed matching; regex; ranking by relevance; arithmetic
beyond `sum`; mod names, tiers, roll ranges (S52); map areas (S14);
league of origin as a fact (S177); judging legacy (S178); worth (S180);
acting on the stash; fetching anything (C79); base or
quality-normalised defences (S26, S90 — `armour` is the displayed
property); a saved *result* — a name saves a query, so it cannot be
stale; a persisted index without the number in section 3. Inherited
whole: S12, S14, S52, S53, S67, S107, S177, S178; S111 but for the
shipped totals (section 4.1). Added: the
million-item one-shot CLI; S131's league is resolved to the store's
current-truth join, league-less characters shown as such.

## 9. Decisions left to the owner

1. **The sum rule.** I commit to it; whether "life ≥ 90" means the
   item's total or one line is a player's reading, and yours decides.
2. **Access method for the reference tables** (class; later trade ids)
   — Q8 and C79 are yours.
3. **User names fire the `user.db` park** (saved searches are its
   listed trigger). Land it, or ship fields and totals only.
4. **C1 and the store's park**: a search crate is a new edge in the
   dependency check, and a frontend-side one moves the store area's
   search-at-scale park ("behind the store API (C48)") rather than
   firing it; it also asks the store for two reads it does not have
   (section 3).

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
