# Response to the crystallization audit

**Written 2026-08-31**, after verifying the audit's code citations.
Position: **endorse the audit nearly wholesale.** This note records the
verification, the few places I would trim or sharpen further, and what
the audit episode itself demonstrates. It is deliberately short; where I
say nothing about an audit item, I accept it as written.

## Verification

All five code citations were checked and hold, and each carries the
weight the audit puts on it:

- `ensure_probe` (`daemon.rs`) submits probe jobs with submitter
  `"daemon"` — the daemon initiates root jobs today, so 03's D3 wording
  ("every job has a submitting client") is factually wrong, not merely
  loose.
- `Store::rebuild` and `acq store import` are frontend-triggered writes
  of fact-side state — 03's D1 "one writer class per layer" is already
  false in the productive way the audit describes. Authority framing is
  correct.
- `eta_for`'s doc comment says "An estimate, not a promise" verbatim —
  the non-reserving quote contract is the code's own self-description.
- The refresh parent re-lists before fan-out — reusing it unchanged
  would silently demote a reviewed plan to advice, exactly as the audit
  warns.
- `JobDb` is a public read/write surface — the read-only facade
  (`JobLedgerReader`) is the right blessing for A1, and "daemon offline"
  must not collapse into "no outstanding work" (persisted waiting jobs
  survive precisely because we built persistence).

## The audit's central move, accepted — with its method-cost named

Redefining Plan from *forecast* to **immutable authorization envelope**
(binding; apply executes the listed actions or a strict subset, never an
addition; offline-computable from facts + intent; quote as optional,
time-stamped, non-reserving enrichment) is the audit's largest
contribution and I accept it, including binding-for-v1 and no dynamic
`--deep` fan-out in the tracer.

Named honestly: this *pre-decides* the question 03 deliberately left
open for build step 6 ("decide with code in hand"). The departure is
justified by the audit's own P3 exception — the binding/advisory choice
shapes the annotation schema and the parent-job design, so it is needed
at step 1, not step 6; it was never really a step-6 decision. But the
method still gets its test: **binding is recorded as
revisable-by-tracer-evidence.** If the owner's live use fights
subset-only reconciliation (vanished tabs skipped, new tabs waiting for
the next plan), that friction is data, and the next session re-rules
with evidence neither model has today.

## Three trims (smaller than the audit's version, same properties)

1. **Stable account identity: key annotations by it now; leave fact
   paths alone.** The audit's step 2 has fact *and* annotation paths
   move to the stable local key. The stable key itself is right and is
   exactly the irreversible-identity case its amended P3 protects. But
   migrating fact paths is deferrable by the audit's own principle:
   facts are refetchable, and CONTEXT.md already accepts rename-orphaned
   fact files. Minimal v1: mint the stable key at first login,
   map it in `accounts.json` (username, aliases, uuid → key), name the
   **annotation** file by it. Fact paths migrate opportunistically,
   later, or never.
2. **`wire_sends` stays coarse in v1.** The logical/wire split is
   correct, but a v1 plan should project wire sends as a range plus
   *named* prerequisites ("a probe may be needed; a token refresh may
   occur; 429 retries possible") — not an accounting. Precise wire
   attribution is the deferred wire-budget feature; letting it leak into
   the Plan type's v1 contract rebuilds the machinery the audit itself
   parked.
3. **D3's final wording needs one more turn of the screw.** Even "every
   root operation has an external initiator" is falsified by probes
   (daemon-submitted roots). The property that is actually true and
   load-bearing: **the daemon creates work only in causal service of
   client-submitted work — probes, children, retries — and never
   originates work spontaneously: no schedules, no policy execution, no
   annotation reads.** That is what keeps blindness (R2) safe, and the
   audit is right that its rationale should be architectural, with the
   macOS keychain fact demoted to corroboration.

## Everything else

Accepted as the audit wrote it, notably: admission-time logical-budget
enforcement replacing 03's mid-fan-out descendant counter (simpler *and*
safer — refuse before spending); reads return freshness while only
asserted freshness preconditions fail with a `RefreshPlan` (R1/D7);
`quote`'s contract as observation-timed, per-policy, non-reserving
projection; revisioned annotation writes now, full change log parked;
`VACUUM INTO`/backup-API export instead of "backup is a file copy";
schema versions behind R4; operation-specific `RefreshPlan` before any
universal Plan grammar; counts-as-heuristic-never-proof on
`metadata.items`; the corrected done-criterion (pin the slice; a CLI
tracer cannot close the GUI/MCP/TUI frontier); and the amended process
lines P2/P3/P4.

## What the episode demonstrates

By the framing's own evidentiary standard — independent agreement is
the strongest signal — this is the strongest validation the design has
received: a second frontier model, auditing adversarially with code
access, accepted every architectural element (four layers, frontend
planner, separate intent, daemon blindness, the tracer choice) and
amended only definitions and contracts. The tower survived contact
with a hostile reviewer; what changed was precision, not shape. That is
also the transmission method working as designed: taste arrived through
boundaries and rationale, and the disagreements landed exactly where
they should — on contracts, where they are cheap to fix before code.

## Proposed next step

Merge 03 + 04 + this note into one final ruling packet (the packet the
owner actually rules on), with the audit's amendments folded into the
decision lines, the revised tracer order (audit's sequence, with trims
1–2 applied), and the binding-plan revisability note attached. Then:
rulings → CONTEXT.md → build.
