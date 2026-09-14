# agent-seat — what an agent needs to search well

Status: first pass complete — 2026-09-14 (one fresh session sat in both seats over a read-only copy of the owner's store; the brief is `e9b2b92c`, deleted at the close)

Headline:

- Of the twelve questions, the CLI seat answered all twelve and the MCP seat five: the CLI's only route to the other seven is pulling the whole corpus to a file (63.5 MB, 2.3 s) and filtering with jq, and the MCP seat has no file. The surface has one predicate (a substring over name, type line, base type) and returns whole GGG bodies.
- Nothing can be learned about an item before pulling one: no field list, no value sets, no mod vocabulary. The first row is the schema (F1).
- Counts do not exist before rows: the only count surface is `tabs`, 1 MB flat for one league, and a league is not enumerable (F4). Truncation is silent: 50 of 81 with no total (F3).
- Under read-only SQL every question became one call (two for Q5, Q7) on both files; the store copy costs a JSON body idiom per question (a number inside text, string-or-object entries, a UNION per array), and the projection cost three silent misses on its template convention and could not answer links or name a character (F12).
- Nineteen R-lines follow the owner's eight; R9 is the owner's R4 and R13 is R1+R2 — the agent's needs and the owner's are one need with a different consumer.

## Question

From the driver's seat: what does an agent need from the search surface — schema discovery, facets and counts before rows, deterministic sort and stable ids, an explain, refining a previous query, errors that name the field — and where does it fail? Written from experience over 22,721 items, both seats, then again under SQL.

## Inputs

| Source | Access | Supplied by |
| --- | --- | --- |
| `MCP-REFERENCE.md`, `CLI-REFERENCE.md`, `decisions/frontends.md` (C53) | the repo | — |
| the spike store, a read-only copy (22,721 items, 3,444 tabs, 65 characters) | `MANIFEST.md` | the owner (approved 2026-09-13, copied 2026-09-14) |
| `../engine-bench/raw/seat-projection.db` (both stores, 36,139 items; phase two only) | `MANIFEST.md` | engine-bench, `93c3a63b` |
| `../owner-seat/data/questions.md` — Q1–Q7 verbatim | this directory | owner-seat |

## Findings

**F1 — Orientation from zero yields the container and nothing of the item.** From the tool list and `acq --help` alone: the corpus size (`store_status`), tabs with counts (Standard only, the league must be named), characters with counts (the leagues were read off their `league` column: the store's own league table is empty), the realm vocabulary (in three descriptions), and one shipped vocabulary, the currency table. No field of an item, no rarity or class values, no mod. The body is "verbatim as GGG returned it", and the first row (call 5, 3.5 KB: a gem, `rarity: null`) is the only schema.

**F2 — One predicate.** `text` is a case-insensitive substring over name, type line and base type; `league`, `realm`, `include_removed` filter; `limit` cuts. No rarity, class, level, mod, tab, sort, offset, count. Order is deterministic (ingest order: characters first) and undocumented.

**F3 — The row is the body, and truncation is silent.** A row is the store's columns plus the whole GGG JSON: 50 default rows are 91.5 KB compact (MCP) or 149 KB pretty (CLI); everything is 38.8 MB / 63.5 MB. `search_items {text:"Staff"}` returned 50 of 81 with no total and no flag (Q4).

**F4 — Facets come from the dump.** Per tab: `tabs` (1,006,740 B pretty, 745,672 B on the MCP with no limit parameter; the text view lists all 3,349 lines, against C53's ten-listed rule). Per league, per rarity, per anything else: the dump and jq. `store_status` says `leagues: 0`.

**F5 — The two seats differ by one file.** The CLI can redirect: every dump question was answered by `items search "" --limit 100000 --json` once, then jq at 0.8–1.5 s a question. The MCP seat cannot: Q3, Q5's items, Q6, Q7, A1's league and rarity counts, and A5 have no route; the nearest attempt, `search_items {text:"Rarity"}` for Q7, returned three support gems.

**F6 — No handle to refine.** A2 ran as jq over the saved file on the CLI; on the MCP it is the same 31.8 KB call again, or the agent's eyes over what is already in context.

**F7 — Nothing explains.** A row has no matched-on field; an empty answer is `[]` (JSON) or `0 item(s)` (text), never "no name, type line or base type contains it; mods are not searched" — which only the description says. `items search Explode` is empty on both seats while 32 items carry an explode line.

**F8 — Ids round-trip only in JSON.** A full id is one exact call on both seats (A4). The text search view prints 10-character prefixes that `items show` refuses (`no item 1d57e63d`), so the decision view's ids are not handles. Tab ids are 10 characters, character ids 64.

**F9 — Every "where" is a join.** Rows carry a location id, never its name; naming Q2's four tabs cost the 1 MB tabs output on the CLI and would cost 745 KB into context on the MCP.

**F10 — Errors.** Every wrong thing was named but one: an unknown league is `[]` on both seats (silent), an unknown realm lists the four, `--limit 0` is empty, a negative limit is a type error, a missing `text` a deserialize error, an unknown id `no item X`, an unknown account names `acq accounts`. The MCP server registered with the brief's literal `…` as `ACQ_STORE_DIR` answered every harness call with `no accounts known (run acq auth)`: the one action this run forbids, and not the directory it looked in. The MCP seat was therefore driven over stdio (`scripts/call.py`), the same binary and answers; `--into context` reproduces the cost.

**F11 — The MCP text (C53), right and wrong.** Right: the store tools name no daemon and no network; `search_items` names its three fields and that socketed gems are rows; realm values are enumerated; `characters` is exact. Wrong or missing: `tabs` promises a "tree" and returns a flat array; nothing says a row is ~2 KB of body, that `limit` truncates without a total, or that `tabs` has no limit; `get_item`'s "verbatim" row is store columns plus `json`. The instructions block spends its length on the refresh loop and jobs, two sentences on reading.

**F12 — SQL: one call each, at a price.** Store copy: every question one call (Q5, Q7 two); the DDL (3.9 KB) sufficed only because phase one had shown the rows — its comment says `location_id` is "tab/substash id, or character name" and the rows hold character ids. The body's idioms: a number inside display text (`substr`/`instr`/`CAST`, A2, A5), entries that are objects in one array and strings in another (`json_each`'s `type` column, A5), "every array" as a `UNION ALL` per array name that the schema does not list (A5, nine lines, 196 ms), class absent (Q6 by base). Projection: one indexed template equality answers A5 in 2 ms and Q7 in 3 ms — after a check, because its template folds the sign into the number (`# to maximum Life`, not `+# to …`) and three queries returned nothing, silently, before that was read; it has no sockets (Q3: `no such column: links`), names a character location by its id (Q6, Q7), and merges two stores with no column to tell them apart. Mistakes counted: phase one two (`location_kind` guessed `tab`; a tab id cut to 8), phase two five (three silent, one error, one comment not tripped).

**F13 — The temptation.** In phase one, at A1, A5 and Q7, the query I would have typed was `SELECT league, rarity, count(*) FROM items GROUP BY 1, 2` and a `json_each` over `explicitMods`; it is recorded per question in `data/questions.json`.

## Numbers (from `data/calls.csv`: 112 calls — 29 CLI, 23 MCP, 22 jq, 38 SQL)

| Q | CLI: calls / B (to file) + jq | MCP: calls / B into context (measured) | SQL store: calls / B / ms | SQL projection: calls / B / ms (errors) |
| --- | --- | --- | --- | --- |
| Q1 | 3 / 108,512 + 1 | 1 / 31,844 | 1 / 1,054 / 7 | 2 / 2,228 / 54 (0, one silent) |
| Q2 | 1 / 20,512 + 2 | 1 / 13,866 | 1 / 149 / 0 | 1 / 369 / 11 |
| Q3 | dump + 1 | none (dump would be 38,820,498) | 1 / 14 / 53 | 1 / 29 / 0 (1: no sockets) |
| Q4 | 1 / 325,852 + 3 | 1 / 178,328 (default 50 of 81) | 1 / 482 / 8 | 1 / 487 / 4 |
| Q5 | tabs 1,006,740 + dump + 3 | tabs 745,672, then none | 2 / 151 / 33 | 1 / 211 / 5 |
| Q6 | dump + 2, price 2,158 | none | 1 / 125 / 11 | 1 / 322 / 8 (id, not name) |
| Q7 | dump + 2 | none (5,105: the wrong thing) | 2 / 183 / 53 | 2 / 146 / 3 |
| A1 | tabs + dump + 5 | tabs 745,672; league filter 1,948,001 | 2 / 296 / 29 | 2 / 530 / 15 |
| A2 | 0 + 1 | 0 (re-read of Q1) | 1 / 38 / 19 | 2 / 51 / 6 (one silent) |
| A3 | 1 / 3 | 1 / 2 | 1 / 46 / 37 | 1 / 23 / 19 |
| A4 | 1 / 3,567 | 1 / 2,155 | 1 / 79 / 1 | 1 / 124 / 3 |
| A5 | dump + 1 | none | 1 / 19 / 196 | 3 / 55 / 7 (one silent) |
| dump | 63,534,917 B, 2,335 ms | 38,820,498 B, 5,644 ms | DDL 3,879 B | DDL 3,432 B + template check 235 B |

## Requirements (numbered on from owner-seat's R1–R8; one line, the failure it prevents)

| # | Requirement | Prevents | Same need as |
| --- | --- | --- | --- |
| R9 | The schema and its vocabularies are readable before any row: fields, value sets (rarity, class, realm, league, container), mod templates ranked by count | F1: the first row as the schema | R4 (the owner's dropdown is the agent's discovery) |
| R10 | A count mode: facets by tab, league, rarity, class, template, returning no bodies | F4: 1 MB or the dump for a number | the owner's "counts and totals" candidate |
| R11 | Every truncated answer says total, returned and how to continue | F3: 50 of 81, silently | — |
| R12 | The caller names the fields; the default is a decision view (id, name, base, rarity, location name, the matching lines), never the body | F3: 31.8 KB for twenty gloves | C53's decision view |
| R13 | Predicates over derived facts — rarity, class, level, links and colours, a mod by template with a numeric value across every array — composed with boolean logic | F5: seven of twelve unanswerable without the dump | R1, R2, and the owner's "specific values" |
| R14 | A query composes: the refinement is the previous query plus one predicate (a query object, not a text argument) | F6 | R2 |
| R15 | Explain: a matched-on per row, an empty answer that names the scope searched, a why-not for one id | F7 | — |
| R16 | The location's name on the row, beside its id | F9 | R6 |
| R17 | Every id the surface prints is one `show` accepts: full ids in text, or prefix lookup | F8 | — |
| R18 | Errors name the wrong value and the fix that applies here: an unknown league lists the known ones; a missing store names the directory | F10 | — |
| R19 | If a lines table is ever exposed (C48 stands), its template convention and the location names are in the schema, or the miss is silent | F12 | — |

**The projection's requirements** (for engine-bench's `--seat` mode): sockets, links and colours; the character's name where `tab_name` now holds its id; a source-store column; the verbatim line beside the template, for an explain; the template convention stated in the DDL ("the sign is part of the number run"); `first_seen`.

## Open questions

- Does R12 alone make the MCP seat viable at this scale? Experiment: a `fields` parameter prototyped on `search_items`, the twelve re-run from `data/questions.json`, bytes compared to the table above.
- Do R10 and R13 make each of the owner's seven one call? Experiment: the design's query object written for each `wanted_query` in `data/questions.json` and dry-run against the projection.
- The `…` registration trap: a second occurrence makes it a line in the mock-session skill; one is a manifest note.

## Provenance

| Artifact | Source | Manifest row |
| --- | --- | --- |
| `data/calls.csv` | every call of the run, `scripts/call.py` | `raw/calls/` |
| `data/questions.json` | the twelve, both phases, hand-written during the run | — |
| `scripts/phase2.sh`, `scripts/q2_where.jq` | the SQL and the one jq join kept as files | — |
| the store copy, the annotations copy, `accounts.json` | `sqlite3 .backup` by the owner, 2026-09-14 | `MANIFEST.md` rows 1–3 |
| the seat projection | read only, not copied | `MANIFEST.md`, last paragraph |

## Review

| # | Finding | Fix |
| --- | --- | --- |
