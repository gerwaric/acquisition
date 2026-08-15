# Review charge: repeated SD-R8 re-close over the F11–F20 repairs

Status: **open — charge for the independent re-close reviewer**
(drafted 2026-08-15 by the analyst session that wrote
`f11-f20-repair-charge.md`; that authorship is a declared interest —
this charge encodes what that session knows about the defect class,
and the reviewer's job includes judging whether its prescribed
approach was *right*, not only whether it was implemented). Read the
mandated documents in AGENTS.md order first; `status.md` is live
authority; the packet under review is `scenario-driver-handoff.md`'s
current four parts; the repair range ends at spike head `1c71763a`
and ground-truth head `3088d6e4`. Close per `slice-review.md` §5 or
record findings and leave the round open. Never contact a live
service; commit before reverting any mutation you run.

## 1. Verify the repairs against the charge, item by item

`f11-f20-repair-charge.md` §§1–4 against the diffs and the §9 repair
entries. For every wording repair (F13–F18), use the final audit's
own method: re-derive each corrected sentence from the evidence
record — never accept that a sentence was softened; check the
softened sentence is *true*. F14/F16 must cite Tom's recorded
decisions. F19/F20 are on `rate-limit-core-ground-truth`: check N31
against B3's ratified conventions in `scenarios.md` §7.2 and N32
against `Actor::schedule`/`pending_probe()` in the actor source,
not against the topic's own summary.

## 2. Reproduce all five mutation signatures

The preserved F5/F9 pins and the three new refusals (F11 relabel →
`MissingScenarioEndpointLane`; F12 exact split → the structural pin;
F12 profile flip → `MissingM8KnownLane`), exact signatures per the
§9 repair entry. Then both authorities: pinned declaration, the
4,096-case declared run (scale is pinned in code), obligations.

## 3. Hunt the fourth generation — the part that matters most

Three generations of one class (F4: profile lane label; F9:
endpoint label; F11/F12: scenario label and profile-configuration
label) each survived the previous generation's repair. Do not
assume the third repair ended the class. Attack it:

- **The `mod lane` pin is load-bearing and possibly lexical.** If
  the structural pin is a source-text check (the X2 pattern),
  probe its blind spots: can an engine or record be constructed
  through a re-export, a helper added outside the module, a test
  building `RunReport` directly, or a second `spawn` call the
  lexical pattern misses? Write the bypass; if it passes both
  authorities, that is F21.
- **The pair requirement trusts the pair.** A report claiming
  `(M2, CharacterList)` whose wire evidence is not M2-shaped — is
  it caught by anything except the M2 assertion the label routes
  to? Check the judge's assertion-id routing actually forces the
  M2 facts for a report so labeled, and try one forgery the pins
  don't already cover.
- **The residual-trust registry note**: verify it exists, names the
  trust surface accurately, and does not claim more binding than
  the code performs.
- **Sweep for remaining run-owned labels** the declaration or
  judge consumes that are bound to nothing: any field of
  `ReproductionRecord` or `RunReport` that the judge does not
  cross-check against observations or construction is a candidate.
  List every one you find, even if you cannot forge a bypass —
  unlisted trust surfaces are how this class survives.

## 4. The packet and the record

Four parts current at the repaired totals; §9 entries complete;
`status.md` accurate; verdict fills remain **suspended** — the
re-close restores them only if everything above holds, and the
repeated final audit still precedes delivery per the F6 gate. If
you close: three acts per `slice-review.md` §5, and state
explicitly that the repeated `final-audit-charge.md` audit is the
next gate. If you find anything: mint SD-R8-F21+ and leave the
round open.
