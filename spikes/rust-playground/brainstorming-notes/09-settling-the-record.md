# Settling the record — diagnosis, ladder, routing

**Written 2026-09-02**, after 07 and 08 both recommended a bounded
documentation pass before pricing. This note is the thinking behind
that pass and the rules it leaves in place, so the pass itself needs no
re-arguing. Disposable history like every note here; what it proposes
is real only where it lands in `AGENTS.md`, `CONTEXT.md`, or a tool.

## Diagnosis

The store's rule is that every fact has exactly one authoritative
source. The documents broke it. `CONTEXT.md` holds copies of review
rounds whose source is git, live results whose source is the run ledger,
and mechanism descriptions whose source is the code. Each copy then rots
on its own, which is where the stale headers came from.

Measured on 2026-09-02:

| Measure | Value |
|---|---|
| Required reading path (AGENTS, README, CONTEXT, LIVE-TESTING, TESTING-NOTES) | 246 KB, 2,806 lines |
| `CONTEXT.md` | 110 KB, 1,140 lines; 800 in three slice narratives |
| `CONTEXT.md` over its last 30 commits (Sep 1–2) | 52 KB → 110 KB, every commit added |
| README store bullet, plan bullet | 120 lines, 77 lines |
| `LIVE-TESTING.md` | 80 KB; the standing rule is 50 lines, closed rung sections ~500 |

The one prune to date (the tracer section, `5e1ce37a`, 2026-09-01) sits
before that window; 58 KB landed after the rule "the property lands in
CONTEXT, the round lands in git" was stated. The rule does not hold on
its own because it fights an incentive: at the end of a session an agent
wants a durable home for what it learned, commit messages feel
ephemeral, and CONTEXT is the only file every next session reads. The
growth is proportional to review rounds, and the rounds were the most
valuable thing that happened — so the fix is not fewer rounds; it is
that a round produces a finding row, not a paragraph.

The project already solved this shape three times: `NETWORK-CLEANUP.md`
(66 lines for seven packages: ranges, a findings table, a hash to the
full text), `TESTING-NOTES.md`, and `LIVE-TESTING.md`'s history move on
2026-08-24. The tracer, characters and legibility slices closed without
that step.

## The ladder

Knowledge here takes six forms, ordered by how it is enforced:

1. **Structure** — the daemon cannot link the store. No reading cost;
   cannot be violated.
2. **Test or lint** — fails when violated. No reading cost; rot is loud.
3. **Script** — the tracer driver, the persist check. Procedure made
   executable; drift shows as the script failing.
4. **On-demand file** — a procedure with a trigger line. Costs reading
   only when relevant; rots silently.
5. **Always-loaded file** — `AGENTS.md`, `CONTEXT.md`. Costs every
   session; rots silently; the only place a boundary can live, because
   a boundary must be known before an agent knows it needs it.
6. **History** — git, closed records, `runs/`. Costs nothing unless
   sought.

Push each piece of knowledge to the lowest rung that can hold it. The
network layer stopped accreting exactly when this was done to it (the
limiter spec became test tables; the ladder's paperwork was retired for
the rails, which are code). Skills are rung 4: where procedures go when
they cannot be scripts, and not a destination. Boundaries never move
below rung 5. A skill written from anticipation is a spec; one is
written after its procedure has run twice and repeated a trap.

## Routing (which file owns which fact)

| Kind of fact | Home |
|---|---|
| A ruling, invariant, or boundary property | `CONTEXT.md`, one line, verbatim |
| A property pinned by a test | the test's name in `CONTEXT.md`, nothing more |
| A review finding | a row in the slice's closed record, with its fix commit |
| A build step's narrative | the commit message |
| A live run | one run-ledger row; journals in `runs/` |
| A fact about GGG | a numbered ground-truth claim (master-side) |
| A mechanism | a doc comment on the code |
| A procedure | its on-demand file, referenced by path from `AGENTS.md` |
| Deliberation | a numbered note here |

A slice's history is its commit range; nothing restates it.

## Target state for the pass

- `CONTEXT.md` ≤ 50 KB (gate), aiming at 40: orientation, invariants,
  decisions (content untouched), interfaces at boundary level, open
  topics, parking lot, working style. Headers carry no status.
- `REFRESH-SLICE.md`: the closed record for tracer + characters +
  legibility in `NETWORK-CLEANUP.md`'s shape, with the ruling inventory
  pointing at each ruling's surviving line.
- `README.md`: the store and plan bullets become a few lines plus
  pointers; their content moves to the crates' module docs.
- `LIVE-TESTING.md`: standing rule, rails, ledger, status; closed rung
  sections become their ledger rows plus a hash. The friction-prompt
  template retires — two runs, zero typed notes, both verdicts in
  conversation.
- `tools/docs-check.sh` in the quality gate: the byte budget and a
  stale-identifier scan.
- Three procedure files: live run, mock session, session close.
- `AGENTS.md`: the routing table, the conditional reading order, the
  procedure triggers.

Not touched: the content of any decision, anything parked, the daemon,
any JSON output, the ad-hoc `refresh --tabs`/`--all` kinds (the design
question — should the plan path take an explicit selection? — goes to
open topics with its evidence).

Method: cut, never paraphrase; where a paragraph holds one ruling inside
narrative, that sentence survives verbatim. Every compression cites the
pre-compression hash in the file. The owner reads the compressed
`CONTEXT.md` and the closed record; the words are recorded verbatim as
the verdict.

## Questions for the owner (asked at the reading)

1. Pattern 1 from 07 as a decision line — *for every fact there is one
   authoritative source, and it is the coarser observation's* — now
   that the five store narratives read as that one property.
2. The Decisions section's longest entries (job persistence, bodies
   verbatim, apply): leave as bought, or trim by the owner's hand.
3. `runs/`: twelve mock rehearsal directories sit beside the live
   evidence under the same naming; the driver now writes rehearsals
   apart, and the existing ones are the owner's to delete.

## Outcome (2026-09-02, the pass as run)

| Document | Before | After | Gate |
|---|---|---|---|
| `CONTEXT.md` | 110 KB | 77 KB | 85 KB |
| `README.md` | 40 KB | 25 KB | 30 KB |
| `LIVE-TESTING.md` | 80 KB | 51 KB | 60 KB |
| always-loaded path (AGENTS + README + CONTEXT) | 153 KB | 106 KB | |

The targets above (50 / 20 / 35 KB) were wrong by construction: the
Decisions section alone is 36 KB and was not to be touched, the run
ledger alone is 27 KB of fact rows, and the README's knobs and command
list are its charter. The gates were set at the measured floors plus
room for one slice's rulings. Lowering them further is the owner's call
on the Decisions section (question 2). Commits `74b3dddb..a43377a5`.

## Step two, the same day: the decision registry

Decisions are now a numbered registry (`C1`–`C63`; `D` was already taken
twice, by the C++ design's properties and the ruling packet's lines).
Each entry is one bullet under an 800-byte limit the check enforces;
the six longest honest entries run 640–770 bytes. The mechanism text of
the twenty long entries and the three Interfaces sections moved verbatim
into the owning modules' docs under "Decisions as recorded", headed by
id. `CONTEXT.md` is 37 KB, gate 45 KB. The check refuses an id cited
anywhere that the registry lacks and reports the decisions nothing
cites — 32 at the start, the clause audit's input, to be worked per
area when the area is next touched (pricing first: intent and plans).

## Step three: the registry split by area

The owner's question: byte size is a proxy; the budget that governs an
agent's behaviour is the number of rules it must hold before it knows
which area it is in (rule-count sensitivity, recency, and compaction —
an always-loaded document is what gets lossy-summarized in a long
session, while a file on disk is re-read whole). So `CONTEXT.md` keeps
the invariants and 12 cross-cutting decisions (capped at 15 by the
check) plus an index; the other 51 live in `decisions/{daemon, network,
store, plans, frontends}.md`, read before touching the area and named
at the top of each module's doc so the reminder sits where the hand
goes. `CONTEXT.md` 37 KB → 17 KB, gate 20 KB; always-loaded path
(AGENTS + README + CONTEXT) 153 KB → 47 KB across the three steps.
