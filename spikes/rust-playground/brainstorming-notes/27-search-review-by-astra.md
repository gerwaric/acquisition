Astra; read and used B, then A; review brief at `a47fa1e5015022746f27fdbcce75971bf6c076eb`.

# Search review by Astra

These are paper walkthroughs of proposals, not executed searches. I did
not inspect items or run a tool from either design. “Works” below means
I could express my question under the proposed semantics, not that a
stash returned matches. I completed B's account before opening A.

## B — before use

My one idea, two lines:

> Ask about the words and numbers on my items, combining small terms.
> Search, vocabulary and explanation are views of the same question.

### My five questions, then two from the acceptance set

**1. I need replacement Spiked Gloves: uncorrupted, item level at least
84, a fractured suppression line at least 14, at least 80 life, and
35 total dexterity.** The suppression requirement is precise; life
and dexterity are the broader contribution I need from the item.

```text
realm:pc league:Standard base:"Spiked Gloves" -is:corrupted ilvl>=84
fractured:"#% chance to Suppress Spell Damage">=14
"# to maximum Life">=80 total-dex>=35
```

I like how little ceremony separates the base, flag and mod. I can
read the query aloud. I supplied the suppression wording; discovery
would have to confirm it. I initially wrote `fractured:"suppress">=14`,
then felt obliged to spell out the template: B says a substring under
comparison means “any template it resolves to”. I want the announced
stat, not any future sentence mentioning suppression. That first try
would be accepted, with the resolution in the answer, rather than
flagged as my mistake. I have to read the answer's terms.

For life, summation is welcome *in this question*. B says “implicit 20
+ explicit 75 is 95 life, each contribution shown”. I want the
contribution from the gloves, not necessarily one large roll. I did
not need to construct a pseudomod just to retain both lines.

**2. Find a wand with one explicit physical-damage line whose low end
is at least 12 and whose high end is at most 28.** I am comparing a
particular displayed range, not total physical damage.

```text
class:wand
explicit:"Adds # to * Physical Damage">=12
explicit:"Adds * to # Physical Damage"<=28
```

The stars let me say the two ends separately, which I enjoyed. Then
I remembered the rule: “An item's value for a line is the **sum of
its lines of that template** across the kinds in scope”. My two
clauses do not give me a binder for one occurrence. If the selected
kind contains repeated occurrences, I cannot claim this answers my
question. B explicitly keeps duplicates, so I cannot silently assume
them away. I found no one-occurrence spelling in its grammar; this is
an unanswered question on my walkthrough, not a request to average
the ends. The answer's contributions could let me inspect candidates,
but inspection would be additional work, not the same query.

**3. Find uncorrupted boots with at least 30% increased movement speed
and at least two of: 35 total fire resistance, 40 strength, 40
dexterity.** I can adjust the rest of the build around any two.

```text
class:boots -is:corrupted "#% increased Movement Speed">=30
holds(total-fire-res>=35, total-str>=40, total-dex>=40)>=2
```

This was the happiest query for me. `holds` counts requirements I
already know how to write. I do not have to distribute three pairwise
OR branches by hand. I first reached for `count`, then looked back at
the grammar and used `holds`; I expect to learn that word quickly.
I would inspect the totals' definitions before using this to select
gear. B promises totals “printed with [their] definition”; I am using
those proposed names, not asserting what their unseen recipes include.

**4. Show my corrupted, unpriced rares in Standard, highest item
level first, so I can decide what to inspect before selling.**

```text
realm:pc league:Standard rarity:rare is:corrupted -priced
--sort ilvl --desc
```

This felt like searching. I did not think about a schema or read an
item body. The built-in name `priced` and negation were sufficient.
I particularly want the promised “place by name” beside each result:
a list I can walk to in game is an answer I can use. No valuation is
being requested.

**5. Before clearing old leagues, how many rare items do I have for
each league-and-class combination?** I want to see whether one league
contains most of my rings, not two unrelated totals.

```text
rarity:rare --count league,class
```

That was my first attempt and it is wrong. B says “one table each,
never crossed” and lists “no cross-tabulated counts” as a gap. I
would receive two valid-looking tables, not a syntax error. Reading
the heading should reveal my mistake, but the commas tempted my
GROUP BY habit. Separate searches per league could recover the
answer; they would cost calls and reconstruction. I leave it a gap
here, without routing to SQL.

**6. OQ5: “I want to practice leveling, so I need to find my leveling
gear.”** I make my present need concrete as equipment usable by level
28, including equipment with no level requirement.

```text
gear (reqlevel<=28 or -has:reqlevel)
```

This is conditional on a user-defined `gear`, not a runnable cold-start
answer. B says “one query only once the user has named `gear` as the
`or` of the classes they mean” and “the model ships no grouping above
class”. I could spell out the classes after a vocabulary read. I
cannot honestly write a complete equipment set from this page alone.
I also nearly omitted the absence branch. The rule “a comparison on
something the item lacks is false” would make that a silent narrowing,
although the lacking-item counts would give me a chance to catch it.
This is where the compact language stopped feeling effortless.

**7. AQ5: “One whole-corpus mod query with a value: +# to maximum Life
at 90 or more.”**

```text
"+# to maximum Life">=90 --sort "+# to maximum Life" --desc
```

I appreciate being allowed to type the plus sign from the question.
B says “the query's text is canonicalised by the same function”.
But I would ordinarily read this request as a line with at least 90,
and B's own 20 + 75 example would qualify here. That is useful for
question 1 and surprising here. I cannot call the acceptance question
unambiguously satisfied without agreeing what its author meant. The
contributions explain the result; they do not make my initial reading
the query's meaning. This was the largest trust cost I encountered.

### Walking in cold

I reset to the proposed tool description for question 1. It supplies
the grammar, field list, arguments and “`count: line` lists the lines
your items carry.” It does not supply the total names: B itself says
“the description names no total”.

My planned calls, with no item row read, are:

1. `search {query:"realm:pc league:Standard base:\"Spiked Gloves\"", count:"line", text:"suppress"}`.
2. The same scoped vocabulary read with `text:"life"`.
3. The same scoped vocabulary read with `text:"dexterity"`.
4. Search the selected templates and thresholds, with dexterity
   expressed as `sum(...)` over the returned contributions I mean.

That is four calls on the successful discovery path, not four
observed calls. I cannot fill the last sum before seeing the vocabulary
and deciding which displayed contributions count. An empty or
ambiguous discovery result adds thought and possibly calls; it does
not authorize inventing a total. This is considerably easier than
pulling bodies, but I am still doing semantic work. The single-call
route is question 4, whose names and operators are all in the help.
I would use that as my first small success, not as evidence that the
harder cold start takes one call.

### Notes I want to retain

Question 3 made composition feel natural. Question 5 caught my own
imported assumption. Question 7 made me realise I had been reading
quoted templates as line predicates while the design was giving them
item-level values. I do not want that discovery diluted into a generic
request for better documentation: the documentation already told me.
I read it, liked the simplicity, and still reverted to another meaning
when using it.

I also like that the answer can teach me the corpus's language.
For question 1, B's “per term: what it resolved to, items matching it
alone, items lacking it” would help distinguish a bad spelling from
an over-demanding combination. I would still need the combined total:
nonzero counts for each term do not establish that any item satisfies
their conjunction.

### B — after use

My one idea now, two lines:

> Give item-level values to the words on my items, then compose tests.
> Each answer shows how those words resolved and contributed.

“Item-level values” replaced my casual “words and numbers”. That is
the learning cost: summation, selected endpoints and substring
resolution are part of what I must remember whenever I type a number.
The short syntax remains attractive. I trust it more after use, but
only because I now know where I would need to slow down.

## A — before use

My one idea, two lines:

> Keep an explicit question whose conditions bind the evidence I mean.
> Return the witnesses, and distinguish a failed condition from ignorance.

I met this page with B's summation rule in my head and felt immediate
relief at “several constraints inside it refer to the **same
occurrence**”. That relief is an order effect, not an independent
discovery that collections are intuitive. I was also tired of reading
contracts and impatient with the longer inventory. Both reactions
belong in this account.

### The same questions

I keep the five questions I wrote for B and the same two acceptance
questions. A explicitly says “`$name` marks an example substitution
(value or predicate), not executable syntax”. Below, `$uncorrupted`,
`$corrupted`, `$wand`, `$boots`, `$level84` and `$strength40` stand for
discovery predicates; `$item_level` is its item-level order expression.
I am not inventing field spellings
that its partial catalogue has not supplied. This is a notation
walkthrough with those holes exposed, not seven executable commands.

**1. The replacement Spiked Gloves.**

```text
in pc/Standard where base = "Spiked Gloves" and $uncorrupted
  and $level84
  and line(template = "#% chance to Suppress Spell Damage"
           and flag.fractured = true and n1 >= 14)
  and sum(lines where template = "# to maximum Life", n1) >= 80
  and total.dexterity >= 35
```

I had to write the summation that B gave me automatically. This time
that felt productive: I can see the difference between my precise
fractured requirement and my broad life contribution. `n1` looks
mechanical, but I know what it binds. For this one-number template it
does not slow me much.

I cannot promise the requested shortlist. A says private fractured
flags “remain unknown wherever no reviewed private mapping exists”,
and that “Unknown applicable lines that cannot be ruled out as
contributors make a semantic total incomplete”. I would accept an
honest unknown for an undecodable fractured flag. I am much less sure
how patient I would be with an unknown dexterity total on otherwise
ordinary gloves. That is the first place I wanted to see actual items,
which this review deliberately does not supply. Replacing the total
with a chosen set of lines would change what was promised, so I do
not count that as solving the problem.

**2. One explicit wand range: low at least 12, high at most 28.**

```text
in account where $wand
  and line(source = explicit
           and template = "Adds # to # Physical Damage"
           and n1 >= 12 and n2 <= 28)
```

This expresses the question I meant. My first impulse was to write
two `line(...)` calls after carrying B's two clauses across. A says
“separate `line(...)` conditions may bind different ones”; that would
be a valid but wrong query, not a type error. I corrected it while
writing. Once both bounds sat in one pair of parentheses, the burden
went away. I would still use the promised highlighted captures to
confirm that the two slots are the range I mean. I like having one
place to check that, rather than checking how a quantity was reduced.

**3. Movement-speed boots with any two of fire resistance, strength
and dexterity.**

```text
in account where $boots and $uncorrupted
  and line(template = "#% increased Movement Speed" and n1 >= 30)
  and at_least(2, total.fire_resistance >= 35,
                  $strength40, total.dexterity >= 40)
```

Here the movement requirement means one qualifying displayed line;
B would total repeated occurrences of that template. My original
wording did not settle that distinction. A made me choose it, and
I would choose the line reading for this task. If I wanted the
selected lines' combined contribution I would spell `sum` as in
question 1.

`at_least` was immediately legible after B's `holds`. B helped A here.
I prefer B's compact version when writing, and A's when checking the
question later. A's rule that at-least-N is true when “N children are
true” means an unresolved third total need not spoil two known
successes. That is a welcome bit of the conservative policy. It
still does not establish how many boots would be left undecidable.

**4. Corrupted, unpriced Standard rares, highest item level first.**

```text
in pc/Standard where rarity = rare and $corrupted and priced = false
  order $item_level desc
```

This is short enough. My resistance to A's syntax was disproportionate
to the actual cost of this everyday query. I would check discovery's
definition of `priced`: A says it is “only a documented alias for a
specified resolution”. That distinction matters to a selling task,
although I do not need the whole pricing architecture in my answer.
I want the default location and observation age, then the candidates.

**5. Rare-item counts by league-and-class combination.**

```text
in account where rarity = rare summarize count() by league, class
```

I wrote this without looking back. A says “`summarize` replaces rows
with grouped aggregates”; it separately says facets add independent
tables. My SQL habit helped this time. There is a real additional
concept here, but for this question I had already paid for it years
ago. I would read the unclassed/unknown contribution before deciding
which league to clear. A count of known classes is not a claim that
all equipment was classified.

**6. OQ5, equipment usable by level 28, including no requirement.**

```text
in account where equipable = true
  and (not present(required.level) or required.level <= 28)
  select identity, location, required.level
  order required.level asc
```

I needed the same absence thought as in B. A did provide the equipment
grouping I reached for: “`equipable` follows that taxonomy and shares
its coverage”. It spared me writing a private list of classes.
I initially thought `not present` would sweep every missing value
into the answer. Then I checked: `present` “itself stays unknown for
undecodable input”. So known absence gets in; unknown data does not
become a reassuring yes. I like that result, but I had to stop and
reason through it. I would not describe this predicate as effortless.

**7. AQ5, a maximum-life line of at least 90, sorted.**

```text
in account where line(template = "# to maximum Life" and n1 >= 90)
  select identity, location, evidence,
    max(lines where template = "# to maximum Life", n1) as life
  order life desc
```

This gives my line reading of the question. It also repeats the
selector to obtain an orderable value. I nearly wrote `order n1 desc`;
outside a particular occurrence, that does not say which line's n1.
A's proposed discovery returns “slot examples with aggregate/order
expressions”, which would save me inventing the `max` expression. I
want that help. Without it, this simple request feels like a small
database exercise. A says unknown candidates have “their own selectable
diagnostic view”; I would check their count before treating the exact
match count as a complete answer about the stash.

### Walking in cold

For the same replacement-gloves task, the MCP description gives the
query skeleton and points me to `search_describe`. I would ask for
account-wide realm/league descriptors and the terms Spiked Gloves,
corrupted, item level, suppression, maximum life and dexterity. A
says one call accepts “optional terms/fields” together. I would then
select Standard and splice the returned fragments into the question,
making the life aggregation explicit.

That is **two proposed calls**, discovery then search, if the bounded
description includes all of those descriptors and needs no continuation.
A scoped follow-up or continuation makes it three or more. The page
does not give a complete JSON encoding for every expression, so I
cannot print a fully specified MCP request for this compound query
without inventing parts. I am relying on its promised ready-to-use
fragments, and count the resulting comfort as conditional on that
promise. The common literal predicate shape *is* demonstrated:

```json
{"all":[{"field":"rarity","eq":"rare"},
        {"line":{"template":"# to maximum Life","n1":{"gte":90}}}]}
```

The second call can correctly report unresolved candidates; it is not
a guarantee of finding gloves or certifying that none qualify. For a
first usable success I would do question 4 after the same discovery.
At a terminal I would add the two help invocations A specifies. I
cannot compare that four-invocation path to B's MCP call count as
though the help calls were a design penalty.

### Notes I want to retain

Question 2 sold me on binding occurrences; question 7 charged me for
it. I do not get to celebrate precision and pretend its aggregation
cost disappears. Question 5 was easier because I am already fluent in
grouping, not because every person will find it simple.

The difficult part of question 1 was no longer whether I had silently
asked a broader numeric question. It was whether I would get enough
decidable answers to finish dressing the character. That is a better
place for *me* to have uncertainty, but it remains unfinished work for
the user. An unknown counter is informative; it is not the gear I need.

### A — after use

My one idea now, two lines:

> State exactly which occurrences and quantities would answer me.
> Let the answer separate witnesses from work the evidence cannot do.

I started thinking this was chiefly about evidence attached to results.
I finished thinking its value was choosing the referent before the
comparison. Its cost is that I sometimes have to act like a query
author to get there. After B, I was unusually receptive to that cost;
after two long documents, unusually impatient with its exposition.

## Having used both

### What I would choose tomorrow

**I would work in A.** My budget includes correcting a plausible wrong
question, not just emitting its first string. On the wand range and
the life-line request, I could finish checking A's meaning and move
on. B saved characters and then made me revisit what the numeric
subject was. A's discovery fragments also fit how I want an MCP tool
to teach me: give me a piece I can reuse in the next question.

This is my choice of working environment, not a verdict that A has
solved the design. I am least sure whether its conservative totals
would leave so many ordinary candidates unknown that I would spend
my session compensating for the tool. The page itself calls this an
“everyday-search gap”. If that happened often on the gloves and boots
tasks, the extra precision would not repay the practical loss. I have
no run from which to estimate the frequency.

At a terminal, doing quick remembered-item searches, I would lean B:
the unpriced-rares query shows why. For a saved query I intended to
reuse for precise gear selection, I would still choose A, even if I
typed more. An agent less comfortable than me with collections might
prefer B's single string. An agent that reliably copies discovery
fragments but reasons poorly about implicit conversions might benefit
more than I do from A. I do not know which describes the typical
agent, and a smaller request is not by itself evidence of lower total
token cost.

### What I would be sorry to lose

**From B: discovering the stash while searching it.** “The vocabulary
read, facets, and autocomplete are **the same call as search**” is its
strongest idea for me. On the gloves task I am already in a useful
scope; asking what dexterity lines those gloves have feels like
continuing the task. The automatic per-term resolutions and counts
make even a poor first query educational. I also like the possibility
of keeping what I learned under a user's name: B calls these names
“where what an agent learned about a stash outlives its session”.
I did not exercise defining one, so that enthusiasm is for the idea,
not for an interaction I have verified.

**From A: choosing one occurrence is an ordinary act.** “Has this line
with this number” should not require me to undo an aggregation. Its
scope, condition and requested evidence make a question something I
can hand to another consumer without also handing over my thought
process. Ready-to-use discovery fragments are the ergonomic half of
that claim; without them, I fear I would be defending an excellent
semantic model that I disliked operating.

### Two steps out

**A GUI answering in a keystroke.** I can picture B's scoped line
counts feeding a very approachable picker; I can picture A's returned
fragments feeding condition rows that preserve the exact binding.
Neither earns a responsiveness verdict from this reading. B explicitly
says “a keystroke after a change pays the whole re-derive”; A says
“initial implementation reloads”. I would expect both to bend between
updates and to risk breaking the *feeling* of immediacy during refresh.
A's visible loading state is honest, but waiting honestly is waiting.
B's unified read is attractive, but does not itself make a reload cheap.

**PoE2 in December.** This is a hypothetical future task, not a
prediction about the game. For unfamiliar displayed text I would feel
less blocked in B, whose vocabulary is “the corpus's own”. A likewise
says an unfamiliar line is “still searchable as text”. I would expect
both to bend for new words and both to need attention when a numeric
shape or source changes. For a new semantic total, A's stated
incompleteness would protect my confidence but could stop my search;
B says a moved template in a shipped total and one absent from the
stash “look the same”. That is where B would break my willingness to
treat an ordinary-looking total as settled. Neither page warrants a
claim that future sockets, classes or game mechanics are already solved.

**An unnamed future interface.** For a voice client, as an example,
I would want a short spoken question and a compact confirmation of
its actual constraints. A's explicit tree and same-occurrence grouping
feel easier to confirm precisely; B's terms block feels easier to
explain conversationally. This is my inference from the walkthrough,
not a tested adapter. The burden I foresee is translating a user's
“life” into a total or a particular line. Neither interface can remove
that ambiguity simply by hiding its notation.

**Tom a year later.** I expect B's short queries to be easier to resume
reading; I expect to forget the sum rule again on AQ5. A's explicit
line and aggregate distinction would help me reconstruct intent.
A says a changed formula meaning gets a new version and a saved
query must upgrade or receive `query_version_unsupported`. I would
trust that boundary, and also resent a saved search that stopped
working when I just wanted boots. B's exact-template diagnostic would
help with a vanished wording, but its admitted shipped-total gap is
the place I would most worry about being reassured incorrectly.
Both preserve observed words rather than the game knowledge in a
person's head; neither can reconstruct a forgotten reason for choosing
one variant merely from its text.

### What using them made me think of

1. A useful future usability test would ask the same person for “90
   life on the item” and “a life line of 90”, separated by another
   task. I want to see which distinction survives ordinary distraction,
   not just whether someone can repeat a grammar immediately after
   reading it.
2. Discovery's output might be the best unit for measuring agent cost:
   how often can I directly reuse what it gives me, and how often must
   I reinterpret it? The number of tools matters less to me than the
   semantic work between their calls. This is a possible measurement,
   not a new API proposal.
3. The unknown-candidate count could become a reason to investigate a
   particular item rather than abandon the whole query. I would want
   to learn whether that actually helps finish a gear task or merely
   turns search into reference-data maintenance. A offers a diagnostic
   view; this is a possibility to test through it, not a replacement
   design.

## What I left out

I did not repeat the authors' citation checks, audit their proposals,
execute a query, measure response sizes or timings, or try every
acceptance case. I did not test trade conversion, saved-name creation,
pagination during a refresh, or a real GUI. I did not infer authorship.
I used text notation to expose meaning and only the JSON shape A
actually supplied; my MCP comfort is partly a judgment about its
promised discovery output. I did not equate a listed gap with a defect
I was commissioned to repair.

The one cut I would most want reversed is **a real gloves-and-boots
session with A's conservative totals**. It bears directly on the
weakest part of my choice: whether knowing exactly what remains unknown
still leaves me able to finish an ordinary task. Another pass over
these pages would not answer that.
