# Item search, step 9c3 — the brief for an outside review

For a reviewer who has the repository and nothing else. It holds the
boundary of the look and no more: what is under review, what it is held
against, what a finding is. What the build claims is in the code's own
documents and is not restated here. This file is deleted when the
look's findings are routed, and cited by hash from then on.

## What is under review

The four commits `23991fe1..8506cc4b` on `spikes/rust-playground`: 19
files. Every finding is against `8506cc4b`: the one commit after it is
this brief's, which touches no other file, and nothing else lands until
the findings are in.

What the step was to build, and what it was to close on, is its row in
`search/BUILD-PLAN.md` at `1662e8ab`; the row left the plan at the
step's close.

## What to read

`AGENTS.md`, "Read before changing anything", in its order, for item
search. For this step beside it:

- the module docs of `crates/acquisition-search/src/totals.rs` and
  `pseudo.rs`, and the header of `reference/totals-v5.toml`;
- `search/pseudo-stats/README.md`, the rows on the four pseudos that
  are no sum, and `tools/trade_rows.py`, `DERIVED`;
- the four commits' messages.

## What it is held against

- The rulings: `decisions/search.md`, C93, C94 and C101 first, and
  `CONTEXT.md`.
- The language reference, `search/DESIGN.md`.
- The plan's rules, `search/BUILD-PLAN.md`, "How a partial build stays
  honest".
- The shapes of fault in `SEARCH-SLICE.md`, "Findings": the checklist.
- What the trade site showed: `search/pseudo-stats/data/captures.json`.
- The answer itself: two parts of one answer that say one thing
  agree, and the text is a function of the JSON (C53).

What the owner has ruled is no finding: `SEARCH-SLICE.md`, "Holes
ruled". What the record already holds as open, "Observations still
open", is no finding repeated; it is one where a fault is shown in it.

## How to look

Through the boundary: a request in, an answer out, by the crate's
`answer` or by `acq search` and `acq show`. Fixture stores are built
through the store's own ingest, as `crates/acquisition-search/tests/`
builds them. The plan's rule 7 binds: no traffic to GGG, the owner's
store never opened, `ACQ_STORE_DIR` always set, `ACQ_PROVIDER=mock`.
The quality gate is `AGENTS.md`'s.

The reviewer changes nothing that is committed and commits nothing. A
probe file of its own is welcome beside its findings.

## What a finding is

One fault, numbered, that a reader can reproduce without the reviewer:

1. the request, and the items it was asked of;
2. the answer seen;
3. the answer expected, and the authority that expects it, by id or by
   path;
4. which shape of the checklist it is, or that it is a new one.

A finding names no fix. A doubt that could not be reproduced is
reported apart, as a doubt. A look that finds nothing says what it
asked.
