# Step 9c3 outside review

Reviewed `23991fe1..8506cc4b`, against `8506cc4b`, following
`search/REVIEW-9c3.md`. No tracked file changed and no commit made.

Two findings: one acceptance failure and one documentation error.
The acceptance failure should keep the step's review open.

## 1. [P2] The capture fixture loses the multiline case that the build misses

**Location:** `crates/acquisition-search/tests/pseudo.rs:1225`.

**Item and request.** `search/pseudo-stats/data/captures.json`,
`searches.R1.fetched[0]`, is Thread of Hope. Its second explicit modifier's
description is one string:

```text
Passive Skills in Radius can be Allocated without being connected to your tree
-17% to all Elemental Resistances
Passage
```

Ingest the capture's modifier arrays unchanged through `Store::record`,
with a fixture id and stash. Ask `pseudo.count_res=3` and
`pseudo.count_ele_res=3` of this item.

**Observed:** both return no match. Both `-has:pseudo.count_res` and
`-has:pseudo.count_ele_res` return the item: the values are reported as
known absence, not undecided.

**Expected:** both counts are 3, as the capture's `pseudoMods` explicitly
show. Step 9c3's closing condition at `1662e8ab` is “the four answering as
the site shows them on the track's captures.” C101 defines these readings
from the resistance totals; the language reference's owner's words also
say that two-line mods should contribute to relevant pseudos while
remaining a single occurrence.

**Why the checks miss it.** The new `thread` fixture replaces the
multiline description with a standalone `-17% to all Elemental
Resistances`. The research checker splits descriptions on newlines
(`tools/trade-evidence.py:89`); the production total matches the whole
occurrence's template. Thus agreement with the research calculation is
not agreement with the built search on this capture. The underlying
total behavior predates 9c3; 9c3 inherits it and its acceptance fixture
conceals it.

**Extent reproduced:** replaying all 110 readings cited for the four
pseudos, including the 16 absent readings, yields 108 agreements and
these two failures on this one item. This does not establish how often
the private API emits this grouping; no private API was queried.

**Checklist shape:** “A claim the code did not make”; the capture-based
acceptance fixture changes an input shape relevant to evaluation.

## 2. [P3] The vocabulary documentation excludes an incomplete count that is included

**Location:** `crates/acquisition-search/src/pseudo.rs:119`–`121`.

**Item and request.** One pc item with explicit modifiers:

```json
["+20% to Fire Resistance", "+20% to Cold Resistance", "+10000000000% to Chaos Resistance"]
```

Ask `undecided(pseudo.count_res)`, then an empty query with
`--count line:count_res`.

**Observed:** the item matches the undecided probe, yet the computed
vocabulary bucket for `pseudo.count_res` counts 1 and routes through
`has:pseudo.count_res`, which matches it. This is correct for a count
whose presence is known while its value is 2..3.

**Expected documentation:** the vocabulary's account must allow that
case. The module currently says that a lacked value and “an incomplete
one do not” carry the computed value. This contradicts the same module's
C93/C94 paragraph, `present`, and the new boundary tests. The code's
behavior is not the finding.

**Checklist shape:** “A claim the code did not make.”

## Reproduction

The untracked `crates/acquisition-search/tests/review_9c3_probe.rs`
contains both probes. It builds isolated stores through ingest, never
opens the owner's store, and sends no requests. The failing capture
probe is ignored by default so ordinary test runs remain usable.

From the workspace root:

```sh
cargo build --workspace
ACQ_STORE_DIR=/tmp/acq-review-9c3-isolated ACQ_PROVIDER=mock cargo test -p acquisition-search --test review_9c3_probe -- --ignored --nocapture
ACQ_STORE_DIR=/tmp/acq-review-9c3-isolated ACQ_PROVIDER=mock cargo test -p acquisition-search --test review_9c3_probe incomplete_count_is_carried_by_the_vocabulary -- --nocapture
```

The first reports 110 readings and the two mismatches, then fails its
assertion. The second passes and prints the vocabulary count and route.

## Verification and review limits

- The committed workspace gate passed: build; all targets' tests
  (**609 passed, 1 ignored**); Clippy with warnings denied; formatting;
  diff check; docs check; rustdoc with warnings denied.
- The first full test attempt hit a sandbox denial binding a local
  socket. The rerun outside the sandbox passed.
- `python3 tools/totals-table.py --check` passed: the v5 table regenerates
  exactly from its committed sources.
- Reviewed the changed evaluator paths, table definitions and loader,
  generation script, fixed and generated tests, CLI rendering test,
  capture evidence, and closure records. The existing suite exercised
  comparison, presence, routes, sort evidence, counts, sums, unsupported
  realms, and partial inputs.
- No fresh performance measurements on the owner's corpus were taken.
  The recorded timings and already parked totals cost are not reported
  again as findings. The builder's conservative treatment of incomplete
  least values and counts that might be absent is explicit in the record;
  this review found no separate contradiction in those choices.
- The private API's representation of the R1 multiline modifier remains
  unverified, as noted in finding 1. No other unreproduced concern is
  promoted to a finding.
