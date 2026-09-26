# Item search — the design: the language reference and the binding contract detail

**Standing: binding.** This is the contract detail of the item-search
rulings (`decisions/search.md`): the rules that decide an answer and do
not fit a registry entry — the language a query is written in, the sort
scalar, how undecided composes, what an incomplete sum compares as, the
diagnostic counts. Each
paragraph is headed by the decision it completes and binds with that
decision's standing (provisional under note 23's decision 15 unless the
owner ruled it in his own words). A paragraph lives here **until code
takes it**: when a module doc in `acquisition-search` carries it under
its id, the paragraph is deleted in that commit, so this file empties as
the crate is built and is never a second registry. The owner chose this
home 2026-09-18 (the stage-5 audit, finding 6: "I agree with (a)"); it
is the one document in `search/` that is an authority.

What is here was moved verbatim from note 28 §2b, where it was recorded
at the stage-5 harvest (2026-09-17) with Astra's completions accepted as
one voice, marked where they apply; the owner's answers verbatim and
Astra's check stay in `brainstorming-notes/28-search-ruling-packet.md`
(§4 and its last section). The language reference above the detail is
the stage-5 audit's finding 5, harvested 2026-09-19 from session C: a
design dialogue between Astra and Fable with two paper walks, then the
owner's amendments (`search/search-forms/`; files 10 and 11 are what was
harvested, and the owner's words are whole in file 11).

## The language reference

One page, every construct with an example, so that the owner's seat, an
agent and the parser's round-trip test read the same thing and nobody
invents notation. Shapes bind; flag and key names are the builder's;
field, class and computed-value names are illustrative, and `--describe`
supplies the real ones. A section leaves as the detail below leaves:
deleted when the parser's module doc, the clap help or `--describe`
carries it.

Three words are fixed here. The **query** is the conditions and nothing
else: C91's tree and its text. The word is the trade site's own — its
request is `{query, sort}` posted to a realm and a league, the conditions
inside `query` and everything else outside it — and *filter* stays the
word for GGG's item filters and for the site's single controls. A
**request** is a query bound to a scope and a view. **`pseudo`** is this
search's namespace for computed values; the trade site's 298 are called
the site's pseudo stats, and neither promises the other (owner,
2026-09-19: "This doesn't need to 100% align with trade").

### Two values

- **The query** — the conditions. Text and tree, each printing to the
  other (C104). Portable: it carries no account, realm or view, so it is
  handed between people, agents and accounts (C91).
- **The request** — `{account, realm | all, membership}`, the query, a
  view, and a basis when it continues or drills down (C98, C100). A
  receiver whose selected account differs refuses it; rebinding is
  deliberate and shown. A request always carries its realm resolved: at
  a terminal an omitted `--realm` over a store holding one realm
  resolves to it and the answer prints it; over a store holding more
  than one it is C96's error.

### The request at a terminal

```
acq search [--account A] [--realm pc|xbox|sony|poe2|all] ['<query>']    no query: every item in scope
   [--membership live|all]    live by default; all adds removed items, each row marked with when it left
   a query that starts with -, the language's not, goes after --, as every route prints it:
       acq search --realm pc -- '-is:corrupted'
   view       rows (default) | --count key,…  (one table each) | --cross key,key  (one table)   [--sum value]
              [--fields name,…] [--sort value [--desc]] [--limit n] [--next token]
   discover   --describe [name,…]             the language: fields, what a line takes, what has a
                                              value, how terms compose, operators, closed value
                                              sets, computed values with their definitions, slots,
                                              what a count takes; a name asks for one entry, or a
                                              block by its name;
                                              structured under --json. This is the help C97 names.
              --count line[:text,…]           the vocabulary: templates by kind, ranked, slot ranges,
                                              each with its exact selector; several texts in one call
                                              (`~pattern` narrows too). Computed values whose name or
                                              definition a narrowing matches are listed beside them,
                                              marked computed; `line` alone lists none (T5).
              --context matches | corpus      corpus evaluates the EMPTY query over the bound scope
                                              and prints that effective query
              --view locations                the full coverage list the scope block summarises
   explain    --explain <path>                literal substitution of one node (below: Explain)
   exchange   --print-request                 print the request as JSON; does not run it
              --request <file|->   --rebind
acq show <id> [--against '<query>'] [--basis B]      the item as the search derives it — its lines and values,
                                                     its place, what was unread; --json the same, structured;
                                                     the stored body on request, and still when deriving fails
```

```
acq search --realm pc 'class:ring' --count rarity,tab          two tables: rings by rarity, rings by tab
acq search --realm pc 'class:ring' --cross rarity,tab          one table: rarity against tab
acq search --realm pc 'has:priced' --count price.currency --sum price.amount
acq search --realm pc 'class:bow' --sort 'line("Adds # to # Cold Damage").avg' --desc --limit 10
acq search --realm pc 'class:ring' --count line:resist,life    the vocabulary rings carry, narrowed twice
```

Computed values in the vocabulary read — owner, 2026-09-19: "agreed and
we will see what this does to implementation, but it's always easier to
cut stuff out than add it in".

Count views (C95; E4 and E5 accepted 2026-09-22): choose `--count` or
`--cross`, never both. `--cross` takes exactly two fields; `line` is
counted in its own vocabulary table. `--sort` orders item rows and is an
error with either count view; tables are ranked by their counts.
`--sum` requires one of those count views: no standalone grand total.
It adds one number per item: a numeric field or an explicit item
`sum( … )`. A raw line projection is an error offering the item sum,
never an implicit choice of its largest occurrence (C95; E2 accepted
2026-09-22). For example, `--sum 'sum("# to maximum Life")'` adds all
matching life lines on each item before adding the items' totals.
A computed value (`pseudo.<name>`; C94, C101) defines its own
contributors and is summed as a numeric field is; a ranged one needs a
slot word, and none is shipped yet (`pseudo.rs`).
`--limit` bounds rows or each table's values (a crossed table's cells),
while `none` and `undecided` remain visible. The bucket selectors and
the vocabulary's presence-based `undecided` (C105; E1 and E3 accepted
the same day) are defined in `acquisition-search/src/counts.rs`,
"As built"; zero undecided does not assert that every line was readable.

Membership (gap 3 accepted 2026-09-23) is a scope value beside realm,
never a term: `live` by default, `all` on request, and every removed row
under `all` says when it left; the scope block states membership and the
count of each. An `id:` term that matches nothing live says whether the
id is among the removed, with its removal time, and prints the `all`
route, so a handle held from an earlier answer never reads as an item
that did not exist; `acq show` shows a removed item, marked. `removed`
alone waits for a question. Until `all` runs, the route names the
unbuilt construct (the plan, rule 5).

### The query

```
COMPOSITION
  t t                 and — at the level you are in; whitespace never binds across levels
  and  or  not  -t  ( )
  a b or c            and beside or, unparenthesised: an error showing both readings
  holds(q, q, q)>=2   holds(q, q, q)=2..3        holds(rarity=rare, is:corrupted)=1
  undecided(x)        x a thing or a term: true when its value / its truth cannot be
                      established. Always decided itself; known absence does not satisfy it.
                        undecided(class)     undecided(pseudo.total_res>=60)
  true()  false()     always true, always false: what --explain prints for a forced node.
                      Legal wherever a term is, inside a member group too.

STRINGS
  "…"                 inside quotes  \"  \\  \n  are the only escapes. \n is the row break of a mod
                      displayed over several rows; such a mod is one occurrence with one template:
                        line("Monsters' Action Speed cannot be modified to below Base Value\nMonsters' Movement Speed cannot be modified to below Base Value")
                      A phrase tests each displayed row on its own; template:words reaches across
                      rows; a total's row may name such a template like any other.
  "+# …"              in a quoted template a + before a # is spelling, as the game and the trade
                      site write a line, and is dropped: the sign is the number's (C90).
                        "+# to maximum Life">=90  →  line("# to maximum Life" arg1>=90)
                      A - there is not spelling — it says the value is negative — so it is an error
                      offering the comparison: line("#% to Fire Resistance" arg1<0). A dash after a
                      # or a ) is a range dash, (#-#), never a sign. #% needs nothing: the % is the
                      template's.

ITEM-LEVEL
  "a phrase"          text: substring, any case, tested against each displayed string on its
                      own, never across two: the name, the type line, the base, and every line as shown with
                      its numbers — properties, requirements, every mod line; markup reduced
                      to what the player sees. Numbers are characters — "Level 1" also finds
                      "Level 10" — where template: sees only the #-form and never a number.
                      The item level is displayed as the game and the trade site show it,
                      "Item Level: 84", where the item has one (ilvl, below).
                      The requirements are one displayed string, the row the game shows:
                      "Requires Level 67, 159 Str"; each requirement is kept on its own beneath it.
                      Not searched: flavour text, description text, the note (its own field).
                      A socketed gem is its own item, found by its own text, its place showing
                      what it sits in; the parent's text does not include it.
                      The row shows which string matched. A hit in what is readable is a match;
                      no hit while part of the item is unread is undecided.
  "T"                 a quoted string with a # in it is a template, never a phrase: alone it means
                      line("T") — the item carries the line, whatever its numbers.
                        "# to maximum Life"      -"+#% to Chaos Resistance"   (the item has no such line)
                      text:"…#…" searches for a literal #.
  word                a bare word is an authoring error that shows its readings, never a guess:
                        rare → rarity=rare · "rare" · line(template:rare)
                        corrupted → is:corrupted · "corrupted" · line(template:corrupted)
                        life → "life" · line(template:life)          ring → class:ring · "ring" · line(template:ring)
                      Quotes are the deliberate signal for text. An adapter may offer a plain
                      text box that wraps its input in text: — the query itself never does.
  name  typeline  base                 the header: three text things, each what GGG gives (name, typeLine,
                                       baseType) under the normalisation every displayed string gets. An
                                       item with no name — a magic or normal item, an unidentified rare —
                                       lacks it: known absence, so name: is false and -has:name finds it;
                                       an unread body is undecided. typeline carries what the base does
                                       not, a magic item's affix names among it:
                                         typeline:staunching     base="Divine Life Flask"
                                       A renderer may print name and type line together; that never
                                       changes what the fields mean, and a phrase never matches across them.
  name:kaom   name="Kaom's Heart"   name~"^Kaom"
                                       `:` contains · `=` whole value · `~` pattern, on every text
                                       thing, each in any case: name="kaom's heart" finds Kaom's
                                       Heart. Rust regex syntax, unanchored; a pattern may turn case
                                       back on for itself, (?-i). A bad pattern is an authoring error.
                                       Like `:`, a pattern is a continuing search: the answer lists
                                       what it resolved to and never freezes it.
  text:word   text~"pattern"           the explicit name of what a quoted phrase searches;
                                       in a pattern ^ and $ are the ends of one displayed string
  rarity=rare   class:ring             closed set: `=` exact; `:` picks among legal values, listed
  rarity=unique   frame=gem            two fields, each what GGG gives. rarity is normal, magic, rare or
                                       unique; a gem, a currency stack, a card lacks it: known absence.
                                       frame (GGG's frameTypeId) is on every item and adds gem, currency,
                                       divinationcard, quest, supporterfoil.
  ilvl>=84   ilvl=80..84   ilvl=..84   = > >= < <= ; a..b inclusive, a side may be blank. GGG's ilvl of 0
                                       is an item with no item level — a gem, a card: known absence, so
                                       ilvl<=10 is false on it and -has:ilvl finds it.
  has:x  -has:x                        presence; absence only when all that could hold x was readable
  is:corrupted  -is:corrupted          yes / no; silence is any. The words are a closed list, GGG's
                                     own spellings in any case (is:abyssJewel), printed by
                                     --describe; so are a line's source= words and its is: flags.
                                     A word outside its list is an authoring error with the near
                                     ones offered. A flag GGG adds is shown by acq show and
                                     cannot be asked for until the list gains it.
  league:  tab:  character:  container:      place, as text fields. tab: is the tab's name, never its
                                     type: a map tab named Maps is found by tab:maps because of
                                     what it is called. An item in a substash — a map tab's or a
                                     unique tab's, whose own names are "1" or empty — is tested
                                     against the substash's name and its tab's. A folder's name
                                     is no part of it. A stash item has no character and a
                                     character's no tab: known absence.
  id:<handle>                          any id an answer printed, whole: the item's own, or its
                                     tab's, substash's or character's, which finds what is in
                                     it. `:` and `=` mean the same. acq show takes an item's id.
  unknown field, operator or closed value → an authoring error, near names offered; mod( and
  stat( are ordinary unknown names. realm: is an error that names the scope words (C96).
  A valid selector nothing carries is NOT an error: it matches nothing known, and is
  undecided where eligible evidence is unread. Validity never depends on the corpus.

MEMBERS — conditions that hold together on one member
  line( … )      one displayed occurrence      linked( … )     one link group
  line:    "T" (leading; means template="T")   template:words   template~"pattern"
           source=explicit   is:fractured   -is:crafted
           source=hybrid     a vaal gem's base skill, displayed on the gem: a source of its own
           arg1 arg2 …   and on a ranged line  low  high  avg        with = > >= < <= a..b
             line("# to maximum Life" source=explicit -is:crafted arg1>=90)
             line(template:resistance is:fractured)
             line("Adds # to # Cold Damage" low>=15 high<=45)
           A quoted template is compared in any case too. GGG has spelled some lines
           two ways — Gain # Life per Enemy Killed · Gain # Life per enemy killed —
           and one spelling typed finds both: the term lists the spellings it found,
           and template~"(?-i)^Gain # Life per enemy killed$" selects one.
  linked:  red green blue white (counts)   size
             linked(red>=3 green>=1)      linked(size>=5 blue>=2)
  inside: and / or / - / ( ); a bare word is an error here as everywhere
  a template typed with its numbers — line("+92 to maximum Life"), "+92 to maximum Life">=90 —
  is an error showing two readings: any value, or that value.

SLOTS — the words that name a number
  arg1 arg2 arg3 …    every number of a template by position, counting from 1 (arg2 is the
                      second # of the template), a ranged pair's two numbers included
  low  high  avg      on a ranged line: a template with exactly one `# to #`. low and high are
                      that pair, avg their mean.
                        "Adds # to # Cold Damage"                  low=arg1  high=arg2
                        "# to # Added Physical Damage per # Armour or Evasion Rating on Shield"
                                                                   low=arg1  high=arg2  arg3 the per-N
  two `# to #`        not ranged: low, high or avg on it is an error that lists arg1 … arg4
  (#-#)               "Bow: Adds (#-#) to (#-#) Cold Damage" describes possible rolls and never
                      reads `# to #`: positional only — a line, and searchable, like any other
  a slot is checked against a quoted template wherever the template sits among the group's
  conjuncts: line(("T" source=explicit) arg3>=0) is the error line("T" source=explicit arg3>=0)
  is. A template under an or or a not states no numbers, and its slots are the evaluator's.
  a number            the search reads a number of up to ten whole digits and four decimals — more
                      than any game displays. One written longer is an unread slot of its line:
                      what asks that slot is undecided, through a not as well, and the line's text,
                      template, source and other numbers answer as read (C93). Arithmetic on what
                      is read — a sum, avg — is exact in decimals and the same in any order.

VALUES
  line(P).<slot>                 a projected value: what a comparison, a sum or a sort consumes.
                                 <slot> is one of the slot words: line(P).high  line(P).avg  line(P).arg1
                                 A comparison on a projection means what it means inside the group:
                                 line(P).avg>=20 lowers to line(P avg>=20). One meaning.
  "T">=90    "T".avg>=20         shorthand; lowers at once to line("T" arg1>=90), line("T" avg>=20)
                                 With no slot named, the template's own #s decide before it lowers:
                                 one # is arg1; several is the error that lists them; none is an
                                 error. sum("T") and --sort obey the same.
  "T" low>=15 high<=45           error: offers line("T" low>=15 high<=45) and "T".low>=15 "T".high<=45
  sum("T")   sum(line(P).<slot>) the item's sum
  pseudo.<name>                  a computed value: anything --describe prints a definition for.
                                 One namespace for named totals (C94: reviewed rows, one meaning on
                                 every surface) and derived fields (C101):  pseudo.total_res
                                 pseudo.dps   pseudo.pdps   pseudo.defence_pct.   It means computed
                                 here, from this definition; the translation report (C99) says
                                 where it corresponds to one of the site's pseudo stats.
  pseudo.cold_damage.avg>=30     a ranged computed value takes a slot word last
                                 Names are words of letters, digits and underscores; a hyphen is
                                 never part of a name, so `-` only ever means not, or a minus sign.
                                 has: applies to a derived field (-has:pseudo.dps: a ring has no
                                 dps), never to a total — an error offering pseudo.total_res>0
                                 and undecided(pseudo.total_res) (T2).
  sockets  links  sockets.red    counts over the socket collection:  sockets>=5  links=6  sockets.red>=2
                                 the colours are the four words; an abyssal socket (GGG's A) or a
                                 resonator's (DV) counts in sockets, under no colour, and is asked for
                                 by its line or its base (K1)
  has:priced   price.amount  price.currency  price.lot
                                 the effective price (C81), joined read-only: has:priced holds where it
                                 carries a price — a listing resolved to skip or no price lacks it (P1,
                                 ruled 2026-09-25); price.lot is a ratio's lot, which a decimal price
                                 lacks — 5 and 5/1 stay two spellings, as the game keeps them (P2, the same)
  --sort takes a number: ilvl, stack, line(P).<slot>, sum( … ); a ranged value needs a slot. An
  item with no satisfying occurrence sorts last either way, and so does one whose largest is not
  established — a source P admits is unread, or an occurrence P may select holds a larger number:
  what was readable is shown, marked incomplete, as an incomplete sum is.
```

**A sum's status.** One rule for computed totals and item sums:

| Situation | Value | A comparison | `undecided(…)` |
| --- | --- | --- | --- |
| contributors complete, including none | the number; zero when none | ordinary — matched or failed | false |
| a required contribution cannot be established | incomplete subtotal | undecided | true |
| the total has no definition for this realm | unavailable, with the reason | undecided | true |

Absent is still false and never zero for a *line or property* the item
lacks (C93); a *sum* of nothing is an honest zero.

**The owner's words on the query** (2026-09-19, verbatim but for
spelling; whole, with the
reasoning, in `search/search-forms/11-owner-amendments.md`):

- `line(…)` keeps its name over `stat(…)` and `mod(…)` — the word says
  what the search sees, what the item displays: "I agree with line()".
- One namespace for every computed value, never a kind of line: "i agree
  with 'pseudo' as the namespace now, which means we can fit stuff like
  the various kinds of dps and other calculated items." A line the item
  lacks is false and a sum of nothing is zero; a condition binds to one
  occurrence and a total has none (C92) — so the reuse is at the value,
  where the same `.<slot>` reads a line and a ranged computed value.
- Names: "for named pseudo-mods, let's use underscored, because dashes
  have meaning elsewhere." With it, the language is shown to be formally
  constructible: a parser and printer with a round-trip test (C104) is
  the first thing built.
- No `has:` on a total, which had read as its opposite on an item with
  no resistances at all: "agree, no has:pseudo"; a derived field takes
  it (T2, 2026-09-24).
- `~`: "Using ~ for regex makes sense". Two notes for the help: patterns
  are for words and comparisons for numbers; a `sum` over
  pattern-selected lines is the user's arithmetic, never a reviewed
  total (the all-elemental line counts once there, three times in
  `pseudo.total_res`). It has no trade equivalent, so it is the
  translation's remainder (C99). Its cost over all displayed text was
  measured at the first surface: 9 ms an ask over 22,623 items (the build
  plan's M4).
- On the case of `=`: "i agree with any-case everywhere after this
  investigation." Six pairs of the census's 6,549 templates differ only
  by capitals, each a line GGG has spelled two ways; an exact `=` would
  find 209 items of one such line and miss the 2 spelled the other way,
  with no sign of it.
- Positions are `arg<N>`, his form, after he asked whether `#1` and `#2`
  could lose the special character: the template is a format string and
  its numbers are the arguments filled into it, so every slot word is a
  plain word and `#` means one thing, the placeholder inside a quoted
  template.
- Bare words: "I approve your proposed rule on bare words." It is C91's
  own rule, and it passes the break-later test in the right direction —
  allowing bare words later breaks nothing, forbidding them later would.
- A mod displayed over several rows: "the two-line mods should
  contribute to relevant pseudo-lines, but it makes sense they are a
  single occurrence." On `true()` and `false()`: "agree with
  recommendation". On the header, asking for the third field: "should we
  add something like type line to the search in addition to name and
  base?" — the C++ app's pretty name, the two joined, is a rendering,
  never a field.
- Rarity and the frame: "Rarity is now a first-class field in Item
  objects, and it can be Normal, Magic, Rare, or Unique. However, there
  is also a frameType (deprecated enum int, but still present) and a
  frameTypeId string … I think this means we need to support both
  'rarity' and 'frame' or 'frameType' as a search." On an ilvl of 0 as
  absent, and on a vaal gem's base skill as lines and displayed strings
  of their own: "Agreed", each. On the requirements, after the builder
  had made each its own string: "In the trade site, item level is
  rendered as "Item Level: <N>" and the required level is "Requires
  Level <N>"", then "I agree on the display row for requirements with
  each requirement kept separately." The C++ app builds the same row
  (`src/ui/itemtooltiptext.cpp`, name before value throughout); here
  GGG's `displayMode` orders each, `159 Str`. On the item level, a
  field only until then: "Let's make the item level a displayed string."
  Measured over the census's store copy
  (22,721 live items): 8,297 carry no rarity, 7,903 an ilvl of 0, 457 a
  base skill of 2,251 lines.
- The trade site's spelling of a line: "The trade site accepts "+# ...",
  "#% ...", and "+#% ...", so our grammar should accept those because
  agents and humans will expect it." On a template alone meaning the
  line: "users may want to search for the presence of the line without a
  value constraint", and on its negation: "critical for some builds.
  allow." No displayed string on the census's store copy holds a `#` (0
  of 22,721 items), so the phrase reading it replaces could never match.
- The ranged line. On the shield line: "average is of the first two
  numbers". On `(#-#)`: "real items will only ever have 'Adds # to # Cold
  Damager'. The ranges-of-range are the underlying mods that allow the
  upper and lower limits to be randomized, but they will not appear in
  real items." The two-pair error was proposed at the harvest and agreed
  the same day. The census (`item-facts/scripts/ranged-split.py`): 167
  templates with one pair alone, 17 with a further number — the pair
  first in every one — 2 with two pairs (two lines in the corpus), 423
  with several numbers and no pair, 166 `(#-#)`, every one under a slot
  prefix in `explicitMods`.

### One worked example

Seven pc items: rare rings r1 (life 95, res 65), r2 (20 implicit + 75
explicit, 60), r3 (100, 55), r6 (readable 95, implicit array unread, res
subtotal 65 incomplete), r7 (no life line, 65); r4 a *magic* ring (95,
70); r5 rare, life 95, res 65, its base missing from the class table.
Two fetched locations, one never fetched. Every count below is worked by
hand from those facts.

**The request** (shape, not the Rust layout; a request sends `text` *or*
`tree`, an answer returns both):

```json
{ "scope":  { "account": "A", "realm": "pc", "membership": "live" },
  "query":  { "tree": { "all": [
      { "field": "league", "op": "=", "value": "Standard" },
      { "field": "class",  "op": "=", "value": "Rings" },
      { "field": "rarity", "op": "=", "value": "rare" },
      { "exists": "lines", "where": { "all": [
          { "attr": "template", "op": "=",  "value": "# to maximum Life" },
          { "attr": "arg1",     "op": ">=", "value": 90 } ] } },
      { "value": { "pseudo": "total_res" }, "op": ">=", "number": 60 } ] } },
  "view":   { "rows": { "limit": 20 } } }
```

**The answer at a terminal:**

```
query   league=Standard class=Rings rarity=rare line("# to maximum Life" arg1>=90) pseudo.total_res>=60
scope   account A · pc · live · 7 items · 2 locations fetched (oldest 3d, newest 2h)
        1 never fetched · location list seen 2h ago                       more: --view locations
basis   store 3f9a1c0be27d · snapshot 41 · intent 12 · totals v1 · classes v3
terms   each term evaluated independently over live pc items in all leagues
  0    league=Standard                        7 matched
  1    class=Rings                            6 matched · 1 undecided
  2    rarity=rare                            6 matched · 1 failed
  3    line("# to maximum Life" arg1>=90)     5 matched · 1 failed · 1 lacked · 1 reaches 90 only together
  4    pseudo.total_res>=60                   5 matched · 1 failed · 1 undecided
total   1 match · 2 undecided
rows    r1  Two-Stone Ring · rare · Standard / Rings
            +95 to maximum Life (explicit) · pseudo.total_res 65
next    1 of 1
routes  (--routes prints all; one shown)
  term 3 failed, over the scope:
    acq search --account A --realm pc 'line("# to maximum Life") -line("# to maximum Life" arg1>=90)'
```

The two undecided items say why and what might help: r5 — *class: base
not in the class table; a refresh will not help, a reference update
may*; r6 — *pseudo.total_res: implicit lines unread; a refresh may
help*. These are hints, never guarantees.

**The same route under `--json`** — directly resubmittable, carrying its
denominator and the basis it was counted at; run on a later basis it
reports both counts:

```json
{ "count": 1, "counted_at": { "snapshot": 41 }, "denominator": "scope",
  "request": { "scope": { "account": "A", "realm": "pc", "membership": "live" },
               "query": { "text": "line(\"# to maximum Life\") -line(\"# to maximum Life\" arg1>=90)" },
               "view":  { "rows": {} } } }
```

It returns r2. Appending it to the original query instead would return
nothing, which is why a route is a request and never a fragment. A
bucket of a count view routes *under the query*: counting every item by
class gives `ring 6 · undecided 1`, and the undecided bucket's request
has the query `undecided(class)` and returns r5, which no root-undecided
list would ever contain; a `none` bucket routes by `-has:<key>` (C105).
A crossed cell's route carries both keys. Paths (`0`…`4`, `3.1`) belong
to this exact returned query; after an edit, use the next answer's.

**When the total is zero**, in place of rows: selectors that resolved to
nothing, with suggestions (tolerant suggestion, exact execution; a
suggestion may be a whole, deliberately broader request); the
root-undecided route; the coverage summary; and the *offer* of
`--explain` and `--context corpus`, never their output.

### Explain

`--explain <path>`: the root's definite and undecided counts as given,
with the node forced true, and with it forced false, each beside its
transformed query. Labelled literally — never as removed, never as a
contribution (`A or B` with an item satisfying both loses nothing when A
is forced false). Inside a member group the substitution is per member:
it never creates a member, and an unread collection is not a population.
Opt-in, one node at a time.

### Invariants of the surface

1. Canonicalisation lowers shorthand and normalises spelling; it never
   reorders, flattens, merges, deduplicates or simplifies.
2. A `:` or `~` selector is never replaced by what it resolved to; the
   answer shows the binding beside the authored selector, and a
   vocabulary row's selector is always the exact form.
3. A selector's validity never depends on the corpus.
4. A count shown has a route to its members, and the route keeps its
   denominator.
5. Every block of an answer is bounded: summaries and a route to the
   whole, never an enumeration by default.
6. Search is a pure read, and no item predicate supports a claim about
   an unfetched location.
7. Parentheses group and do nothing else: a rewrite that changes no
   meaning changes no match, no count, no error and no route — only the
   canonical text and the paths (invariant 1).

### Outside the first surface

Directions agreed in the dialogue, nothing specified; none is a park,
since none has a trigger yet. Meeting one is a listed limit (C102).

| Thing | State |
| --- | --- |
| a refresh plan from a selection of uncovered locations | an integration gap: `--plan` compiles stored policy and excludes `--tabs`; C76's selected-location plan is parked in `decisions/plans.md`. Search reports coverage honestly meanwhile. |
| alternatives for a node | deferred. If built: evaluate the query with the leaf replaced, by the normal evaluator; any optimisation must reproduce that. |
| a slot's distribution under the rest of the query | deferred; an edited bound and another search serve. |
| user-authored weighted sums | direction agreed — an unnamed total; operands are anything with a value, a definition and a status. Range weights, negative weights on ranges and guarded trade groups need their own definition first. |
| continuing a retained basis | a resident client's improvement. Default everywhere: a changed basis refuses the continuation and names both; an expired or never-held basis is its own error; `show --against` obeys the same. |
| cross-currency price comparison | needs an explicit valuation input; the currency table holds no rates. |
| `in pc: …` as a scope-bearing text | untested; try copying the full invocation and the JSON request first. |
| substring over a displayed occurrence inside `line(…)` | unavailable; item-level phrase search reaches displayed lines. |

## Contract detail, by decision

What remains is detail no module carries yet: the derived fields not
yet built, what reference data admits (every table to come), the trade
translation (not in this plan). The
paragraphs the crate took on 2026-09-23 — C89/C103, C90/C102, C92,
C93, C95, C105, C96, C97, C98, C100, the derivation's shape, and C94
with C101's DPS at step 7 — are their module docs' now (`eval.rs`,
`group.rs`, `counts.rs`, `answer.rs`, `corpus.rs`, `derive.rs`,
`show.rs`, `totals.rs`, `pseudo.rs`) and this file at `aeeba6d3`. The
site's 298 pseudo stats by the mechanism each needs, with the input
that admits each, is `search/pseudo-stats/README.md` (2026-09-18; 109
unresolved, each with the one read that closes it).

- **C101 — the derived fields not yet built.** Base
  defence percentile is the item's defence against its base's range
  from reference data (C68, C106) — the RePoE export carries the ranges
  (`search/repoe/data/base-defences.csv`: 476 bases, 466 with armour,
  evasion or energy shield, 9 with ward; pseudo-stats open question 3,
  closed); an item above its base's current maximum reads over 100 %,
  and a realm with no table is undecided with that reason, never zero;
  a quality-normalised defence follows S26; each is a named pure
  function in the search crate, listed with the totals under `pseudo.`
  by `--describe` (C97; the reference), as `pseudo.dps` and
  `pseudo.pdps` are (`pseudo.rs`). The sockets are `sockets.rs`'s
  (step 8).
- **C106 — what reference data admits, and the test.** Three kinds:
  *data*, what the game files state and a registered surface exports;
  *convention*, a concept the community has published a definition for
  (the owner, 2026-09-18, on "equipment": "This is something that's
  well-defined by the community, e.g. RePoE, the trade site categories,
  poedb.tw, and https://www.poewiki.net/wiki/Equipment"); *judgment*,
  never embedded. C79 never required a surface to be GGG's: it requires
  registration, no runtime fetch, and a human's commit (C68). A
  convention behaves as a named total does (C94): defined once, printing
  its definition and source on request, naming one authority with the
  rest as corroboration and any disagreement noted on the row — order of
  authority: GGG-authored surfaces (the API, the trade site's own
  categories, the item-filter documentation), then game-file exports
  (RePoE, Path of Building), then community prose (the wiki, poedb). A
  word that is the trade site's says so. **The admission test:** (a) a
  published definition at a registered surface; (b) reviewable —
  generated by a committed script from a pinned source and reviewed as a
  diff at each pull (a "small enough to read every row" test would have
  refused the 476-row base-defence extract); (c) the word prints its
  definition and its source version, which the basis cites (C98); (d)
  every question can still be asked without it. A table enters one at a
  time — "may back a field", never "the app carries game data" — and a
  surface's row changes only when the first definition needs it, its
  licence read then (the wiki is CC BY-NC 3.0; poedb states no terms and
  is unregistered). "A roll outside the base's current range" is data;
  calling it legacy is history the export does not carry — legacy is
  never a field (C107); a unique's variant, a label its source prints,
  is the parked candidate (`decisions/search.md`, "Parked").
  **Three class readings ruled at the first seat (V1, V2, 2026-09-26;
  built at 9b):** a blighted or blight-ravaged map's class is read from
  the base past the prefix the API adds (`Blighted Map (Tier 13)` reads
  as `Map (Tier 13)`, Maps); an invitation's class is its frame's — the
  quest frame Quest Items, any other Misc Map Items; a captured beast's
  class follows the trade site's categories, the grouping above class's
  first case (9d).
- **C93 — the undecided block's shape (V8, 2026-09-26; built at 9b).**
  An answer shows its undecided items as one line per distinct reason,
  each with one example item, then the one route to them all; after the
  rows, before the routes. No item list and no count per reason: no
  term selects a reason, so such a count would have no route (C105). The
  JSON has the same shape. A per-reason count and its selector wait for
  someone missing the number.
- **C99 (K5).** `count` carries its minimum and its maximum; `weight`
  and `weight2` differ in how a per-stat requirement gates a
  contribution (S57), and a weighted group translates only where that
  guard has a representation; a defence bound is inexact while the site
  normalises quality (S26) and sits in the remainder until a normalised
  field exists. Several ids for one line go out as a `count` of at
  least one (S106). A site bound on a ranged line is `avg` on the line
  (S47), exact once the word exists.
