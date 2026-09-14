# 23 — The search synthesis: the plan and the owner's answers (2026-09-14)

**Written 2026-09-14, from the conversation of 2026-09-13/14.** The plan
the synthesis runs under, with the owner's decisions verbatim, so that
every session — Fable here, Astra from Codex — reads the same plan from
the repository and none of it lives in a chat. Disposable once the
slice's closed record cites it; until then the index (`search/README.md`)
points here.

## Why stages

Ten track READMEs are about 120 KB, and the research has pulled toward
the intricate end. The owner's goal, verbatim: "a powerfully simple,
usable, comprehensive system of search that eschews overcomplication
and tricky edge cases and implementations." One session synthesizing
everything skims or drowns, and a design shaped by what was read last
is the failure. So: compression, framing, proposal, review and ruling
are separate stages, each producing one committed artifact sized to be
loaded whole by the next, and each stage's prompt is a committed file
the owner edits before it runs — "I want to make sure all the prompts
during this synthesis phase are similarly challenging, aspirational,
and deeply thought-provoking" (the register is note 00, at `a6a8f53e~1`).

## The stages

| # | Stage | Runner | Artifact | Brief |
| --- | --- | --- | --- | --- |
| 1 | Digest: one claim per line with a stable `S<n>`, a kind, a weight, a pointer; a limits register; a kill list per track | Opus subagents per track, Fable review | `search/DIGEST.md` — the entry point in place of the READMEs once accepted | note 21 |
| 2 | Framing: the goal function, the owner's stances, the sources and their lanes, the settled floor, the vocabulary map, triage, convergences, seeds, gravity, the acceptance test, the questions | the owner edits; Fable drafted | note 22 | — |
| 3 | Two proposals, blind: the model on one page, the grammar, an answer per framing question, the appendix (every seat question in the model's notation), the concept inventory, refusals, decisions left to the owner; under 10 KB | Fable and Astra, in parallel, neither seeing the other's until both are committed | notes 24 (Fable), 25 (Astra) | note to write, after the owner's edit of 22 |
| 4 | Cross-review, then reconciliation: a decision table — where they agree (essential), where they differ (each side, the claims that decide, a recommendation) | each reviews the other's; Fable writes the reconciliation, Astra checks it | notes 26, 27 | — |
| 5 | Ruling: candidate decision lines in registry form, a parking lot with triggers, the owner's verdicts verbatim; harvested into `decisions/search.md` (new area file, one index row in `CONTEXT.md`) | the owner, with Fable | note 28, then `decisions/search.md` | — |
| 6 | Build plan: slice steps with their evidence; the seats' question files as acceptance tests; the closed record at the end in `PRICING-SLICE.md`'s mold | Fable | `search/` | — |

## The owner's decisions, verbatim (2026-09-13/14)

1. "Astra is an OpenAI model I will run from codex." — so every brief
   is a self-contained repository file Codex can be pointed at, and
   Codex commits its own artifacts, as it has for reviews.
2. Blind proposals: "agreed".
3. Reconciliation: "Agree fabe writes. I've been using codex for
   reviews and this has worked well."
4. The rulings' home: "yes, new decisions file".
5. The digest as entry point: "Yes, replaces README as the entry point
   for each track".
6. Simplicity: offered capped, judged, or both — "I agree with you on
   1" (judged, not capped; recorded in note 22).
7. On the reading of his own questions: "We need a system that is
   flexible, simple, and generalizable here, not one tuned to a
   specific set of questions from a specific single user." And on game
   knowledge: "We should let users provide that knowledge for now
   instead of trying to embed or access it from within the app."

## Guardrails

- Every stage's brief is committed before it runs and deleted at its
  close, cited by hash (the research-track skill's pattern).
- A session loads the previous stage's artifact and reaches a README
  only to verify a claim.
- The limits register from stage 1 rides through every later stage.
- The acceptance test: every owner-seat question and every agent-seat
  scenario expressible in the model, written out; simplicity judged
  from the concept inventory, never capped.

## The question the design will meet first

The owner, 2026-09-14: in the product, agents get "a read-only
interface", and whether that is the on-disk store or a projected
database is open. The agent-seat's phase two measures it (its brief);
engine-bench's seat projection is the file it measures against. The
candidate amendment it will put to stage 5: raw SQL over the facts file
stays no surface (C48); a published, versioned, read-only projection
derived from facts (C34) may be one.
