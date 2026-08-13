# Slice hand-off: live/history doc split

Status: **open — review round one (DS-R1) filed and fixed
2026-08-13; awaiting Tom's closure.** Tom commissioned an external
consistency audit of the whole spike (five independent readers plus
a test run) and adopted its findings as this slice's review round;
findings and fixes are in the `result-draft.md` §9 DS-R1 entry. Two
decisions remain his at closure: the gate-summary marker
(`status.md` §3 item 6, now with both candidate wordings) and the
core-slice closure-record gap (marker in `core-handoff.md`).
Originally presented for review 2026-08-12. Doc-only
slice per `kickoff-doc-split.md` (adjusted at `6dd3c79b`); unblocked
by the REG-R1 closure. Commit 1 (`77aee08`) pays the round-four doc
debt; commit 2 (`087dc56`) creates `status.md` and re-points both
authorities in the same commit. No code changed; no dated paragraph
was rewritten — corrections carry dated markers with the original
text preserved.

## 1. Silences taken

Short, as predicted — the kickoff is prescriptive. Three readings
were still mine to take:

| Silence | Reading taken | Consequence |
|---|---|---|
| The kickoff collapses AGENTS.md's "what remains" paragraph to a pointer, but that paragraph carried the fragment/verdict rule, which is a rule, not status. | The rule keeps one sentence in AGENTS.md; the status content moved to `status.md`. | A session that skips `status.md` still cannot fill a verdict slot from a fragment run. The rule also appears as `status.md` §5's last line — duplication accepted because one occurrence is a standing rule, the other a sequence entry. |
| The kickoff drops the hand-off table's status column but does not say where per-slice closure *dates* go. | Closure dates are history: they stay in each hand-off's own status line and the §9 changelog; the table keeps only hand-off ↔ slice. | A reader wanting "when did mock close" opens `mock-handoff.md` or the changelog — one hop, never a second live authority. |
| The kickoff does not say whether the chain table lists this doc-only slice. | It does — the split is a slice under the slice process with its own hand-off. | The chain stays complete; a future reader can trace who restructured the docs and under what review. |

## 2. Seam map and invariants walk

No code, test, or fixture changed, so every seam touched is
documentary:

- **AGENTS.md** (process authority): read order now leads with
  `status.md`; the hand-off table keeps the chain and loses its
  status column. The chain rows themselves are unchanged.
- **`slice-review.md` §5** (the three-acts rule): act 1 now flips
  `status.md` first and, on slice closure, adds the historical
  marker to the hand-off. This makes act compliance *easier* — the
  status flip is one named file instead of "whichever docs claim
  liveness" — which is the kickoff's stated success test for the
  split.
- **The five closed hand-offs**: one dated marker line each; all
  dated text untouched. The actor hand-off's marker carries one
  extra clause (see judgment call 3).
- **`result-draft.md`**: commit 1's corrections, each traceable to
  the round-four owed list; one preamble sentence in §3. No register
  cell was edited — the gate-summary rows are exactly as they were
  (deferred, §4 below).
- **Cross-slice invariants 1–6** (AGENTS.md): no engine, actor,
  mock, or test state was touched, so each holds exactly as at the
  REG-R1 closure; the re-run gate matrix below is the evidence, per
  the standing docs-only precedent.

## 3. Coverage confession

- **The naive-reader probe passed, twice.** Two cold sessions with
  no context from this slice read the directory per AGENTS.md's read
  order — one with the kickoff's verbatim prompt, one lightly
  rephrased. Both transcripts are below, verbatim. Both match
  `status.md` on open/next/blocked/coverage and cite no superseded
  source as live.
- **Two prompts is a sample, not proof.** The probe tests one
  property ("a fresh reader is not misled") along two phrasings of
  one entry path (AGENTS.md-first). A reader who enters via a
  hand-off file directly still meets a dated status line before any
  claim of liveness — that is the marker's job — but no probe
  exercised those entries.
- **Nothing machine-checks `status.md`.** Its truth decays by
  process discipline (`slice-review.md` §5 act 1), unlike the
  registry it points to. A registry-style checker for status prose
  was out of scope and is probably not worth its weight; noted so
  the asymmetry is a decision, not an oversight.
- **Deliberately untouched:** F14–F16 (the driver slice's debt);
  every gate and verdict slot; `scenarios.md`; the §8 discrepancies
  that are decisions rather than doc debt (now enumerated in
  `status.md` §3).

- **Probe 1 flagged a real mid-slice artifact, disclosed here.** It
  ran in the window between commit 2 and the creation of this file,
  so it correctly reported that `doc-split-handoff.md` — cited by
  `status.md` §2 and the AGENTS.md table — did not yet exist. The
  file it found missing is this hand-off; the gap closed in the
  commit that carries it. Evidence the probe was genuinely naive,
  and a reminder that the split's pointers are only as good as the
  commit boundary they land on.

### Probe transcript 1 (kickoff's verbatim prompt, cold session)

> **Open** (`status.md` §2): two slices. The scenario-driver slice
> (M1–M13 driver/judge) is in review round four with findings
> F14–F16 (driver twin-guard, duplicated floor literal, mirror
> fallbacks) unaddressed — confirmed by the status line in
> `scenario-driver-handoff.md`. The doc-split slice is presented for
> Tom's review. Core, bootstrap, mock, actor, and clause-registry
> slices are closed.
>
> **Next** (`status.md` §5): fix F14–F16 and close round four; raise
> M-row fragments toward full contracts; G3/G4 finalization; §7.4
> capture replay when unblocked; verdict slots last, only from a
> `verdict_eligible()` run.
>
> **Blocked** (`status.md` §4): §7.4 capture replay awaits raw
> `networkcapture` input from Tom via the `scenarios.md` §4
> sanitizer. Six decisions await Tom (§3), of which the G3-epsilon
> one blocks every verdict slot.
>
> **Coverage** (`status.md` §1): machine-checked via
> `src/obligations.rs` (`CLAUSES`, `OPEN_UNTESTED`) verified by
> `tests/obligations.rs` (uniqueness, owner coverage,
> citation/coverage consistency, cited fns exist, computed open-set
> match). I confirmed `OPEN_UNTESTED` holds 13 ids. Prose tables in
> `obligation-map.md` are superseded/history.
>
> **Discrepancy**: `doc-split-handoff.md`, cited by both `status.md`
> §2 and `AGENTS.md`, does not exist; only `kickoff-doc-split.md` is
> present.

### Probe transcript 2 (varied phrasing, cold session)

> **Open slice** — the **scenario-driver slice** (M1–M13
> driver/judge integration), in review round four. It owes fixes for
> findings **F14–F16** (driver twin-guard, duplicated floor literal,
> mirror fallbacks), all unaddressed. The **doc-split slice** is also
> open, but only awaiting Tom's review. All other slices (core,
> bootstrap, mock, actor, clause registry) are closed. *(status.md
> §2)*
>
> **Next in sequence** — fix F14–F16 and close round four; then
> raise M-row fragments toward full contracts; then G3/G4
> finalization; then §7.4 capture replay; verdict slots last, and
> only from a `verdict_eligible()` full-contract run. *(status.md
> §5)*
>
> **Blocked** — §7.4 capture replay, on raw `networkcapture` input
> from Tom passed through the `scenarios.md` §4 sanitizer; no
> synthetic stand-in permitted. Separately, G3/G4 finalization waits
> on Tom's G3-epsilon decision, which gates every verdict slot.
> *(status.md §4, §3.1)*
>
> **Coverage** — machine-checked via `CLAUSES` in
> `src/obligations.rs`, verified by `tests/obligations.rs`
> (`cargo test --locked --test obligations`). Open items are the
> 13-id `OPEN_UNTESTED` constant, not prose — including
> `x2-single-send-path`, `c3-trip-latched`, and
> `m7-threshold-tuning` (an ambiguity, not an owed test).
> *(status.md §1; obligations.rs)*
>
> The open slice's live coverage confession is in
> `scenario-driver-handoff.md` §3: every M row is a `Fragment`, and
> dropped dispatched `RequestTicket` coverage is absent.

## 4. Judgment calls

1. **`status.md` is item 1 in the read order**, ahead of the
   charter. The kickoff says "immediately after `AGENTS.md` itself,"
   and AGENTS.md is not in its own list, so first is the literal
   reading; a different session might have slotted it after the
   contract docs.
2. **`status.md` §1 restates the `OPEN_UNTESTED` count** (13, dated
   2026-08-12) beside the pointer. A stale count misleads at most
   until the reader runs the test the same paragraph tells them to
   run; a bare pointer seemed less useful than a dated hint plus
   REG-R1-F2's warning that the ids are not all owed tests.
3. **The actor hand-off's marker supersedes one sentence by name.**
   Its status paragraph said "§3's 'not yet covered' list is the
   live one" — itself a liveness claim, so the standard one-line
   marker would have left the paragraph asserting two authorities.
   The marker names that sentence superseded. Dated text was not
   edited.
4. **The gate-summary partial-evidence marker is deferred — flagged
   here per the kickoff.** Audit §8.2 item 6 suggests marking the
   G1–G6 rows "unfilled pending full contracts; fragment-level gate
   evidence green at φ=0/1, see driver status note." It edits
   register cells, so it needs Tom's confirmation at this review;
   it is `status.md` §3 item 6. If confirmed, it is a one-commit
   follow-up.
5. **Commit 1's numeric span correction is in place with the
   original preserved in a bracketed note**, following the
   strikethrough-plus-note precedent already inside doc finding 11,
   rather than a separate supersession paragraph for a two-character
   figure fix.

## Gate matrix

Docs-only change; standard evidence re-run 2026-08-12, offline:

- `cargo test --locked` — green, 135 debug.
- `cargo test --locked --release` — green, 133 (drop-bomb tests
  debug-only, as ever).
- `PROPTEST_CASES=4096 cargo test --locked` — green, 135.
- `cargo clippy --locked --all-targets -- -D warnings` — green.
- `cargo fmt --all --check` and `git diff --check` — green.

Identical to the REG-R1 closure matrix, as expected for a slice that
touches no code.
