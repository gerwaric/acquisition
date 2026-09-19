# 10 — Item search: the consolidated forms and one worked example

*Harvested 2026-09-19 into `search/DESIGN.md` (the language reference), which is the authority; this page is history as it stood, and says* filter *where the reference says* query.

*Amended 2026-09-19 with the owner's changes (file 11): the `pseudo.` namespace, computed values in discovery, the `~` pattern match, the projection written `.<slot>`, what `text` searches, `arg1` for positions, underscored names, no `has:` on a computed value, and bare words refused.*

The consensus of files 1–9 between Astra and Fable, for the owner's seat. It is **not harvested**: `decisions/search.md` and `search/DESIGN.md` are unchanged, and §8 lists what adopting this would touch. Flag and key names are the builder's; the shapes are the agreement. Field, total and class names are illustrative — `--describe` supplies the real ones.

## 1. Two values

- **Filter** — the conditions. Text and tree, each printing to the other (C104). Portable: it carries no account, realm or view, so it can be handed between people, agents and accounts.
- **Bound request** — `{account, realm | all, membership}` + filter + view, and a basis when it is a continuation or a drill-down. A receiver whose selected account differs refuses it; rebinding is deliberate and shown.

## 2. The request at a terminal

```
acq search [--account A] --realm pc|xbox|sony|poe2|all ['<filter>']     no filter: every item in scope
   view       rows (default) | --count key,…  (one table each) | --cross key,key  (one table)   [--sum value]
              [--fields name,…] [--sort value [--desc]] [--limit n] [--next token]
   discover   --describe [name,…]             the language: fields, operators, closed value sets,
                                              totals and derived fields with definitions, slots;
                                              structured under --json
              --count line[:text,…]           the vocabulary: templates by kind, ranked, slot ranges,
                                              each with its exact selector; several texts in one call
                                              (`~pattern` narrows too). Computed values whose name or
                                              definition matches are listed beside them, marked computed.
              --context matches | corpus      corpus evaluates the EMPTY filter over the bound scope
                                              and prints that effective filter
              --view locations                the full coverage list the scope block summarises
   explain    --explain <path>                literal substitution of one node (§5)
   exchange   --print-request                 print the bound request as JSON; does not run it
              --request <file|->   --rebind
acq show <id> [--against '<filter>'] [--basis B]
```

## 3. The filter

```
COMPOSITION
  t t                 and — at the level you are in; whitespace never binds across levels
  and  or  not  -t  ( )
  a b or c            and beside or, unparenthesised: an error showing both readings
  holds(q, q, q)>=2   holds(q, q, q)=2..3
  undecided(x)        x a thing or a term: true when its value / its truth cannot be
                      established. Always decided itself; known absence does not satisfy it.

ITEM-LEVEL
  "a phrase"          text: substring, any case, tested against each displayed string on its
                      own, never across two: the name, the base, and every line as shown with
                      its numbers — properties, requirements, every mod line; markup reduced
                      to what the player sees. Numbers are characters — "Level 1" also finds
                      "Level 10" — where template: sees only the #-form and never a number.
                      Not searched: flavour text, description text, the note (its own field).
                      A socketed gem is its own item, found by its own text, its place showing
                      what it sits in; the parent's text does not include it.
                      The row shows which string matched. A hit in what is readable is a match;
                      no hit while part of the item is unread is undecided.
  word                a bare word is an authoring error that shows its readings, never a guess:
                        rare → rarity=rare · "rare"      ring → class:ring · "ring"
                        life → "life" · line(template:life)
                      Quotes are the deliberate signal for text. An adapter may offer a plain
                      text box that wraps its input in text: — the filter itself never does.
  name:kaom   name="Kaom's Heart"   name~"^Kaom"
                                       `:` contains · `=` whole value · `~` pattern, on every text
                                       thing. Rust regex syntax, unanchored, any case unless the
                                       pattern says otherwise; a bad pattern is an authoring error.
                                       Like `:`, a pattern is a continuing search: the answer lists
                                       what it resolved to and never freezes it.
  text:word   text~"pattern"           the explicit name of what a quoted phrase searches;
                                       in a pattern ^ and $ are the ends of one displayed string
  rarity=rare   class:ring             closed set: `=` exact; `:` picks among legal values, listed
  ilvl>=84   ilvl=80..84               = > >= < <= ; a..b inclusive, a side may be blank
  has:x  -has:x                        presence; absence only when all that could hold x was readable
  is:corrupted  -is:corrupted          yes / no; silence is any
  league:  tab:  character:  container:      place, as text fields
  id:<handle>                          any id an answer printed
  unknown field, operator or closed value → an authoring error, near names offered.
  A valid selector nothing carries is NOT an error: it matches nothing known, and is
  undecided where eligible evidence is unread. Validity never depends on the corpus.

MEMBERS — conditions that hold together on one member
  line( … )      one displayed occurrence      linked( … )     one link group
  line:    "T" (leading; means template="T")   template:words   template~"pattern"
           source=explicit   is:fractured   -is:crafted
           arg1 arg2 …   and on a ranged line  low  high  avg        with = > >= < <= a..b
  linked:  red green blue white (counts)   size
  inside: and / or / - / ( ); a bare word is an error here as everywhere
  a template typed with its numbers — line("+92 to maximum Life"), "+92 to maximum Life">=90 —
  is an error showing two readings: any value, or that value.

VALUES
  line(P).<slot>                 a projected value: what a comparison, a sum or a sort consumes.
                                 <slot> is one of the reserved slot words — arg1 arg2 arg3 … by position
                                 (arg2 is the second # of the template, counting from 1);
                                 low high avg on a ranged line — so: line(P).high  line(P).avg  line(P).arg1
                                 A comparison on a projection means what it means inside the group:
                                 line(P).avg>=20 lowers to line(P avg>=20). One meaning.
  "T">=90    "T".avg>=20         shorthand; lowers at once to line("T" arg1>=90), line("T" avg>=20)
  "T" low>=15 high<=45           error: offers line("T" low>=15 high<=45) and "T".low>=15 "T".high<=45
  sum("T")   sum(line(P).<slot>) the item's sum
  pseudo.<name>                  a computed value: anything --describe prints a definition for.
                                 One namespace for named totals (reviewed rows, one meaning on every
                                 surface) and derived fields:  pseudo.total_res   pseudo.dps
                                 pseudo.pdps   pseudo.defence_pct.   It means "computed here, from
                                 this definition" — not "identical to the trade site's pseudo of a
                                 similar name"; the translation report says where they correspond.
  pseudo.cold_damage.avg>=30     a ranged computed value takes a slot word last
                                 Names are words of letters, digits and underscores; a hyphen is
                                 never part of a name, so `-` only ever means "not" or a minus sign.
                                 has: does not apply to a computed value — an error offering
                                 pseudo.total_res>0 ("has some") and undecided(pseudo.total_res).
  sockets  links  sockets.red    counts over the socket collection
  has:priced   price.amount  price.currency  price.lot
  --sort takes a value; a ranged value needs a slot; no satisfying occurrence sorts last either way.
```

**A sum's status.** One rule for named totals and item sums:

| Situation | Value | A comparison | `undecided(…)` |
| --- | --- | --- | --- |
| contributors complete, including none | the number; zero when none | ordinary — matched or failed | false |
| a required contribution cannot be established | incomplete subtotal | undecided | true |
| the total has no definition for this realm | unavailable, with the reason | undecided | true |

Absent is still false and never zero for a *line or property* the item lacks; a *sum* of nothing is an honest zero.

## 4. One worked example

Seven pc items (Astra's scenario, file 3): rare rings r1 (life 95, res 65), r2 (20 implicit + 75 explicit, 60), r3 (100, 55), r6 (readable 95, implicit array unread, res subtotal 65 incomplete), r7 (no life line, 65); r4 a *magic* ring (95, 70); r5 rare, life 95, res 65, its base missing from the class table. Two fetched locations, one never fetched. Every count below is worked by hand from those facts.

**The bound request** (shape, not the Rust layout; a request sends `text` *or* `tree`, an answer returns both):

```json
{ "scope":  { "account": "A", "realm": "pc", "membership": "live" },
  "filter": { "tree": { "all": [
      { "field": "league", "op": "=", "value": "Standard" },
      { "field": "class",  "op": "=", "value": "ring" },
      { "field": "rarity", "op": "=", "value": "rare" },
      { "exists": "lines", "where": { "all": [
          { "attr": "template", "op": "=",  "value": "# to maximum Life" },
          { "attr": "arg1",       "op": ">=", "value": 90 } ] } },
      { "value": { "pseudo": "total_res" }, "op": ">=", "number": 60 } ] } },
  "view":   { "rows": { "limit": 20 } } }
```

**The answer at a terminal:**

```
filter  league=Standard class=ring rarity=rare line("# to maximum Life" arg1>=90) pseudo.total_res>=60
scope   account A · pc · live · 7 items · 2 locations fetched (oldest 3d, newest 2h)
        1 never fetched · location list seen 2h ago                       more: --view locations
basis   snapshot 41 · intent 12 · totals v1 · classes v3
terms   each term evaluated independently over live pc items in all leagues
  0    league=Standard                        7 matched
  1    class=ring                             6 matched · 1 undecided
  2    rarity=rare                            6 matched · 1 failed
  3    line("# to maximum Life" arg1>=90)       5 matched · 1 failed · 1 lacked · 1 reaches 90 only together
  4    pseudo.total_res>=60                   5 matched · 1 failed · 1 undecided
total   1 match · 2 undecided
rows    r1  Two-Stone Ring · rare · Standard / Rings
            +95 to maximum Life (explicit) · pseudo.total_res 65
next    1 of 1
routes  (--routes prints all; one shown)
  term 3 failed, over the scope:
    acq search --account A --realm pc 'line("# to maximum Life") -line("# to maximum Life" arg1>=90)'
```

The two undecided items say why and what might help: r5 — *class: base not in the class table; a refresh will not help, a reference update may*; r6 — *pseudo.total_res: implicit lines unread; a refresh may help*. These are hints, not guarantees.

**The same route under `--json`** — directly resubmittable, carrying its denominator and the basis it was counted at; run on a later basis it reports both counts:

```json
{ "count": 1, "counted_at": { "snapshot": 41 }, "denominator": "scope",
  "request": { "scope":  { "account": "A", "realm": "pc", "membership": "live" },
               "filter": { "text": "line(\"# to maximum Life\") -line(\"# to maximum Life\" arg1>=90)" },
               "view":   { "rows": {} } } }
```

It returns r2. Appending it to the original filter instead would return nothing — which is why a route is a request and not a fragment. A bucket of a count view routes *under the filter*: counting every item by class gives `ring 6 · undecided 1`, and the undecided bucket's request has the filter `undecided(class)` and returns r5, which no root-undecided list would ever contain. A crossed cell's route carries both keys. Paths (`0`…`4`, `3.1`) belong to this exact returned filter; after an edit, use the next answer's.

**When the total is zero**, in place of rows: selectors that resolved to nothing, with suggestions (tolerant suggestion, exact execution; a suggestion may be a whole, deliberately broader request); the root-undecided route; the coverage summary; and the *offer* of `--explain` and `--context corpus` — not their output.

## 5. Explain

`--explain <path>`: the root's definite and undecided counts as given, with the node forced true, and with it forced false, each beside its transformed filter. Labelled literally — never "removed", never "contribution" (`A or B` with an item satisfying both loses nothing when A is forced false). Inside a member group the substitution is per member: it never creates a member, and an unread collection is not a population. Opt-in, one node at a time.

## 6. Invariants

1. Canonicalisation lowers shorthand and normalises spelling; it never reorders, flattens, merges, deduplicates or simplifies.
2. A `:` selector is never replaced by what it resolved to; the answer shows the binding beside the authored selector, and a vocabulary row's selector is always the exact form.
3. A selector's validity never depends on the corpus.
4. A count shown has a route to its members, and the route keeps its denominator.
5. Every block of an answer is bounded: summaries and a route to the whole, never an enumeration by default.
6. Search is a pure read, and no item predicate supports a claim about an unfetched location.

## 7. Outside the first surface (directions agreed, nothing specified)

| Thing | State |
| --- | --- |
| a refresh plan from a selection of uncovered locations | **an integration gap**: today `--plan` compiles stored policy and excludes `--tabs`; C76's selected-location plan is parked in `decisions/plans.md`. Search reports coverage honestly meanwhile. |
| alternatives for a node | deferred. If built: evaluate the filter with the leaf replaced, by the normal evaluator; any optimisation must reproduce that. |
| a slot's distribution under the rest of the filter | deferred; an edited bound and another search serve. |
| user-authored weighted sums | direction agreed — an unnamed total; operands are anything with a value, a definition and a status. Range weights, negative weights on ranges and guarded trade groups need their own definition first. |
| continuing a retained basis | a resident client's improvement. Default everywhere: a changed basis refuses the continuation and names both; an expired or never-held basis is its own error; `show --against` obeys the same. |
| cross-currency price comparison | needs an explicit valuation input; the currency table holds no rates. |
| `in pc: …` as a scope-bearing text | untested; try copying the full invocation and the JSON request first. |
| substring over a displayed occurrence inside `line(…)` | unavailable; item-level phrase search reaches displayed lines. |

## 8. For the owner: what adopting this would touch

Nothing here is ruled. If harvested, these are the rulings and contract paragraphs it amends or completes:

- **C92** — the owner's spaced slot syntax (`"T" avg>15 high>=20`) gives way to the group `line("T" avg>15 high>=20)` and the one-bound `"T".avg>15`; the words `low`, `high`, `avg` and positional slots stay. Reason: whitespace should never bind conditions to an occurrence.
- **C96** and its contract paragraph — the `realm:pc` front shorthand is removed; scope, with the account, lives in the bound request.
- **C91, C104** — the travelling value splits into filter and bound request; a bare word is an authoring error showing its readings and a standalone quotation is a phrase (owner, file 11); two new ambiguity errors (and beside or; a template typed with numbers).
- **C90, C97** — source and flags are attributes inside `line(…)`, not a prefix; discovery has three contexts (language, corpus, matches), text-narrowed, structured, bounded.
- **C93** — the fourth undecided reason ("a name the search cannot bind") is retired in favour of three cases: unknown name is an authoring error; a valid selector nothing carries is a valid selection; only what cannot be established on an item is undecided. `undecided(…)` is new. Reasons carry a hint of what might resolve them.
- **C94, C95** and their contract paragraph — a complete empty sum is a present zero and lands in matched or failed, never lacked.
- **C94, C101** (owner, file 11) — named totals and derived fields share one namespace, `pseudo.<name>`: whatever `--describe` prints a definition for. It need not match the trade site's pseudo of a similar name.
- **C91** (owner, file 11) — a third text-match strength, `~` pattern, beside `:` and `=`; `text` is the explicit name of what a quoted phrase searches.
- **C97** (owner, file 11) — a text-narrowed vocabulary read also lists the computed values whose name or definition matches.
- **C100** — counts carry routes with denominators; the zero-match block; the coverage summary; node paths.
- **C105** — its open question closes: `none` routes by `-has:<key>`, `undecided` by `undecided(<key>)`, both as requests under the filter.
- **C101** — `linked(…)`, `sockets.red`; one internal member node for lines and link groups, no general collection syntax in the text.
- Note 35's N1–N12 and G1–G3 are superseded by this page; its verdict lines need no answers.
