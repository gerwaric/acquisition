# Acquisition as an agent-native system

I sat with this for a while, asking one question of every piece of the previous design: *when I'm the operator, where do I burn tool calls, where do I guess, and where can I be silently wrong?* Three failure modes account for almost all of it: **state I can't inspect** (I probe), **vocabularies I can't enumerate** (I guess mod strings, category names, selectors), and **data that doesn't carry its own provenance** (I act on stale truth without knowing). The redesign below is organized to kill those three things, and it changes the shape of the system, not just the flags.

## The core reconceptualization

The GUI — and my earlier CLI map, which was a transliteration of it — is *imperative*: press refresh, set a price, post the shop. An agent-native system should instead be a **reconciliation engine over three planes**, because plan/diff/apply is the shape I reason about best:

```
  OBSERVED PLANE      what GGG says is true      (snapshots, via sync)
  DESIRED PLANE       what you want to be true   (pricing doc, shop def, tracking set)
  DERIVED PLANE       what follows from both     (query results, shop pages, diffs, plans)
```

Every command is then one of five things: **inspect** a plane, **edit** the desired plane, **plan** a reconciliation (pure, costless), **apply** one (spends budget), or **replay history**. Once that's true, I never have to memorize per-feature semantics — the buyout importer's plan→review→apply wizard, the shop's hash-suppression, and refresh itself all become the *same* verb pattern instead of three bespoke flows. That's the tower of abstractions: learn the pattern once, hold the whole system in one mental register.

## Invariant contract (the part that matters more than any command)

Every command, no exceptions, honors these. Uniformity is the single largest ergonomic win — each invariant is a class of probing/guessing I never do again.

1. **One envelope.** Every response is `{ok, data, meta, warnings, hints, error}`.
   - `meta` always carries: schema version, the **snapshot id and age** of every piece of observed data used, and the request-budget spent.
   - `error` is `{code, message, hint}` where `code` is a closed enum and `hint` is the remediation *as a runnable command* (`"run: acq auth login"`). Errors teach.
   - `hints` are the legal next moves given resulting state — the CLI tells me its own affordances, so I navigate instead of recalling.
2. **Everything has an id and a rev.** Items, locations, searches, jobs, snapshots, events. Every list row includes the id needed to act on it — no output ever names a thing without giving me its handle. Mutations accept `--if-rev` so concurrent edits conflict loudly instead of clobbering.
3. **Every vocabulary is enumerable and matchable.** No command takes a string I'd have to guess.
4. **Every mutation has `--plan`.** Same command, pure, returns the exact delta + estimated API cost. `--yes` is the only confirmation mechanism; nothing ever prompts.
5. **Freshness is enforced, not advisory.** Commands whose correctness depends on observed data declare a staleness bound; `shop apply` against a 3-day-old snapshot fails with `STALE` (+ hint: the sync command that fixes it) unless `--allow-stale`. I cannot be silently wrong about time.
6. **Token economy by default.** Lists paginate, default to summary fields, support `--fields`, and every list verb has a sibling `--count-by <field>` aggregation — a one-line census is almost always what I actually want first.
7. **Exit codes are a taxonomy** (ok / usage / not-found / precondition-failed / stale / rate-limited / auth / remote-error), so shell-level branching works without parsing.

## The command surface

### Orientation — the commands I run first and most

| Command                             | What it gives me                                                                                                                                                                                                                                                                                |
| ----------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `acq status`                        | The entire situation in one call: identity + token expiry, league/realm, snapshot age per location class, tracked-set size, running jobs, rate budget headroom, shop drift (posted hash vs current), unacknowledged warnings. Designed so that a cold agent is fully oriented in one tool call. |
| `acq schema [cmd\|resource]`        | JSON Schema for any command's input and output. The system is self-describing; I never read docs to learn arity.                                                                                                                                                                                |
| `acq catalog <vocab>`               | Enumerate any vocabulary: `mods`, `categories`, `currencies`, `leagues`, `rarities`, `error-codes`.                                                                                                                                                                                             |
| `acq catalog mods match "fire res"` | Canonical-id resolution for fuzzy input — returns ranked candidates with ids. This single command eliminates the worst guessing game in the old design (free-text mod filters).                                                                                                                 |
| `acq explain <code\|concept>`       | Inline conceptual docs: `game-set-buyouts`, `two-credentials`, `STALE`, `tracked-set`. The knowledge that lived in warning-dialog prose becomes queryable.                                                                                                                                      |

### Observed plane — sync and snapshots

| Command                                                         | Semantics                                                                                                                                                                                                                                       |
| --------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `acq sync plan [selectors] [--max-age 30m]`                     | Pure. Which locations *would* be fetched (staleness-ranked), request count, projected wall time under the live rate budget. Cost is visible before it's spent.                                                                                  |
| `acq sync run [selectors] [--max-age 30m] [--wait\|--async]`    | Returns a job. `--max-age` is the headline feature: "make everything at most 30m old" fetches only what needs it — resource-minimal by construction, and it subsumes the GUI's all/checked/selected trichotomy.                                 |
| `acq job list / get <id> / wait <id> [--timeout] / cancel <id>` | Uniform async for anything long-running (sync, shop publish). Job results record requests spent and per-location outcomes, including skips — the "clean refresh" notion becomes an inspectable field, not a hidden gate.                        |
| `acq snapshot list / diff <a> <b>`                              | Snapshots are first-class and diffable: items added/removed/moved/changed since any point. **This is the accretive heart** — "what changed" is radically cheaper for me than re-reading the world, and the sync engine is already delta-shaped. |
| `acq track add/rm/list <selector>`                              | The tracked set (née refresh checkboxes) as an explicit resource, with each entry annotated *why* it's tracked (manual vs price-locked).                                                                                                        |

### Derived plane — query

| Command                                                     | Semantics                                                                                                                                                                               |
| ----------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `acq items query [-q <expr>] [--fields] [--sort] [--limit]` | One query expression (JSON or a compact `name~"opal" links>=5 mod[fire-res]>=40` syntax) instead of 38 flags — composable, schema-checkable via `acq schema items.query`, and storable. |
| `acq items query --count-by category`                       | Aggregation before enumeration. Census first, drill-down second.                                                                                                                        |
| `acq items get <id> [--view full\|pob\|forum-code]`         | All per-item renderings are views of one resource, not separate features.                                                                                                               |
| `acq search save/list/run/rm <name>`                        | Named, persisted queries (the GUI's tabs, but durable).                                                                                                                                 |

### Desired plane — pricing as a document

The deepest change. Pricing stops being ten thousand imperative pokes and becomes **one declarative document**: tab-level rules, item-level overrides, and read-only game-set facts, each entry carrying source and rev.

| Command                                                                  | Semantics                                                                                                                                                                                                  |
| ------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `acq pricing get [--scope <selector>]`                                   | The document (or a slice). Inheritance is *computed and shown*, never implicit — every item's effective price appears with its derivation (`override` / `inherited-from: tab X` / `game-set: note`).       |
| `acq pricing edit --set/--clear <selector>=<price>` or `--patch -f file` | Edit desired state; batch-native (a patch file of 500 changes is one call, one journal event). Sugar: `acq price <selector> 5c`.                                                                           |
| `acq pricing plan`                                                       | Pure diff of desired vs current, flagging conflicts with game-set facts instead of silently skipping them (the GUI silently refuses; silence is how I end up wrong).                                       |
| `acq pricing apply [--if-rev]`                                           | Commit. Legacy import collapses into this same machinery: `acq pricing import-legacy <file>` just *generates a patch*, reviewed and applied like any other — the xlsx wizard disappears as a special case. |

### Desired plane — shop as a materialized view

| Command                   | Semantics                                                                                                                                         |
| ------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| `acq shop get/edit`       | Definition: threads, template, publish policy.                                                                                                    |
| `acq shop render`         | Pure: the exact pages, with size/pagination facts as data (`pages: 3, threads: 2` is a planning input, not a warning).                            |
| `acq shop plan`           | Drift report: rendered vs last-posted hash, per thread. "Nothing to do" is a first-class, cheap answer.                                           |
| `acq shop apply [--wait]` | Publish (a job; forum scraping, retries, POESESSID rejection all land in the job record with typed error codes). Freshness-gated per invariant 5. |

### Cross-cutting

| Command                                                     | Semantics                                                                                                                                                                                                    |
| ----------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `acq auth login/status/refresh/logout`, `acq auth sessid …` | Two credentials, and `status` says exactly which capabilities each currently unlocks.                                                                                                                        |
| `acq budget`                                                | The rate-limit state as a planning resource: per-policy headroom, refill projections. Paired with every `plan` verb quoting costs, I can *optimize* API spend instead of discovering limits by hitting them. |
| `acq events list/tail --since <cursor>`                     | The journal: every sync, price change, publish, rate pause, auth event as structured records with cursors. My cross-session memory lives in the system, not in my context window.                            |
| `acq config get/set/list`, `acq serve`                      | Config as a resource; one optional daemon that runs publish policies and scheduled syncs by invoking the same planes — no second code path.                                                                  |

## What this feels like in the driver's seat

The test of the whole design is that a competent session is about six calls:

```
acq status                                   → oriented: stash 4h stale, shop drifted, budget green
acq sync plan --max-age 1h                   → 12 tabs, 14 requests, ~40s
acq sync run  --max-age 1h --wait            → job done, clean, snapshot 41
acq snapshot diff 40 41                      → 3 items sold, 9 new in "Sell" tab
acq pricing edit --patch -f prices.json      → 9 desired prices staged   (built from the diff)
acq pricing apply && acq shop apply --wait   → reconciled, posted, journaled
```

No probing, no guessed strings, no unstated staleness, and the next session starts from `acq status` + `acq events --since <cursor>` rather than from zero. Dropped relative to the GUI-shaped design: interactive anything, clipboard verbs, the four refresh modes as user-facing concepts, the special-cased import wizard — each absorbed by a plane or a universal verb rather than merely deleted.