I've read the whole corpus — framing, both CLI maps, both redesign essays, the CLI guide, and `CONTEXT.md` — and sat with it as one object rather than five documents. Here is what I see when I look for the *system* underneath, and where I think the genuinely new findings are. This is long because the framing asked for depth; the questions for you are at the end.

---

## 1. The central recognition: the essays and the spike describe the same tower, from opposite ends

Fable's four layers (facts / annotations / derivations / effects) and Sol's three planes (observed / desired / derived, plus jobs and history) are near-isomorphic — the framing already noted that convergence. What the framing *doesn't* say, and what I think is the deepest fact available in this room, is this:

**The spike has already built the top and bottom of that tower, and the two-surface rule is the tower's own shadow.**

- **Effects** — the essays' journaled, costed, id-bearing operations layer — *is* the job model. Not "maps to": is. Ids, states, priorities, ETAs, parent fan-out, persistence across restarts, a send journal. The essays, written blind to the daemon, independently specified the thing the spike spent its whole life proving.
- **Facts** — the essays' provenance-stamped observed plane — *is* the store. Verbatim bodies, items lifted on a stable GGG id, `fetched_at`, and `item_events` as the ingest-time change log. The essays' `snapshot diff` is already running in production form.
- **Derivations** exist in embryo: search, the tab tree, ingest counters — plain functions in the store crate.
- **Annotations** — the desired plane — is the one layer with no implementation. And `CONTEXT.md` already knows it: "user state on items: the store has the key, no table yet; needs the first frontend that writes." The open topic *is* the missing layer.

So the brainstorm's real question is not "what command surface should we build" — it's **"the system is a four-layer tower with one layer missing and one grammar unnamed; what does finishing it minimally look like?"** Everything else falls out of that.

And the two-surface rule turns out not to be a separate architectural stance that the layer model has to be squeezed through. It's the same partition:

| Layer       | Surface                                 | Sole writer              | Change log                           | Free?        |
| ----------- | --------------------------------------- | ------------------------ | ------------------------------------ | ------------ |
| Effects     | door 1 (daemon protocol)                | daemon                   | jobs table + send journal            | costs budget |
| Facts       | door 2 (store read API)                 | daemon (`record`, blind) | `item_events`                        | free         |
| Annotations | door 2 (store crate, **new write API**) | frontends                | (predicted: annotation events)       | free         |
| Derivations | shared crates behind door 2             | nobody — computed        | none needed (recomputable, hashable) | free         |

The epistemological boundary the essays wanted ("I must never mistake cached for current, and a sync must never clobber my intent") and the architectural boundary the spike proved ("reads are daemon-free and network-free by construction") are the *same boundary*. That's why the essays' `--sync-if-stale` felt wrong to whoever wrote the gravity warning: it's the one proposal that reaches *across* the layer boundary, and the architecture rejects it for the same reason the epistemology does.

This also generalizes the write discipline into something clean enough to be a decision line: **each layer has exactly one writer class.** The daemon writes facts and never reads them; frontends write annotations (through one shared store-crate write API, so all frontends share the semantics — the two-rules-fused reading of the surface rule is satisfied); nobody writes derivations; only the daemon writes the effects ledger. "A sync can never clobber my pricing" stops being a promise and becomes a structural property, exactly the way the choke point made rate-limit discipline structural.

## 2. The keystone finding: what a *plan* actually is in this architecture

Both essays make plan-before-apply the universal grammar. The framing warns against adopting it surface-wide up front. But sitting with the layer table, I think the plan grammar is much cheaper than either essay imagines — and locating it precisely is the most load-bearing design act available.

A plan is a pure function of three inputs: **facts** (what's stale, what changed — door 2), **annotations** (what the policy/pricing says should be true — door 2), and **cost** (what the reconciliation would spend under live limiter state — which lives only in the daemon, door 1). Two of its three inputs are behind door 2; one is behind door 1. Neither surface can compute it alone — the daemon never reads the store (so it can't know what's stale), and the store never sees the limiter (so it can't price anything).

The resolution is not a third door and not a violation. It's this: **a plan is a frontend-side derivation, composed from both existing surfaces, living in a shared crate.** The store side answers "which fetches does the policy imply" (it has `fetched_at` and, eventually, the API's free `metadata.items` counts). The daemon side answers one new, tiny, *pure* question: **`quote(job set) → {requests, ETA, headroom after}`** — which is nothing but the ETA machinery it already runs for queued jobs, asked hypothetically. Plausibly not even a new verb: `submit --plan`, a dry-run flag on the verb that exists.

That means the entire universal grammar — the thing both essays built cathedrals around — costs, at the boundary, **one dry-run flag on the protocol and one shared crate behind door 2.** The essays put plan/apply in "the CLI"; the architecture says almost none of it is CLI code, and the CLI, MCP server, and GUI all inherit it from the same crate. This is the framing's "daemon/store split means ergonomics are built once" seed, made concrete.

And it yields a unification I find genuinely beautiful, which I'd propose as its own decision line: **an error's remedy is a plan.** The essays converged on structured errors carrying runnable remedies, and on reads that refuse with "here is the exact sync it would take." A stale-read refusal, a `--plan` preview, and an error's `remedy` field are one type appearing in three positions. Build the Plan type once and the error idiom, the cost-visibility idiom, and the plan/apply grammar are the same code. `hints`, `remedy`, `plan`: one object.

## 3. The symmetry that predicts the rest: every layer is a ledger with a cursor

Look at what's already true. Facts have current state (`items`) plus a change log (`item_events`). Effects have current state (the jobs table) plus a change log (the send journal). The pattern *ledger + change-log + durable position* is the system's native idiom — nobody designed it twice, it emerged twice.

The essays' `state`-with-a-cursor and `diff --since` are then not features to bolt on; they're the observation that the tower should expose this symmetry uniformly. `acq state` is a summary of every ledger's head; `diff --since <cursor>` is a merge-read of the change logs past a position. Both are door-2 reads — free, daemon-optional (and the effects ledger is *still* readable with the daemon down, because `daemon.db` is already blessed as frontend-readable; "no daemon" even implies "nothing in flight," so the orientation document degrades gracefully rather than failing).

The symmetry also makes one prediction: when annotations land, they'll eventually want their own change log too, or `diff --since` can't answer "what got repriced while I was away." I would *note* that prediction and not build it — pin-after-the-consumer — but writing it down now means the annotations schema gets designed so that adding the log later isn't a migration.

The same symmetry resolves the rails question the framing seeded. The tripwire, `ACQ_MAX_SENDS`, and the journal are the effects ledger's *integrity mechanisms* — and the essays' `acq budget`, `--max-requests`, and cost-before-spend are the same mechanisms *read as product features*. One enforcement point, two framings. Graduating rails to product features isn't new machinery; it's promoting existing knobs from environment variables to first-class citizens of the plan grammar (`--max-requests` on a plan is `ACQ_MAX_SENDS` scoped to an operation instead of a daemon lifetime).

## 4. The hard-won lesson the C++ app testifies to, recast as a property

Here's where the C++ evidence speaks in its lane — about a *rule*, not a widget — and I think it's speaking louder than either essay heard.

Why does the legacy-buyout import wizard exist at all? Because the C++ app keyed user intent (buyouts) to unstable hashes inside a fact cache, and when the cache's world moved, years of the user's pricing work nearly died with it. The entire plan→review→apply grammar that both essays canonized was *invented as disaster recovery for intent trapped in a cache.* That's the origin story of the system's best idea.

The property underneath: **facts are refetchable; annotations are the only irreplaceable local state.** Losing the fact store costs API requests. Losing annotations costs the user's actual work — there is no server to refetch intent from. The spike has already half-solved this by accident (items keyed on stable GGG ids kills the unstable-hash half of the problem). The other half is a real design decision for the annotations layer: intent should be separable from the cache — its own file, or at minimum its own backup/export story — so that a schema migration, a corruption, or even a future rewrite of the fact store can never again threaten it. The legacy-import saga is the one place the C++ app paid full price for getting this wrong; we get to accept the lesson at design time instead.

(Pleasing corollary: if annotations are cleanly separable and keyed on stable ids, then "legacy import" stops being a wizard and becomes what Sol said it should be — just a patch generator into the ordinary annotation plan/apply path. The special case dissolves.)

## 5. Where multi-account actually bites: annotation scope

Applying the "what does this look like with three accounts?" test to the tower, three layers pass trivially — jobs carry `account`, facts are per-account files, the limiter keys per account. The annotations layer is where the test finds something real:

- **Item- and tab-scoped intent** (prices, notes, the sync policy / tracked set): naturally per-account. Lives beside — or keyed against — that account's facts. Easy.
- **But some intent is not account-shaped.** Currency ratios describe a league's economy, not an account. Saved searches are questions, not possessions — you'd plausibly want to run one *across* accounts. A shop template might be shared across two accounts' shops or might not. There is currently no home for user-scoped, account-agnostic state — the store directory has per-account files, `accounts.json`, and `daemon.db`, and nothing else.

So the multi-account stance forces a question the single-account essays could never see: **does the store grow a user-scoped annotations home** (a `user.db` beside `accounts.json`), with each annotation *kind* declaring its scope — per-item, per-account, per-league, per-user? I don't think this is expensive, but it's exactly the kind of thing that's cheap now and a retrofit later, which is the framing's stated reason multi-account is headline stance 2. It also touches the CLI grammar: `acq state` with three accounts is presumably a roster with shared budget/daemon lines plus per-account ledger summaries, and `--account` selects the deep view — the limiter's existing two-level keying (shared IP gate, per-account policies) already tells us which lines are shared and which are per-account.

## 6. Why the framing's suggested tracer is right — in system terms, not convenience terms

The framing nominates refresh-with-`plan` against the delta/selection topic and says "the session may find better." Having built the tower picture, I don't think there is better, and I can now say *why* rather than just nodding:

**The sync policy is the only object that touches all four layers at once, and the triple convergence noticed exactly that.** The policy is an annotation (declared freshness intent — first row in the missing layer's table). The plan is a derivation (policy compared against `fetched_at` facts, priced by the daemon's quote). The apply is an effect (the refresh parent job, unchanged). The next `state` reads the facts it produced. One tracer, and every new boundary the rest of the roadmap needs is validated: the annotations write path through the store crate, the scope question from §5 (the policy is per-account — the easy case, good for a first row), the `quote` dry-run on the protocol, and the Plan type with its error-remedy unification. Pricing-as-document then arrives to *validated* machinery — it's another annotation table plus the same plan/apply grammar, not a new architecture. Shop after that (with its own boundary thinking, since POESESSID traffic is outside the choke-point invariant — parking lot, per the framing).

It's also honest to the delta/selection topic on its own terms: `--max-age`/policy compilation with `metadata.items` counts is precisely the "skip tabs that cannot have changed" lever `CONTEXT.md` already names, and "retry just the failed set" falls out as re-planning against unchanged intent.

## 7. What I'd bring to you as the genuine open questions

These are the places where I hit a decision that's yours (boundaries) or where the evidence underdetermines the answer:

1. **The annotations write path.** Frontends writing to store files directly (through a shared store-crate write API) extends door 2 from read-only to read/intent-write. I believe it's the coherent answer — one writer class per layer, no daemon involvement in intent, no third door — but it amends the current "frontends read the file directly" discipline, and that's a boundary you hold. The mechanical sub-question (annotations in the per-account `.db` beside facts, versus their own file for the §4 separability property) I'd argue toward *separate file*, on the legacy-buyout lesson.
2. **Annotation scope taxonomy** (§5): is a user-scoped home (`user.db`) acceptable as part of the store's shape, with each annotation kind declaring per-item / per-account / per-league / per-user scope?
3. **`quote` as `submit --plan`** — a protocol change, small but yours: does a dry-run flag on submit (compute cost and ETA, enqueue nothing) fit your sense of the protocol, or does "plan" deserve its own verb?
4. **How far to pin the grammar now.** My instinct honors the gravity warning: record the five-verb grammar (look / assert / plan / commit / attend) and the Plan-type unification as *direction* lines, but pin only what the refresh tracer exercises. Pricing and shop stay in the parking lot with their landing spots named, so deferring them never needs re-arguing.

If this synthesis holds up under your pushback, I'd crystallize it into the framing's three artifacts — candidate decision lines in `CONTEXT.md` style sorted by the four buckets, the tracer slice specification, and the parking lot. But the framing was right that the session should start with conceptualization rather than proposals, so I've led with the tower. Where do you want to push first? The place I'm least certain is §1's cleanest claim — that annotations belong wholly behind door 2 with the daemon permanently blind to them; if there's a future where the daemon must read intent (say, daemon-resident scheduled syncs executing the policy without a frontend awake), that blindness is the assumption that breaks, and I'd rather stress it now than after the tracer pins it.