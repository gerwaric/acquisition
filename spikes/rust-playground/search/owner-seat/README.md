# owner-seat — what the owner actually asks of the stash

Status: first pass complete — 2026-09-13. The owner answered five prompts in conversation; seven questions and four answers are verbatim in `data/questions.md`, the agent's reading columns accepted uncorrected.

Headline:
- Seven questions, three kinds: find one remembered item (by name, by league of origin, by an unusual mod — Q2, Q4, Q6), which items carry mod X (Q1, Q3, Q7) and a set by level (Q5). None filters on defence or damage numbers: "well-crafted endgame gear is already on bases with the right armour or evasion" (F1).
- The mod is asked for by identity, not by total: "a crafting base with a specific fractured modifier, or a key build-enabling modifier, or a stat that is needed to complete a defensive layer" — the line-identity question, stated from the seat (F1).
- Pricing and organising are not the owner's questions: pricing is other users' (buyouts survive upgrades; no direct forum updates, "because of the use of POESESSID"), and the app cannot act on the stash. Counts by kind are "useful, or at least interesting" (F2).
- What the trade site does better, and where both fail: yes/no/any flags; a stat autocomplete that is instant, categorised by mod kind and sensibly ordered ("The c++ mod search box is terrible by comparison on both fronts"); boolean logic — the site only over stats, never "(Armour > 1000) OR (Required Level < 80)" (F3).
- Wanted: a trade search run against the stash, "fantastic", by copy/paste or a browser add-on; pricing through Awakened PoE Trade from within the app, asked for by newer players. Neither is core (F4).

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

- `data/questions.md` — one row per question, verbatim: purpose,
  answer shape, today's route, what it needs and which track holds it;
  then every answer verbatim. They double as acceptance scenarios
  later, beside the agent's.

## Findings

**F1 — The questions** (`data/questions.md`, the table). Three kinds. *Find one item I remember* (Q2 a legacy unique, Q4 "that staff from Cruicible league", Q6 "that legacy explode chest"): by name, by league of origin — which no item field carries (item-facts F2) — or by an unusual mod; the answer is one item and its tab. *Which items carry mod X* (Q1 resist or attribute lines for a build, Q3 gear for a new interaction, Q7 "the modifier GGG just anounced is going away"): the mod by identity — fractured, build-enabling, a defensive-layer stat — sometimes as a total; the answer is a list with the lines visible and the tab. *A set by level* (Q5 leveling gear). None filters on armour, evasion or damage numbers; for other players "the modifiers on their weapong" and damage mods by mechanism — hit, spell, over time, ailment — matter. Legacy-ness (Q2, Q6) is a mod value outside the current range, which needs unique data the repoe extract leaves out.

**F2 — Not the owner's** (the prompt-2 and prompt-3 answers). Pricing: the consumer is a small set of other users with unknown workflows; their two known needs are rulings already (C35; the parked publishing item, now carrying the POESESSID reason). Organising: the app cannot move items or rename tabs, so the owner does not use it for that; counts and totals by kind — currencies, equipment, maps, fragments — are a candidate derivation, not a need (the parked currency-totals item, `decisions/pricing.md`).

**F3 — The trade site, and the shared gap** (the prompt-4 answer). Better on the site: tri-state flags; the stat autocomplete — instant, distinguishing implicit, explicit, pseudo, fractured and the rest, ordered sensibly; boolean composition `(A or (B and C))`. Missing on both: composition across fields — "(Armour > 1000) OR (Required Level < 80)" — the site composes stats only (trade-query F2), the C++ app ANDs everything (cpp-search F1).

**F4 — Wanted, not core** (the prompt-5 answer). A trade search run against the stash — the URL is the query (trade-query F9) — by copy/paste or a browser add-on; Better Trading, an extension that manages trade searches, as a possible integration, "fun, but not a core features". Pricing through Awakened PoE Trade from within the app (GUI, CLI or MCP), asked for by newer players — a dependency on a governed surface (`SURFACES.md`) and a design topic, not a search requirement.

## Requirements (one line each, the failure it prevents)

| # | Requirement | Prevents |
| --- | --- | --- |
| R1 | A line has an identity finer than its text: its kind (implicit, explicit, fractured, crafted, pseudo, …) and, where a mod is named, the mod | Q1, Q3, Q7 missing a mod that renders several ways, or a template that hides its kind |
| R2 | Boolean composition across every field, not stats alone | the gap both existing searches share (F3) |
| R3 | Tri-state flags: yes, no, any | the C++ checkbox that can only demand |
| R4 | The mod vocabulary served instantly, categorised by kind, ranked sensibly — a read-model property | the C++ dropdown (F3); the same need as an agent's schema discovery |
| R5 | A trade search, as a URL or its JSON, is a stash query; the join is the trade id → stat id map (repoe F1, F2) | re-typing a query the site already holds (F4) |
| R6 | Every answer carries the item's location — tab or character, realm, league | Q2, Q4, Q6 answered without "where" |
| R7 | League of origin is not promised: no field carries it; a derivation from league-specific mods or first-seen is the most a search can offer | Q4 answered wrongly |
| R8 | Legacy detection: a unique's mod values against the current version's ranges | Q2, Q6; needs the uniques data (repoe) |
| — | Non-goals: acting on the stash; direct forum updates; a pricing engine | F2, F4 |

## Open questions

- **Q1 — Legacy, operationally.** A mod value outside the current range, a mod the unique no longer rolls, or both? Closes: the repoe unique-mods extract against one known legacy item of the owner's.

## Provenance
