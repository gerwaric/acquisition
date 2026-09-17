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
| 1 | Digest: one claim per line with a stable `S<n>`, a kind, a weight, a pointer; the acceptance set; a limits register; a kill list per track; the reviewer's question index and convergence marks | one subagent per track, in series; the runner model chosen at the pilot (decision 10); Fable review and merge after each | `search/DIGEST.md` — partial and committed after every track, the entry point in place of the READMEs once accepted | note 21 |
| 2 | Framing: the goal function, the owner's stances, the sources and their lanes, the settled floor, the vocabulary map, triage, convergences, seeds, gravity, the acceptance test, the questions | the owner edits; Fable drafted | note 22 | — |
| 3 | Two proposals, blind, in the output shape note 22 sets (the model, the grammar, the derivation after it, an answer per framing question, the appendix with each question one query or a listed gap, the cold start, the concept inventory, refusals, decisions left to the owner; under 12 KB) | Fable and Astra, in parallel, neither seeing the other's until both are committed | notes 24 (Fable), 25 (Astra) | note to write, after the owner's edit of 22 |
| 4 | Cross-review, then reconciliation: a decision table — where they agree (essential), where they differ (each side, the claims that decide, a recommendation) | each reviews the other's; Fable writes the reconciliation, Astra checks it | notes 26, 27 | — |
| 5 | Ruling: candidate decision lines in registry form, a parking lot with triggers, the owner's verdicts verbatim; harvested into `decisions/search.md` (new area file, one index row in `CONTEXT.md`) | the owner, with Fable | note 28, then `decisions/search.md` | — |
| 6 | Build plan: slice steps with their evidence; the seats' question files as acceptance tests; the closed record at the end in `PRICING-SLICE.md`'s mold | Fable | `search/` | — |

## The owner's decisions, verbatim (2026-09-13 to 16)

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
8. SQL on the surface (2026-09-14, after the reframing in note 22,
   stance 6): "MCP gets an SQL tool for symmetry, so an agent using it
   isn't tempted to waste tokens figuring out how to use the CLI for
   something the mcp can't do." And "read-only annotations should be
   viewable. This kind of thing is present in the c++ app ('priced')
   and the trade site in more detail ('Trade Filters')."
9. On the digest brief (2026-09-16), taking note 21 as committed: "I
   don't have any edits to note 21. It's mostly process and I trust you
   to manage process better than me for this effort."
10. On the dry run of stage 1 (2026-09-16), after Fable's review of
    the brief found ten gaps in its mechanics — the acceptance set had
    no home a proposer loads, the budget did not add up, ids could not
    be stable under parallel writers, the `ruling` kind collided with
    the registry, weight was undefined for most kinds, no `idiom` kind,
    runner isolation incomplete, the digest a cap on the design, the
    merge unspecified — and proposed the wording: "I generally
    approve. I'm also not pressed for time, so we could run each of
    the track in series, which would simplify some of this work. it
    would also allow us to check the quality of the process
    incrementally without fully commiting resources to run every
    track." On the runner model: "I'm on the fence about the model" —
    settled by the pilot's measurement (note 21, "The run, in series"),
    the choice recorded here when it is made.

## Guardrails

- Every stage's brief is committed before it runs and deleted at its
  close, cited by hash (the research-track skill's pattern).
- A session loads the previous stage's artifact and reaches a README
  only to verify a claim.
- The limits register from stage 1 rides through every later stage,
  as the evidence floor a design inherits; a design's own limits are
  its output.
- The digest is a floor, never a cap: a proposal may cite a finding the
  digest dropped, by its README id and marked undigested, and the
  reconciliation adds it to the digest as a new `S` id. Both proposers
  still start from the same page.
- Stage 1 runs in series, one track per run, the partial digest
  committed after each with its status at the top; the index names it
  the entry point only at acceptance.
- When the digest is accepted, note 22's citations of track findings
  (F- and R-numbers) are re-pointed to the digest's `S<n>` ids, so a
  proposer holding only the digest can resolve them.
- The acceptance test: every owner-seat question and every agent-seat
  scenario expressible in the model, written out; simplicity judged
  from the concept inventory, never capped.

## The question the design met first

The owner, 2026-09-14: in the product, agents get "a read-only
interface", and whether that is the on-disk store or a projected
database was open. The agent-seat's phase two measured it (its brief)
against engine-bench's seat projection, and the framing (note 22,
settled floor and stance 6) settled it the same day with decision 8:
the search reads a projection derived from facts (C34), persisted as
SQLite, kept in step with its facts by construction; SQL is a second
language over it through the store, on the CLI and the MCP alike, and
a file opened outside the store is the only bypass. The amendment
stage 5 records: raw SQL over the facts file stays no surface (C48); a
published, versioned, read-only projection derived from facts is one.
Open for the proposals: what it carries, who holds it, its contract,
and how the model itself reads it.
