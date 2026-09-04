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
first — that is the parked publishing item in `CONTEXT.md`. The API
reference (`pathofexile.com/developer/docs`) and the API itself are
sanctioned and need no row; what GGG's own client writes into item
notes reaches us through the API and is cited as `game:` evidence.

| Surface | Status | Terms exposure | Access method | Cadence | Yields |
| --- | --- | --- | --- | --- | --- |
| The trade site, `pathofexile.com/trade` (its price-note formats, currency vocabulary, listing display, the About page) | **registered 2026-09-03**; first dated read 2026-09-03: the owner saved `/trade/about` in a browser (kept locally in `pricing-info/`, gitignored — GGG's text is quoted in claims, never committed whole); the agent read the saved copy | GGG's site terms; not part of the API reference; no published terms for automated reads | `browser` — a human reads it and records a dated observation. **No automated fetch**; a tool may only *propose* rows from what a human recorded | on demand: the bulk-exchange item list (parked, `decisions/pricing.md`), the seller-account search as the oracle for the listing state | dated observations cited in `crates/acquisition-plan/reference/currency-v1.toml`; T1–T4 (`docs/design/trade-ground-truth.md`) |
| The trade site's internal static-data endpoint (`/api/trade/data/static`) | **rejected as a tooling source** (packet §1(d), 2026-09-03) | not in the API reference; an undocumented internal endpoint | `browser` only, as evidence for a human-committed row | — | proposals for the currency table, never the table itself |
| The forum shop threads, `pathofexile.com/forum` | known; **not used by the spike**. The owner posts by hand: one character item linked and priced on 2026-09-03 (T7, T8); automated posting and bumping are parked (`CONTEXT.md`) | credentialed session (POESESSID) for posting; reading is public | `browser` — the owner posts what `shop render` writes to stdout; tooling sends nothing (C74) | one hand experiment per open matrix cell (`PRICING-SLICE.md`, "Observations still open") | `T<n>` claims on link codes, resolution at post time, indexing and visibility |
| PoE Wiki, `poewiki.net` (the forum-listing guide) | **corroboration only**, read 2026-09-03 from a copy the owner saved (`pricing-info/`, gitignored; CC BY-NC 3.0) | the wiki's licence | `browser`; a third party's reading of GGG's surfaces, never authority | — | T14, cited as wiki, dated |
| Procurement, `github.com/Procurement-PoE/Procurement` | **corroboration only**, read 2026-09-03 through the GitHub API (last release 1.29.2, 2022-12-22) | Artistic-2.0 | a public code read; a third party's prior reading of the same folklore as the C++ app | — | T15; nothing the C++ code did not already say |
| Third-party price feeds (market and suggested prices) | **not registered** — parked (`decisions/pricing.md`) | each feed's own terms | — | — | — |
| RePoE static game data | known; used by the C++ app on `master` for mod data, **not by the spike** | the project's own licence | — (register with an access method when a spike consumer appears) | — | — |
