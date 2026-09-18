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
and a remainder (row 5). There is no SQL surface (the owner,
2026-09-17, §4 question 3); the derivation keeps a table shape — lines
as rows, fields as columns, place by name — so an export can be added
when a recorded question asks for it.

### The concept inventory, and what it cost

Astra's check asked for this and said plainly that subtracting A's 16
from B's 11 would mislead. The combined model has **15**: item and its
three parts · line as occurrence · kind (two coordinates) · name (field,
total) · term and comparison · slot comparisons in one term ·
composition with bounded `holds` · `sum` and the totals table · absent
against unreadable · place terms and the realm rule · ask and the one
answer shape · the three views, with two count shapes and a sum · the
vocabulary as counts by line · basis and continuation · the trade
translation and its remainder.

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

- **C94 — A named total is a sum that answers, defined once as reviewed reference data (C68).** A row names the contributing line — template, kind where it matters, slot — and its weight; the evaluator is generated from that table and nothing else defines a total, so the shipped totals have one meaning in every surface. The answer prints a total's definition on request and each row's contributions. A definition is checked against the reference data, loudly, when the table is built; a template the game has since moved and one the stash merely lacks look the same in a corpus, and that stays a listed limit (S111). *Why:* a total that may decline failed the everyday question in use (stance 3). *Evidence:* S29, S49, S111. 2026-09-17.

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

### C48 — not amended; the SQL surface withdrawn (owner, 2026-09-17)

Note 23 owed stage 5 an amendment of C48 that made read-only SQL over
a published contract a surface. The owner withdrew the surface instead
(§4, question 3), so C48 stands as written on 2026-08-31: the schema is
internal, raw SQL is not a surface, no cached search service. The
draft amendment and its open question (an in-memory SQLite the search
crate would fill on request — Astra's check below reads it as
"defensible, but still open") are history at `a0d85b23`. What the
floor said SQL was for, the search now answers otherwise: reach is a
property of the derivation, the vocabulary read shows what exists, a
gap is listed and closed by a field, and every answer is JSON under
C53. The parking lot carries the export form and its trigger.

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
  <item> for this"); the query language has no path into it. The store's bulk read (C103) hands bodies
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
- **The derivation's shape.** Lines as rows (one per occurrence), fields
  as columns, place by name (S200): the shape the model produces, kept
  so that a JSON or SQLite export, or a persisted projection, is a
  writer over it and not a redesign.

## 3. The parking lot (→ `decisions/search.md`, "Parked")

- Saved queries, user names, tags → user-scoped intent through the store. Trigger: the `user.db` park firing (`decisions/store.md`). Then: A's version rule, B's composability, and the bare-word collision solved first. A query is already a value that travels between clients (C91); only the write is parked.
- A persisted projection (A's transactional one is a candidate; direct SQLite for short-lived consumers is another, S152) → under the store, the deriver moving with it (C89). Trigger: a measured number — the streaming body read, the re-derive under an active refresh, or a CLI ask over 500 ms.
- Whole-corpus export of the derived lines — JSON first; a SQLite file stamped with its basis if JSON in hand is not enough — → the search crate's CLI/MCP adapters, as a writer over the derivation's table shape. Trigger: a recorded question the term language, the counts view and the JSON answer could not serve. *The SQL surface the floor once named was withdrawn 2026-09-17; SQL as an engine was declined by measurement (S146–S148); a surface over a persisted projection is the one form that would be cheap, and rides on C98's trigger, not its own.*
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
3. **The SQL surface's home** while nothing is persisted. First: "i'm
   also unsure about sql. Let's come back to this after resolving the
   others." Then, with Astra's C48 reading in hand, the owner stepped
   back: "i have been insisting on SQL as a surface, but I didn't create
   acquisition and I'm not a CRUD or database expert. Is it possible the
   SQL surface as a requirement is doing some unaccounted-for harm like
   constraining our design or implementation? Are we duplicating work,
   adding complexity, or creating more room for bugs? should we consider
   making SQL the primary interface language for search?" Fable's
   reading: SQL as a surface is a second *meaning* where the design
   allows only second *spellings* (C46, stance 5); the insurance it
   bought is bought otherwise by stance 6; as the primary language it
   makes stance 3's everyday case the advanced one (S187); as an engine
   it lost by measurement. Verdict: "I agree with (b) now, and possibly
   (c) if json export isn't sufficient for some future need. Between
   pseudo-mods, counting, weighting, and all the other search features
   we need, I suspect the only way an SQL surface makes sense is if we
   have an SQL table behind it, which so far nobody has been pushing
   for." — **the SQL surface is withdrawn**; C48 stands unamended; note
   22's floor bullet is marked changed; the export is parked with its
   trigger.
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
(`tools/docs-check.sh`, `ENTRY_LIMIT`); C1 today is 793 and its amendment above is 670; C48 is unchanged.

| Line | Bytes |
| --- | --- |
| C89 | 598 |
| C90 | 705 |
| C91 | 708 |
| C104 | 683 |
| C92 | 714 |
| C93 | 777 |
| C94 | 744 |
| C95 | 724 |
| C96 | 666 |
| C97 | 740 |
| C98 | 712 |
| C99 | 689 |
| C100 | 775 |
| C101 | 792 |
| C102 | 709 |
| C103 | 785 |
| C1 | 669 |

## What I left out, and am least sure of

I did not re-read notes 26 and 27 or either audit; I took note 31's
account of them, as corrected by Astra. I read B's model and grammar
pages and only the binder lines of A, so §1 is written in B's idiom by
B's author — the bias note 31 named is compounded here, and Astra's eye
on §1 is the remedy I can name. The slot-comparison spelling in §1 has
been typed by nobody. With SQL withdrawn, I am least sure of whether
"the largest satisfying occurrence" is what a person sorting by life
expects when a second, smaller line also matched.

## Astra's check — stage 5

**Verdict: ready with named changes, not ready to harvest as written.**
The combined direction survives this check: compact terms, explicit
occurrences, discovery as a scoped read, and answers that carry their
evidence. The binder fixes K1's counterexample; K5's translation
remainder and K6's socket correction are preserved. The changes needed
are to finish C92's scalar and diagnostic rules, complete C93/C95's
unavailable-value semantics, make C98's reuse rule cover its stated
basis, reconcile C102 with the limits it inherits, and resolve the
open C48 home. C96/C97 need their realm boundary made explicit, and
the page and registry need the smaller corrections below. These are
meaning differences a builder should not settle accidentally, not a
reason to reopen the whole design. C48 remains the owner's explicitly
open question; this check supplies a recommendation, not its ruling.

Astra, from Codex, 2026-09-17, checking the packet at `a0d85b23` under
`brainstorming-notes/33-search-ruling-check-brief.md`. Read notes 28 and
31 whole, note 22, the digest, note 23, the registry form and pricing
area; reached into notes 24 and 25 only for the attributed sections.
Checked the relevant store/frontend rulings and the existing snapshot
read for the specific claims below. No new corpus experiment, external
fetch or user seat was run. My recommendations remain one voice; my
stage-4 check and my own proposal are not independent support for them.

### C92 — the binder works; a vector has no largest scalar

**Sentence:** “Conditions on several numbers of one line are written
in one term and hold on one occurrence together”. This resolves the
important part of K1. For occurrences `[15,40]` and `[5,20]`, the
page's `"Adds #>=12 to #<=28 Cold Damage"` matches neither. Separate
existential terms would match. The compound term is a sufficient
binder; it does not need A's general collection syntax. Its exact
punctuation can remain an internal choice, tested by the mistake walk.

**Sentence:** “Where a term sorts or is shown as one number and several
occurrences satisfy it, the largest is used”. This is complete for
one-number lines, but not for the multi-slot term just introduced.
Occurrences `[15,28]` and `[20,25]` both meet low ≥ 12 and high ≤ 28.
The largest low is 20; the largest high is 28; they belong to different
occurrences. Even one occurrence `[15,28]` leaves two candidate scalar
values. The sentence also does not say what an unbounded sort uses, or
what is shown for a row admitted by another branch of an `or` when
this term has no satisfying occurrence.

**Resolution; my reading:** keep maximum for a selected scalar slot,
over the occurrences satisfying the whole term. With no comparison,
use the occurrences selected by the line selector. Require the slot
when more than one remains; show the actual vector as a vector, never
invent a representative vector from independent maxima. When no
occurrence qualifies there is no scalar; put it last in either direction, with
an explicit unavailable status where appropriate. This keeps AQ5's
simple maximum and does not silently restore the mean. The rejection
of an unqualified multi-number `sum`, naming its slots, is expressly
in note 31 §5 and missing here: preserve it in the harvest or mark its
replacement as a new choice. “Never averaged” in C90 forbids one wrong
answer but does not choose among all the others.

**Sentence in §2b:** “items whose lines reach a bound only together —
defined for a single-number bound and nothing else”. Restricting the
diagnostic to one number is right, but is not yet a definition. Specify
the comparison and source/flag selection, whether the selected
occurrences are completely readable, and its denominator. A useful
initial rule, **my reading**, is a lower bound on a one-slot line
selector: over C93's item scope, count an item once when no occurrence
meets the bound and their complete sum does. Thus 20 + 75 counts for
≥ 90; 95 + 5 does not; 20 plus an unreadable possible contributor
cannot establish the diagnostic. Upper bounds, equality and compound
terms need no such counter initially; report not applicable, not zero.
This is an additional diagnostic, not a fifth disjoint bucket beside
C93's four. The promise that the diagnostic exists belongs in C92;
the evaluation recipe can stay in §2b.

### C93 — retain undecided, and finish its composition

**Sentence:** “and, or and not carry it (true or undecided is true; not
undecided is undecided)”. I would make this choice too: it preserves
a known positive without admitting unreadable data through negation.
It resolves K2's original problem, and the fourth per-term bucket is
the right correction. However, C91 also admits bounded `holds`, which
neither this sentence nor §2b defines for undecided children.

**Counterexample:** exactly one of `(true, undecided)` cannot be called
true: the unreadable child might make the count two. Treating it as
false would produce that wrong answer. At least one of the same
children is already true; demanding all children be readable would
discard that known answer.

**Resolution; my reading:** let `t` be true children and `u` undecided
children. For an inclusive count bound `[lo, hi]`, the possible count
interval is `[t, t+u]`: true when wholly within the bound, false when
disjoint, otherwise undecided. An omitted upper bound is unbounded.
This is a conservative composition rule, not a demand to prove
correlations between child expressions. Record the complete Boolean
rule beside it, including `false and undecided = false`.

**Sentence:** “A body or array the deriver could not read makes the
terms that needed it undecided”. Give line existence the same witness
rule. A readable explicit life line of 95 already establishes an
unqualified life ≥ 90 atom even if another eligible array is unreadable;
with only a readable 20 and the same unreadable array, it is undecided.
A sum needing that array is still unavailable. Do not turn a whole
item invalid merely because one array failed. Conversely, do not
infer absence from an incomplete collection without a positive
witness. C102 below also names unavailable evidence that is not a
body/array parse failure.

**Sentence:** “Per term … matched, compared and failed, lacked,
unreadable — four that sum to the scope”. State whether these are
pre-negation atomic diagnostics or results of compound terms. For a
known-absent requirement, `-has:reqlevel` matches while `has:reqlevel`
lacks it; counting both against the same four-way partition would
double-count the item. **My reading:** classify each atomic term before
outer Boolean composition, over the fixed item scope and before
pagination, with no short-circuit omission of its diagnostics. Root
matches and root undecided remain separate answer counts. This retains
B's useful explanation without making the four buckets contradictory.

### C95, C94 — missing is not unreadable, and a subtotal is not a total

**Sentence in C95:** “an item lacking the thing adds nothing and is
counted as lacking, an unreadable one is counted apart”. The distinction
is good. It still leaves the value of an empty sum and the status of
the number returned alongside unreadable inputs unspecified (K3).

**Counterexample:** an emitted group contains stack values `10`, known
absent, and unreadable. Its item count is three. A builder may return
`sum: 10` plus two diagnostics; another may return no complete sum plus
`known_subtotal: 10`. Those are different claims about the answer.
Likewise, a group whose values are all known absent could yield zero
or no value under “adds nothing”.

**Resolution; my reading:** sum the known contributors; an empty
complete sum is zero, with its lacking count. If any required
contribution is unreadable, return the known subtotal and an explicit
incomplete status, never an unqualified total. Counts still cover all
matching items before pagination. Apply the same distinction to C92's
explicit sums and C94's recipe sums: a numeric predicate on a sum whose
required input could not be read is undecided. This does **not** restore
A's conservative recipe coverage policy. A readable line absent from
the reviewed recipe is not, for that reason alone, an unknown
contributor. The recipe answers its declared definition; unreadable
inputs cannot supply a known value for that definition.

**Sentence in C94:** “A row names the contributing line — template,
kind where it matters, slot — and its weight”. K3's source/flag and
slot correction is present. Make realm applicability part of that
definition too, consistent with C90's identity and C96's explicit
all-realms mode. A same-text line in two games is not evidence for
reusing a recipe across them. Do not treat absence of a recipe for a
realm as a known zero. The one table and the build-time validation
remain supported; the moved-template limitation is honestly retained.

### C98, C100 — the basis is right; the reuse condition is narrower

**Sentence:** “The basis is one snapshot of facts with their location
metadata, the intent revision, and the reference and derivation
versions; a held corpus is reused only while the store's revision …
still matches”. The first half correctly harvests K4. The second names
only facts as the reuse condition, although §1's corpus also joins
intent. It is safe if “corpus” means only reusable fact derivations
and intent is read afresh against each answer's consistent basis; it
is unsafe if it includes a resident effective-price column.

**Counterexample:** facts stay at revision 7 while another client
changes a price at intent revision 12 to 13. A resident `priced`
value from 12 cannot answer the next ask just because facts still
equal 7. Reading the new revision number without the corresponding
intent values would mislabel the result rather than fix it.

**Resolution:** say what is reusable and require every reused component
to agree with the new answer's basis. A consistent facts/intent basis
means compatible held snapshots, not sequential revision numbers read
around unrelated data. The mechanism for obtaining that basis belongs
in §2b/code; K4 did not require A's particular retry mechanism. Facts
revision must also advance for changes to joined location metadata and
membership, not merely changed item bodies. C103's read must use the
store's authoritative current joins: choosing `items.league` alone
would preserve S131's old-stamp bug despite using a store column.
Include store/provider/account identity in the basis binding, as
well as the versions: revision 7 in two accounts is not one snapshot.
C100's continuation then names that basis and the query/view/order it
continues, rather than accepting a matching integer from another
answer. These are identity and consistency properties, not a request
for a particular token encoding.

I would retain the unpersisted first build and revision validation.
Neither my proposal's persistence nor an event subscription substitutes
for this contract. **Standing correction:** §4 answer 2 already rules
the revision-check interpretation “by construction”. The opening
“every line but C89, C103 and C1 is provisional” and C98's annotation
must acknowledge that ruled part, while leaving its unpersisted-first
implementation choice provisional. Do not ask the owner to approve
the same interpretation again.

### C48 — an in-memory SQL home is defensible, but still open

**Sentence below the amendment:** “the search crate fills an in-memory
SQLite from its held corpus”. **My reading:** choose that as the first
SQL adapter, over the same basis and derived values as ordinary search,
constructed when SQL is requested. It satisfies the already-settled
CLI/MCP symmetry without requiring a persisted projection or another
process. It is already proposed in B §3; seeing it here is not an
independent discovery. Its actual load and memory cost are unmeasured;
S157 measured a different, indexed file build and is not an upper
bound on this implementation. Measure before choosing its successor.

**Sentence:** “SQL is a second language over the model and never a
door around it”. Preserve that as the boundary, with the published
relations' version, basis and availability meanings specified before
building the SQL adapter. One SQL `NULL` for both known absence and
undecided would lose C93's distinction under negation; the relation
needs enough status information to express the model's meaning.
C94's common totals table prevents duplicated recipes, not every
possible semantic disagreement. Likewise, SQL arithmetic over two
slots does not close a missing expression in the ordinary query model
(K5); that remains a listed model gap. The SQL adapter must actually
confine reads to its derived representation; accepting arbitrary file
attachment would not satisfy “read-only over published relations”.
The confinement mechanism belongs in code, not a new framework here.

**Resolution:** retain the amendment as a candidate until the owner
returns to §4 question 3, and present the in-memory adapter with these
conditions as the recommendation. Parking SQL behind persistence is
not an equivalent alternative under the current floor: it would
require an explicit change to the already-ruled surface. The sentence
“The amended text above holds under either” needs that qualification.
This check does not supply the missing approval, and does not need
another research round to name the available choice.

### C96, C97 — naming a realm is not necessarily restricting to it

**Sentences:** “Realm … [is a term] like any other” and “a query naming
none is an error”. Mere presence of a realm term does not define a
single-realm scope under arbitrary Boolean composition.

**Counterexample:** `realm:pc or "# to maximum Life">=90` names pc but
can match a poe2 item through the second branch. One builder could
search both realms; another could extract pc as an outer restriction,
silently changing the tree's meaning. `-realm:pc` exposes the same
problem without an `or`.

**Resolution:** define the realm domain independently of whether a
realm token appears somewhere in the tree; show the resolved domain
in the canonical request/answer. **My reading:** require an explicit
single-realm or all-realms domain when the store holds several, while
allowing the adapter to abbreviate an unambiguous positive realm
restriction. Do not hoist a realm from an arbitrary branch. This keeps
place predicates and the permitted all-realms mode without prescribing
A's `in S` syntax.

**Sentence in C97:** “lists the templates … by kind … every row
carries the exact term that selects it”. In all-realms mode, retain
C90's realm coordinate in vocabulary grouping and in a row's selecting
fragment. Otherwise `(pc, kind, template)` and `(poe2, kind, template)`
collapse to one row despite the proposed identity. Value ranges must be
per ordered slot, not a min/max pooled across a damage line's two
numbers. These are consequences of C90, not new reasons to prefer A's
discovery operation. Source and flag selectors must both remain
expressible; keeping both in storage alone would not settle row 1's
`explicit`/`fractured` interpretation.

### C102, C93 — the limits register has more than decode failure

**Sentences:** “The digest's limits register is inherited whole” and
“unreadable is the only form unknown takes”. Taken literally, the
second contradicts the first. S14's map area is unavailable despite a
perfectly readable body. S53's unclassed item can be a readable base
missing from the reviewed class table. S107/S111 also describe
unresolved names, rather than necessarily malformed arrays. Within
the existing registry, C81 requires an unreadable decisive annotation
to leave the effective price unresolved; C93 names only bodies and
arrays, although C100 queries that price.

**Counterexample:** a fully readable item has no established class
mapping. Treating this as ordinary absence makes negation of a class
comparison a known positive. Neither the lack of a mapping nor C102
establishes that the item belongs outside that class. Similarly, an
unresolved effective price must not become confidently unpriced.

**Resolution:** distinguish known absence, unavailable evidence or
derivation, and an unsupported/unresolved query name. **My reading:**
retain one undecided evaluation outcome where the item's answer cannot
be established, with a named reason; unsupported requests and unbound
templates get their explicit diagnostics. Decode failure is a reason,
not the exhaustive definition of unknown. Known noncontributors to a
recipe remain C94's chosen case, as explained above. Say the limits
are inherited **with the stated S111 exception**, rather than both
verbatim and excepted. This preserves the simplification of ordinary
absence without pretending the register contains only damaged data.

### C89, C100, C101, C103 — correct the raw-JSON account

**Sentence in §2b:** “Tab and character bodies have no read today and
gain none”. The character half is factually too broad. The public
`Store::refresh_snapshot` returns `CharacterSnapshot`, whose `listed`
is the listing entry and whose `fetched` is the character envelope
minus its lifted item arrays (`crates/acquisition-store/src/snapshot.rs`,
the struct and `read_characters`). Those are existing neutral reads,
not a new search escape hatch. A full original character response is
not what that API returns, but “no read” does not describe it.

**Resolution:** state the distinction precisely: search results expose
one requested item body through `show`; no raw-path query or bulk raw
search output is proposed; existing in-process store snapshot reads
remain what they are. §2b already makes that distinction for item
bodies. Carry it through the explanatory gloss in §4 question 1
without rewriting the owner's quoted verdict. Nothing here calls for
changing the accepted crate edges or revoking an existing store API.

### C90–C104 — trace of the checked harvest and the page

The following checks are in addition to the semantic findings above;
an unchanged judgment is not another vote for its evidence.

| Line | Row/K trace and remaining harvest issue |
| --- | --- |
| C90 | Row 1's text identity, signed ordered slots and both kind coordinates are retained. No new game fact is claimed. Do not let the phrase “set of lines” on the page erase repeated occurrences; it is a collection with multiplicity. |
| C91 | Row 2 and K5's bounded composition are retained. “Text that could be read two ways is an error that shows both readings” is a stronger choice than row 2's “one visible rule or an error”, as its annotation admits. Mark that sentence chosen here, and restrict the promise to defined language ambiguities; the parser cannot diagnose every possible human reading. I prefer visible deterministic rules plus errors for actual unresolved alternatives, with the mistake walk deciding the idiom. |
| C104 | Row 12's adapter preference is properly discounted. “The tree parses from and prints to the text bijectively” should mean the canonical text and canonical tree. Parentheses and whitespace already allow distinct input strings for one query; literal bijection over all accepted text contradicts canonicalization. No evidence establishes that text is always shorter than JSON; the comparative rationale is a design judgment, not S195's finding. |
| C92 | Row 2a's occurrence default and K1's binder are preserved. Scalar selection and the together-count need the completion above; note 31 §5's multi-slot sum rejection was dropped. |
| C93 | Row 2b is correctly changed under K2, with its change to internal three-valued evaluation disclosed. Complete it for bounded counts, line witnesses and diagnostic partitions; do not broaden “decode failure only” into a claim about every limit. |
| C94 | Row 2c and K3's shared recipe are retained, and definition validation is distinguished from detecting a renamed line in the corpus. Realm and unreadable-input handling need the qualifications above. |
| C95 | Row 2d keeps both the independent facets and crossed counts K3 required. Its new missing rule needs the complete/incomplete sum distinction above. |
| C96 | Row 2e's new default is labelled, and the stage-4 all-realms qualification survives. The interaction with arbitrary Boolean place terms needs the boundary above. S199 supports the informative error, not the single-realm policy itself. |
| C97 | Row 3 keeps both scoped vocabulary and ready fragments without a call-count victory. Preserve realm/slot identity as above. Its help-list promise is supported; the combined page does not yet supply that list. |
| C98 | Row 4 and K4 are substantially retained; the later persisted projection is correctly a candidate. The complete-basis property must govern reuse as well as the answer label. |
| C99 | Row 5 with K5 is faithfully narrowed: bounded counts, guarded weights only where representable, remainder otherwise, defence normalization gap. No new mapping is proven. The average “exists only in the translation” must not be read as claiming an exact public-tree representation that the packet has not supplied. |
| C100 | Row 6's compact answer and K4's continuation are retained. S193 also requires caller-selected fields; the line states only the default, although B §1 allowed extra columns. Preserve that reach or list the omission; do not cite S193 as wholly met by the compact default. C81 is the correct effective-price authority. |
| C101 | Rows 8/10 preserve any qualifying socket group, undecoded socket shapes, and the open S42 catalogue. A single group must also bind a combined link-count/colour request when that S58 shape is implemented; independent witnesses cannot stand in for one group. No general collection language is required. |
| C102 | Row 7's non-goals and the twin qualification are intact. Its stronger “only” and “inherited whole” formulations need the exceptions and unavailable-evidence distinction above. |

**§1's provenance:** most sentences trace to the named rows and Ks;
the slot punctuation is correctly marked new. The compound-term
choice is an implementation of K1's required binding, and the fourth
diagnostic bucket follows K2. The choice of undecided propagation is
one of K2's offered alternatives, not its verdict: copy C93's *chosen
here* mark onto the page so the citation does not obscure that
distinction. The unsupported attribution in the next paragraph is a
substantive exception to the page's traceability claim.

**§1's intent sentence:** “Intent joins read-only: `priced`, `price`,
`note` (row 6).” Row 6 joins price; B §3 sources `note` from the body.
Do not imply a user note annotation kind was ruled. Describe the
observed item note as a fact; a new local note kind would be new scope.

**Omissions that matter to coherence:** the page does not state C96's
single-realm default or explicit all-realms mode, C101's same-link-group
term, or the C92 scalar/together diagnostic choices. The inventory
mentions place/realm and the prose says group binding survives, but
neither teaches the reader how those work. C91's safe `(old) new`
refinement and `has:` for deliberate absence also belong on this page;
the history names both as places users made mistakes. The page's
answer has four term buckets but no together-count. Its SQL sentence
describes the intended surface without disclosing §4's open home.
These omissions do not require a second specification: add the small
rules and the open marker, leaving the detailed recipes in §2b.

The inventory is a useful admission that the combination costs more
than B. Its 15 is not comparable with either proposal's total until
the counting convention matches: it bundles both facet shapes and
their sum, while socket-group binding has no explicit inventory entry.
Also, “SQL is a second language … not a concept” does not remove its
learning cost. **Resolution:** make group binding visible and treat
the inventory as a list of obligations, not a numerical simplicity
result. The deferred mistake walk remains necessary; this logical
check is not that seat.

### Registry form — C92, C98, C100, C102, C1

All 18 candidate bullets have a date and fit the existing 800-byte
limit. The 17 listed sizes in §5 reproduce exactly; C1 is 669 bytes
without its newline, 670 with it. This one-byte inconsistency is only
bookkeeping. All cited `S` ids exist in the digest; every `C` id exists
in the current registry or this candidate packet. The important
citation issues are the overstatements identified above, not invented
identifiers.

One attribution in §2b also needs narrowing: “Both are the register's”
includes showing templates that share words with an unbound template.
That particular suggestion mechanism comes from B §4.1, not the
limits register's S107 wording. It is compatible with showing an
unknown without guessing a match; cite its proposal provenance rather
than treating it as prescribed by S107.

C98, C100 and C102 lack the registry's explicit *Why:* sentence. Add
it without converting their evidence list into a claim of necessity.
The basis, answer shape, occurrence binding and unavailable-value
behavior are boundary properties and belong in the bullets. Exact
slot punctuation, the chosen maximum's implementation, snapshot
acquisition and SQL population belong in §2b/code. Conversely, the
presence of C92's promised diagnostic is observable behavior; leaving
its entire promise only under “Mechanism” makes it easy to omit during
harvest. The repeated C89/C103 and C89/C1 dependency-check paragraphs
in §2b should have one home. Neither the byte gate nor moving text to
§2b can substitute for completing a boundary's meaning.

### Parking lot — C91, C94, C98, C99, C101 and the stage-6 work

| Park | Check and resolution |
| --- | --- |
| Saved queries, names, tags (C91) | The storage home is correct, but “when the user.db park fires” points to “the first user-scoped kind actually written”, potentially the saved query being postponed. Name the demand that authorizes that first write, such as the first consumer asking to save/name/tag; the store home is built as its prerequisite. Another kind creating user.db is not itself evidence to build all three features. Query values passed between clients are needed now. |
| Persistence (C98) | “A measured number — the streaming body read, the re-derive … or a CLI ask over 500 ms” gives a threshold only for the last case. Every body read produces a number, including an acceptable one. Treat the first two as measurements owed by the initial build; move the persistence decision on a measured failure of an agreed response budget, not merely their existence. Do not invent a GUI budget here. Any fired trigger opens the experiment among candidates, not automatic persistence. |
| Category (C101; K7) | Stage 6 reaching OQ5 is recognizable. Membership/base-name rules and the user's equipment definition remain distinct, correctly. This is an acceptance task before claiming OQ5 covered, not an optional later enhancement if the first slice claims that answer. |
| Query by example (C92) | “The first seat that asks” is recognizable and the feature is not required by the other lines. Keep it parked. |
| Quality-normalised defence (C99) | The remainder clause is an observable trigger. A listed translation/model gap is permitted until then; no claim of full stance-2 reach before its closure. |
| Totals coverage trial (C94) | Shipping the first total is recognizable, but the trial informs what is shipped: run it before accepting that first recipe. A word-based trial measures coverage; it is not proof that every line with “Resistance” belongs in a particular total. |
| Mistake walk (C91/C104) | “Stage 6, step 1, before the grammar” is explicit and already required. Put it in the build plan as a prerequisite, not among indefinite parks. It must use the combined page after the corrections above, not either old proposal. |
| Kill-list check (C102) | The acceptance tests' arrival is recognizable; rechecking before any deletion is right. The surviving floor gaps are evidence to retain claims even when neither proposal cited them. |

The remaining S42 item-reading catalogue and K5's unrepresentable
translation expressions also need explicit dispositions in the build
plan: implemented or listed gaps, never silently counted as reach.
The error would be to call the floor satisfied merely because the
unbuilt part has a park. None of this authorizes a new live read or a
fetch from a governed surface. C48's open owner choice is an open
decision, not a park that can be discharged by a latency measurement.
