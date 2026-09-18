# Item search — the design: binding contract detail (and, from session C, the language reference)

**Standing: binding.** This is the contract detail of the item-search
rulings (`decisions/search.md`, C89–C104): the rules that decide an
answer and do not fit a registry entry — the sort scalar, how undecided
composes, what an incomplete sum compares as, the diagnostic counts. Each
paragraph is headed by the decision it completes and binds with that
decision's standing (provisional under note 23's decision 15 unless the
owner ruled it in his own words). A paragraph lives here **until code
takes it**: when a module doc in `acquisition-search` carries it under
its id, the paragraph is deleted in that commit, so this file empties as
the crate is built and is never a second registry. The owner chose this
home 2026-09-18 (the stage-5 audit, finding 6: "I agree with (a)"); it
is the one document in `search/` that is an authority.

What is here was moved verbatim from note 28 §2b, where it was recorded
at the stage-5 harvest (2026-09-17) with Astra's completions accepted as
one voice, marked where they apply; the owner's answers verbatim and
Astra's check stay in `brainstorming-notes/28-search-ruling-packet.md`
(§4 and its last section). **Not here yet:** the language reference — one
self-contained page with an example per construct, which the mistake
walk needs (the audit's finding 5); session C adds it above this detail.

## Contract detail, by decision

- **C89, C103 — the check and the store.** `Store::search` and its
  `items_names` index (S124, serving no read) become deletable once the
  search answers what they answer (S136). When the crate exists, the
  rules and the printed summary in `tools/docs-check.sh` §5 gain: search
  links store and plan only, never daemon, client, HTTP or an async
  runtime; daemon ∌ search; plan ∌ search — each with a breaker case in
  `tools/docs-check-breakers.sh`, so the new rule is proven to refuse
  before it is trusted. The crate adopts the store's
  `unwrap_used`/`expect_used` denial (C47). The store's bulk read joins
  location the way `read_items` does (S131), never `items.league` alone.
- **C90, C102.** Twins stay one line (S52, S72), and binding occurrences
  stops them being summed by accident. A template that resolves to
  nothing answers with the templates sharing its words: B §4.1's
  suggestion, compatible with S107's "never guessed" and not prescribed
  by it (Astra).
- **C92 — slots, the sort scalar, the together count.** A bound of 90 on
  a life line holds when one line reaches 90, never when an implicit 20
  and an explicit 75 do together. A ranged line is one whose template
  reads `# to #`: `low` and `high` are its two slots and `avg` their
  mean; any line's slots are also addressable by position (`#1`, `#2`),
  the general case a non-ranged multi-number line uses — the owner's
  words apply to ranged modifiers only (note 28 §4, question 7). Sorting on a
  term uses the named slot over the occurrences that satisfy the whole
  term, the largest of them; with no comparison, the occurrences the
  line selector picks; a row admitted by another branch with no
  satisfying occurrence has no scalar and sorts last in either
  direction, its status shown; a vector is shown as a vector, never a
  representative built from independent maxima (Astra's reading,
  accepted). The together count: for a lower bound on a one-slot line
  selector, over C93's item scope, an item counts once when no
  occurrence meets the bound and the complete sum of its occurrences
  does — 20 + 75 counts for ≥ 90, 95 + 5 does not, and an unreadable
  possible contributor cannot establish it; upper bounds, equality and
  compound terms report not applicable, never zero. It is a diagnostic
  beside C93's counts, not a fifth bucket (Astra's reading, accepted).
- **C93 (K2) — composition and witnesses.** `true or undecided` is true;
  `false and undecided` is false; `not undecided` is undecided;
  at-least-N-of with `t` true and `u` undecided children has the count
  interval `[t, t+u]`: true when wholly inside the bound, false when
  disjoint, otherwise undecided, an omitted upper bound unbounded
  (Astra's reading, accepted). A readable line is a witness: an explicit
  95 life establishes life ≥ 90 even when another eligible array is
  unread; a readable 20 with the same unread array is undecided; absence
  is never inferred from an incomplete collection. The four counts are
  taken per atomic term before outer composition, over the fixed item
  scope, before pagination, with no short-circuit omission; root matches
  and root undecided are separate answer counts. `has:` is the spelling
  of presence; `-has:reqlevel` is how OQ5 asks absence, and an unread
  requirements array does not satisfy it.
- **C94, C95 — sums and the totals table.** Sum the known contributors;
  an empty complete sum is zero with its lacking count; if any required
  contribution is unreadable, return the known subtotal and an explicit
  incomplete status, never an unqualified total, and a comparison on it
  is undecided (Astra's reading, accepted). A readable line absent from a
  recipe is not an unknown contributor: the recipe answers its declared
  definition. A total's row: template, kind (source and flag) where it
  matters, realm, slot or none, weight. A row with no slot contributes
  its weight when the line is present, so the all-elemental line counts
  three times into "# total Resistances" as the site does; a total whose
  rows are ranged lines sums low with low and high with high and is a
  ranged value taking `low`, `high`, `avg`. The site's 298 pseudo stats
  by the mechanism each needs, per entry with the input that admits it
  (`search/pseudo-stats/`, 2026-09-18): 123 a field or property the
  private item carries (63 of them by an attested sibling, flagged);
  36 a weighted sum (the C++ app's 35 tables, S29, plus the site's own
  `pseudoMods` line proving a 0.5 weight, S49); 15 a ranged total; 14
  needing the mod behind the line (S52; one crafted modifier renders as
  two lines); 1 a count of lines; 0 computed; and **109 unresolved**,
  each with the one read that closes it. The pattern classification of
  2026-09-17, which partitioned all 298, is withdrawn (the stage-5
  audit, finding 1); it is history at `3c4331f5`.
- **C105 — the buckets of a count.** Known absence is not undecided
  (C93): a stash item counted by character has no character and lands in
  `none`; a readable base the class table lacks has a class that cannot
  be established and lands in `undecided`. Each is printed only when
  nonzero. A key an item holds several of (`line`, influence) puts the
  item in several buckets, so for it only the first invariant holds; the
  exact sum to the total holds for a key with one value per item (class,
  league, tab, rarity), and the first counts test pins that case — ten
  rare items by class: armour 6, weapon 3, undecided 1, total 10.
  Beneath `undecided`, a tally by kind of reason — C93's closed list: a
  body unread, a base the class table lacks, a price unresolved, a name
  unbound — where it costs no undue complexity (owner, 2026-09-18: "If
  we can bucket the undecided reasons by kind without undue complexity,
  go ahead"). The tally is a diagnostic, not a partition: an item
  undecided for two reasons counts under both, so it need not sum to the
  bucket, whose count is the contract. Item-specific detail (which base)
  shows on the item's row, never in the aggregate. A crossed table gains
  at most one `none` and one `undecided` row or column per name, the
  tally once per table. Open for the language reference: whether the
  `none` and `undecided` buckets carry a pasteable selecting term as
  C97's rows do.
- **C96 — the realm scope.** The domain is stated outside the tree, shown
  resolved in the canonical request and answer, and never hoisted from a
  branch: `realm:pc or "# to maximum Life">=90` names pc and still admits
  a poe2 item through its second branch, and `-realm:pc` has the same
  problem without an `or` (Astra). An adapter may abbreviate an
  unambiguous positive realm restriction — `realm:pc` at the front of a
  plain query — into the scope; anything else is an error that names the
  scope words.
- **C98 (K4) — notification beside the check.** A long-lived frontend
  may listen for the daemon's job events and reload the moment a refresh
  lands, as the C++ app's signal did; that is responsiveness and is
  allowed. The revision check before every answer is what makes the
  answer correct when the change came from another client, a missed
  event, or a machine waking — ruled "by construction" (note 28 §4, question 2).
  Whether events reach a client that did not submit the job is
  unverified. Reload is whole; S150's 200 ms at 36,139 items is a benchmark
  baseline for the JSON parse alone, not this design's reload; an
  incremental update over `item_events` is parked.
- **C98 (K4) — the basis.** The facts revision advances in the
  transaction that changes bodies, locations or membership; the basis
  carries the store and account identity, so revision 7 in two accounts
  is never one snapshot; a resident joined value (an effective price) is
  part of the held corpus and is reused only under the intent revision
  it was read at (Astra's counterexample: facts at 7, a price moved at
  intent 12 → 13). The numbers that would move persistence: the
  streaming body read; the re-derive under an active refresh; a CLI ask
  over 500 ms. S152 keeps direct SQLite open for a consumer that does
  not outlive its query.
- **C100, C101, C103 — where raw JSON is seen.** A result row never
  carries a body. One item's raw body is seen through `show <item>`, on
  request, one at a time (owner, 2026-09-17: "i agree with using show
  <item> for this"); the query language has no path into it. The store's
  bulk read (C103) hands bodies to the search crate in-process and,
  being a public store read, to any code that links the store — as
  `Store::search` does today. The store's public snapshot read
  (`Store::refresh_snapshot`, `CharacterSnapshot`: the listing entry and
  the fetched envelope minus its lifted item arrays) stays what it is,
  an in-process neutral read; no raw-path query, no bulk raw search
  output and no new tab or character body read is proposed (Astra's
  correction of the earlier "no read today and gain none").
- **C101 — the derived-field pseudos and the sockets.** DPS is attacks
  per second times the average of the range (S22); base defence
  percentile would be the item's defence against its base's range from
  reference data (C68) — unresolved until the export is shown to carry
  those ranges (pseudo-stats, open question 3); a quality-normalised defence follows S26; each is
  a named pure function in the search crate, listed with the totals in
  the help (C97). Socket shapes the census shows and the deriver does
  not decode stay counted as unread (S16); a combined link-count and
  colour request is bound to one group when S58's shape is built.
- **C99 (K5).** `count` carries its minimum and its maximum; `weight`
  and `weight2` differ in how a per-stat requirement gates a
  contribution (S57), and a weighted group translates only where that
  guard has a representation; a defence bound is inexact while the site
  normalises quality (S26) and sits in the remainder until a normalised
  field exists. Several ids for one line go out as a `count` of at
  least one (S106). A site bound on a ranged line is `avg` on the line
  (S47), exact once the word exists.
- **The derivation's shape.** Lines as rows (one per occurrence), fields
  as columns, place by name (S200): the shape the model produces, kept
  so that a JSON or SQLite export, or a persisted projection, is a
  writer over it and not a redesign.
