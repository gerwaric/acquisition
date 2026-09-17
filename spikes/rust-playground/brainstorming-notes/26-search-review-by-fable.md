# 26 — The search review, by Fable

Fable; read and used design A, then design B; under the review brief (note 30) at a47fa1e5. Design A's section below was written before design B was opened.

## Design A

### The one idea, before use (5 lines)

A search is a typed question document — a scope, one Boolean tree over
fields and same-occurrence line tests, and what to give back — and the
answer hands that document back with true / false / unknown counts and
the evidence per row. Discovery, refinement, saving and trade are that
document produced, appended to, stored or translated.

### My questions, in its notation

**F1 — boots: movement speed 30+, and any two of the three resists at
35+** (the precise-among-broad one).

```text
in pc/Standard where rarity = rare and class = Boots
  and line(template = "#% increased Movement Speed" and n1 >= 30)
  and at_least(2, line(template = "#% to Fire Resistance" and n1 >= 35),
                  line(template = "#% to Cold Resistance" and n1 >= 35),
                  line(template = "#% to Lightning Resistance" and n1 >= 35))
```

Wrote first time, and reads back as what I meant. `at_least` over
`line(...)` children is exactly the trade site's count group, across
fields. Then I saw it misses `+#% to Fire and Cold Resistances`. The
design's answer is `total.fire_resistance >= 35`, and the design itself
says that may come back unknown: "the predicate on the total remains
unknown". So I hold two spellings of one wish — one exact and
incomplete, one complete and possibly unanswerable — and nothing tells
me in advance which the stash will reward. I could not work out *when* a
total is unknown: "Unknown applicable lines that cannot be ruled out as
contributors" — ruled out by what? The design has no mod spine, and the
ruling-out rule is not on the page.

**F2 — how much currency do I have, by kind.**

```text
in pc/Standard where class = Currency summarize count(), sum(stack.count) by base
```

One line, no rows. I enjoyed this most. First try I added `order
sum(stack.count) desc` and it is not in the grammar: `order E`, and
`sum` inside `E` is the other sum — `sum(collection where P, E)` against
`A := … sum(E)`. Two `sum`s of different arity and meaning, and
`summarize` has no `as`, so a summary cannot be ordered by its own
aggregate as written.

**F3 — a crafting base: six-link body armour, three blue in the link,
uncorrupted, ilvl 84+.**

```text
in pc/Standard where class = BodyArmour and corrupted = false and ilvl >= 84
  and any(socket_groups, size >= 6 and blue >= 3)
```

`size`, `ilvl` and `corrupted` are my guesses; the page names `red`,
`blue`, `required.level` and no others, so `describe` is a required
stop. The same-occurrence rule doing the link for free is lovely. The
worry is `corrupted = false`. GGG omits the key when false; the design
says "A missing known optional field is absent … ordinary comparisons
to either are unknown", and also "Boolean flags are `= true`, `= false`,
or unconstrained". I trust the convenience flag reads false; by the
letter `fact["corrupted"] = false` would be unknown on nearly every
item. The same fact behaves two ways by which door I use, and the raw
door is the one offered for anything unnamed.

**F4 — the jewel I half remember: corrupted-blood immunity and some
life.**

```text
in account where line(text contains "corrupted blood") and line(text contains "maximum life")
```

Exactly how I think. Two `line()`s bind two occurrences; text ranges
over every array, so the corrupted implicit is found without my knowing
it is one. No friction at all.

**F5 — what left my stash this week that was unique.** I could not
write it. Scope "abbreviates a document naming … live/removed
membership" but the token is deferred ("scope tokens (§5)" in the
contract gap), and "Literals include numbers, quoted strings and
booleans" — no date, no duration. Observation age is on the row; I
cannot put a condition on it from what I was given.

**F6 — rank my rare rings by life plus half their elemental resists.**

```text
in pc/Standard where rarity = rare and class = Ring
  select identity, location,
    sum(lines where template = "# to maximum Life", n1) + total.elemental_resistance / 2 as score
  order score desc
```

My first try used `max(...)`. "Min/max of an empty collection are
absent", so every ring without life would have sorted last with no
score instead of scoring its resists. `sum` is right because "An empty
complete sum is zero". The design would have shown me blank scores, not
told me why. This is the question that felt like programming.

**OQ5 — leveling gear**, as I would ask it: `… and required.level <=
28`. Tabula Rasa has no level requirement. It lands in the unknown
count, not the matches, unless I write `not present(required.level) or
required.level <= 28` — which the design does teach (§4.2), and which
its own appendix row does not use. Rings, amulets, jewels and low bases
lack the requirement by nature, so on my stash the unknown count for
this query is thousands, and "absent by nature" and "could not decode"
share it.

**AQ2 — refine.** Over MCP: take the canonical query out of the last
answer, wrap it `{"all":[old, new]}`, resubmit. Cheap, stateless, and I
like that the answer is where the query lives. At a terminal `refine`
is up-arrow and retype; the design says as much ("the adapter's local
operation").

### The cold start, as I walked it

MCP, for F1. The tool description gives the skeleton and names
`search_describe`. Call 1: `search_describe`, scope `pc/Standard`… and
I do not know the league token is valid, so honestly call 1 is
`describe` on `account` for realm and league, call 2 is `describe` with
terms `boots, movement speed, fire resistance, cold resistance,
lightning resistance`, call 3 the query. Three calls, and I believe it,
because describe returns "ready-to-use predicate fragments" — the
feature that makes this design usable at all. What I do not believe yet
is the JSON. The text grammar is what the page taught me; MCP "accepts
its structured JSON", and the page shows two lines of it. For F6's
arithmetic and `select … as` I have no JSON to write: "general typed
expression nodes are the JSON encoding of the grammar above" is a
promise. As an agent I would rather send the text.

### Notes, as the user

- F1, F6, OQ5: three of eight questions ran into unknown. It is the
  design's conscience and its tax. A counter that is nonzero on every
  ordinary query is one I will stop reading by the second day.
- F1: I could not tell whether I may ask for a fractured mod. The
  owner's refined OQ1 is that case, and the page says `flag.fractured`
  "remain unknown wherever no reviewed private mapping exists".
- Whole read: 44 KB, and much of it is disclaimers set into the model
  ("Text is a proposed notation, not existing CLI"; "not this design's
  performance promise"; a bolded "Gap:" thirteen times — counted). Honest, and it cost
  me: I kept having to decide whether a sentence offered a feature or
  withdrew one.
- F2, F4: where it is a small language over my items, it is a pleasure.
  I already know how to think in it; nothing was alien.
- F3, F5: every field name beyond a dozen is a describe call away. Fine
  for me; page one alone would not let a person type a query.
- I never once wanted SQL, and never thought about the engine.

### The one idea, after use (5 lines)

A small typed expression language over items and their line
occurrences — aggregates, arithmetic, `if`, raw paths; bigger than "a
question" — whose one hard commitment is that don't-know is never
turned into no. It pays for that where it hurts most: its own named
totals, the everyday shortcut, may decline to answer.

What changed: the echoed question document moved from the centre to the
furniture — useful, quiet, never in my way. The three-valued truth moved
to the centre, because I met it on three questions of eight and it
decided what I would type each time. And the language grew: page one
reads as seven lines, and using it I needed §4.2's rules for empty sums,
absent maxima and `present` to get two of my eight right.

## Design B

I met it tired and relieved: 20 KB after 44. Relief reads as quality,
so discount my warmth by some of that. A also taught me the questions —
occurrences, absence, hybrids, set totals — so I arrived knowing where
to press, which B's first reader would not.

### The one idea, before use (4 lines)

An item is the lines it displays, where it is, and what you said about
it. You type the stash box — words, and lines with comparisons, side by
side — and one call answers in one shape that says how each term
resolved. Counting is the same call, so discovery is search.

### My questions, in its notation

**F1 — boots.**

```text
rarity:rare class:boots "#% increased Movement Speed">=30
  holds(total-fire-res>=35, total-cold-res>=35, total-lightning-res>=35)>=2
```

A third the length, written without looking anything up but the total
names (guessed from "`total-fire-res` …"; the page lists five of 35).
The hybrid-resist problem A left me holding is gone: the total is a
sum, and a sum answers. I noticed I believed it more easily because it
did not hedge, and that is not evidence. The mode switch — a `#` in the
quotes "names one whole template; without #, it is a case-insensitive
substring" — I got right only because I had just spent an hour on
templates.

**F2 — currency by kind.** `class:currency --count base` gives me the
number of *stacks* of each. I found no way to total `stack`: "*Counts*
group the set by any name", `sum(thing, 2*thing, …)` is a per-item
quantity, and the design refuses "arithmetic beyond `sum`". How many
chaos orbs I own is not askable. It is not in the design's own gap
list.

**F3 — the crafting base.** `class:"body armour" links>=6 colors:bbb
-is:corrupted ilvl>=84`. Ten seconds, every word from the printed
list, and `-is:corrupted` has no absence puzzle in it. `colors:bbb` is
the item's sockets as a multiset, not the link's; on a six-link that is
the same thing and on a five-link it is not. A was exact here and B is
close enough that I would not notice until it mattered.

**F4 — the half-remembered jewel.** `"corrupted blood" "maximum life"`.
Shorter than A and the same thought. Unquoted, a bare word "is a name
when one exists and text otherwise", so `tier`, `note`, `stack`,
`links` typed as words mean has-that-field, and every name I define
later widens the set of words that stop being text. The terms block
would show me; I would still be surprised once.

**F5 — what left this week.** Closer than A and still no: `removed` is
a printed field, but "default scope is … live items" and nothing says
whether naming `removed` widens it, and a comparison takes a "number" —
no date.

**F6 — ranked rings.** `rarity:rare class:ring --sort sum("# to maximum
Life", 0.5*total-ele-res) --desc`. One line. But "Absent is never a
number", and the page never says what `sum` does with a ring that has
no life line. A total is a `sum` over templates no item carries all of,
so absent members must count as nothing — I inferred the answer the
design needs; it did not tell me.

**OQ5 — leveling gear.** The appendix needs `gear` defined, and the page
has no syntax for defining a name — only that "defining a name that
exists is an error". "One operation: ask" has an unshown second one. I
would type six `class:` terms in an `or`. The `-has:reqlevel` clause is
the same lesson as A's, but here the answer would have taught it:
"items lacking it", per term, is Tabula Rasa waving at me.

**AQ2 — refine.** Append a term. Except: "`or` binds looser than
adjacency", so appending `ilvl>=80` to the appendix's own OQ3 text
narrows only the Kaom's Roots branch. "The previous query plus one
term" is true only as `(old) new`. A said this out loud; B leaves it
for my first bad refinement.

### The cold start, as I walked it

The description carries the grammar and the twenty-nine fields, so for
F3 my first call is the answer. For F1 I would not make the design's
discovery calls; I would send the query, and the terms block tells me
what each text resolved to, or shows "the templates sharing its words".
One call, two if I was wrong. A misspelt total quietly becomes text
that resolves to nothing — visible, though it will offer me templates,
not names. At a terminal the stranger's first query is `ilvl>=84`
unquoted, and the shell redirects into a file called `=84`; the page
does not say the query is always one quoted argument.

### Notes, as the user

- F1, F3, F4: I typed, and it was right. That has not happened to me
  with a query language before I had read its reference.
- OQ5, F1: the terms block — "items matching it alone, items lacking
  it" — is what would make me trust an empty answer. It names the term
  that emptied it. A counts unknowns per answer; B counts lack per
  term, and per term is where I can act on it.
- F2, F3, F6: everything B cannot say, I learned from A an hour before:
  set totals, the same socket group, one occurrence rather than their
  sum. B's first reader would not miss them as soon. I do not know how
  often I would.
- F1: `fractured:"…"` is offered flatly here and doubted at length
  there. The designs disagree about a fact of the data and I cannot
  settle it from either.
- There is no "could not read this" anywhere. "A line is never unknown
  to the search" holds for text; an array the deriver has never seen is
  simply not lines, and no count would show it.

### The one idea, after use (4 lines)

The stash's text box, grown up — and the *answer* is the manual: every
ask reports what each term became and what lacked it, so a wrong query
is one call from a right one. It buys being typable first time with
sums in place of occurrences and no arithmetic over the set.

What changed: "bag of lines" was the idea I read; the terms block was
the idea I used. The printed closed field list mattered more than I
expected — I never made a discovery call for a field — and "one
operation" turned out to be two, with the write unshown.

## Having used both

**I would work in B tomorrow.** With a budget, the cost that dominates
is a wrong first query, and B makes it cheap twice: the query is short
enough to get right, and the answer says which term failed. In A I
would make a describe call before most queries and read a counter I
half trust. At a terminal I would choose B more strongly still; nobody
types `line(template = "…" and n1 >= 90)` at a prompt for pleasure. A
different agent changes it. One that splices better than it composes —
smaller, or raised on JSON schemas — is safer in A, where describe
hands over fragments to insert and the tree has no modes; B's switches
(`#` or not, name or word, `:` a substring except on `colors`) are
forgiving to me and would not be to it. An agent whose job is audit, not
finding, wants A's `inspect` and its refusal to say no. My choice is
for finding things; for *accounting* for a stash (F2) A is the only one
of the two that can do it.

**What should survive, at its strongest.**
From A: *same-occurrence binding over collections* — "this line, this
kind, this value" and "these colours in this link" are one idea, and B
has no way to say either exactly. *Summaries over the set* — count and
sum by anything, no rows: the stash as an inventory, which the owner
asked for in S170. *Don't-know is not no* — for what could not be read,
which is rare and therefore legible. And `where (old) and P`: refinement
defined with its parentheses.
From B: *the terms block* — resolved-to, matching alone, lacking, per
term; the best answer either design gives to "what would make you trust
an empty answer". *Discovery is search*: `--count line resist` under a
query is a vocabulary scoped to what I am looking at, which a separate
describe call can imitate and never quite be. *The closed field list in
the help.* *Totals written in the grammar*, so what counts is readable
and a user can write their own. And the string as the thing a person
and an agent hand each other.

**Two steps out.**
*A GUI on a keystroke.* A's tree is already form rows and its persisted
projection survives a refresh in progress; its three-valued answers
need a third bucket in every list, which bends. B's `--count line`
under the current query is the autocomplete I would want; it breaks
where it says it does — "a keystroke after a change pays the whole
re-derive, and under an active refresh that is the common case".
*PoE2.* Both stand on displayed text, so both work on day one. B bends
well through lines (`"Spirit: #">=30` needs no field) and badly at its
closed list — `links`, `colors`, `tier`, `es`, `pdps` are PoE1's nouns —
and its default scope, "every realm and league", will put two games'
items under one `total-res`. A's explicit scope, "no implicit 'last
league'", was a chore in use and is right here.
*The unnamed interface.* B's string pastes into a chat, a URL, a
commit message. A's document carries a version and "never reinterpret
itself", which is what a saved thing needs.
*The owner in a year.* In B he types `explode` and is working again;
the answer re-teaches him. What breaks is a bare word that is secretly
a name he defined and forgot, and a shipped total that shrank silently
when a template moved (the design lists this). In A he must re-read
before he can type, and then sees "unknown: 4,312" with no memory of
why that is normal.

**What they made me think of.**
1. *Three numbers per term, not two.* Matched, lacked, could-not-read —
   A's conscience inside B's terms block. The third is almost always
   zero, which is what would make it read when it is not.
2. *Judge the wrong first query.* Both cold starts count calls to a
   right answer assuming a right first query; of my eight first tries,
   four in A (F1, F2, F6, OQ5) and two in B (F3's colours, AQ2) were
   wrong or silently short. A set of plausible wrong queries,
   and whether the answer contains the correction, would measure what I
   actually felt.
3. *Set totals as a column of the counts view.* `--count base` with a
   summed field beside the count would answer F2 without arithmetic
   entering the language — a possibility for either design, since A's
   `summarize` could equally be read as a view.

## What I left out

The derivation sections of both: I read them and never felt them, so I
have no testimony on persisted against re-derived beyond the two-steps
paragraph. The trade boundary — neither can be *used* from its page, and
I did not try. SQL, which I never reached for in either. PoE2
questions of my own. A's `inspect` and B's `--against`, read and not
exercised.

**The cut I would most want reversed** is a deliberate mistake walk:
the five likeliest first-try errors made on purpose in each design,
with what the answer would say back, quoted. My claim that B corrects
where A prevents rests on eight questions apiece and on errors I
happened to make.
