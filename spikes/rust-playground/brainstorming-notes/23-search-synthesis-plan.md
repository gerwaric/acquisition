# 23 — The search synthesis: the plan and the owner's answers (2026-09-14, cut 2026-09-17)

The plan the synthesis runs under, with the owner's decisions verbatim,
so that every session — Fable here, Astra from Codex — works from the
repository and none of it lives in a chat. **This note is for whoever
runs a stage, never for a proposer:** a proposer's whole reading is its
brief's load list (below). Stage 1's story — the dry run, the pilot,
the budget amendments, the digest's size — is history: note 23 at
`a4dec10a`, and the commits it cites. The note is disposable once
nothing cites its path; the closed record cites it by commit.

## Why stages

Ten track READMEs are about 120 KB, and the research has pulled toward
the intricate end, away from the owner's goal (note 22, "The goal
function"). One session synthesizing everything skims or drowns, and a
design shaped by what was read last is the failure. So: compression,
framing, proposal, review and ruling are separate stages, each
producing one committed artifact sized to be loaded whole by the next,
and each stage's prompt is a committed file the owner edits before it
runs — "I want to make sure all the prompts during this synthesis phase
are similarly challenging, aspirational, and deeply thought-provoking"
(the register is note 00, at `a6a8f53e~1`).

## The stages

| # | Stage | Runner | Artifact | Brief |
| --- | --- | --- | --- | --- |
| 1 | Digest — closed, accepted 2026-09-17 | — | `search/DIGEST.md` | note 21 at c56404ca |
| 2 | Framing — closed; the owner's edits done 2026-09-17 | the owner edits; Fable drafted | note 22 | — |
| 3 | Two proposals, blind — closed; merged `f2b06a3e` 2026-09-17 | Fable and Astra, each in its own worktree | notes 24 (Fable), 25 (Astra) | note 29 |
| 4 | Closed 2026-09-17: note 31 at `2ad2986d`, Astra's check at `8d26eb37`, the owner's Link Groups answer at `9b0b3070`. Audit (a check: does each design hold against its page and its citations), the author's repair, the review (a seat: what each design is like to use), then the reconciliation: a decision table — where the designs agree, where they differ (each side, the claims that decide, what the users said, a recommendation) | audit: one Opus subagent per proposal. Repair: each author, its own proposal. Review: Fable and Astra, each using both designs. Reconciliation: a fresh Fable session with the owner; Astra checks it | `search/proposal-audit/audit-24.md`, `audit-25.md`; notes 24 and 25 repaired in place; notes 26 (Fable's review), 27 (Astra's); note 31, the reconciliation with Astra's check | `search/proposal-audit/BRIEF.md`; note 32 (repair); note 30 (review); this note (the reconciliation) |
| 5 | Closed 2026-09-17: harvested into `decisions/search.md` (new area file, one index row in `CONTEXT.md`, C1 amended to 670 bytes) with Astra's check (note 28 at `f20fbf11`, "ready with named changes") worked in and the owner's verdicts on every finding that changed a line's meaning verbatim in note 28 §4, questions 6–9 (the realm as scope, `low`/`high`/`avg` on a ranged line, undecided with a reason, the pseudo classes); the brief was note 33 at `a0d85b23`. Under decision 15 the rulings are provisional; note 28 keeps the answers and the check (its model page gave way to `search/DESIGN.md`'s language reference, 2026-09-19). An external audit of the stage (2026-09-18, `audits/search-stage-5-audit-results.md`, seven findings and three cleanups, each with its fate) is worked in three sessions: facts and cleanup (done 2026-09-18), the contract gaps with the owner (done 2026-09-18: five verdicts, note 34 at `c2fbb374`, harvested as C105, C106, C96–C97's amendment, note 22's stances 2, 4 and 6, and `search/DESIGN.md`, the binding contract detail), the language reference (done 2026-09-19: the surface chosen in `search/search-forms/`, harvested into `search/DESIGN.md` at `4b88d582`; the audit has no open row) | the owner, with Fable | note 28, then `decisions/search.md` | — |
| 6 | Build plan: slice steps with their evidence, the parser and printer with a round-trip test first (the owner, 2026-09-19, `search/DESIGN.md`); the digest's acceptance set as acceptance tests; the closed record at the end in `PRICING-SLICE.md`'s mold | Fable | `search/` | — |

## Stage 3: what a proposer reads

Agreed 2026-09-16, so that no session's deliberation pre-shapes a
proposal: each proposal runs cold, in a fresh session. The brief is
self-contained (decision 1) and carries all of this itself:

- **Load:** the brief, `search/DIGEST.md`, note 22. The registry by
  `C<n>` and a track README by pointer, only to verify a claim.
- **Never:** the other proposal; this note; any other numbered note;
  for Fable, the memory files on item search (they hold conclusions
  Astra does not have); for either, anything recalled from an earlier
  session.
- **The digest is a floor, never a cap:** a proposal may cite a finding
  the digest dropped, by its README id and marked undigested. Both
  proposers still start from the same page.
- **The limits register is inherited whole;** a design's own limits are
  its output.
- Where the proposal is written and that its author commits it (notes
  24 and 25; Codex commits its own, as it has for reviews).

**Blind by construction, not by instruction.** Each proposer runs in
its own worktree on its own branch, both cut from the commit that holds
the brief (`git worktree add <dir> -b search-proposal-fable <commit>`,
and `-astra`), so neither tree ever contains the other's file and the
two can run at once. Both branches merge into `spikes/rust-playground`
only after both proposals are committed; the files are distinct, so the
merge cannot conflict. A worktree at a new path should also start a
Fable session without this project's memory files; check that at the
start of the run rather than assume it.

## Stage 4: how it runs

Four steps, in order, each with one input. A first design of this stage
had two auditors per design, anonymised audits, a parts merge and a
fate for every finding; the owner cut it (decision 14): checking is not
judging, so model bias matters little there, and a design may be edited
— so its author repairs it, and the findings are spent where they are
found.

1. **The audit.** One Opus subagent per proposal under
   `search/proposal-audit/BRIEF.md`, launched from any session (prompt:
   the brief's path, the proposal's number, the prohibitions repeated).
   The launching session checks each audit against the brief's
   acceptance sentence — form, not substance — and commits. The check
   on a wrong finding is the next step.
2. **The repair,** one pass under note 32, skipped for a proposal whose
   audit found nothing that matters. A fresh Fable session repairs note
   24; Astra, from Codex, repairs note 25; each sees only its own
   audit. Every finding is fixed, listed as a gap, or disputed with the
   line; nothing else changes; a `## Repairs` table is appended. No
   re-audit.
3. **The review,** under note 30, on the repaired designs. First run
   `python3 search/proposal-audit/anonymise.py` and commit
   `search/designs/` (both retired at the close, `548ecc4f`): the proposals with provenance, titles and the
   `## Repairs` table removed.
   **Design A is note 25 (Astra's); design B is note 24 (Fable's)** — a
   coin flip, recorded here and in that script, where no reviewer
   reads. It lowers a reviewer's preference for its own model's work;
   it cannot remove it (the sizes differ, and style shows). Fable and
   Astra each use both designs, in opposite orders, each in its own
   worktree and branch cut from the commit that holds the copies
   (`search-review-fable`, `search-review-astra`), merged only after
   both reviews are committed: a review written in sight of the other
   is one review.
4. **The owner's seat** (optional, and worth more than either review's
   opinion): before reading any review, ten minutes asking OQ1 of each
   design's model page, and a few lines on what happened. Both
   reviewers are agents; stance 5 says "both the agent and the human".
5. **The reconciliation** (note 31), a fresh Fable session here, with
   the owner. It reads this note, note 22, the digest, both repaired
   proposals with their `## Repairs` tables, both audits and both
   reviews, and writes:
   - the decision table: one row per design question (note 22's seven,
     then any the proposals raised) — what each proposal says, whether
     they agree, the `S` ids that decide, what the users said, and a
     recommendation.
   - the disputed findings, each settled by the line or left to the
     owner; and any place where a design audited clean and was a labour
     to use, or the reverse.
   - the withheld checklist, seed by seed: reached unprompted by which
     proposal, or by neither — and then proposed here.
   - the digest's changes: undigested findings the proposals cited, to
     become new `S` ids; claims nothing cited, as candidates for the
     kill list.
   - what stage 5 needs from the owner, as questions he can answer in a
     line.
6. **Astra's check**, from Codex, appended to note 31 as its last
   section and committed by Codex: is every row traceable to both
   proposals and to the claims it names; does each recommendation
   follow from its row; is any voice counted twice. At this step Astra
   reads this note too — nothing is blind any more.

## The owner's decisions, verbatim (numbers are stable; citations use them)

The owner's words are verbatim but for spelling, corrected at his
request in every file a model reads (2026-09-17: "I don't want their
attention distracted by my mistakes"); the uncorrected text is history,
at `e490196e`.

1. "Astra is an OpenAI model I will run from codex." — so every brief
   is a self-contained repository file Codex can be pointed at, and
   Codex commits its own artifacts, as it has for reviews.
2. Blind proposals: "agreed".
3. Reconciliation: "Agree Fable writes. I've been using codex for
   reviews and this has worked well."
4. The rulings' home: "yes, new decisions file".
5. The digest as entry point — carried out; `search/README.md` says so.
6. Simplicity judged, not capped — recorded in note 22.
7. The seat questions test reach, and game knowledge is the user's —
   recorded in note 22, stance 1 (verbatim) and stance 4 (refined
   2026-09-18: judgment is the user's; game data and community
   convention enter as reference data, C106).
8. SQL on the surface, and read-only annotations viewable — distilled
   into note 22 (stance 6, the settled floor); the owner's words are
   at `a4dec10a`, this decision.
9. The digest brief taken as committed — spent with stage 1.
10. Stage 1's run: series, the pilot, the budgets, the acceptance —
    spent with stage 1; what it leaves owed is under "Owed by later
    stages".
11. On the audit that cut this note to what stages 3–6 need
    (2026-09-17): "I approve your changes."
12. On the proposal's size and the seeds (2026-09-17). The estimate was
    that note 22's output shape needs about 16 KB against its 12 KB
    cap, and stage 1 showed what a binding cap cuts: whole items, here
    the gaps, limits and inventory lines stage 4 judges by. The
    recommendation: the model and grammar hold to one page, the rest
    gets a guide. "I approve your recommendation on the cap, but
    let's make 16kB the guide. Several times our estimates on the
    amount needed have been low, so I'd like to give the models more
    room to be expressive so we have more to judge." And on moving the
    answer-shaped seeds and the one thesis among the convergence
    signals out of the proposers' reading: "I agree to the rest of your
    recommendations on the seeds and signals."
13. On the stage-3 brief (note 29, 2026-09-17): the body taken as
    drafted ("it looks good"), the owner's own final word added, and
    Fable's edits to it — scoped to the search, the human seat kept, "a
    few linked abstractions" for "a tower" — "All of your changes
    improve the text. Accepted." On isolation: "either branches or
    worktrees based on whichever best serves our purposes in a simple,
    reliable way."
14. On stage 4 (2026-09-17). Audit and review are split at the owner's
    suggestion: "What if we do a 2-stage process here, where the first
    stage is purely an audit, so we can devote the second-stage to
    purely what adds value as a review to the reconciliation?" Then,
    when Fable's first design of the audit had grown to six sessions
    and a fate for every finding: "you and I are spiraling into some
    process complexity here. Is there a way to simplify the audit stage
    significantly? I'm ok if we have to edit the designs, and since we
    are checking things instead of forming opinions on them, I'm
    thinking the model bias might be less important." — one auditor,
    the author repairs. "I confirm. Go ahead."
15. On the reconciliation (note 31, 2026-09-17), asked twelve one-line
    questions: "If I'm honest, this is over my head, and my input is not
    likely to add value until the design is close to a UI such as trying
    to replicate the trade site, or c++ app, or even the SQL-like
    surface. In that spirit, my feeling is that we should press ahead
    according to plan and I'll be able to add my input later, even if it
    means cycling back on the design." — so, Fable's reading: Astra's
    check runs as planned; stage 5 takes note 31's recommendations as
    provisional rulings, each carrying its revisit trigger (the owner's
    first seat at a surface he can use), and marks the three proposals
    no audit or reviewer has seen as such; stage 6 orders the build so
    a surface the owner can sit at comes as early as the evidence
    allows, because that is where his input enters. The owner's seat
    (step 4) moves there.

16. On the SQL surface (stage 5, 2026-09-17), having ruled decision 8 and
    then asked whether it was "doing some unaccounted-for harm like
    constraining our design or implementation": "I agree with (b) now,
    and possibly (c) if json export isn't sufficient for some future
    need. Between pseudo-mods, counting, weighting, and all the other
    search features we need, I suspect the only way an SQL surface makes
    sense is if we have an SQL table behind it, which so far nobody has
    been pushing for." — (b) is no SQL surface in the slice, (c) an
    export parked with a trigger (note 28). Decision 8's SQL half is
    withdrawn; its other half, read-only annotations viewable, stands.
    The C48 amendment once owed by stage 5 is no longer owed (its bullet
    removed at the harvest).

## Withheld from the proposers: stage 4's checklist

Note 22 carried eight synthesis seeds, Fable's drafts. Three restated
framing that stays (limits as an output: question 7; the holder:
question 4; one grammar, four adapters: stance 5) and are dropped. Five
were answers to ranked questions 1–4, and one convergence signal was a
design thesis; handed to both proposers they would have returned as
agreement. They wait here (full text: note 22 at `5e0d0681`). The
reconciliation asks of each: did a proposal reach it unprompted — which
is evidence — and if neither did, the reconciliation proposes it then,
so nothing is lost.

- **The query as a value.** Immutable, serializable, named: saved,
  shared, sent to the site as a search, received from the site as a
  URL, handed from an agent to a human. C38's plan is the mold.
- **The vocabulary is a read.** Autocomplete for the owner and schema
  discovery for the agent are the same read (C53's views): what lines
  exist in *this* corpus, by kind, ranked by how many items carry them.
- **Identity is the template plus its kind; the stat id lives at the
  boundary.** The template names every line, the stat id names 67 %,
  and only the trade boundary needs the id.
- **Query by example.** "Like this item, but with life at least 70":
  the refinement both seats want (agent R14, S195), with an item id as
  the starting point either seat can write.
- **The model's semantics as views.** A derived meaning — a pseudo
  total, the named location, "priced" — defined once as a view in the
  projection's DDL, owned by the store crate (C46), so the model and a
  SQL caller cannot disagree; what stance 6 needs to be true.
- **Every tool that lasted asks in the item's own terms** (the thesis):
  the site's form is the tooltip as a form; item-filter names
  predicates after tooltip fields; the C++ buckets are the tooltip's
  mod sections; Awakened starts from an item and makes a query of it.

## Owed by later stages

- **Stage 4 pruned and grew the digest** — spent: the reconciliation
  added S16, S17, S57, S58 and S159; pruning went to the kill list,
  parked in `decisions/search.md` behind stage 6's acceptance tests
  (the owner: "i agree with not trimming to 30 aggressively.").
- **Stage 6 reads the digest's acceptance set as its tests.**

## Guardrails

- Every stage's brief is committed before it runs and deleted at its
  close, cited by hash (the research-track skill's pattern).
- A session loads the previous stage's artifact and reaches a README
  only to verify a claim.
- The acceptance test and the output shape have one home, note 22;
  nothing here restates them.
- Agreement is counted by its evidence, not its voices (stage 4): two
  proposals that agree while resting on the same text — one `S` id,
  one passage of note 22 or of the brief, a response the limits
  register already prescribes — are one piece of evidence, because two
  readers of one text agreeing is evidence about the text.
  Independence enters through the acceptance test and through what
  each proposer reached without being handed it.
- Three kinds of evidence reach the reconciliation and are never added
  together. An audit finding counts by the line it points at.
  Testimony is one seat's experience: evidence about what a design is
  like to use, never about what is true of it. And a reviewer's taste and its own
  model's proposal are one voice, never two: a reviewer preferring the
  other model's design is strong evidence; preferring its own is weak.
- A brief's terms are defined by what they are for, with the wrong
  reading named (stage 1: most runner-marked `limit` claims were the
  source's limitations, not edges the search declines). Under a binding
  constraint a comparison of runners reports the constraint.
- Before a brief or a framing runs, read it top-down for an answer
  sitting where a boundary belongs (notes 22 and 29, 2026-09-17, found
  each of these): a floor that names a mechanism; a question that lists
  its answer's parts; a source's lane stated as its conclusion; a
  warning that carries a ruling; an example that pulls where a stance
  pushes; a word that invites a reading nobody meant ("loose" read as
  fuzzy, "maximally" read as more). The last words of a prompt weigh
  most, so what closes it must agree with everything above it.
