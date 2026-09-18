# 34 — The stage-5 audit's contract gaps, laid out for the owner (session B)

Written by Fable at the close of session A, 2026-09-18, so that session B
opens on answers rather than on re-reading. The audit is
`audits/search-stage-5-audit-results.md` (its fate table names these as
findings 2, 3, 4, 7 and 6). Each question is the case, a small example,
what changes, and a recommendation; the owner answers in a line under
**Verdict:**, in this file or in chat, and session B harvests the answers
into `decisions/search.md` and the contract detail with the owner's
words beside each. Deleted at session B's close, cited by hash.

Standing: the rulings these touch are provisional under note 23,
decision 15; an answer here is a ruling in the same sense.

## 1. Stance 6 still says SQL (finding 2)

**The case.** Note 22's "Headline stances", stance 6, still reads "SQL
is a second language on the surface … The MCP carries the SQL tool as
the CLI does", while the floor section of the same note records the
withdrawal (2026-09-17, decision 16). C101 and C102 cite the stances,
so the framing has to say one thing. This is your framing language;
the old text stays in history at `6b3b4e49`.

**Proposed replacement for stance 6, verbatim:**

> *Reach is a property of the derivation, not the language. What the
> derivation carries, every adapter over it can ask, and a file opened
> outside the store is the only bypass. A question the model cannot ask
> in one query is a gap in the model, listed as a limit and closed by a
> field or a total (C101, C94), never routed around it.* The SQL surface
> this stance once named was withdrawn 2026-09-17 (the floor below;
> note 23, decision 16); the earlier text is note 22 at `6b3b4e49`.

**Verdict:**

## 2. Counting by a key an item does not have (finding 3)

**The case.** C95 says what a *summed* value does when it is missing or
unreadable, but not what a *grouping key* does. Count matching items by
class, and one matching item is a readable base the class table does not
know. Does it land in an "unclassed" bucket, an undecided bucket, or
drop out with a diagnostic? Crossed tables make it worse: a row for
every reason times a column for every reason.

**Example.** Ten rare items match; count by class:

```
armour     6
weapon     3
undecided  1    class: base "Runic Crown" not in the class table
total     10
```

The buckets add up to the answer's total, so nothing vanishes, and the
one item says why it is apart. In a crossed table (class × league) the
same item is one row `undecided × Settlers`, again with its reason.

**What changes.** C95 gains one sentence: an item whose grouping key
cannot be established is counted in one undecided bucket named by its
reason (the form C93 gives every "can't tell"), never omitted and never
merged with a real value; the buckets always sum to the total, and the
first counts test pins that. Recommendation: accept; it is the only
rule that keeps C93 and the total consistent.

**Verdict:**

## 3. A vocabulary row from the other game (finding 4)

**The case.** Under an all-realms scope, the vocabulary read lists
`"+# to maximum Life"` twice — once for pc, once for poe2 — because a
line's identity carries its realm (C90). C97 promises that every row's
pasted fragment selects exactly that row. But C96 moved the realm out of
the conditions and into the scope, so what does the poe2 row's fragment
look like, and how does it stay composable?

**Example.** The kind word already sits inside a term and names the
line's coordinate: `explicit "+# to maximum Life"`. A realm word inside a
term would do the same for the line's realm coordinate:

```
poe2 "+# to maximum Life" >= 90        selects the poe2 row's lines
poe2 explicit "+# to maximum Life"     both coordinates
```

Under a single-realm scope the word is redundant, and if it contradicts
the scope (`poe2 "…"` under a pc scope) the query is an error naming
both, never a silent empty answer.

**What changes.** C96 or C97 gains one sentence: a realm word inside a
term names the line's realm coordinate (C90), distinct from the search's
scope; it selects under an all-realms scope, is redundant under a
matching single-realm scope, and is an error under a contradicting
one. Recommendation: accept; it is the smallest rule that closes the
gap without reopening C96.

**Verdict:**

## 4. Reference data against game knowledge (finding 7)

**The case.** C102 says "base defences, and which categories count as
equipment, are game knowledge and the user's (stance 4)", while the
contract detail prescribes a base-defence percentile computed from
reference data. Both can stand only if the line between them is drawn.
Session A drew it in data: the RePoE export you cloned carries armour,
evasion and energy-shield ranges on 466 bases (`search/repoe/`, the
`base-defences.csv` extract), so the ranges are game *data*, exported
from the game files and reviewable under C68.

**Proposed distinction.** Game *data* — what the game files state and a
registered surface exports (base ranges, class membership, the mod
table) — may back a derived field or a total when it is reviewed,
committed and cited under C68. Game *judgment* — what counts as
equipment for you, what is good, what is legacy, what is obtainable —
is the user's and is never embedded (stance 4, decision 7). The
base-defence percentile is then an authorized derived field over C68
data; "which categories count as equipment" stays yours.

**What changes.** C102's clause becomes "base ranges and class tables
are game data under C68; which categories count as equipment, and what
is good or legacy, is the user's (stance 4)". Recommendation: accept.

**Verdict:**

## 5. Where the binding detail lives (finding 6)

**The case.** The rules that decide answers — the sort scalar, truth
propagation, the diagnostic denominators, incomplete sums — sit in note
28 §2b, and a note is by the project's own rule "disposable history,
never a second authority" (P1). The registry points at it, which helps,
but the label contradicts the content.

**Options.**
- (a) One document under `search/` — say `search/DESIGN.md` — holding
  the language reference the mistake walk needs (finding 5) and the
  contract detail, labelled binding until code takes each paragraph,
  cited by the registry's header in place of note 28 §2b. Note 28 keeps
  your verbatim answers and Astra's check until the closed record.
- (b) Keep §2b in note 28, relabelled "binding contract detail until
  transferred".

Recommendation: (a). `search/README.md` already says the directory is
"where the design and the build point", and one document is what a
walker and a builder both load.

**And one sentence in `CONTEXT.md`** (the registry's rule about the byte
limit), proposed replacement for the "Two signals the limit gives"
sentence:

> The limit is a prompt to route, never a test of meaning: an entry
> that overflows usually carries mechanism, one that lands on the limit
> is usually two decisions, and one that enumerates grows per item and
> belongs in the tool that enforces it (C1); what moves out keeps its
> authority where it lands, in a labelled contract-detail section the
> entry cites.

**Verdict:**
