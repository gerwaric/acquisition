# agent-seat — what an agent needs to search well

Status: not started — 2026-09-12

Headline: (none yet)

## Question

From the driver's seat over a few hundred thousand items: what does an
agent need from the search surface — schema discovery, facets and counts
before rows, deterministic sort and stable ids, an explain, refining a
previous query, errors that name the field — and where does it fail (a
query pulling fifty thousand rows into context)? Written from
experience, checked against how the existing MCP tools shape output
(C53's views).

## Inputs

| Source | Access | Supplied by |
| --- | --- | --- |
| `MCP-REFERENCE.md`, `CLI-REFERENCE.md`, `decisions/frontends.md` (C53) | the repo | — |
| The trade grammar and the C++ catalogue, once they exist | this directory | the other tracks |

## Outputs planned

- `data/questions.json` — a dozen concrete questions an agent would ask,
  each as a candidate query object with its expected result shape; they
  double as acceptance scenarios later.
- The requirements list, one line each, with the failure it prevents.

## Findings

## Open questions

## Provenance
