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
| 2 | Framing — the owner's edit is the last step | the owner edits; Fable drafted | note 22 | — |
| 3 | Two proposals, blind, in the output shape note 22 sets | Fable and Astra, in parallel, neither seeing the other's until both are committed | notes 24 (Fable), 25 (Astra) | note to write, after the owner's edit of 22 |
| 4 | Cross-review, then reconciliation: a decision table — where they agree, where they differ (each side, the claims that decide, a recommendation) | each reviews the other's; Fable writes the reconciliation, Astra checks it | notes 26, 27 | — |
| 5 | Ruling: candidate decision lines in registry form, a parking lot with triggers, the owner's verdicts verbatim; harvested into `decisions/search.md` (new area file, one index row in `CONTEXT.md`) | the owner, with Fable | note 28, then `decisions/search.md` | — |
| 6 | Build plan: slice steps with their evidence; the digest's acceptance set as acceptance tests; the closed record at the end in `PRICING-SLICE.md`'s mold | Fable | `search/` | — |

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
   recorded in note 22, stance 1 (verbatim) and stance 4.
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

- **Stage 4 prunes the digest, with evidence.** The digest stayed at
  its accepted size on the owner's verdict ("i agree with not trimming
  to 30 aggressively."); which `S` ids the two proposals cite is the
  evidence a pruning waits for, and a pruned claim goes to the kill
  list.
- **Stage 4 grows the digest.** A finding a proposal cited as
  undigested gets a new `S` id in the reconciliation.
- **Stage 5 amends C48.** Raw SQL over the facts file stays no surface;
  read-only SQL over the published contract note 22's settled floor
  describes becomes one, with the contract the ruling gives it. `search/README.md` lists the other
  standing rulings the design revisits.
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
  one passage of note 22 — are one piece of evidence, because two
  readers of one text agreeing is evidence about the text.
  Independence enters through the acceptance test and through what
  each proposer reached without being handed it.
- A brief's terms are defined by what they are for, with the wrong
  reading named (stage 1: most runner-marked `limit` claims were the
  source's limitations, not edges the search declines). Under a binding
  constraint a comparison of runners reports the constraint.
