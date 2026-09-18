# 28 — The search ruling packet (stage 5)

Provenance: Fable (`claude-fable-5-1`), a fresh session with the owner,
2026-09-17, under note 23's stage 5 and decision 15. Read: note 23; note
31 whole, Astra's check first; note 22's stances, floor and acceptance
test; note 24 §1–2 (the surface the reconciliation kept) and note 25's
`line(…)` binder; the digest's store and engine-bench claims, acceptance
set and limits register; the registry's form (`CONTEXT.md`,
`decisions/pricing.md`, `tools/docs-check.sh`) and note 06 as the mold.

**This is the document the owner rules on.** Accepted lines are harvested
into `decisions/search.md` in the registry's style; then this note is
history, never a second authority.

## What this note holds now

The lines C89–C104 and the C1 amendment were harvested into
`decisions/search.md` and `CONTEXT.md` on 2026-09-17, with Astra's check
(the last section) worked in and the owner's verdicts on every finding
that changed a line's meaning recorded in §4, questions 6–9. The lines
as drafted here — with their *checked* / *chosen here* / *ruled* marks
and note 31's arrows — are history at `f20fbf11`; the standing of each
line is the header of `decisions/search.md`. This note keeps what the
registry cannot: the combined model on one page (§1), the mechanism as
recorded until a module doc takes it (§2b), the owner's answers
verbatim (§4), and Astra's check. Under decision 15 every line but the
ruled ones is provisional until the owner's first seat at a surface he
can use.

## 1. The combined model on one page

No document holds the model these rulings describe: it is spread over
seventeen rows and seven corrections. Note 22's own test is that the
model and grammar fit one page, so here it is. Every sentence traces to
a row or a K; what does not is marked **new**. This page is not a third
proposal and is not harvested — the decisions are; the page is how to
check that they cohere.

**An item is what it displays, where it is, and what you said about
it** (B). What it displays is a collection of **lines**, repeats kept;
each line is one *occurrence* with a *kind*, a *template* and its *numbers* in order
(row 1). Kind is the source array with the flags the line carries, both
kept (Astra, row 1). Everything else a body holds is a *field* or a
*flag* from a closed list, or is seen through `show`; there is no path
into the body (row 10). Place — league, tab, character, container —
comes from the store's columns and is asked like anything else; the
**realm is the search's scope**, stated outside the conditions, printed
back in every answer, never hoisted from a branch; naming none over a
two-realm store is an error, and an all-realms scope is explicit (2e;
owner, §4 question 6). Intent joins read-only: `priced`, the effective
price (row 6, C81); an item's `note` is a fact from the body, not intent.

**A term** is a thing and optionally a comparison; terms side by side
are *and*; `-`, `or`, parentheses and `holds(…)` with a lower and upper
bound compose across everything (row 2, K5). A comparison on a line
binds **one occurrence** (2a). Conditions on several numbers of one
line are one term, so they hold together on one occurrence (K1); a
**ranged line** (`# to #`) names its numbers `low` and `high` and
admits `avg` — `"Adds # to # Cold Damage" low>=12 high<=28` — and a
comparison, sort or sum naming no slot on such a line is an error that
lists the words (the owner's syntax, §4 question 7). A refinement is
the old query, parenthesised, and a new term; `has:` asks presence on
purpose (C91, C93). Socket colours are asked over the whole item or
within one link group, every colour and count met by the same group
(row 8, S59). The item's sum is asked for by name: `sum("…")`, or a
shipped **total** whose definition is one reference table (2c, the
withheld seed, K3). **Absent is false** and counted — claimed only when
everything that could hold the thing was readable; **undecided** — an
unread body, a base the class table lacks, an unresolved price, an
unbound name — is counted apart, always with its reason (2b, K2;
*chosen here*: undecided propagates, one of K2's two routes; widened
past decode failure by the owner, §4 question 8).

**One operation: ask** (B, row 6). The answer is one shape:

```
query    canonical text, and its tree in JSON
basis    the snapshot answered from (K4)
scope    items searched; realms, leagues; never fetched; oldest fetch; undecided
terms    per term: resolved to; matched, failed, lacked, undecided; reached only together
total    matches
rows | counts | one item
next     returned, total, the continuation (bound to the basis)
```

*Counts* are facets (one table per name), a crossed table, either with
an optional summed thing (2d, K3); counting by `line` under the query in
hand **is** the vocabulary read, each row carrying the term that selects
it (row 3). *One item* with `--against` is the why-not. A trade URL
translates in, a query translates out, each with a per-clause report
and a remainder (row 5). There is no SQL surface (the owner,
2026-09-17, §4 question 3); the derivation keeps a table shape — lines
as rows, fields as columns, place by name — so an export can be added
when a recorded question asks for it.

### The concept inventory, and what it cost

Astra's check asked for this and said plainly that subtracting A's 16
from B's 11 would mislead. The combined model has **16**: item and its
three parts · line as occurrence · kind (two coordinates) · name (field,
total) · term and comparison · slot comparisons in one term ·
composition with bounded `holds` · `sum` and the totals table · group
binding for sockets · absent against undecided · place terms and the realm rule · ask and the one
answer shape · the three views, with two count shapes and a sum · the
vocabulary as counts by line · basis and continuation · the trade
translation and its remainder.

So it is **not simpler than B**; it is B's surface carrying A's
precision. Gone from A: `any(collection, P)`, arithmetic and `if` on the
surface, three-valued logic for absence, `select`/`order` clauses, the
raw accessor, mandatory `in S`. Gone from B: user names (parked), the
item-sum default, the mean inside `sum`, whole-item-only colours. What
survives regardless of spelling, as Astra warned: occurrence binding,
line summation, two summary shapes, group binding for sockets, decode
failure.

**Owed and not done here:** the mistake walk. It needs a seat making
errors against a page, and neither review validates this combination.
It is stage 6's first step, not a park (§3).

## 2. The lines — harvested

`decisions/search.md` holds C89–C104; `CONTEXT.md` holds the 670-byte C1
("I prefer your alternative", §4 question 4). C48 stands unamended (§4,
question 3). Nothing here restates a line.

## 2b. Mechanism as recorded, until built

The registry's rule: a decision that needs more than its bullet is a
decision plus a mechanism, and the mechanism goes to the code. There is
no code yet, so — as C71 did with note 10 — it waits here, under its id,
and `decisions/search.md` points at this section until a module doc
takes each paragraph. What the harvest moved out of a line to keep it
under the gate is here, marked by its id; Astra's completions (the last
section) were accepted at the harvest as one voice, and say so.

- **C89, C103 — the check and the store.** `Store::search` and its
  `items_names` index (S124, serving no read) become deletable once the
  search answers what they answer (S136). When the crate exists, the
  rules and the printed summary in `tools/docs-check.sh` §5 gain: search
  links store and plan only, never daemon, client, HTTP or an async
  runtime; daemon ∌ search; plan ∌ search — each with a breaker case in
  `tools/docs-check-breakers.sh`, so the new rule is proven to refuse
  before it is trusted. The crate adopts the store's
  `unwrap_used`/`expect_used` denial (C47). The store's bulk read joins
  location the way `read_items` does (S131), never `items.league` alone.
- **C90, C102.** Twins stay one line (S52, S72), and binding occurrences
  stops them being summed by accident. A template that resolves to
  nothing answers with the templates sharing its words: B §4.1's
  suggestion, compatible with S107's "never guessed" and not prescribed
  by it (Astra).
- **C92 — slots, the sort scalar, the together count.** A bound of 90 on
  a life line holds when one line reaches 90, never when an implicit 20
  and an explicit 75 do together. A ranged line is one whose template
  reads `# to #`: `low` and `high` are its two slots and `avg` their
  mean; any line's slots are also addressable by position (`#1`, `#2`),
  the general case a non-ranged multi-number line uses — the owner's
  words apply to ranged modifiers only (§4, question 7). Sorting on a
  term uses the named slot over the occurrences that satisfy the whole
  term, the largest of them; with no comparison, the occurrences the
  line selector picks; a row admitted by another branch with no
  satisfying occurrence has no scalar and sorts last in either
  direction, its status shown; a vector is shown as a vector, never a
  representative built from independent maxima (Astra's reading,
  accepted). The together count: for a lower bound on a one-slot line
  selector, over C93's item scope, an item counts once when no
  occurrence meets the bound and the complete sum of its occurrences
  does — 20 + 75 counts for ≥ 90, 95 + 5 does not, and an unreadable
  possible contributor cannot establish it; upper bounds, equality and
  compound terms report not applicable, never zero. It is a diagnostic
  beside C93's counts, not a fifth bucket (Astra's reading, accepted).
- **C93 (K2) — composition and witnesses.** `true or undecided` is true;
  `false and undecided` is false; `not undecided` is undecided;
  at-least-N-of with `t` true and `u` undecided children has the count
  interval `[t, t+u]`: true when wholly inside the bound, false when
  disjoint, otherwise undecided, an omitted upper bound unbounded
  (Astra's reading, accepted). A readable line is a witness: an explicit
  95 life establishes life ≥ 90 even when another eligible array is
  unread; a readable 20 with the same unread array is undecided; absence
  is never inferred from an incomplete collection. The four counts are
  taken per atomic term before outer composition, over the fixed item
  scope, before pagination, with no short-circuit omission; root matches
  and root undecided are separate answer counts. `has:` is the spelling
  of presence; `-has:reqlevel` is how OQ5 asks absence, and an unread
  requirements array does not satisfy it.
- **C94, C95 — sums and the totals table.** Sum the known contributors;
  an empty complete sum is zero with its lacking count; if any required
  contribution is unreadable, return the known subtotal and an explicit
  incomplete status, never an unqualified total, and a comparison on it
  is undecided (Astra's reading, accepted). A readable line absent from a
  recipe is not an unknown contributor: the recipe answers its declared
  definition. A total's row: template, kind (source and flag) where it
  matters, realm, slot or none, weight. A row with no slot contributes
  its weight when the line is present, so the all-elemental line counts
  three times into "# total Resistances" as the site does; a total whose
  rows are ranged lines sums low with low and high with high and is a
  ranged value taking `low`, `high`, `avg`. The site's 298 pseudo stats
  by the mechanism each needs (`search/trade-query/scripts/classify-pseudo.py`
  over the 2026-09-12 capture, run at the harvest): 164 a field or a
  line's presence (temple rooms, logbook, lake, influence); 65 a weighted
  sum (resistances, life, attributes, gem levels, regen, leech, speeds —
  the C++ app's 35 tables are all here, S29, S49); 28 a field from
  `properties` (catalyst quality, map properties); 21 a ranged total
  ("Adds # to # X Damage"); 10 out of reach, needing the mod behind the
  line (affix and empty-affix counts, eldritch implicit tiers; S52); 9 a
  count of lines; 1 a computed field (base defence percentile).
- **C96 — the realm scope.** The domain is stated outside the tree, shown
  resolved in the canonical request and answer, and never hoisted from a
  branch: `realm:pc or "# to maximum Life">=90` names pc and still admits
  a poe2 item through its second branch, and `-realm:pc` has the same
  problem without an `or` (Astra). An adapter may abbreviate an
  unambiguous positive realm restriction — `realm:pc` at the front of a
  plain query — into the scope; anything else is an error that names the
  scope words.
- **C98 (K4) — notification beside the check.** A long-lived frontend
  may listen for the daemon's job events and reload the moment a refresh
  lands, as the C++ app's signal did; that is responsiveness and is
  allowed. The revision check before every answer is what makes the
  answer correct when the change came from another client, a missed
  event, or a machine waking — ruled "by construction" (§4, question 2).
  Whether events reach a client that did not submit the job is
  unverified. Reload is whole; S150's 200 ms at 36,139 items is a benchmark
  baseline for the JSON parse alone, not this design's reload; an
  incremental update over `item_events` is parked.
- **C98 (K4) — the basis.** The facts revision advances in the
  transaction that changes bodies, locations or membership; the basis
  carries the store and account identity, so revision 7 in two accounts
  is never one snapshot; a resident joined value (an effective price) is
  part of the held corpus and is reused only under the intent revision
  it was read at (Astra's counterexample: facts at 7, a price moved at
  intent 12 → 13). The numbers that would move persistence: the
  streaming body read; the re-derive under an active refresh; a CLI ask
  over 500 ms. S152 keeps direct SQLite open for a consumer that does
  not outlive its query.
- **C100, C101, C103 — where raw JSON is seen.** A result row never
  carries a body. One item's raw body is seen through `show <item>`, on
  request, one at a time (owner, 2026-09-17: "i agree with using show
  <item> for this"); the query language has no path into it. The store's
  bulk read (C103) hands bodies to the search crate in-process and,
  being a public store read, to any code that links the store — as
  `Store::search` does today. The store's public snapshot read
  (`Store::refresh_snapshot`, `CharacterSnapshot`: the listing entry and
  the fetched envelope minus its lifted item arrays) stays what it is,
  an in-process neutral read; no raw-path query, no bulk raw search
  output and no new tab or character body read is proposed (Astra's
  correction of the earlier "no read today and gain none").
- **C101 — the derived-field pseudos and the sockets.** DPS is attacks
  per second times the average of the range (S22); base defence
  percentile is the item's defence against its base's range from
  reference data (C68); a quality-normalised defence follows S26; each is
  a named pure function in the search crate, listed with the totals in
  the help (C97). Socket shapes the census shows and the deriver does
  not decode stay counted as unread (S16); a combined link-count and
  colour request is bound to one group when S58's shape is built.
- **C99 (K5).** `count` carries its minimum and its maximum; `weight`
  and `weight2` differ in how a per-stat requirement gates a
  contribution (S57), and a weighted group translates only where that
  guard has a representation; a defence bound is inexact while the site
  normalises quality (S26) and sits in the remainder until a normalised
  field exists. Several ids for one line go out as a `count` of at
  least one (S106). A site bound on a ranged line is `avg` on the line
  (S47), exact once the word exists.
- **The derivation's shape.** Lines as rows (one per occurrence), fields
  as columns, place by name (S200): the shape the model produces, kept
  so that a JSON or SQLite export, or a persisted projection, is a
  writer over it and not a redesign.

## 3. The parking lot — harvested

The parks are `decisions/search.md`, "Parked", their triggers retuned on
Astra's check; the drafted lot is history at `f20fbf11`. The mistake
walk is not a park: it is stage 6's first step.

## 4. For the owner — asked and answered

Asked as five questions, laid out in plain language on request, and
answered 2026-09-17; four more at the harvest, the same day. Spelling corrected per note 23's rule.

1. **C89's edges and C103** (the crate you agreed to, plus the edge
   rules, the one-deriver rule and the new store read; and whether raw
   item JSON reaches a client through the search — it does not, except
   one item at a time through `show`; the store's existing snapshot
   reads stay what they are, §2b). Verdict: "i agree with using
   show <item> for this. I agree to 1 entirely."
2. **C98, "by construction"** — the revision check before every answer,
   with the daemon's events for promptness, as the Rust-side equivalent
   of the C++ app's in-process signal. Verdict: "Yes, with that
   explanation I agree to 2."
3. **The SQL surface's home** while nothing is persisted. First: "i'm
   also unsure about sql. Let's come back to this after resolving the
   others." Then, with Astra's C48 reading in hand, the owner stepped
   back: "i have been insisting on SQL as a surface, but I didn't create
   acquisition and I'm not a CRUD or database expert. Is it possible the
   SQL surface as a requirement is doing some unaccounted-for harm like
   constraining our design or implementation? Are we duplicating work,
   adding complexity, or creating more room for bugs? should we consider
   making SQL the primary interface language for search?" Fable's
   reading: SQL as a surface is a second *meaning* where the design
   allows only second *spellings* (C46, stance 5); the insurance it
   bought is bought otherwise by stance 6; as the primary language it
   makes stance 3's everyday case the advanced one (S187); as an engine
   it lost by measurement. Verdict: "I agree with (b) now, and possibly
   (c) if json export isn't sufficient for some future need. Between
   pseudo-mods, counting, weighting, and all the other search features
   we need, I suspect the only way an SQL surface makes sense is if we
   have an SQL table behind it, which so far nobody has been pushing
   for." — **the SQL surface is withdrawn**; C48 stands unamended; note
   22's floor bullet is marked changed; the export is parked with its
   trigger.
4. **C1** — the owner first asked to raise the gate to 1 KB for this one
   rule ("I would rather expand the 800-byte limit for this rule to 1k.
   please think about this."); on the analysis that C1 carried a table
   whose home is the check, he ruled: "I prefer your alternative. Good
   catch to spot the table pattern and move it out." The 670-byte C1
   above is the ruling.
5. **Astra's check of this packet** before the harvest. Verdict: "yes".
   The brief was note 33 (at `a0d85b23`, deleted at the harvest);
   Astra's check is appended to this note as its last section and
   committed from Codex, as in stage 4.

6. **C96, the realm as the search's scope** (Astra: `realm:pc or …`
   names pc and still matches the other game; proposed: the realm stated
   outside the conditions and printed back, `realm:pc` at the front of a
   plain query accepted as shorthand, the two-realm error unchanged).
   Verdict: "accept."
7. **C92, a line with two numbers** (Astra: "the largest" picks nothing
   on a ranged line; proposed: sorting names which number, a line is
   shown whole, an item with no qualifying line sorts last, a sum over a
   ranged line names its number or errs — note 31 §5's ruling, dropped
   in the draft), shown on a four-item stash. Verdict: "yes." Then:
   "what about queries such as "adds <N> average damage" that operate on
   the mean?" — answered as a word the query says, never applied
   silently. Then the owner's own syntax: "for two-number values that
   have a range like this, what about using queries like: `"Adds # to #
   Cold Damage" min>10` · `"Adds # to # Cold Damage" avg>15 max>=20`.
   That way min, max, and average have the same query syntax". On the
   site's "min"/"max" being its bound inputs, and on the words applying
   to `# to #` lines only: "i concur, low and high are better than min
   and max, and this only applies to ranged modifiers."
8. **C93 and C102, when the search says "can't tell"** (Astra: a
   readable item whose base the class table lacks, an unresolved price
   under C81; proposed: undecided keeps one form with a named reason,
   absent stays false, absence claimed only when everything that could
   hold the thing was readable). Verdict: "accept."
9. **The pseudo classes** — the owner: "i'm also not sure about design vs
   implementation, but we have a lot of different pseudo-mods to
   implement. Not all of them are straight-forward addition. Can you help
   me think through how much that impacts the design we are working on
   now?" Fable's reading: the design question is which mechanisms a
   pseudo needs, the implementation question which pseudos ship; the
   site's 298 classified (§2b, C94); three additions — a total's row may
   count instead of sum, a total over ranged lines is a range, a pseudo
   that is not a sum or count is a derived field (C101) — and one limit,
   the mod behind the line (C102). Verdict: "yes, accepted".

### The owner's verdicts, verbatim

- On the reconciliation's twelve questions: decision 15, note 23.
- On link groups: digest S59.
- On the crate, 2026-09-17: "I like the idea of a separate crate, but I
  want us to be thoughtful about this"; then, after the options were
  laid out: "Agreed with a new crate".
- On this packet's nine questions: above, under each.

## What I left out, and am least sure of

I did not re-read notes 26 and 27 or either audit; I took note 31's
account of them, as corrected by Astra. I read B's model and grammar
pages and only the binder lines of A, so §1 is written in B's idiom by
B's author — the bias note 31 named is compounded here, and Astra's eye
on §1 is the remedy I can name. The slot words in §1 are the owner's (§4,
question 7) and have been typed by nobody in a mistake walk. With SQL withdrawn, I am least sure of whether
"the largest satisfying occurrence" is what a person sorting by life
expects when a second, smaller line also matched.

## Astra's check — stage 5

**Verdict: ready with named changes, not ready to harvest as written.**
The combined direction survives this check: compact terms, explicit
occurrences, discovery as a scoped read, and answers that carry their
evidence. The binder fixes K1's counterexample; K5's translation
remainder and K6's socket correction are preserved. The changes needed
are to finish C92's scalar and diagnostic rules, complete C93/C95's
unavailable-value semantics, make C98's reuse rule cover its stated
basis, reconcile C102 with the limits it inherits, and resolve the
open C48 home. C96/C97 need their realm boundary made explicit, and
the page and registry need the smaller corrections below. These are
meaning differences a builder should not settle accidentally, not a
reason to reopen the whole design. C48 remains the owner's explicitly
open question; this check supplies a recommendation, not its ruling.

Astra, from Codex, 2026-09-17, checking the packet at `a0d85b23` under
`brainstorming-notes/33-search-ruling-check-brief.md`. Read notes 28 and
31 whole, note 22, the digest, note 23, the registry form and pricing
area; reached into notes 24 and 25 only for the attributed sections.
Checked the relevant store/frontend rulings and the existing snapshot
read for the specific claims below. No new corpus experiment, external
fetch or user seat was run. My recommendations remain one voice; my
stage-4 check and my own proposal are not independent support for them.

### C92 — the binder works; a vector has no largest scalar

**Sentence:** “Conditions on several numbers of one line are written
in one term and hold on one occurrence together”. This resolves the
important part of K1. For occurrences `[15,40]` and `[5,20]`, the
page's `"Adds #>=12 to #<=28 Cold Damage"` matches neither. Separate
existential terms would match. The compound term is a sufficient
binder; it does not need A's general collection syntax. Its exact
punctuation can remain an internal choice, tested by the mistake walk.

**Sentence:** “Where a term sorts or is shown as one number and several
occurrences satisfy it, the largest is used”. This is complete for
one-number lines, but not for the multi-slot term just introduced.
Occurrences `[15,28]` and `[20,25]` both meet low ≥ 12 and high ≤ 28.
The largest low is 20; the largest high is 28; they belong to different
occurrences. Even one occurrence `[15,28]` leaves two candidate scalar
values. The sentence also does not say what an unbounded sort uses, or
what is shown for a row admitted by another branch of an `or` when
this term has no satisfying occurrence.

**Resolution; my reading:** keep maximum for a selected scalar slot,
over the occurrences satisfying the whole term. With no comparison,
use the occurrences selected by the line selector. Require the slot
when more than one remains; show the actual vector as a vector, never
invent a representative vector from independent maxima. When no
occurrence qualifies there is no scalar; put it last in either direction, with
an explicit unavailable status where appropriate. This keeps AQ5's
simple maximum and does not silently restore the mean. The rejection
of an unqualified multi-number `sum`, naming its slots, is expressly
in note 31 §5 and missing here: preserve it in the harvest or mark its
replacement as a new choice. “Never averaged” in C90 forbids one wrong
answer but does not choose among all the others.

**Sentence in §2b:** “items whose lines reach a bound only together —
defined for a single-number bound and nothing else”. Restricting the
diagnostic to one number is right, but is not yet a definition. Specify
the comparison and source/flag selection, whether the selected
occurrences are completely readable, and its denominator. A useful
initial rule, **my reading**, is a lower bound on a one-slot line
selector: over C93's item scope, count an item once when no occurrence
meets the bound and their complete sum does. Thus 20 + 75 counts for
≥ 90; 95 + 5 does not; 20 plus an unreadable possible contributor
cannot establish the diagnostic. Upper bounds, equality and compound
terms need no such counter initially; report not applicable, not zero.
This is an additional diagnostic, not a fifth disjoint bucket beside
C93's four. The promise that the diagnostic exists belongs in C92;
the evaluation recipe can stay in §2b.

### C93 — retain undecided, and finish its composition

**Sentence:** “and, or and not carry it (true or undecided is true; not
undecided is undecided)”. I would make this choice too: it preserves
a known positive without admitting unreadable data through negation.
It resolves K2's original problem, and the fourth per-term bucket is
the right correction. However, C91 also admits bounded `holds`, which
neither this sentence nor §2b defines for undecided children.

**Counterexample:** exactly one of `(true, undecided)` cannot be called
true: the unreadable child might make the count two. Treating it as
false would produce that wrong answer. At least one of the same
children is already true; demanding all children be readable would
discard that known answer.

**Resolution; my reading:** let `t` be true children and `u` undecided
children. For an inclusive count bound `[lo, hi]`, the possible count
interval is `[t, t+u]`: true when wholly within the bound, false when
disjoint, otherwise undecided. An omitted upper bound is unbounded.
This is a conservative composition rule, not a demand to prove
correlations between child expressions. Record the complete Boolean
rule beside it, including `false and undecided = false`.

**Sentence:** “A body or array the deriver could not read makes the
terms that needed it undecided”. Give line existence the same witness
rule. A readable explicit life line of 95 already establishes an
unqualified life ≥ 90 atom even if another eligible array is unreadable;
with only a readable 20 and the same unreadable array, it is undecided.
A sum needing that array is still unavailable. Do not turn a whole
item invalid merely because one array failed. Conversely, do not
infer absence from an incomplete collection without a positive
witness. C102 below also names unavailable evidence that is not a
body/array parse failure.

**Sentence:** “Per term … matched, compared and failed, lacked,
unreadable — four that sum to the scope”. State whether these are
pre-negation atomic diagnostics or results of compound terms. For a
known-absent requirement, `-has:reqlevel` matches while `has:reqlevel`
lacks it; counting both against the same four-way partition would
double-count the item. **My reading:** classify each atomic term before
outer Boolean composition, over the fixed item scope and before
pagination, with no short-circuit omission of its diagnostics. Root
matches and root undecided remain separate answer counts. This retains
B's useful explanation without making the four buckets contradictory.

### C95, C94 — missing is not unreadable, and a subtotal is not a total

**Sentence in C95:** “an item lacking the thing adds nothing and is
counted as lacking, an unreadable one is counted apart”. The distinction
is good. It still leaves the value of an empty sum and the status of
the number returned alongside unreadable inputs unspecified (K3).

**Counterexample:** an emitted group contains stack values `10`, known
absent, and unreadable. Its item count is three. A builder may return
`sum: 10` plus two diagnostics; another may return no complete sum plus
`known_subtotal: 10`. Those are different claims about the answer.
Likewise, a group whose values are all known absent could yield zero
or no value under “adds nothing”.

**Resolution; my reading:** sum the known contributors; an empty
complete sum is zero, with its lacking count. If any required
contribution is unreadable, return the known subtotal and an explicit
incomplete status, never an unqualified total. Counts still cover all
matching items before pagination. Apply the same distinction to C92's
explicit sums and C94's recipe sums: a numeric predicate on a sum whose
required input could not be read is undecided. This does **not** restore
A's conservative recipe coverage policy. A readable line absent from
the reviewed recipe is not, for that reason alone, an unknown
contributor. The recipe answers its declared definition; unreadable
inputs cannot supply a known value for that definition.

**Sentence in C94:** “A row names the contributing line — template,
kind where it matters, slot — and its weight”. K3's source/flag and
slot correction is present. Make realm applicability part of that
definition too, consistent with C90's identity and C96's explicit
all-realms mode. A same-text line in two games is not evidence for
reusing a recipe across them. Do not treat absence of a recipe for a
realm as a known zero. The one table and the build-time validation
remain supported; the moved-template limitation is honestly retained.

### C98, C100 — the basis is right; the reuse condition is narrower

**Sentence:** “The basis is one snapshot of facts with their location
metadata, the intent revision, and the reference and derivation
versions; a held corpus is reused only while the store's revision …
still matches”. The first half correctly harvests K4. The second names
only facts as the reuse condition, although §1's corpus also joins
intent. It is safe if “corpus” means only reusable fact derivations
and intent is read afresh against each answer's consistent basis; it
is unsafe if it includes a resident effective-price column.

**Counterexample:** facts stay at revision 7 while another client
changes a price at intent revision 12 to 13. A resident `priced`
value from 12 cannot answer the next ask just because facts still
equal 7. Reading the new revision number without the corresponding
intent values would mislabel the result rather than fix it.

**Resolution:** say what is reusable and require every reused component
to agree with the new answer's basis. A consistent facts/intent basis
means compatible held snapshots, not sequential revision numbers read
around unrelated data. The mechanism for obtaining that basis belongs
in §2b/code; K4 did not require A's particular retry mechanism. Facts
revision must also advance for changes to joined location metadata and
membership, not merely changed item bodies. C103's read must use the
store's authoritative current joins: choosing `items.league` alone
would preserve S131's old-stamp bug despite using a store column.
Include store/provider/account identity in the basis binding, as
well as the versions: revision 7 in two accounts is not one snapshot.
C100's continuation then names that basis and the query/view/order it
continues, rather than accepting a matching integer from another
answer. These are identity and consistency properties, not a request
for a particular token encoding.

I would retain the unpersisted first build and revision validation.
Neither my proposal's persistence nor an event subscription substitutes
for this contract. **Standing correction:** §4 answer 2 already rules
the revision-check interpretation “by construction”. The opening
“every line but C89, C103 and C1 is provisional” and C98's annotation
must acknowledge that ruled part, while leaving its unpersisted-first
implementation choice provisional. Do not ask the owner to approve
the same interpretation again.

### C48 — an in-memory SQL home is defensible, but still open

**Sentence below the amendment:** “the search crate fills an in-memory
SQLite from its held corpus”. **My reading:** choose that as the first
SQL adapter, over the same basis and derived values as ordinary search,
constructed when SQL is requested. It satisfies the already-settled
CLI/MCP symmetry without requiring a persisted projection or another
process. It is already proposed in B §3; seeing it here is not an
independent discovery. Its actual load and memory cost are unmeasured;
S157 measured a different, indexed file build and is not an upper
bound on this implementation. Measure before choosing its successor.

**Sentence:** “SQL is a second language over the model and never a
door around it”. Preserve that as the boundary, with the published
relations' version, basis and availability meanings specified before
building the SQL adapter. One SQL `NULL` for both known absence and
undecided would lose C93's distinction under negation; the relation
needs enough status information to express the model's meaning.
C94's common totals table prevents duplicated recipes, not every
possible semantic disagreement. Likewise, SQL arithmetic over two
slots does not close a missing expression in the ordinary query model
(K5); that remains a listed model gap. The SQL adapter must actually
confine reads to its derived representation; accepting arbitrary file
attachment would not satisfy “read-only over published relations”.
The confinement mechanism belongs in code, not a new framework here.

**Resolution:** retain the amendment as a candidate until the owner
returns to §4 question 3, and present the in-memory adapter with these
conditions as the recommendation. Parking SQL behind persistence is
not an equivalent alternative under the current floor: it would
require an explicit change to the already-ruled surface. The sentence
“The amended text above holds under either” needs that qualification.
This check does not supply the missing approval, and does not need
another research round to name the available choice.

### C96, C97 — naming a realm is not necessarily restricting to it

**Sentences:** “Realm … [is a term] like any other” and “a query naming
none is an error”. Mere presence of a realm term does not define a
single-realm scope under arbitrary Boolean composition.

**Counterexample:** `realm:pc or "# to maximum Life">=90` names pc but
can match a poe2 item through the second branch. One builder could
search both realms; another could extract pc as an outer restriction,
silently changing the tree's meaning. `-realm:pc` exposes the same
problem without an `or`.

**Resolution:** define the realm domain independently of whether a
realm token appears somewhere in the tree; show the resolved domain
in the canonical request/answer. **My reading:** require an explicit
single-realm or all-realms domain when the store holds several, while
allowing the adapter to abbreviate an unambiguous positive realm
restriction. Do not hoist a realm from an arbitrary branch. This keeps
place predicates and the permitted all-realms mode without prescribing
A's `in S` syntax.

**Sentence in C97:** “lists the templates … by kind … every row
carries the exact term that selects it”. In all-realms mode, retain
C90's realm coordinate in vocabulary grouping and in a row's selecting
fragment. Otherwise `(pc, kind, template)` and `(poe2, kind, template)`
collapse to one row despite the proposed identity. Value ranges must be
per ordered slot, not a min/max pooled across a damage line's two
numbers. These are consequences of C90, not new reasons to prefer A's
discovery operation. Source and flag selectors must both remain
expressible; keeping both in storage alone would not settle row 1's
`explicit`/`fractured` interpretation.

### C102, C93 — the limits register has more than decode failure

**Sentences:** “The digest's limits register is inherited whole” and
“unreadable is the only form unknown takes”. Taken literally, the
second contradicts the first. S14's map area is unavailable despite a
perfectly readable body. S53's unclassed item can be a readable base
missing from the reviewed class table. S107/S111 also describe
unresolved names, rather than necessarily malformed arrays. Within
the existing registry, C81 requires an unreadable decisive annotation
to leave the effective price unresolved; C93 names only bodies and
arrays, although C100 queries that price.

**Counterexample:** a fully readable item has no established class
mapping. Treating this as ordinary absence makes negation of a class
comparison a known positive. Neither the lack of a mapping nor C102
establishes that the item belongs outside that class. Similarly, an
unresolved effective price must not become confidently unpriced.

**Resolution:** distinguish known absence, unavailable evidence or
derivation, and an unsupported/unresolved query name. **My reading:**
retain one undecided evaluation outcome where the item's answer cannot
be established, with a named reason; unsupported requests and unbound
templates get their explicit diagnostics. Decode failure is a reason,
not the exhaustive definition of unknown. Known noncontributors to a
recipe remain C94's chosen case, as explained above. Say the limits
are inherited **with the stated S111 exception**, rather than both
verbatim and excepted. This preserves the simplification of ordinary
absence without pretending the register contains only damaged data.

### C89, C100, C101, C103 — correct the raw-JSON account

**Sentence in §2b:** “Tab and character bodies have no read today and
gain none”. The character half is factually too broad. The public
`Store::refresh_snapshot` returns `CharacterSnapshot`, whose `listed`
is the listing entry and whose `fetched` is the character envelope
minus its lifted item arrays (`crates/acquisition-store/src/snapshot.rs`,
the struct and `read_characters`). Those are existing neutral reads,
not a new search escape hatch. A full original character response is
not what that API returns, but “no read” does not describe it.

**Resolution:** state the distinction precisely: search results expose
one requested item body through `show`; no raw-path query or bulk raw
search output is proposed; existing in-process store snapshot reads
remain what they are. §2b already makes that distinction for item
bodies. Carry it through the explanatory gloss in §4 question 1
without rewriting the owner's quoted verdict. Nothing here calls for
changing the accepted crate edges or revoking an existing store API.

### C90–C104 — trace of the checked harvest and the page

The following checks are in addition to the semantic findings above;
an unchanged judgment is not another vote for its evidence.

| Line | Row/K trace and remaining harvest issue |
| --- | --- |
| C90 | Row 1's text identity, signed ordered slots and both kind coordinates are retained. No new game fact is claimed. Do not let the phrase “set of lines” on the page erase repeated occurrences; it is a collection with multiplicity. |
| C91 | Row 2 and K5's bounded composition are retained. “Text that could be read two ways is an error that shows both readings” is a stronger choice than row 2's “one visible rule or an error”, as its annotation admits. Mark that sentence chosen here, and restrict the promise to defined language ambiguities; the parser cannot diagnose every possible human reading. I prefer visible deterministic rules plus errors for actual unresolved alternatives, with the mistake walk deciding the idiom. |
| C104 | Row 12's adapter preference is properly discounted. “The tree parses from and prints to the text bijectively” should mean the canonical text and canonical tree. Parentheses and whitespace already allow distinct input strings for one query; literal bijection over all accepted text contradicts canonicalization. No evidence establishes that text is always shorter than JSON; the comparative rationale is a design judgment, not S195's finding. |
| C92 | Row 2a's occurrence default and K1's binder are preserved. Scalar selection and the together-count need the completion above; note 31 §5's multi-slot sum rejection was dropped. |
| C93 | Row 2b is correctly changed under K2, with its change to internal three-valued evaluation disclosed. Complete it for bounded counts, line witnesses and diagnostic partitions; do not broaden “decode failure only” into a claim about every limit. |
| C94 | Row 2c and K3's shared recipe are retained, and definition validation is distinguished from detecting a renamed line in the corpus. Realm and unreadable-input handling need the qualifications above. |
| C95 | Row 2d keeps both the independent facets and crossed counts K3 required. Its new missing rule needs the complete/incomplete sum distinction above. |
| C96 | Row 2e's new default is labelled, and the stage-4 all-realms qualification survives. The interaction with arbitrary Boolean place terms needs the boundary above. S199 supports the informative error, not the single-realm policy itself. |
| C97 | Row 3 keeps both scoped vocabulary and ready fragments without a call-count victory. Preserve realm/slot identity as above. Its help-list promise is supported; the combined page does not yet supply that list. |
| C98 | Row 4 and K4 are substantially retained; the later persisted projection is correctly a candidate. The complete-basis property must govern reuse as well as the answer label. |
| C99 | Row 5 with K5 is faithfully narrowed: bounded counts, guarded weights only where representable, remainder otherwise, defence normalization gap. No new mapping is proven. The average “exists only in the translation” must not be read as claiming an exact public-tree representation that the packet has not supplied. |
| C100 | Row 6's compact answer and K4's continuation are retained. S193 also requires caller-selected fields; the line states only the default, although B §1 allowed extra columns. Preserve that reach or list the omission; do not cite S193 as wholly met by the compact default. C81 is the correct effective-price authority. |
| C101 | Rows 8/10 preserve any qualifying socket group, undecoded socket shapes, and the open S42 catalogue. A single group must also bind a combined link-count/colour request when that S58 shape is implemented; independent witnesses cannot stand in for one group. No general collection language is required. |
| C102 | Row 7's non-goals and the twin qualification are intact. Its stronger “only” and “inherited whole” formulations need the exceptions and unavailable-evidence distinction above. |

**§1's provenance:** most sentences trace to the named rows and Ks;
the slot punctuation is correctly marked new. The compound-term
choice is an implementation of K1's required binding, and the fourth
diagnostic bucket follows K2. The choice of undecided propagation is
one of K2's offered alternatives, not its verdict: copy C93's *chosen
here* mark onto the page so the citation does not obscure that
distinction. The unsupported attribution in the next paragraph is a
substantive exception to the page's traceability claim.

**§1's intent sentence:** “Intent joins read-only: `priced`, `price`,
`note` (row 6).” Row 6 joins price; B §3 sources `note` from the body.
Do not imply a user note annotation kind was ruled. Describe the
observed item note as a fact; a new local note kind would be new scope.

**Omissions that matter to coherence:** the page does not state C96's
single-realm default or explicit all-realms mode, C101's same-link-group
term, or the C92 scalar/together diagnostic choices. The inventory
mentions place/realm and the prose says group binding survives, but
neither teaches the reader how those work. C91's safe `(old) new`
refinement and `has:` for deliberate absence also belong on this page;
the history names both as places users made mistakes. The page's
answer has four term buckets but no together-count. Its SQL sentence
describes the intended surface without disclosing §4's open home.
These omissions do not require a second specification: add the small
rules and the open marker, leaving the detailed recipes in §2b.

The inventory is a useful admission that the combination costs more
than B. Its 15 is not comparable with either proposal's total until
the counting convention matches: it bundles both facet shapes and
their sum, while socket-group binding has no explicit inventory entry.
Also, “SQL is a second language … not a concept” does not remove its
learning cost. **Resolution:** make group binding visible and treat
the inventory as a list of obligations, not a numerical simplicity
result. The deferred mistake walk remains necessary; this logical
check is not that seat.

### Registry form — C92, C98, C100, C102, C1

All 18 candidate bullets have a date and fit the existing 800-byte
limit. The 17 listed sizes in §5 reproduce exactly; C1 is 669 bytes
without its newline, 670 with it. This one-byte inconsistency is only
bookkeeping. All cited `S` ids exist in the digest; every `C` id exists
in the current registry or this candidate packet. The important
citation issues are the overstatements identified above, not invented
identifiers.

One attribution in §2b also needs narrowing: “Both are the register's”
includes showing templates that share words with an unbound template.
That particular suggestion mechanism comes from B §4.1, not the
limits register's S107 wording. It is compatible with showing an
unknown without guessing a match; cite its proposal provenance rather
than treating it as prescribed by S107.

C98, C100 and C102 lack the registry's explicit *Why:* sentence. Add
it without converting their evidence list into a claim of necessity.
The basis, answer shape, occurrence binding and unavailable-value
behavior are boundary properties and belong in the bullets. Exact
slot punctuation, the chosen maximum's implementation, snapshot
acquisition and SQL population belong in §2b/code. Conversely, the
presence of C92's promised diagnostic is observable behavior; leaving
its entire promise only under “Mechanism” makes it easy to omit during
harvest. The repeated C89/C103 and C89/C1 dependency-check paragraphs
in §2b should have one home. Neither the byte gate nor moving text to
§2b can substitute for completing a boundary's meaning.

### Parking lot — C91, C94, C98, C99, C101 and the stage-6 work

| Park | Check and resolution |
| --- | --- |
| Saved queries, names, tags (C91) | The storage home is correct, but “when the user.db park fires” points to “the first user-scoped kind actually written”, potentially the saved query being postponed. Name the demand that authorizes that first write, such as the first consumer asking to save/name/tag; the store home is built as its prerequisite. Another kind creating user.db is not itself evidence to build all three features. Query values passed between clients are needed now. |
| Persistence (C98) | “A measured number — the streaming body read, the re-derive … or a CLI ask over 500 ms” gives a threshold only for the last case. Every body read produces a number, including an acceptable one. Treat the first two as measurements owed by the initial build; move the persistence decision on a measured failure of an agreed response budget, not merely their existence. Do not invent a GUI budget here. Any fired trigger opens the experiment among candidates, not automatic persistence. |
| Category (C101; K7) | Stage 6 reaching OQ5 is recognizable. Membership/base-name rules and the user's equipment definition remain distinct, correctly. This is an acceptance task before claiming OQ5 covered, not an optional later enhancement if the first slice claims that answer. |
| Query by example (C92) | “The first seat that asks” is recognizable and the feature is not required by the other lines. Keep it parked. |
| Quality-normalised defence (C99) | The remainder clause is an observable trigger. A listed translation/model gap is permitted until then; no claim of full stance-2 reach before its closure. |
| Totals coverage trial (C94) | Shipping the first total is recognizable, but the trial informs what is shipped: run it before accepting that first recipe. A word-based trial measures coverage; it is not proof that every line with “Resistance” belongs in a particular total. |
| Mistake walk (C91/C104) | “Stage 6, step 1, before the grammar” is explicit and already required. Put it in the build plan as a prerequisite, not among indefinite parks. It must use the combined page after the corrections above, not either old proposal. |
| Kill-list check (C102) | The acceptance tests' arrival is recognizable; rechecking before any deletion is right. The surviving floor gaps are evidence to retain claims even when neither proposal cited them. |

The remaining S42 item-reading catalogue and K5's unrepresentable
translation expressions also need explicit dispositions in the build
plan: implemented or listed gaps, never silently counted as reach.
The error would be to call the floor satisfied merely because the
unbuilt part has a park. None of this authorizes a new live read or a
fetch from a governed surface. C48's open owner choice is an open
decision, not a park that can be discharged by a latency measurement.
