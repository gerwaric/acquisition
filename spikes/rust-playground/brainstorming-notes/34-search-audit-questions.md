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

**Verdict:** "Accepted after dropping the historical reference and fixing
the floor." (owner, 2026-09-18, session B) — *harvest gloss, not the
owner's words:* the stance is the italic text alone; the closing "The SQL
surface this stance once named …" sentence is dropped (the floor bullet
and the commit carry the history); the floor bullet loses its last
sentence about stance 6's SQL clauses, which the rewrite makes stale.

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

**Verdict:** (owner, 2026-09-18, session B, three answers)
- On the undecided bucket: "Yes, undecided is good, but I'm wary about
  requiring a reason at the aggregate level because different items may
  be undecided for different reasons." Then, on the tally: "Undecided in
  aggregate doesn't have a reason. If we can bucket the undecided reasons
  by kind without undue complexity, go ahead."
- On known absence: "Yes, a separate bucket for none."
- On the reconciliation promise: "I agree with the two invariants."

*Harvest gloss, not the owner's words.* The session found two holes in
the draft above. (A) Known absence is not undecided (C93): an item that
has no value for the key (a stash item counted by character) lands in a
`none` bucket, apart from `undecided`; each printed only when nonzero.
(B) "Buckets sum to the total" is false for a key an item holds several
of (`line`, influence). The two invariants: every matching item is in at
least one real bucket, or in `none`, or in `undecided`; and for a key
with one value per item the buckets sum exactly to the total — the case
the first counts test pins. The bucket is named `undecided` and its count
is the contract; the reason stays on the item (C93). Beneath the bucket,
a tally by reason *kind* (C93's closed list), if it costs no undue
complexity: a diagnostic, not a partition — an item with two reasons
counts under both, so the tally need not sum. A crossed table gains at
most one `undecided` and one `none` row or column per name, the tally
once per table. Open for session C: whether the `undecided` bucket
carries a pasteable selecting term as C97's rows do.

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

**Verdict:** (owner, 2026-09-18, session B)
- On the realm word inside a term, this section's proposal: "Should we
  consider disallowing realm within search terms, if it's already part of
  the scope? I don't fully understand it's value in this scenario yet."
  Then: "Disallow realm as a search term." **The proposal above is
  declined.**
- On the unstated scope, after weighing an all-realms default: "I now
  agree that a search should error without a realm named. The sticky
  default idea should be parked for future development."

*Harvest gloss, not the owner's words.* Realm lives in the scope only
(C96). What the word bought was one query with different conditions per
game; what it cost was realm in two places, a redundant case and a
contradiction error, and another closed list of bare words. So finding 4
closes by amending C97's promise, not by syntax: under an all-realms
scope a term matches its line in every realm in scope; the vocabulary
still lists a template once per realm (C90), each row naming its realm;
the row's term with the scope narrowed to that realm selects exactly
that row. A per-realm condition inside one query is a listed limit, its
workaround two searches. Proposed park (adding the word later breaks
nothing; removing it would): a realm word in a term, trigger a recorded
question needing different conditions per realm in one answer. C96's
error is read as written — it fires only over a store holding more than
one realm. C96's default stands unamended.
The sticky default is the owner's idea ("a way to set the realm as a
state variable that applies to searches until changed or unset"): an
adapter convenience, never model state — the library holds none (C98), a
query is a value (C91), every request carries a resolved scope and every
answer prints it (C96); the adapter supplies the scope when none is typed
(a CLI knob, a GUI's realm dropdown, an MCP default; C46). Persisted
across sessions it is user-scoped intent (the `user.db` park). Lands as a
park in `decisions/search.md`; proposed trigger: the first seat holding
two realms that tires of stating the scope.

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

**Verdict:** (owner, 2026-09-18, session B, in the order given)
- "I accept the data/judgement line. However, we should discuss the
  judgement element before proceeding. You used the example of
  'equipment' to show a case where players may have conflicting
  definitions. This is something that's well-defined by the community,
  e.g. RePoE, the trade site categories, poedb.tw, and
  https://www.poewiki.net/wiki/Equipment. Can we import concents and
  definitions from those as registered surfaces to keep game data aligned
  to community standards, even if they aren't directly from GGG?"
- On the three-way line with the admission test: "I agree. However, I
  want to stop and think about the legacy concept. Many of acquisition's
  users have been playing for over a decade, so being able to easily
  search for legacy gear would be valuable."
- After the test's (b) was corrected: "I approve the proposed threee-way
  line as amended."

*Harvest gloss, not the owner's words.* The line is three-way. **Data**:
what the game files state (base ranges, the mod table). **Convention**: a
concept with a published community definition (equipment, the trade
site's categories). Both enter as reference data under C68 from a
registered surface (C79 — which never required the source to be GGG's),
never fetched at runtime. **Judgment**: what is good, what it is worth,
what to keep — never embedded. A convention behaves as a named total does
(C94): defined once, prints its definition and source, names one
authority with the rest as corroboration and disagreement noted on the
row (order: GGG-authored surfaces, then game-file exports, then community
prose), and removes nothing — the user composes their own meaning from
class terms. The admission test: (a) a published definition at a
registered surface; (b) *reviewable* — generated by a committed script
from a pinned source and reviewed as a diff per pull (the first wording,
"small enough to review every row", would have refused the 476-row
base-defence extract); (c) the word prints its definition and source;
(d) every question can still be asked without it. Surface rows change
only when the first definition needs one; licence is per row (the wiki is
CC BY-NC 3.0, poedb states no terms). "Which categories count as
equipment" moves from the user's column to convention; the `category`
park gains its source of truth. Stances 2 and 4 get wording drafted for
the owner's approval (stance 2's last clause, read literally, gives every
reference-backed field to the user). **Legacy stays the user's for now**
(S178, R8 stand) pending a research track: Path of Building, a registered
clone at `16de4b82d`, labels unique variants by version — the owner's own
S165 example, Ashes of the Stars "Pre 3.23.0", is there; roughly 1,383
such labels over about 715 uniques by a grep, not a parse. The track
measures, over a `.backup` of the owner's store: uniques matching one
variant, several, none; how often "either" (undecided, C93); what the
rares case has for a source, if anything. Lands as a park in
`decisions/search.md` whose trigger is that track's result.

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

**Verdict:** "I agree with (a) and accept the CONTEXT.md sentence."
(owner, 2026-09-18, session B) — *harvest gloss, not the owner's words:*
the owner named no other file, so the document is `search/DESIGN.md`;
this session creates it with the contract detail moved in and labelled
binding until code takes each paragraph, session C adds the language
reference; the overflow from questions 2–4 lands there.
