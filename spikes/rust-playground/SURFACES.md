# Governed surfaces — the register under C79

C79 (`CONTEXT.md`): surfaces GGG does not sanction — the trade site, the
forums, third-party feeds — are governed inputs, never runtime
dependencies, and permission attaches to the access method. This file
is the register that ruling names. One row per surface: its status,
its terms exposure, the access method permission is granted for, the
cadence, and what it yields. An entry changes only by a commit that
says why. The daemon never touches any of these; no store read, plan
compile or apply depends on one; tooling fetches one only from an
official export or with the permission its row records; what a surface
yields lands as `T<n>` claims (`docs/design/trade-ground-truth.md`,
master-side) or reviewed reference data with sources cited (C68). A
surface used as an *effect* (posting) needs its own boundary session
first — that is the parked publishing item in `CONTEXT.md`.

| Surface | Status | Terms exposure | Access method | Cadence | Yields |
| --- | --- | --- | --- | --- | --- |
| The trade site, `pathofexile.com/trade` (its price-note formats, currency vocabulary, listing display) | **registered 2026-09-03**, pricing step 2d | GGG's site terms; not part of the API reference; no published terms for automated reads | `browser` — a human reads it and records a dated observation. **No automated fetch**; a tool may only *propose* rows from what a human recorded | on demand: the currency table's alias and emitted-form columns (C68), the forum matrix's price-format cells (step 8) | dated observations cited in `crates/acquisition-plan/reference/currency-v1.toml`; `T<n>` claims |
| The trade site's internal static-data endpoint (`/api/trade/data/static`) | **rejected as a tooling source** (packet §1(d), 2026-09-03) | not in the API reference; an undocumented internal endpoint | `browser` only, as evidence for a human-committed row | — | proposals for the currency table, never the table itself |
| The forum shop threads, `pathofexile.com/forum` | known; **not used by the spike**. Manual posting with junk items in a disposable thread is step 8's instrument; automated posting is parked (`CONTEXT.md`) | credentialed session (POESESSID) for posting; reading is public | `browser` — the owner posts by hand; tooling renders text, sends nothing (C74) | step 8's matrix, run once and repeated once | `T<n>` claims on indexing, visibility and mechanics |
| Third-party price feeds (market and suggested prices) | **not registered** — parked (`decisions/pricing.md`) | each feed's own terms | — | — | — |
| RePoE static game data | known; used by the C++ app on `master` for mod data, **not by the spike** | the project's own licence | — (register with an access method when a spike consumer appears) | — | — |
