# Brief — the proposal audit (stage 4 of the search synthesis)

You audit one design for Acquisition's item search against the page it
was asked to fit and the evidence it cites. The prompt that launched
you names your proposal (note `24` or note `25`). An audit is a
measurement: what holds, what does not, and the text that decides it.
It is not a review. Whether the design is good, pleasant or wise is
another step's question, asked of other readers who will never see
your work; every sentence of yours must survive the question "where
does it say that?". The design's author reads your audit next and
repairs the design against it, so a finding is only useful if the
author can find the line.

Exacting, not generous and not harsh: the ruling that follows can only
be as sound as this audit is accurate. A softened mark, a finding
without its line, or a defect inflated to look thorough is a decision
made for the owner.

## Rules that bind you

- Read, in this order: this brief; `search/DIGEST.md`;
  `brainstorming-notes/22-search-framing.md` (the goal, stances, floor,
  questions, output shape); `brainstorming-notes/29-search-proposal-brief.md`
  (what the designer was asked); your one proposal,
  `brainstorming-notes/24-*.md` or `brainstorming-notes/25-*.md`.
- Open a decision by its `C<n>` (`CONTEXT.md`, `decisions/`) or a track
  README by its finding id only to verify a claim.
- Never: the other proposal; the other audit;
  `brainstorming-notes/23-*`, `30-*`, `32-*` or any other note; the
  index `search/README.md`; the code; memory of any other session. Who
  wrote the design does not matter to any check.
- Nothing here needs the daemon, the store or the network; do not run
  them. Never push, fetch or spawn.
- Write one file: `search/proposal-audit/audit-24.md` or `audit-25.md`.
  Touch nothing else, and never commit — the session that launched you
  does.

## The checks

Each is work you do. Quote the design by its section; a number stays a
number; a quotation is verbatim.

1. **The twelve questions against the page.** For each question in the
   digest's acceptance set (OQ1–OQ7, AQ1–AQ5), take the query the
   design wrote and check every word of it against the design's own
   model-and-grammar page. Mark each: **holds** (every construct is
   defined on the page); **holds off-page** (needs something defined
   only elsewhere in the design — name it); **gap, listed** (the design
   says it cannot, and says so as a gap); **claimed, does not hold**
   (say which construct fails); **missing** (the question is not
   written out). A gap closed by SQL is *claimed, does not hold*.
2. **The citations.** For every `S<n>` a choice rests on, open the
   claim. Does the claim say what the design uses it for? Is the source
   in its lane (note 22's table)? List each miss. List the choices that
   rest on no claim and are not labelled as judgment.
3. **The cold start, counted.** Walk it as the stranger the design
   describes — tool descriptions and help only, no row seen, only what
   the design says those contain. Number each call. Give your count,
   the design's count, and the first call where they part.
4. **The concepts.** For each entry in the inventory: can the twelve
   still be asked without it? Then list every concept the text uses
   that the inventory does not name.
5. **The floor, the stances, the limits.** One row per breach, with the
   line: a gap routed to SQL; a reading that could be stale; game
   knowledge the app is asked to hold; an everyday question (stance 3)
   that needs the advanced machinery; a limit in the digest's register
   that the design drops or contradicts. No breach found is a result:
   say which you checked.
6. **The one page.** Do the model and its grammar fit one page as the
   design presents them? Could a second implementer build the same
   search from that page alone? List what they would have to fetch from
   the other sections.
7. **The output shape.** Note 22 lists nine parts and a closing line
   (what was left out; the one cut to reverse). Present, partial or
   absent, each.

## The audit you write

1. One line: the proposal, and the commit you ran at (`git rev-parse
   --short HEAD`).
2. Findings, most consequential first, one row each: id (`F1`…), check
   number, the claim in one sentence, the evidence (design section
   quoted, `S<n>`, or the call number), what would settle it. No
   recommendation, no redesign, no adjectives.
3. The twelve-question table: question, the design's query verbatim,
   your mark, the page text that decides it.
4. The citation table: `S<n>`, what the design uses it for, holds or
   the miss.
5. The cold start, call by call.
6. The concepts: deletable, and unlisted.
7. The output-shape table.

No size limit and no size target: completeness is the point, and a
table is cheaper than prose.

**Acceptance:** every one of the twelve questions has a mark with the
page text that decides it; every `S<n>` the design cites was opened;
the cold start is numbered call by call; every finding points at a
line.

**Report** (your final message, at most 300 words): what you checked,
what you could not check and why, what you left out, and the one
finding you are least sure of.
