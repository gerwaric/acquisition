# proposal-audit — do the two search designs hold against their own page and their evidence?

Status: closed — two audit rounds, two repair passes; the reviews read `search/designs/`, retired with the briefs at the stage-5 audit's cleanup (last at `548ecc4f`) — 2026-09-17

- Stage 4 of the synthesis, the checking step: one auditor per proposal
  under `BRIEF.md` (deleted at the close, at `a47fa1e5`) writes `audit-24.md` and `audit-25.md` — marks,
  citations, a counted cold start, findings that each point at a line.
  No opinion of either design lives here.
- Each author then repairs its own proposal against its audit, one
  pass (note 32 at `a47fa1e5`): fixed, listed
  as a gap, or disputed with the line. The audits are not re-run.
- `anonymise.py` (at `548ecc4f`) wrote the copies the reviewers used,
  after the repair pass. Reviewers never read this directory.
