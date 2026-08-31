# Framing for the CLI/frontend design conversation

**Written 2026-08-31, before the session.** This is the context-setting
document: read it first, then the four notes in this directory, with
`../CLI-GUIDE.md` as the reference for what the spike actually is. It
records the owner's stances, the rules of engagement, and the seeds worth
chasing — so the session starts framed instead of winging it.

## The goal function

We are building something **new, better, simpler, and more usable** — not
porting the C++ app, and not decorating the spike. The backward-looking
C++ analysis tells us about the *product*. The spike tells us about the
*new way of building*. The synthesis of both might land somewhere
unexpectedly better than either source alone; that possibility is the
point of the session, and the framing below exists to protect it.

## Headline stances (owner, 2026-08-31)

These go first so they cannot be buried under the longer material.

1. **Traffic to GGG from any frontend is acceptable, because the daemon
   regulates it.** The rate limiter and request daemon were a large
   investment; the payoff is exactly this — CLI, MCP, and GUI traffic is
   properly regulated by construction, so we no longer have to treat
   every new caller as a fresh risk. Bugs excepted, and that is what a
   lightweight tripwire (in the mold of the spike's current rails) is
   for. Design accordingly: do **not** treat agent-originated traffic as
   forbidden. The narrower recorded deferral — verifying GGG's stance on
   agent traffic before the MCP path *ships* — is a shipping gate and the
   owner's call, not a design constraint for this room.

2. **Multi-account is first-class, not a footnote.** The spike already
   has it (session map, per-account stores, per-account limiter keys);
   the C++ notes and both redesign essays assume a single account
   throughout. Every proposed verb, resource, or document in this
   session must answer: *what does this look like with three accounts?*
   `state`/`status`, cursors, sync policy, a pricing document, a shop
   definition — all of it. Retrofitting account-awareness later is how
   the C++ app got where it is.

## The three sources, and the one rule about them

**Let each source be authoritative only where it is evidence.**

- **The C++ app** (surveyed in `acq-fable-cli.md`, `acq-sol-cli.md`) is
  evidence of *product semantics*: what users actually do, and the
  hard-won rules — buyout inheritance, game-set prices being read-only,
  priced tabs locked into the refresh set, the clean-refresh gate on
  shop auto-publish, the two-credential wrinkle. It is **not** evidence
  about shape: its shapes are artifacts of Qt and of assuming a human
  with eyes. When C++ testifies about a rule, listen; when it testifies
  about a widget, translate to the property underneath and discard the
  widget.
- **The spike** (`../CLI-GUIDE.md`, `../CONTEXT.md`) is evidence of
  *architecture and method*: how to talk to GGG safely, and how to pin a
  design (properties not mechanisms, decisions after the code teaches,
  pin after the consumer validates). It is **not** evidence about scope:
  it fetches and stores; it has never priced an item or published a
  shop. Its silence on a product feature is not an argument against it.
- **The redesign essays** (`acq-fable-redesign.md`,
  `acq-sol-redesign.md`) are evidence of *operator ergonomics*: what an
  agent (or script, or human in a terminal) needs — orientation, cost
  visibility, epistemics, memory. They are **not** evidence about
  feasibility or fit: they were written against the C++ GUI and know
  nothing of the daemon, the store, the job model, or the two-surface
  rule.

When a source speaks outside its lane, discount it.

## The settled floor (not up for debate here)

Naming what's settled is what makes it safe to be radical everywhere
else. Out of scope for this session:

- The five invariants (`CONTEXT.md`): one choke point + one send gate,
  headers are truth, never retry through Cloudflare, user-agent
  continuity, no plaintext refresh tokens.
- The architecture: a daemon owns all GGG traffic; frontends are thin;
  a frontend consumes exactly two surfaces (daemon protocol + store
  read API), no third door.
- The job model: API requests are jobs with id/state/priority/ETA,
  persisted, parent/child fan-out.
- The store split: daemon writes through one call and never reads;
  frontends read the file directly.
- The method: properties over mechanisms; pin after a real consumer
  validates; ADR 0003 stays the owner's and is not this conversation.

Everything else — verbs, resources, grammars, scope — is open.

## Shared vocabulary: what the essays call things the spike already has

Do this mapping *before* debating proposals, or the session will
reinvent existing primitives under new names (or worse, propose parallel
mechanisms beside them):

| Essay concept | Existing spike primitive |
|---|---|
| journal / ops log | persisted job queue (`daemon.db`) + send journal |
| `op_id`, `ops wait`, `ops show` | `--detach` + `acq status <id>` / `acq result <id>` |
| `events tail --since cursor` | subscribe channel + `item_events` |
| `snapshot diff` | `item_events` (ingest-time diff, shared by all consumers) |
| cost/ETA before spend | the daemon's ETA machinery (it can predict; it sees everything) |
| `daemon run` / `serve` | lazy-spawned daemon, idle exit |
| per-row batch outcomes | ingest counters; the legacy-import plan/apply flow (C++ side) |
| `as_of` freshness stamps | per-tab `fetched_at` (partial — generalizing it is cheap) |
| `budget` / `--max-requests` | rails: `ACQ_MAX_SENDS`, tripwire, journal (see synthesis seeds) |

Some mappings are exact, some are partial — where partial, the gap *is*
the design question.

## Triage: four buckets before any evaluation

Sort every proposal from the notes into:

- **(a) Already built** — wants at most a rename or an exposure through
  a surface.
- **(b) Fits an existing open topic** — the high-leverage bucket.
  Known landings: declarative sync policy / `--max-age` → the
  delta/selection-for-refresh topic; pricing-as-document → user state on
  items (store has the key, no table); interactive-vs-background → the
  priority-levels topic.
- **(c) New scope** — shop, currency, POESESSID, PoB export, catalogs:
  C++ product features the spike has never touched. Real, but must not
  crowd out (b).
- **(d) Conflicts with a recorded decision or invariant** — argue these
  explicitly or drop them; never adopt one by not noticing.

## Convergence signals

Independent agreement is the strongest evidence in the pile.

**Both essays converge on:** one-call orientation (`state`/`status` with
a cursor) · durable cursors + diff · plan-before-apply as a universal
grammar · enumerable vocabularies (no guessable strings) · per-row batch
outcomes · structured errors that carry their own remedy as a runnable
command · epistemics stamped on every output (`as_of`, `complete`,
provenance).

**Triple convergence, the strongest single signal:** C++'s tracked-set /
clean-refresh semantics, the spike's open "delta/selection for refresh"
topic, and both essays' declarative sync policy are three independent
descriptions of the **same object** — a persistent, inspectable
statement of what should be kept fresh, compiled into minimal requests.
Whatever else the session produces, this object probably deserves a
design.

Where the essays diverge (38 flags vs. one query expression; "four
layers" vs. "three planes"), the choice matters much less — treat
divergences as implementation taste, convergences as candidate
requirements.

## Synthesis seeds — where "unexpectedly better" might live

- **Rails graduate from test scaffolding to product features.** The
  owner's traffic stance rests on trustable regulation plus a tripwire;
  both essays independently asked for budget visibility, per-operation
  request bounds, and cost-before-spend. These are the same object:
  `ACQ_MAX_SENDS` and the tripwire, made first-class, *are* the essays'
  `acq budget` and `--max-requests`. The trust mechanism becomes the
  product feature.
- **Plan→review→apply × the job model.** The essays generalize the
  legacy-buyout importer's grammar to every effect; the spike's jobs
  already are the effect layer (ids, ETAs, persistence, events). A
  `plan` phase on jobs may unify both without new machinery.
- **Agent-native turns out to be everyone-native.** `state`, remedies,
  plan/apply, epistemics help scripts, cron, and humans in terminals as
  much as agents. Don't frame these as agent features; frame them as the
  CLI being an API — which is already a recorded decision.
- **The daemon/store split means ergonomics are built once.** Freshness
  stamps, cost quoting, diffs — built in the daemon or store, every
  frontend (CLI, MCP, GUI) inherits them for free. The essays put these
  in "the CLI"; the architecture says they mostly aren't CLI code at all.
- **Every stateful concept needs a surface-home decision.** Cursors,
  sync policy, notes, pricing doc, saved searches: each lives in the
  daemon or the store (no third door). Rough instinct: durable user
  intent → store (shared by all frontends for free); live orchestration
  state → daemon. Deciding the home is a real design act each time.

## Stress-tested: the two-surface rule (2026-08-31)

Before the session, the owner deliberately pushed back on the "two
surfaces, no third door" line, using product-scale item search — the one
big capability the spike has only in miniature — as the test case. The
rule survived, but came back better specified. The brainstorm inherits
these conclusions rather than re-running the argument:

- **The rule is two rules fused**: (1) shared semantics have exactly one
  implementation, so every frontend sees the same results; (2) wire
  contracts are minimized — a linked crate is a cheap contract, a wire
  protocol is an expensive one (lifecycle, handshake, respawn story).
- **The rule counts doors, not rooms.** Product-scale search stresses
  the store *crate's* identity, not the surface count. Search may grow
  into its own crate layered on the store's read API — a third leg of
  code with zero new contract. Crate factoring behind door #2 is
  internals.
- **Search indexes can live in the store file.** Ingest already does
  domain work (item lifting, column extraction); maintaining FTS or
  canonicalized-mod tables at ingest is the same species. Readers then
  get fast search with no warm-up, no cache, no coherence problem — and
  the daemon still never reads the store.
- **The schema is not a surface.** The store being a SQLite file means
  raw SQL is an invisible third door already standing open; nobody has
  agreed to hold the schema stable. Declare it internal, and defend it
  by making door #2 expressive enough (query language, `--count-by`,
  `--fields`) that going around it is never worth it. An agent-native
  design makes this acute: agents will find `sqlite3` if the blessed
  surface is weaker than the schema underneath.
- **A cached search service would reintroduce, inside our own
  architecture, the exact failure the essays warn about** — stale
  results mistaken for current truth — where today "read the file" is
  trivially coherent.
- **Tripwires for reopening** (so refusal today never needs
  re-arguing): a true third leg becomes discussable when multiple
  long-lived frontends measurably duplicate an expensive in-memory
  index, or when a concrete consumer shows latency that in-process
  reads over the store file cannot meet. Until then, the door count
  stays two — the forcing-function value is real: it is what turns a
  frontend's workaround into a design event.

## Gravity warnings

- **C++ detail gravity.** The C++ notes are the most detailed source by
  an order of magnitude and will dominate an unframed discussion. A
  feature-walk produces a transliteration — the trap both essay authors
  caught themselves in. Counterweight: extract properties, discard
  shapes.
- **Total-grammar gravity.** The universal plan/diff/apply grammar is
  genuinely elegant, but the project's proven method is to pin after a
  consumer validates, one tracer at a time. Adopt the grammar as a
  *direction*; commit to it slice by slice, not surface-wide up front.
- **Fused reads.** `--sync-if-stale` (a read that fetches when stale) is
  seductive but breaks the load-bearing property that store reads are
  daemon-free and network-free by construction. The architecture-fitting
  version is the essays' own error idiom: the read *fails with the exact
  sync it would take*, and the caller decides to spend. (This is about
  the read/write split, not about whether traffic is allowed — see
  headline stance 1.)
- **Shop is a different risk category.** Forum posting via POESESSID is
  outward-facing traffic not covered by the choke-point invariant, which
  is scoped to the API. If shop lands in scope, it needs its own
  boundary thinking, not inheritance from the API's.

## Output shape

The session should end with artifacts, not vibes:

1. **Candidate decision lines** in `CONTEXT.md`'s style — one-liners
   with rationale, phrased as boundary properties — sorted by the four
   buckets, for the owner to accept, amend, or refuse.
2. **One chosen tracer slice** to build next, honoring
   pin-after-the-consumer. The obvious candidate is refresh-with-`plan`
   against the delta/selection topic: it is bucket (b), it exercises
   cost-visibility and the sync-policy object, and it needs no new
   scope — but the session may find better.
3. **A parking lot** for everything real but not next, so deferring a
   good idea never requires re-arguing it.
