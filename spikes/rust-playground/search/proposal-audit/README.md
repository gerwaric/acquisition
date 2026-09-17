# proposal-audit — do the two search designs hold against their own page and their evidence?

Status: second audits in after one repair pass (18 findings on note 24's design, 20 on note 25's); first audits at 3a7c86e6 — 2026-09-17

- Stage 4 of the synthesis, the checking step: one auditor per proposal
  under `BRIEF.md` writes `audit-24.md` and `audit-25.md` — marks,
  citations, a counted cold start, findings that each point at a line.
  No opinion of either design lives here.
- Each author then repairs its own proposal against its audit, one
  pass (`brainstorming-notes/32-search-repair-brief.md`): fixed, listed
  as a gap, or disputed with the line. The audits are not re-run.
- `anonymise.py` writes the copies the reviewers use, after the repair
  pass. Reviewers never read this directory.
