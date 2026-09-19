# Brief — the language reference (stage-5 audit, finding 5; run by a subagent)

**Do not run: a first draft, kept as the record of what was stepped back
from.** It calls the work "a reference, no design" and asks for the
language's whole surface; its size guide is an estimate that left out
the stash and the answers. The surface is ruled first, in
`brainstorming-notes/35-search-forms-table.md`; this brief is rewritten
against the ruled table before any run.

You are an Opus subagent drafting one section of one file. Read this
brief, then `search/README.md` (rules and traps), then the inputs below
in the order given. This is not a track: no README holds the question,
so this brief does, and your one writable file is `search/DESIGN.md` —
one new section, `## The language reference`, between the header
paragraphs and `## Contract detail, by decision`. Change nothing else in
that file, header included, and no other file: never the index,
a track, `CONTEXT.md`, `decisions/`, `SURFACES.md` or
`brainstorming-notes/`. Never commit, push, pull, fetch, run a network
request or spawn an agent. The reviewer commits; this brief is deleted
at the close, cited by hash.

**A reference, no design.** The model is ruled. You add no construct,
field or rule, and you settle no question the inputs leave open. What you do choose is spelling, where no input fixes one,
and every such choice is visible (below).

## The question

The stage-5 audit's finding 5 (`audits/search-stage-5-audit-results.md`)
is that the model's one page (note 28 §1) cannot be walked: a reader
must invent notation or consult the old proposals. The next step of the
slice is a mistake walk — a seat making errors against a page, before
any grammar is built. What page lets that seat write a request for
every construct of the ruled model, and say what it expects back,
holding nothing but the page?

## Inputs, in order of authority

1. The owner's words: note 28 §4
   (`brainstorming-notes/28-search-ruling-packet.md`, "For the owner —
   asked and answered" through "The owner's verdicts, verbatim"), and
   the quotations on the lines of 2 and 3. The note is 42,742 bytes:
   read it by section, never whole.
2. `decisions/search.md` — the lines C89–C107 and "Parked".
3. `search/DESIGN.md` — the contract detail, as it stands.
4. Note 28 §1, "The combined model on one page", with its concept
   inventory.
5. For spelling only, where 1–4 fix none: note 24 §1–2
   (`brainstorming-notes/24-search-proposal-fable.md`) and the
   `line(…)` binder of note 25 §1–2
   (`brainstorming-notes/25-search-proposal-astra.md`; read that
   section only). Both predate the rulings, and the rulings changed
   some of what they say: on any conflict 1–4 win.
6. For the words inside an example: `search/item-facts/data/` —
   `mod-templates.csv` (it carries bare CRs: open with `newline="\n"`),
   `field-census.csv`, `properties-census.csv`, `class-evidence.csv`.

Where two inputs disagree, the page follows the higher and the report
says so, first. Nothing else is a source; `search/DIGEST.md` may be
opened to read an `S` id a line cites.

## The page

- **Self-contained.** A reader holding only the new section — not the
  registry, not a note, not the contract detail beneath it — can write
  each construct and read each answer. Nothing on the page sends the
  reader elsewhere to learn a spelling.
- **One sample stash, stated once.** Every example runs against one
  small stash given at the top of the section, small enough that a
  reviewer checks every printed answer by hand, and holding whatever
  the examples need and nothing more. The case C105's contract detail
  pins is reproduced exactly. Its numbers
  are illustration and say so; its templates, fields and classes are
  real (input 6).
- **Per construct:** one sentence of what it means, then an example —
  the request as it is given, and the part of the answer the construct
  changes, laid out in the answer shape of note 28 §1. An edge rule the
  contract detail holds is shown by an example and cited by its
  decision id, never restated in prose: that paragraph stays the rule's
  one home.
- **What is outside the conditions.** Scope, view, order, named fields
  and continuation are stated outside the query text (C96, C100). No
  input says how a request spells them. Choose one form, use it
  throughout, and table it.
- **Spelling.** A spelling inputs 1–4 fix is used as written. Any other
  — taken from input 5, or yours — is one row of a closing table,
  *Spellings chosen here*: the construct, the spelling, where it came
  from, and what input 5 said when a ruling overrode it. That table is
  the agenda of the owner's review, and each row is provisional until
  his first seat (note 23, decision 15). Anything a form implies that
  no input states — precedence, quoting, escaping — is a row too.
- **Not in the language.** What "Parked" and C102 put out of reach, as
  a short closing list: the thing, and the workaround the entry names.
  No example spells a parked construct.
- **Open for the owner.** The contract detail leaves one question to
  this page — whether the `none` and `undecided` buckets carry a term
  that selects them, as C97's rows do. Lay out both readings with what
  each would print; choose neither. A question of the same kind that
  you find gets the same treatment.

## Constructs to cover

The floor of coverage, by the decision that rules each; the order and
grouping of the page are yours. A construct the inputs hold and this
list lacks is added, and named in the report.

| Construct | Ruled by |
| --- | --- |
| naming a line by its displayed text; a text that resolves to no template | C90 |
| an ambiguity the grammar defines: the error that shows the readings | C91 |
| selecting by kind: the source array, the flag, both | C90, C97 |
| a comparison on a line, holding on one occurrence | C92 |
| slots: a ranged line's words, slots by position, several conditions in one term, the error when no slot is named | C92 |
| fields and flags from the closed lists; presence and absence | C93, C97, C101 |
| `priced` | C100 |
| place terms: league, tab, character, container | C96 |
| composition: and, not, or, parentheses, at-least-N-of with both bounds; a refinement | C91 |
| the item's sum of a line; a named total and its definition on request; a ranged total | C92, C94 |
| a derived field and its definition | C101 |
| socket colours over the whole item, and within one link group | C101 |
| the scope: a realm named, the front shorthand, all realms, none named over two realms, the default within a realm | C96 |
| the answer's blocks, each once: query as text and tree, basis, scope, terms, total, view, next | C98, C100, C104 |
| the terms block: matched, failed, lacked, undecided; absent against undecided with its reason; the together count | C92, C93 |
| undecided under composition, and a readable line as a witness | C93 |
| an incomplete subtotal, and a comparison on one | C94, C95 |
| rows: the compact row, named fields, a sort and what sorts last | C92, C100 |
| counts: facets, a crossed table, a count with a summed thing; `none`, `undecided` and the tally | C95, C105 |
| the vocabulary read and the term a row carries, in one realm and under all realms | C97 |
| one item: `show`, the why-not, an id accepted back | C100 |
| a continuation, and its refusal across a change | C98, C100 |
| the trade translation in and out: the per-clause report and the remainder | C99 |
| a limit met: what the answer says | C102 |

## Size

A guide, not a budget: about 12 KB for the section (note 28 §1 is
5,112 bytes; `DESIGN.md` is 15,022 today). A
construct without an example is the failure, never an overrun. Cut
nothing to reach a number; say in the report what you would cut.

## Acceptance

Holding only the new section, a reader can write a request for every
row of the table and say which part of the answer changes, inventing no
notation; every printed answer follows by hand from the sample stash;
every spelling that inputs 1–4 do not fix is a row of *Spellings chosen
here*; the contract detail is byte-for-byte unchanged (`git diff` shows
one inserted hunk); `tools/docs-check.sh` and `git diff --check` pass.

## Report (at most 300 words)

First, any disagreement between inputs and which you followed. Then:
constructs you added to the table; every sentence elsewhere the new
section makes stale (the header of `DESIGN.md` and of
`decisions/search.md`, note 28, `search/README.md`) — listed, not
edited; the three spellings you were least sure of; what you left out,
and the one cut you would most want reversed.
