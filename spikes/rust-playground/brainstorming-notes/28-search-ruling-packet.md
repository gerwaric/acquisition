# 28 — The search ruling packet (stage 5)

Provenance: Fable (`claude-fable-5-1`), a fresh session with the owner,
2026-09-17, under note 23's stage 5 and decision 15. Read: note 23; note
31 whole, Astra's check first; note 22's stances, floor and acceptance
test; note 24 §1–2 (the surface the reconciliation kept) and note 25's
`line(…)` binder; the digest's store and engine-bench claims, acceptance
set and limits register; the registry's form (`CONTEXT.md`,
`decisions/pricing.md`, `tools/docs-check.sh`) and note 06 as the mold.

**This is the document the owner rules on.** Accepted lines are harvested
into `decisions/search.md` in the registry's style; then this note is
history, never a second authority.

## How to read a line's standing

Under decision 15 every line but C89, C103 and C1 is **provisional**: note 31's
recommendation, harvested *with* its K-correction, standing until the
owner's first seat at a surface he can use, which may cycle any of them
back. Each candidate carries one of three marks, because they are not
the same kind of thing:

- **checked** — a note-31 recommendation that Astra's check traced and
  left standing, or corrected in a way that only narrows it.
- **chosen here** — Astra's check said stage 5 "must choose and state"
  and neither note 31 nor the check chose. The choice is Fable's alone:
  no audit, no reviewer, no second model has seen it. One voice.
- **ruled** — the owner's own words, this session.

The arrows of note 31 (→A, →B) are kept so the weak direction stays
visible: I am Fable, B is Fable's.

## 1. The combined model on one page

No document holds the model these rulings describe: it is spread over
seventeen rows and seven corrections. Note 22's own test is that the
model and grammar fit one page, so here it is. Every sentence traces to
a row or a K; what does not is marked **new**. This page is not a third
proposal and is not harvested — the decisions are; the page is how to
check that they cohere.

**An item is what it displays, where it is, and what you said about
it** (B). What it displays is a set of **lines**; each line is one
*occurrence* with a *kind*, a *template* and its *numbers* in order
(row 1). Kind is the source array with the flags the line carries, both
kept (Astra, row 1). Everything else a body holds is a *field* or a
*flag* from a closed list, or is seen through `show`; there is no path
into the body (row 10). Place — realm, league, tab, character,
container — comes from the store's columns and is asked like anything
else (2e). Intent joins read-only: `priced`, `price`, `note` (row 6).

**A term** is a thing and optionally a comparison; terms side by side
are *and*; `-`, `or`, parentheses and `holds(…)` with a lower and upper
bound compose across everything (row 2, K5). A comparison on a line
binds **one occurrence** (2a). Conditions on several numbers of one
line are one term, so they hold together on one occurrence (K1); the
spelling offered, **new**: each `#` in a quoted template may carry its
own comparison, `"Adds #>=12 to #<=28 Cold Damage"`, and B's `*` is the
slot left free. The item's sum is asked for by name: `sum("…")`, or a
shipped **total** whose definition is one reference table (2c, the
withheld seed, K3). **Absent is false** and counted; **unreadable is
undecided** and counted apart (2b, K2).

**One operation: ask** (B, row 6). The answer is one shape:

```
query    canonical text, and its tree in JSON
basis    the snapshot answered from (K4)
scope    items searched; realms, leagues; never fetched; oldest fetch; undecided
terms    per term: resolved to; matched, failed, lacked, unreadable
total    matches
rows | counts | one item
next     returned, total, the continuation (bound to the basis)
```

*Counts* are facets (one table per name), a crossed table, either with
an optional summed thing (2d, K3); counting by `line` under the query in
hand **is** the vocabulary read, each row carrying the term that selects
it (row 3). *One item* with `--against` is the why-not. A trade URL
translates in, a query translates out, each with a per-clause report
and a remainder (row 5). Read-only SQL runs over the same derived
relations (row 4, stance 6).

### The concept inventory, and what it cost

Astra's check asked for this and said plainly that subtracting A's 16
from B's 11 would mislead. The combined model has **15**: item and its
three parts · line as occurrence · kind (two coordinates) · name (field,
total) · term and comparison · slot comparisons in one term ·
composition with bounded `holds` · `sum` and the totals table · absent
against unreadable · place terms and the realm rule · ask and the one
answer shape · the three views, with two count shapes and a sum · the
vocabulary as counts by line · basis and continuation · the trade
translation and its remainder. (SQL is a second language over the same
relations, not a concept of the model.)

So it is **not simpler than B**; it is B's surface carrying A's
precision. Gone from A: `any(collection, P)`, arithmetic and `if` on the
surface, three-valued logic for absence, `select`/`order` clauses, the
raw accessor, mandatory `in S`. Gone from B: user names (parked), the
item-sum default, the mean inside `sum`, whole-item-only colours. What
survives regardless of spelling, as Astra warned: occurrence binding,
line summation, two summary shapes, group binding for sockets, decode
failure.

**Owed and not done here:** the mistake walk. It needs a seat making
errors against a page, and neither review validates this combination.
Parked below as stage 6's first step.

## 2. Candidate decision lines

Registry form, one bullet each, ids C89–C104 (the registry ends at
C88). Sizes are measured (`wc -c`, the 800-byte gate) and listed in §5.
The file's header will carry the provisional standing once, so no entry
spends bytes on it.

### Ruled

- **C89 — Item search is its own library crate, `acquisition-search`, over the store's read API.** It links the store, and `acquisition-plan` once a query reads the effective price (C81); never the daemon, the client, an HTTP client or an async runtime, and it opens no file of its own, so it is no third door (C12): a frontend links it as it links the planner (C1). The daemon and the planner never link it — a plan takes item ids, never a query. *Why:* each property is an edge the check can refuse, and a search edit recompiles no daemon. Ruled 2026-09-17 (owner: "Agreed with a new crate").

  *Standing:* **ruled**, edges included — question 1 below asked
  whether the agreement covered them: "I agree to 1 entirely."

### Provisional

- **C90 — A line's identity is (realm, kind, template), and the displayed text is the only name a line needs.** The template is the displayed English with each number replaced by `#`, markup reduced to its display half and the sign carried in the number; numbers are ordered slots, never averaged. Kind keeps both coordinates the body gives — the source array and the flags the line carries (`crafted`, `fractured`, `mutated`; S17) — so neither bucket rule is chosen silently. A rename is a new line. A stat id exists only at the trade boundary (C99). *Why:* the template names every line and the stat id two thirds (S155); the text is the moving part. *Evidence:* S3, S4, S17, S68, S71. 2026-09-17.

  *checked* · row 1 →both, with Astra's "preserve both coordinates".

- **C91 — A query is one typed tree, and a value: immutable, serializable, handed between a human and an agent, to the site and from it.** And, or, not and at-least-N-of (with a lower and an upper bound) compose across every field, line and place alike, and every surface is an encoding of the one tree. A refinement is the old query, parenthesised, and a new term. There are no silent modes: text that could be read two ways is an error that shows both readings. Every answer returns its canonical query. *Why:* composition the C++ app lacks and the site allows only among stats (S169); an answer that says how it was understood is what both designs reached unprompted. *Evidence:* S169, S172. 2026-09-17.

  *checked* · row 2's tree and composition (one text: stance 2, S169,
  S172); "the query as a value" is the withheld seed both reached. "No
  silent modes" as an error showing both readings is my sharpening of
  note 31's "one visible rule or an error".

- **C104 — The text is carried on every seat; the tree is accepted and always returned.** The tree parses from and prints to the text bijectively; the CLI and the MCP pass the text, a client that holds a tree (a GUI's rows) may send it, and every answer carries both. *Why:* both reviewers wrote text in both designs, and the string is far shorter than its JSON. It departs from the letter of R14 — "a query object, not a text argument" (S195) — and keeps its intent, a refinement being the previous query plus one term. No review compared the text with an equally specified JSON, so this is an adapter choice, the first the owner's seat revisits. *Evidence:* S195. 2026-09-17.

  *checked, with a discount* · row 12 →B, the weak direction; Astra:
  rows 2 and 12 reuse one testimony, and this is "a provisional adapter
  choice, not a second vote for row 2". Split from C91 so the line most
  likely to cycle back can move alone. (The first draft held both in one
  bullet and landed on exactly 800 bytes — a line doing two jobs.)

- **C92 — A comparison on a line binds one occurrence; the item's total is asked for by name.** A bound of 90 on a life line holds when one line reaches 90, never when an implicit 20 and an explicit 75 do together; `sum` of the line, or a named total (C94), is the item's sum. Conditions on several numbers of one line are written in one term and hold on one occurrence together — a low bound and a high bound met by two different lines is no match. Where a term sorts or is shown as one number and several occurrences satisfy it, the largest is used, and the answer says so. *Why:* a sum is built from occurrences by a word; an occurrence cannot be recovered from a sum. *Evidence:* S27, S28, S68. 2026-09-17.

  *checked* for the one-line default (2a →A, against Fable's grain, both
  reviewers). **Chosen here** (K1): several conditions in one term as the
  shared binder; the largest satisfying occurrence as the sort scalar;
  the "together" count limited to a single-number bound.

- **C93 — Absent is false; unreadable is undecided, and the two are never one count.** A comparison on something the item lacks is false, never zero (S158), and `has:` is how presence is asked. A body or array the deriver could not read makes the terms that needed it undecided, and only those: and, or and not carry it (true or undecided is true; not undecided is undecided), and an item whose whole query stays undecided is no match and is counted on the scope line. Per term the answer counts, over the items in scope: matched, compared and failed, lacked, unreadable — four that sum to the scope. *Why:* an unknown count that is nonzero on every ordinary query stops being read; one that is nearly always zero is read when it is not. *Evidence:* S107, S158. 2026-09-17.

  *checked* for absent-is-false with per-term counts (2b →B with A's
  conscience). **Chosen here** (K2): of Astra's two routes I took "retain
  an internal indeterminate outcome", confined to decode failure, because
  the other route gives up a known positive under `true or unreadable`.
  So three-valued logic does not "leave the model" as note 31 said; it
  shrinks to the one case that is almost never there. The fourth count
  is Astra's "name the ordinary failed comparison".

- **C94 — A named total is a sum that answers, defined once as reviewed reference data (C68).** A row names the contributing line — template, kind where it matters, slot — and its weight; the evaluator and the SQL view (C48) are both generated from that table, so the model and a SQL caller cannot disagree. The answer prints a total's definition on request and each row's contributions. A definition is checked against the reference data, loudly, when the table is built; a template the game has since moved and one the stash merely lacks look the same in a corpus, and that stays a listed limit (S111). *Why:* a total that may decline failed the everyday question in use (stance 3). *Evidence:* S29, S49, S111. 2026-09-17.

  *checked* · 2c →B; the one table is the withheld seed note 31
  proposed, with K3's "source/flag and slot, not merely a template and
  a weight"; the build-time check is Astra's distinction between
  validating a definition and detecting a renamed line in a stash.

- **C95 — The counts view has two shapes and one optional sum; there is no aggregation language.** Several names give either independent facets, one small table each (AQ1), or one crossed table — two spellings, never one redefined as the other. A count may carry one summed thing beside it (a stack total); an item lacking the thing adds nothing and is counted as lacking, an unreadable one is counted apart (C93). Counting by `line` is the vocabulary read (C97). *Why:* the owner's "counts and total of different currencies" (S170) and the agent's own `GROUP BY 1, 2` (S189) are in reach, and collection aggregation was the largest learning cost A named and its first deletion. *Evidence:* S170, S189, S191. 2026-09-17.

  *checked* · 2d →A's reach, with K3 (keep facets; give the sum a
  missing rule). The missing rule itself is **chosen here**.

- **C96 — Place is terms, and a query never spans realms by default.** Realm, league, tab, character and container are terms like any other. Over a store holding more than one realm, a query naming none is an error that lists them (S199), and an explicit all-realms scope exists. Within a realm the default is every league, live items. The answer's scope block says what was searched: items, realms, leagues, locations never fetched, the oldest fetch. *Why:* a template and a total mean different things in two games (C58). The default is new here — A required every scope explicit, B spanned realms silently — and is labelled so. *Evidence:* S199. 2026-09-17.

  *checked* · 2e, with Astra's two notes: label the default new, keep an
  explicit all-realms scope.

- **C97 — The vocabulary is a read of this corpus, and it is the same call as search.** Counting by `line` under the query in hand lists the templates the matching items carry, by kind, ranked by how many items carry each, with the value range; several texts may be asked in one call, and every row carries the exact term that selects it, ready to paste. The closed lists — fields, kinds, flags, totals — are printed in the help. The owner's autocomplete and the agent's schema discovery are this one read. *Why:* S174 and S190 state one need; scoped discovery and the ready fragment were each the other model's reviewer's strongest praise. No call-count victory is claimed for either design. *Evidence:* S174, S181, S190. 2026-09-17.

  *checked* · row 3 →B with A's fragment; both halves the strong
  direction.

- **C98 — No search service: a consumer holds the corpus, and every answer is derived from one consistent basis and labelled with it.** The basis is one snapshot of facts with their location metadata, the intent revision, and the reference and derivation versions; a held corpus is reused only while the store's revision — advanced in the transaction that changes facts — still matches, checked before every answer, with no timer and no message to miss. A write after the basis is chosen does not unmake a correctly labelled answer. Nothing is persisted first: a persisted projection is a candidate, never a preselected successor, and waits for a measured number. *Evidence:* S135, S150–S153. 2026-09-17.

  *checked* for unpersisted-first (row 4 →B, weak direction) as "a
  provisional implementation choice" in Astra's words. **Chosen here**
  (K4): the basis as the contract, and — note 31's question 4 — that a
  revision comparison before every answer is "by construction", which
  is a reading of the owner's own floor text in note 22 and is his.

- **C99 — Direction. The trade boundary is a translation with a report, in both directions, and the stat id lives only here.** A site URL decodes with no network (S56). Each clause lands as exact, ambiguous, unsupported or listing-only, and a nonempty remainder means the result is never run as equivalent; a clause the published model cannot represent stays in the remainder. The site's average of a two-number line (S47) exists only in the translation; an undecided pair is shown with both candidates (S67). A line with no stat id cannot cross (S155). *Why:* the report makes a missing mapping honest; it does not supply one. *Evidence:* S26, S41, S47, S56, S57, S67, S106. 2026-09-17.

  *checked* · row 5 →both, with K5 whole. "Direction" because building
  it is new scope (note 31) and its place in the order is stage 6's.

- **C100 — One answer shape, whatever was asked: the canonical query, the basis, the scope, the terms block, the total, a view, and how to continue.** A row is compact: id, name, base, rarity, place by name beside its id, and the lines the query touched with their values — matched-on is the decision view (C53); the body only through `show`, whose `--against` form is the why-not. Every printed id is accepted back (S198). A continuation is bound to its basis (C98) and refused across a change, with the query that restarts. `priced` is the effective price the pricing area defines (C81), joined read-only; the manual and game sides are fields to add when a question needs them; legacy is never a field (S178). *Evidence:* S176, S192, S193, S196, S197, S198. 2026-09-17.

  *checked* · row 6 →both, with Astra's "bind continuation to all
  semantic inputs" (the basis, C98).

- **C101 — A missing reach is a field to add, never a general door.** There is no raw-path accessor over the body and no general collection syntax. Socket colours are asked two ways: over the whole item, and within a link group, where every colour named must be met together by one group — any qualifying group, never the largest (S59). Socket shapes the census shows and the deriver does not decode stay counted as unread (S16). The trade site's item-reading filters not yet mapped stay an open catalogue (S42), listed as gaps. *Why:* reach is a property of the derivation (stance 6) and a derived field is the cheap class of change (S135); in use, `fact["corrupted"] = false` read unknown on nearly every item where the named flag read false. *Evidence:* S16, S42, S58, S59. 2026-09-17.

  *checked* · rows 8 (→A's reach; K6, settled by the owner's S59) and 10
  (→B, with Astra's "carry the S42 catalogue gap forward").

- **C102 — The digest's limits register is inherited whole, and each limit is an output.** S12, S14, S52, S53, S67, S107, S111, S177, S178: what the search says on meeting each is the register's wording. Added here: unreadable is the only form unknown takes (C93); binding occurrences stops twins being summed by accident and does not say which is local (S52, S72); a moved template under a shipped total looks like a lacking one (C94); a site defence bound is inexact (C99); base defences, and which categories count as equipment, are game knowledge and the user's (stance 4). The search never fuzzy-matches, values, judges legacy, fetches, acts on the stash, or recovers the mod behind a line. 2026-09-17.

  *checked* · row 7 →both, with Astra's twin qualification.

- **C103 — One deriver per fact.** Place and every other ingest fact — realm, league, location, container — come from the store's columns and are never re-derived from a body; what a body displays is derived in the search crate, by a pure function of the body and those columns that names no store type, so it can move under the store unchanged if persistence fires (C98). The store gains what that needs and nothing more: a streaming read of bodies with their ingest columns under one snapshot, and a revision. `Store::search` is superseded when the search answers its questions. *Why:* two reads in one crate already mean different things by "in this league" (S131); two derivers over one body is that bug with a crate line through it. *Evidence:* S127, S128, S131. 2026-09-17.

  *Standing:* **ruled** with C89 under question 1 ("I agree to 1
  entirely"), the store read included. Split from C89 so each holds one
  thing: the crate and its edges; the deriver and what the store owes it.

### C48, amended in place (`CONTEXT.md`) — owed by note 23

- **C48 — The facts schema is internal, and raw SQL over it is not a surface; read-only SQL over the search's published relations is one, on the CLI and the MCP alike.** The published contract is the derived relations, under their own version, never the facts file's — filled by the same derivation and reference tables the model evaluates (C94), so SQL is a second language over the model and never a door around it; a gap in the model is never closed by SQL. Facts keep schema versions and compatibility errors, defended by a store API expressive enough that going around it is never worth it. No cached search service (C98). *Why:* stale results mistaken for current truth is the failure a cache reintroduces. Decided 2026-08-31; amended 2026-09-17.

  The first half is the owner's settled floor (note 22), so it is
  *ruled* in substance; the wording is mine. **Open inside it:** with
  nothing persisted there is no database for SQL to run over. My
  leaning (**chosen here**, and the thing in this packet I am least
  sure of): the search crate fills an in-memory SQLite from its held
  corpus — it links `rusqlite` and still opens no file — and the DDL
  moves under the store with the deriver if persistence fires. The
  alternative is to park the SQL surface behind persistence, against the
  floor. The amended text above holds under either.

### C1, amended in place (`CONTEXT.md`) — 670 bytes

C1 was 793 bytes because it did two jobs: the ruling, and a crate-by-
crate table of edges that grows about 100 bytes per crate. The table's
home is the check that enforces it from the real graph and prints it on
every run; C1 keeps the ruling and the three edges that make it a
boundary. Ruled 2026-09-17 ("I prefer your alternative").

- **C1 — Cargo workspace, library-centric; the daemon is its own artifact.** Every crate's edges are declared by what it links and refused by the check: `acquisition-daemon` (binary `acqd`), the only GGG sender, links protocol and store and nothing else; the store links none of them; the protocol crate is serde-only; a frontend links client, protocol, store, plan and search as it needs them, never the daemon; the daemon and the planner never link search (C89). The full graph and its rules are the check's own record. *Why:* write/test logic once, and C12's two surfaces as edges the check refuses. *Pinned:* `tools/docs-check.sh`. Amended 2026-09-09, 2026-09-17.

## 2b. Mechanism as recorded, until built

The registry's rule: a decision that needs more than its bullet is a
decision plus a mechanism, and the mechanism goes to the code. There is
no code yet, so — as C71 did with note 10 — it waits here, under its id,
and `decisions/search.md` points at this section until a module doc
takes each paragraph. Nothing here is trimmed from a line to fit a
number; it is what the first drafts carried that was never a boundary.

- **C89, C103.** `Store::search` and its `items_names` index (S124, serving
  no read) become deletable once the search answers what they answer
  (S136). The check gains: search ∌ daemon, client, HTTP, async runtime;
  daemon ∌ search; plan ∌ search — with a breaker case each
  (`tools/docs-check-breakers.sh`). The crate adopts the store's
  `unwrap_used`/`expect_used` denial (C47).
- **C89, C1 — the check.** The rules and the printed summary in
  `tools/docs-check.sh` §5 gain: search links store and plan only, never
  daemon, client, HTTP or an async runtime; daemon ∌ search; plan ∌
  search — each with a breaker case in `tools/docs-check-breakers.sh`,
  so the new rule is proven to refuse before it is trusted.
- **C90, C102.** Twins stay one line (S52, S72); a template that
  resolves to nothing answers with the templates sharing its words
  (S107). Both are the register's, and C102 carries them.
- **C92 (K1).** The terms block counts, beside "items matching", the
  items whose lines reach a bound only together — defined for a
  single-number bound and nothing else. The spelling offered for several
  conditions in one term is §1's; it is internals.
- **C93 (K2).** `true or undecided` is true; `false and undecided` is
  false; `not undecided` is undecided; an item undecided at the root is
  excluded and counted. `-has:reqlevel` is how OQ5 asks absence, and an
  unreadable requirements array does not satisfy it.
- **C98 (K4) — notification beside the check.** A long-lived frontend
  may listen for the daemon's job events and reload the moment a refresh
  lands, as the C++ app's signal did; that is responsiveness and is
  allowed. The revision check before every answer is what makes the
  answer correct when the change came from another client, a missed
  event, or a machine waking. Whether events reach a client that did not
  submit the job is unverified. Reload is whole (about 200 ms at 36,139
  items from JSON, S150); an incremental update over `item_events` is
  parked until a real GUI feels it.
- **C98 (K4).** The numbers that would move persistence: the streaming
  body read; the re-derive under an active refresh; a CLI ask over
  500 ms. S152 keeps direct SQLite open for a consumer that does not
  outlive its query.
- **C100, C101, C103 — where raw JSON is seen.** A result row never
  carries a body. One item's raw body is seen through `show <item>`, on
  request, one at a time (owner, 2026-09-17: "i agree with using show
  <item> for this"); the query language has no path into it; SQL runs
  over the derived relations. The store's bulk read (C103) hands bodies
  to the search crate in-process and, being a public store read, to any
  code that links the store — as `Store::search` does today. Tab and
  character bodies have no read today and gain none.
- **C99 (K5).** `count` carries its minimum and its maximum; `weight`
  and `weight2` differ in how a per-stat requirement gates a
  contribution (S57), and a weighted group translates only where that
  guard has a representation; a defence bound is inexact while the site
  normalises quality (S26) and sits in the remainder until a normalised
  field exists. Several ids for one line go out as a `count` of at
  least one (S106).
- **C48.** The template convention and place by name are in the
  published schema (S200).

## 3. The parking lot (→ `decisions/search.md`, "Parked")

- Saved queries, user names, tags → user-scoped intent through the store. Trigger: the `user.db` park firing (`decisions/store.md`). Then: A's version rule, B's composability, and the bare-word collision solved first. A query is already a value that travels between clients (C91); only the write is parked.
- A persisted projection (A's transactional one is a candidate; direct SQLite for short-lived consumers is another, S152) → under the store, the deriver moving with it (C89). Trigger: a measured number — the streaming body read, the re-derive under an active refresh, or a CLI ask over 500 ms.
- A grouping above class (`category`) → the class table, by reviewed membership and base-name rules, never a dotted prefix (K7: the 14 parents are unions; seven leaves need base names). Trigger: stage 6 reaching OQ5. *New in note 31; logic-checked by Astra, used by no one.*
- Query by example (the one-item view prints the item as an editable query) → a rendering over C92. Trigger: the first seat that asks. *Reached by neither proposal.*
- A quality-normalised defence field → the derivation. Trigger: a translated trade query with a defence bound in its remainder (C99).
- The totals coverage trial (how many lines containing "Resistance" does `total-res` not count, on the real corpus) → one script over the census. Trigger: stage 6 shipping the first total.
- The mistake walk over the combined model (the silent modes first: bare word as name, `#` in quotes, `colors`, `or` under an appended term) → stage 6, step 1, before the grammar is built.
- The digest's kill-list candidates (note 31) → stay candidates. Trigger: stage 6's acceptance tests written; recheck against them and the combined model before any removal (Astra).

## 4. For the owner — asked and answered

Asked as five questions, laid out in plain language on request, and
answered 2026-09-17. Spelling corrected per note 23's rule.

1. **C89's edges and C103** (the crate you agreed to, plus the edge
   rules, the one-deriver rule and the new store read; and whether raw
   item, tab or character JSON reaches a client — it does not, except
   one item at a time through `show`). Verdict: "i agree with using
   show <item> for this. I agree to 1 entirely."
2. **C98, "by construction"** — the revision check before every answer,
   with the daemon's events for promptness, as the Rust-side equivalent
   of the C++ app's in-process signal. Verdict: "Yes, with that
   explanation I agree to 2."
3. **The SQL surface's home** while nothing is persisted. Verdict:
   "i'm also unsure about sql. Let's come back to this after resolving
   the others." — **open**; C48's amendment waits on it, and Astra's
   check is asked for its reading as one more input.
4. **C1** — the owner first asked to raise the gate to 1 KB for this one
   rule ("I would rather expand the 800-byte limit for this rule to 1k.
   please think about this."); on the analysis that C1 carried a table
   whose home is the check, he ruled: "I prefer your alternative. Good
   catch to spot the table pattern and move it out." The 670-byte C1
   above is the ruling.
5. **Astra's check of this packet** before the harvest. Verdict: "yes".
   The brief is `brainstorming-notes/33-search-ruling-check-brief.md`;
   Astra's check is appended to this note as its last section and
   committed from Codex, as in stage 4.

### The owner's verdicts, verbatim

- On the reconciliation's twelve questions: decision 15, note 23.
- On link groups: digest S59.
- On the crate, 2026-09-17: "I like the idea of a separate crate, but I
  want us to be thoughtful about this"; then, after the options were
  laid out: "Agreed with a new crate".
- On this packet's five questions: above, under each.

## 5. Measured sizes

Bytes of each candidate bullet as it stands in §2, against the 800 gate
(`tools/docs-check.sh`, `ENTRY_LIMIT`); C1 today is 793 and its amendment above is 670.

| Line | Bytes |
| --- | --- |
| C89 | 598 |
| C90 | 705 |
| C91 | 708 |
| C104 | 683 |
| C92 | 714 |
| C93 | 777 |
| C94 | 730 |
| C95 | 724 |
| C96 | 666 |
| C97 | 740 |
| C98 | 712 |
| C99 | 689 |
| C100 | 775 |
| C101 | 792 |
| C102 | 709 |
| C103 | 785 |
| C48 | 756 |

## What I left out, and am least sure of

I did not re-read notes 26 and 27 or either audit; I took note 31's
account of them, as corrected by Astra. I read B's model and grammar
pages and only the binder lines of A, so §1 is written in B's idiom by
B's author — the bias note 31 named is compounded here, and Astra's eye
on §1 is the remedy I can name. The slot-comparison spelling in §1 has
been typed by nobody. I am least sure of the SQL home, then of whether
"the largest satisfying occurrence" is what a person sorting by life
expects when a second, smaller line also matched.
