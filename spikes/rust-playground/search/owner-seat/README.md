# owner-seat — what the owner actually asks of the stash

Status: not started — 2026-09-13

Headline: (none yet)

## Question

P2: the validating consumer is real use, and the owner's use is the
product. In the owner's own words, what questions does the owner put to
their stash — to price and list, to gear a character, to clear junk, to find
what a trade query would match — and for each, what shape of answer
they want (a count, a list, one item, a table by tab), how they answer
it today (a C++ filter, by hand, not at all), and which tracks' facts
it touches. Written by the owner, before the agent's seat is drafted,
so the agent's questions are checked against a human's rather than
invented.

## Inputs

| Source | Access | Supplied by |
| --- | --- | --- |
| The owner, in their words | conversation, recorded verbatim | the owner |
| `../cpp-search/data/filters.toml` (what the app can already answer) | this directory | the cpp-search track |

## Outputs planned

- `data/questions.csv` — one row per question, verbatim: purpose,
  answer shape, today's route, fields and derivations it needs, the
  track that holds them. They double as acceptance scenarios later,
  beside the agent's.
- The findings table: what the questions have in common, what none
  of the three existing vocabularies names, and what the answer shapes
  demand of the read model.

## Findings

## Open questions

## Provenance
