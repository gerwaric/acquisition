# 32 — The repair: brief for the author's pass in stage 4 of the synthesis

You wrote one of the two search proposals — Fable, note 24; Astra, note
25 — or you are a fresh session of the model that did, which is the
same thing here. Your proposal has been audited: checked line by line
against its own model page, its citations and its brief. Users will
work in your design next, and the owner will rule after that. This is
your one chance to make sure they meet the design you meant, not the
slips in how you wrote it down.

## What you read

This brief; your audit, `search/proposal-audit/audit-24.md` or
`audit-25.md`; your own proposal; and whatever the proposal brief
(`brainstorming-notes/29-search-proposal-brief.md`) let you read —
`search/DIGEST.md`, `brainstorming-notes/22-search-framing.md`, a
decision or a track README only to verify a claim.

**Never:** the other proposal; the other audit; any other note; the
index `search/README.md`; the code; anything remembered from another
session, and if you are Fable, the memory files on item search. You are
still blind to the other design, and it matters as much as it did.

## What you do

Take the findings one at a time. Each gets exactly one of three answers:

- **Fixed** — the finding is right and the design can carry the fix.
  Make the smallest edit that makes the page true: define the construct
  on the model page, correct the query, re-cite the claim, recount the
  cold start.
- **Gap** — the finding is right and the honest answer is that the
  design cannot do this. Say so where the design claimed otherwise, and
  list it with the design's other gaps. A listed gap is worth more than
  a patch that bends the model to hide it.
- **Disputed** — the finding is wrong. Quote the line of your proposal,
  or the claim, that shows it. A dispute without a line is not one.

Change nothing the audit did not find. This is a repair, not a second
proposal: no new ideas, no restructuring, no polishing of what was not
challenged. If a fix cannot be made without changing the model, that is
a gap, and saying so is the repair. The one limit still holds — the
model and its grammar fit one page — and a fix that breaks it is not a
fix.

## What you hand back

Your proposal, edited in place, with one section appended at the end:

`## Repairs` — one row per finding: the audit's id; fixed, gap or
disputed; the section you changed, or the line you quote. Then one
sentence: what the audit taught you about your own design.

Commit that one file yourself, with a message that counts the three
answers. Touch nothing else. Do not push.
