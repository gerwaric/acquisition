# Acquisition CLI, redesigned as an agent-native system

## 1. What actually costs an agent (the meditation, condensed)

When I drive a system through a CLI, my scarce resources are **turns** (each command round-trip), **context tokens** (each output I must read), **the API rate budget** (the system's own scarce resource), and — the one people forget — **certainty**. Most of my failure modes are not "couldn't do it" but "confidently acted on a wrong model of the world": stale cache mistaken for live truth, a mutation whose effect I assumed instead of verified, a prior session's work I couldn't see.

The v1 design I gave you was a faithful transliteration of the GUI: human verbs behind flags. It would *work*, but it inherits the GUI's operating model — a human sitting in front of it, accumulating situational awareness through their eyes over minutes. I have no eyes and no persistence between sessions. So the redesign optimizes for four properties:

1. **One-call situational awareness** — I should never need five probing commands to know where things stand.
2. **Every answer carries its own epistemics** — freshness, provenance, completeness stamped on every output, so I cannot mistake cached for current.
3. **Mutations return the world, plan before apply** — no read-after-write round trips, no irreversible surprises.
4. **The system remembers so I don't have to** — journal, cursors, notes. Each session accretes onto the last instead of restarting from zero.

And one structural insight: the GUI's best feature, discovered by accident, is the legacy-buyout importer — *plan → reviewable artifact → apply → per-row outcome report*. That pattern shouldn't be a corner of one wizard. It should be the universal grammar for every consequential action.

## 2. The conceptual reframe: four layers, one grammar

Instead of an assemblage of features (refresh, buyouts, shop, currency…), the system is a tower of four abstractions. Every piece of the GUI inventory lands cleanly in exactly one:

```
┌─ EFFECTS ──── touch the outside world: API fetches, forum posts.
│               Always: plan → cost → execute → journaled outcome.
├─ DERIVATIONS ─ pure functions of the layers below: search results,
│               shop pages, currency totals, diffs. Hashed, recomputable, free.
├─ ANNOTATIONS ─ what WE assert: prices, sync policy, saved searches,
│               ratios, notes. Ours to write; never touched by fetches.
└─ FACTS ────── the local mirror of GGG's truth: tabs, characters, items.
                Read-only to us; every fact stamped with as_of + provenance.
```

This isn't taxonomy for its own sake — each boundary kills a class of my errors:

- **Facts vs. annotations** means a sync can never clobber my pricing work, and I can reason about "their truth changed" separately from "our intent changed." (The GUI already lives this — buyouts survive refreshes — but never names it.)
- **Derivations being pure and hashed** means they're free to recompute and cheap to compare. `shop render` twice with the same inputs gives the same hash; if the hash matches what was last published, publishing is provably a no-op. (The GUI's `shop_hash` suppression is this idea, buried.)
- **Effects being journaled operations** means the system, not my context window, is the memory of what happened.

On top of the layers, **universal idioms** — every command speaks them, so learning one command is learning all of them:

- `--json` is the default when stdout isn't a TTY; schemas are versioned and self-served (`acq schema <command>`).
- Every output that includes facts carries `as_of`, `source`, and `complete: true|false` (with a `skipped` list when false). Staleness is data, never a surprise.
- Every mutating command supports `--dry-run` (report what would change, touch nothing) and returns the **post-state** of what it changed (no read-after-write).
- Every effect supports `--plan` (typed plan + request cost + ETA under current rate budget) and lands in the journal with an operation id.
- Batch is native: any `set`-shaped command accepts `-f batch.jsonl` and returns per-row outcomes (`applied / already-set / refused-game-locked / error`), atomically.
- Errors are structured: `{code, message, remedy: {command: "acq auth sessid set", why: "forums do not support OAuth"}}`. The remedy field means a failure is one turn, not a debugging session. Exit codes are enumerated: 0 ok, 2 precondition (auth/config), 3 rate-limited (with `retry_after`), 4 partial, 5 conflict.
- Time and change are addressed by **cursors** — opaque, durable tokens usable across sessions: `--since <cursor>` everywhere.

## 3. The command surface, rebuilt

### The keystone: `acq state`

One command, one small document, complete situational awareness — the fusion of `git status` and a cockpit:

```json
{
  "identity":  {"account": "…", "league": "…", "realm": "pc",
                "oauth": {"ok": true, "expires_in": "8h"},
                "sessid": {"present": false}},
  "facts":     {"tabs": 41, "characters": 12, "items": 8214,
                "freshest": "3m", "stalest": "6d",
                "stale_vs_policy": 7, "items_hash": "a3f1…"},
  "annotations":{"priced_items": 312, "priced_tabs": 4,
                "saved_searches": 5, "notes": 3},
  "shop":      {"threads": 2, "rendered_hash": "9c2e…",
                "published_hash": "9c2e…", "in_sync": true},
  "budget":    {"paused": false, "headroom": "38 req/60s"},
  "ops":       {"running": null, "last": {"id": "op_0142", "verb": "sync",
                "outcome": "complete", "at": "2026-08-31T09:12Z"}},
  "cursor":    "c_8f31"
}
```

That last field matters most: `state` hands me a cursor, and `acq diff --since c_8f31` later tells me *everything that changed while I was away* — new/removed/repriced items, tab changes, ops run by the GUI or another agent. Cold-starting a session is two calls, not fifteen.

### Facts (read)

```
acq tabs list [--stale-over 1h]
acq chars list
acq items search <filter flags as v1> 
        [--fields id,name,price] [--limit N] [--count] [--summary --by tab]
acq items show <id> [--explain]
acq diff --since <cursor|op_id|timestamp> [--filter …]
```

Context economy is built in: `--count` and `--summary` answer questions without dumping items; `--fields` trims what I must read; results paginate with cursors. `--explain` on an item shows the full derivation of its effective price: *inherited from tab "$$ chaos 5" ← tab-name auto-buyout ← game-set, locked*. I never have to re-derive the inheritance rules in my head — the system that owns the rules explains them.

One more idiom here, the biggest single ergonomic win: **declarative freshness on reads.**

```
acq items search --priced --max-age 30m [--sync-if-stale --max-requests 20]
```

Instead of me orchestrating *check staleness → decide → sync → wait → re-query* (five turns, easy to get wrong), I state the freshness my question requires. The system either serves it, syncs the minimal set first (within my request budget), or fails with exit 2 telling me exactly what sync it would take. The read-decide-fetch-reread loop collapses into one turn.

### Annotations (write intent)

```
acq policy show | set …            # THE sync policy (subsumes v1 "tracked set"):
                                   #   which tabs, staleness targets, include
                                   #   maps/uniques, priced-tabs auto-tracked
acq price set <sel> --type bo --value 5 --currency chaos [-f batch.jsonl] [--dry-run]
acq price clear <sel>
acq search save/list/run/rm <name>     # persisted, unlike the GUI
acq ratio set <currency> --chaos N --exalt N
acq note set <target> "text" | list | rm    # breadcrumbs on items/tabs/searches
acq price import-legacy <file> --plan-only -o plan.jsonl   # the xlsx wizard,
acq price apply-plan plan.jsonl                            # now in the universal grammar
```

Two deliberate upgrades over v1: the GUI's checkbox "tracked set" becomes a **sync policy** — I declare *"priced tabs fresher than 30m, everything else daily, skip map stashes"* and the effect layer compiles that into requests. Imperative refresh modes still exist underneath, but intent lives in one inspectable object instead of forty checkboxes.

And **notes** are new: durable, listable annotations addressed to future sessions ("holding this at 2 divine until league economy settles — recheck ~Sep 15"). That's the accretive channel. The GUI has a human's memory; the CLI's users have none, so the system must carry it.

### Derivations (compute, free)

```
acq shop render [-o pages/]        # hashed pages from priced items + template
acq shop diff                      # rendered vs last-published, per thread
acq shop status                    # threads, hashes, pages-vs-threads arithmetic
acq currency list [--ratios] | history [--since] | export --csv
acq export --json|csv|xlsx [--search <name>]    # the items export the GUI never had
```

`shop diff` is the derivation layer earning its keep: before any publish I can see *exactly* what would change on the forum, because both sides are hashed artifacts. No GUI equivalent exists — a human clicks "Update shops" and hopes.

### Effects (spend budget, touch the world)

```
acq sync [--plan] [--all | <selectors> | --lists-only] [--max-requests N] [--watch]
acq shop publish [--plan] [--force]
acq ops list | show <id> | wait <id> | tail
acq daemon run                     # executes the policy + auto-publish;
acq events tail --since <cursor>   # JSONL event stream with durable cursors
```

`sync --plan` prices the operation before I commit: `{requests: 23, est_duration: "41s", budget_after: "15/60s", fetches: [...]}`. When budget is the binding constraint, cost must be visible *before* spending, not discovered through a rate-limit pause. `--max-requests` makes partial-but-bounded syncs a first-class choice.

Every effect returns an `op_id` immediately; `ops wait` blocks, `ops show` gives the typed outcome — including the honest partial: `{outcome: "partial", skipped: [...], reason: "rate_limited"}`. The shop's clean-refresh gate becomes an explicit, checkable predicate instead of hidden coupling.

`shop publish` keeps the two-credential wrinkle loud (structured error + remedy when POESESSID is missing) and gains an **idempotency guarantee**: publishing an already-published hash is a recorded no-op, so a retried command can never double-post. Outward-facing effects should be safe to retry blindly — agents retry.

### Self-description and the journal

```
acq schema [command]      # JSON Schema for any command's output; --help is machine-readable
acq ops log [--since]     # every effect ever run: who (gui|cli|daemon), what, cost, outcome
```

Cold-starting agent, zero docs: `acq schema` + `acq state` and it knows the whole system. And because the verb surface is now a clean set of typed, schema'd, idempotent-or-planned operations, exposing it as MCP tools (`acq mcp`) is a binding, not a redesign — one line worth knowing, not scope worth adding today.

## 4. Proof by walkthrough

**Cold start, new session:** `acq state` → everything, plus cursor. `acq diff --since <last session's cursor from my notes>` → what changed while I was gone. Two turns to full situational awareness. (v1 design: status + auth status + tabs list + ratelimit + shop status + guesswork ≈ 6 turns, no change detection at all.)

**"Keep my shop honest," nightly:** `acq items search --priced --max-age 1h --sync-if-stale --summary` (one turn: fresh facts, bounded cost) → `acq shop diff` (is the forum stale?) → `acq shop publish` if so. Three turns, every one verifiable, journaled, safe to re-run.

**Reprice 200 items after economy shift:** `acq items search --tab "$$" --fields id,name,price --json` → transform → `acq price set -f new-prices.jsonl --dry-run` (per-row preview: 3 refused, game-locked) → drop the flag → post-state returned, done. Two mutations, zero read-backs, an auditable plan artifact in between.

## 5. What changed from v1, and what didn't

**Unchanged:** the functional inventory — every GUI capability from the survey still has a home, and the filter vocabulary, buyout semantics (inheritance, game-set locks, postability), and forum mechanics carry over intact.

**Changed:** `state` + durable cursors + `diff` replace scattered status commands · the tracked-set becomes a declarative sync policy · `--max-age`/`--sync-if-stale` collapse the freshness loop into reads · plan/apply and per-row batch outcomes generalize from the legacy importer to every effect and every bulk write · outputs gain epistemics (`as_of`, `complete`), errors gain remedies, effects gain ids, costs, idempotency, and a journal · `notes` added as the cross-session memory channel · `--explain` added wherever the system applies rules on my behalf.

The through-line: v1 asked "what buttons does the GUI have?" This asks "what does a driver with no eyes, no memory, and a metered fuel line need from the cockpit?" — and the answer is a system that *tells you where you are, prices every move, proves what changed, and remembers for you.*
