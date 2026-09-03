# Reflection after the tracers — where the tower stands, what grows next

**Written 2026-09-02**, the evening the legibility run closed. The owner
asked for a deliberate stop before the next direction: research the
state, the history, and the driving vision; think about where we are and
what we ought to build next; do not be seduced by shiny objects. This
note is that thinking. It reads after 00–06 and `../CONTEXT.md`, and like
them it is disposable history: anything it proposes becomes real only as
a ruling in `CONTEXT.md`.

The 2026-08-31 session had the C++ survey, the redesign essays, and a
spike that fetched and stored. It did not have the tracer's live runs,
the store's five review rounds, PoE2 first contact, or an owner reading
the output at a terminal. Several patterns below were not visible then
and are visible now. That is what this note is for.

## The goal function, as a scorecard

"New, better, simpler, more usable than the C++ app" decomposes into
properties. Each has a status; the status tells us where the C++ app
still wins.

| Property | Status |
|---|---|
| Safe to the GGG relationship, by structure | Done. Proven live, zero 429s, one choke point enforced by the dependency graph |
| Usable by scripts and agents as an API | Done. `--json` total, MCP as a thin client with no parallel semantics |
| Multi-account, multi-realm, first-class | Done. Sessions, per-account files and limiter keys, realm above league |
| Never loses intent | Designed. Never tested: the intent layer has held one row |
| Finds items | Not done. Substring match over three columns; C++ has ~38 filters plus mods |
| Prices and publishes | Not done |
| Runs where players are (Windows, a GUI) | Not done |

The first three are the architecture's payoff and they are real. The
C++ app wins today on the last three, which are the three a player
touches daily. The fourth is the gate between them: pricing is the first
irreplaceable data, and the layer that will hold it is the least
exercised code in the workspace.

## The tower as built

The four layers exist and are enforced by what links against what, not
by discipline. Their maturity is uneven, and the unevenness is the most
useful fact for choosing direction.

| Layer | Maturity |
|---|---|
| Effects (daemon, limiter, gate, jobs, rails) | Proven live. Over-invested relative to the rest: one 8,000-line file, half of it harness |
| Facts (store) | Hardened. Seven schema versions in four days, each bought by a review finding; membership and liveness now have one owner each |
| Intent (annotations) | Embryonic. One kind, one row, nine tests. Never load-bearing |
| Derivations (planner, search, rendering) | Planner mature for refresh. Search is a stub. Nothing derives a price, a total, or a page |
| Frontends | CLI and MCP validated on one slice. Dash exists. GUI absent |

A coherent system grows where the thinnest layer meets the strongest
pull. The thinnest layers are intent and derivations. The strongest
product pull is "find my items" and "price and sell them". The strongest
open architectural question is whether Plan is one grammar. All three
point at the same two layers, and none points at the daemon or at a new
frontend.

## What the runs taught that the last session could not know

Ten patterns, each with its consequence. The first three are
generalizations of things the code already does; the rest are
predictions the next slice can test.

**1. Authority is the coarser observation's.** Every store fight this
week was the same lesson in a new place: a listing owns membership,
address, league and liveness; a fetch owns contents and nothing more; a
fetch never revives what a listing retired. The same shape already
governed the other layers: headers own rate-limit truth and local state
only predicts; the policy revision owns authorization and the plan only
derives; the plan owns the action set and apply only spends. Stated
once: *for every fact there is exactly one authoritative source, and it
is the coarser observation.* This should be a decision line. It would
have saved three of the five review rounds, and it decides pricing's
precedence question below without a new argument.

**2. A refusal keeps what it refused.** The `refused` table, the
`withheld` body, the journal line for every failed HEAD, the orphaned
annotation kept when its item vanishes. The lesson from PoE2 first
contact ("a refusal that destroys its evidence turns every failure into
a re-fetch") is general. A pricing import that rejects a row must keep
the row and say why. A policy write that fails validation should hand
back what it refused. Cheap to state, and it stops a class of design
error before it is designed.

**3. Every human surface is a derivation over a machine surface.** The
legibility ruling said it for one surface: text is a function of the
envelope, grouping is presentation, the authorization is untouched.
Generalized, it relocates one parking-lot item. The shop page is a
rendering of facts plus intent, hashed, exactly the C++ `shop_hash` idea
the essays noticed. That render is a derivation: pure, free, sends
nothing. Only the *post* is the outward effect that needs its own
boundary session. So "shop" splits: render belongs with pricing, publish
stays parked. This matters because it gives pricing a real-use consumer
before any publishing exists (below).

**4. The intent layer is approaching its durability cliff.** Facts went
v3 to v7 in a day and nobody flinched, because facts are refetchable.
Plans went v3 to v6 the same way, because a plan is a derivation. The
first real buyout row ends that agility for one file: an annotation
value schema that changes after data exists is a migration of the
irreplaceable state. Consequences, in P3's register (identity and
durability get first-consumer treatment):
- intent *values* are version-stamped and strict-parsed from day one,
  the sync policy's existing pattern, now factored rather than repeated;
- the annotation crate gets a targeted review before it carries a
  price — not a general code review, a review of the one durability
  boundary about to become load-bearing;
- **provenance on intent rows now.** The table has scope, key, kind,
  value, revision, timestamps. It has no author. Who set this price, and
  through what — a human at the CLI, an agent over MCP, an import from
  the C++ store — cannot be reconstructed later. This is the uuid
  argument again: cheap today, unrecoverable after the fact.

**5. Intent has several authors now, and pricing multiplies them.** The
MCP rule that replacing a policy must name the revision it replaces
existed for one row. Pricing brings thousands of rows and an agent that
will reprice on request. The compare-and-swap discipline earns its keep
here, and the parked annotation event log's trigger ("conflicts need
history") will probably fire at pricing rather than later. Predict it,
so the schema leaves room; do not build it first.

**6. The owner-truth channel is conversation, not prompts.** Two live
runs offered friction prompts; zero notes were typed; both verdicts
arrived in conversation and were recorded by the agent. The method-test
caveat ("owner-truth channel under-exercised") diagnosed a symptom of
the wrong channel. Record the owner's words from the conversation,
verbatim and marked as such, and retire the prompt as the primary
channel. For pricing this matters doubly: there is no live run at all.
The owner's truth will be real buyouts flowing through the import and a
rendered page read against the forum. Design the slice so those two
readings happen.

**7. Coherence debt is the shadow of review velocity.** Five same-day
review rounds produced five narratives in `CONTEXT.md`; the file is now
about 110 KB and every session reads it. Its own charter (current state
only, boundaries not mechanisms, never a parallel description of code)
is broken by its size. The tracer section was pruned to properties on
2026-09-01. Make that the forward rule: **the property lands in
CONTEXT, the round lands in git.** The same disease is the CLI density
verdict and the README's 100-line store bullet. Density is a system
property of this project's writing register, not a bug in one surface.

**8. Reference data is an input the tower has not placed.** The currency
list, item categories, the mod catalog, the league list. Not an account
fact (no account fetched it), not intent (nobody declared it), not a
derivation (nothing computes it). The store's `leagues` table is the
tell: a per-account file holding account-independent truth. Hypothesis:
reference data is *facts with build provenance* — shipped inside the
binary, versioned by the build like the schema, read-only, never in a
store file. The mod semantics work in the sibling repo would ship the
same way. Pricing meets this at the currency list; search meets it at
the mod catalog. Rule it at the first meeting.

**9. Plan is a family sharing an envelope discipline, not one grammar.**
The 08-31 packet left this as evidence to collect. The evidence now
suggests the answer's shape. Three apply targets are in view: the daemon
(refresh), the intent file (a price patch, an import), and the outside
world (a forum post). They cannot share a vocabulary. What they can
share is the envelope: a fact basis, the intent revisions relied on, an
explicit action set, a schema stamp, a strict parse that re-serializes
exactly, and a staleness gate. One refinement falls out: the refresh
gate checks *one* revision (the policy's); a price patch's gate is a
*set* of `(key, revision)` preconditions, one per row it touches, and
the policy case is that set with one element. Factor the set; the
singleton is free. Test this prediction at pricing before generalizing
anything.

**10. Derivations report gaps; they never rewrite intent.** `policy
show` shows what the human typed. The plan reports policy ids the facts
lack rather than inventing actions. The C++ rule "a priced tab is locked
into the refresh set" is intent silently rewritten by another kind of
intent. In the tower the planner reads both kinds and *reports* priced
locations outside the declared coverage, as a remedy the human accepts
or ignores. Same protection, no lock, and the policy stays what its
author wrote.

## Pricing, seen through the tower

The C++ `item_buyouts` row fuses three layers: `source` distinguishes a
game-set price (a fact parsed from a tab name or item note) from a
manual one (intent); `inherited` marks a derived row stored as if it
were data; `value` is the intent itself. The locks exist because the
layers were fused: a game price is frozen so a refresh cannot fight a
manual edit. Separate the layers and the locks dissolve.

- **Intent**: a manual buyout on an item or a location, an annotation
  keyed on the GGG id, with type, currency, value, provenance, revision.
  Type `[Inherit]` is not stored; it is the absence of a row.
- **Derivations**: the game-set price, parsed from the fact (tab name,
  item note) on read, never stored; inheritance, item falls back to its
  location; the effective price, one function over facts and intent
  with a stated precedence; the shop page, rendered from effective
  prices and a template, hashed.
- **Precedence** follows pattern 1: the finest explicit statement wins,
  so an item's own price beats its tab's. At equal grain, game against
  manual is not a fact-beats-intent question. A game-set price is
  *already published* — the trade site indexes the stash. A manual price
  on the same item would publish a second, contradicting price. Report
  it as a conflict with a remedy (edit the note in game, or drop the
  manual price). Better than the C++ lock: the user can state intent and
  be told exactly what contradicts it.
- **The consumer is the owner's own C++ store.** The 0.18 userstore is
  already GGG-id keyed, so the hard matching problem the legacy import
  solved is solved. What remains is honest: import only `source =
  manual` rows (game rows are facts we re-derive; inherited rows are
  derivations we recompute), resolve C++ character locations, which are
  names, to the store's character ids, and produce a plan the owner
  reviews before it lands. Plan, review, apply, applied to the intent
  file with no daemon in the loop. This is the sharpest test of pattern
  9 available, and it is real use with real stakes.
- **`shop render` is the validation surface.** The C++ user guide
  documents pasting the generated markup by hand. With render built, the
  owner can read the page the prices produce and paste it, which makes
  pricing a usable product before any automated publishing exists, with
  zero outward traffic.
- **Multi-account.** Prices are per account; the account owns the
  items. Currency ratios and a shop template are user-scoped, which is
  the `user.db` trigger. Name it in the session; decide it when the
  first user-scoped kind is actually written.
- **Which crate.** The second intent kind fires the factoring rule for
  "versioned, strict, validate-then-CAS intent values", today living in
  the planner for one kind. Whether the price derivations join
  `acquisition-plan` or a sibling is a design-session question, not a
  pre-decision.
- **Out of pricing's scope**: publishing (its own boundary session, as
  recorded), currency totals and history (a derivation over facts,
  cheap, and still scope creep until pricing stands), any GUI.

## The order I recommend

**First, a bounded settling pass.** Not a cleanup project; the
precondition for the pricing session being as good as the 08-31 one.
- `CONTEXT.md` and `README.md` back to their charters: the characters
  and legibility narratives compressed to properties, the rounds left
  in git, the store bullet returned to a paragraph and pointers.
- The CLI density item, while the verdict is fresh. Cut words.
- A ruling on the ad-hoc `refresh --tabs` / `--all` kinds: keep as the
  explicit-selection primitive, or retire. Two doors to one task is the
  incoherence this project removes everywhere else.
- The two small known gaps (the MCP quote note; the per-realm policy
  merge stays parked with its trigger).

**Second, pricing** as framed above, with a design session first, in
the 03→06 shape: candidate decision lines, one slice, a parking lot,
rulings before code. The annotation crate review precedes the build.

**Third, search semantics** as the derivations slice. It is what makes
the store worth a person's time. It also keeps a recorded promise: the
schema is internal *because* door 2 is expressive enough that going
around it is never worth it, and today an agent with `sqlite3` gets
more than `acq items search`. The mod catalog lands here, and with it
the sibling repo's stat semantics.

**Fourth, the GUI**, when there is something to show that beats the C++
app. A GUI over substring search with no prices loses that comparison
on the first day, and it is the most seductive object on the list for
exactly the reason it should wait: it is visible.

## Not now, and why each is a shiny object

A working test for shininess: the proposal grows the strongest layer,
or adds a surface before the thin layers can back it, or has no consumer
whose truth is real use.

- **The GUI**: adds a surface before the thin layers can back it.
- **A broad code or test review**: diffuse. The two targeted reviews
  (annotations before pricing; the daemon's harness extraction on the
  day a fresh build starts, per `TESTING-NOTES.md`) are already placed.
- **Refactoring the daemon**: grows the strongest layer. The 8,000-line
  file is the least modular thing here and also the most-proven code;
  every touch spends live-test trust for no product gain.
- **Windows and code signing**: shipping gaps, real, not directional.
- **A universal Plan grammar**: pattern 9 says collect the evidence at
  pricing; the packet already said so.
- **Scheduling**: the daemon never originates work by decision, and the
  "small frontend" that runs a policy on a timer has no keychain from
  cron on macOS. Real, and downstream of a GUI or a signed build.

## Questions for the owner

1. The order: settle, pricing, search, GUI. Accept, reorder, or refuse?
2. Pricing's consumer as the C++ userstore import plus `shop render`,
   with publishing out. Is the render in scope, given it sends nothing?
3. Provenance on annotation rows now, as a P3 item, before the first
   price is written?
4. Pattern 1 as a decision line: one authoritative source per fact, the
   coarser observation's.
5. Reference data as facts with build provenance, shipped in the binary.
   Rule now, or at the first meeting?
6. The freshness-window question the tracer raised: is a window the
   right handle for "keep these fresh" when a cycle runs thirteen
   minutes? Rule it, or park it with a trigger.
7. The self-description. The README and CONTEXT still call this branch
   a reference implementation whose CLI is evidence, not promotion.
   Multi-account, persistence, realm, and legible output are product
   work. Whether the language changes is yours; that it has drifted is
   recorded here so the next reader is not confused.

## Output shape

The same three artifacts as 00: candidate decision lines in
`CONTEXT.md` style, one chosen slice, and a parking lot with triggers.
The settling pass produces none of these and should be done before the
session that does.
