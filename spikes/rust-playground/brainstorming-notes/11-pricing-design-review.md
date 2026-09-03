The packet has the right architectural backbone, but I would not harvest it verbatim yet. Pricing is correctly placed in intent + derivations, the plan/apply loop fits, and render-before-publish is exactly the right slice. The weak point is the domain model between stored intent and rendered publication: the packet currently collapses game observations, manual forum intent, effective price, and publication eligibility into one precedence function. That is where inherited C++ assumptions are still steering the design.

## Overall judgment

Keep these foundations:

- Pricing is offline intent; no daemon or job.
- Store explicit assertions only; derive inheritance.
- Strict, versioned intent types before the first real price.
- Atomic `PricePlan` application with row-level preconditions.
- Pricing never edits refresh policy.
- Legacy import is a producer of desired state, not a privileged writer.
- `shop render` is in scope; forum publishing is not.
- A mutation receipt lands atomically with the intent mutation.
- CLI and MCP are thin adapters over the same Rust semantics.

Before ruling, I would amend the packet around five issues: publication channels, target identity, provenance, concurrency semantics, and the deliberately open Plan-family experiment.

## 1. Do not force game and manual prices into one scalar “effective price”

C69 says a game-set price wins at equal grain because it is “already public” ([packet](/Users/tom/Development/GitHub/gerwaric/acquisition/spikes/rust-playground/brainstorming-notes/10-pricing-design-packet.md:135)). That premise is conditional. Current stash facts expose `metadata.public`; a price-like tab name or item note can exist without the stash being public. The official API documents both the public flag and item notes. [GGG API reference](https://www.pathofexile.com/developer/docs/reference)

More fundamentally, these are different statements:

- Game price: an observed fact about an in-game note.
- Manual price: intent for Acquisition’s forum-shop output.
- Ignore/no-price: publication disposition, not really a price.
- Renderability: whether the item can be represented safely in a forum post.
- Public visibility: whether GGG already exposes the item through a public stash.

I recommend deriving the game and manual sides independently, then reconciling them into an explainable listing state:

- Resolve manual inheritance by specificity: item → substash → parent tab or character.
- Resolve game pricing by specificity: item note → tab name.
- If both exist, report whether they agree or conflict.
- Let each consumer decide what that means. A price display can show both; `shop render` can suppress, include, or refuse based on public visibility and the ruled duplication policy.
- `ignore` should suppress forum rendering without pretending the observed game price ceased to exist.

This is better than both proposed options in question 1. I would neither refuse the manual write nor silently call it shadowed. Store the intent and return a first-class conflict/corroboration state. Refusal protects only one ordering; unconditional shadowing invents an authority rule that the evidence has not established.

C64 should therefore say “manual listing intent is stored; game pricing and effective/listing states are derived,” rather than implying one universal effective-price scalar.

## 2. C67’s target identity contradicts the Rust store’s learned coordinate model

C67 proposes keys based on GGG IDs alone. That imports a C++ storage shape precisely where the Rust work already found it unsafe.

The Rust store deliberately keys stash locations by `(realm, league, id)` and records that “a location is its full coordinate” ([store decisions](/Users/tom/Development/GitHub/gerwaric/acquisition/spikes/rust-playground/decisions/store.md:10)). Realm is explicitly “above league, everywhere” ([store decisions](/Users/tom/Development/GitHub/gerwaric/acquisition/spikes/rust-playground/decisions/store.md:17)). Meanwhile, the C++ userstore reverted stash identity to `id` alone to fit its upsert machinery; that is evidence of its implementation constraints, not a new-system invariant.

Use a typed `PriceTarget`, even if the annotation table internally encodes it into `scope` and `key`:

- `Item { id }`
- `Tab { realm, id }`
- `Substash { realm, parent, id }`
- `Character { realm, id }`, unless C55’s global character identity is deliberately ruled sufficient

League can remain absent so intent follows a tab through league migration. Realm should not be discarded without evidence that identifiers are globally collision-free across realms.

This target type should be part of the public pricing API; frontends should never construct raw annotation keys.

## 3. The amount grammar is not ready to freeze

“Positive decimal string” is directionally better than a float, but underspecified:

- Are ratios allowed?
- Is exponent notation refused?
- What precision and length limits apply?
- Are `1`, `1.0`, and `01.00` equal?
- Does the system preserve a lexeme or a mathematical value?
- How does import convert a C++ `REAL`, where the original text is already lost?

The old parser accepts only a narrow decimal regex, and the old store holds `REAL`. Neither proves the current forum grammar. The packet’s claim that the forum tag remains exactly what the human typed cannot hold for imported values.

I would rule “no binary floating point; validated exact amount representation” now, but leave the precise decimal/rational grammar to a small evidence step before finalizing Buyout v1. Equality must be semantic and deterministic so a second import is truly `unchanged`.

Retire `current_offer` from Buyout v1, but preserve every encountered row as an import non-action. A read-only census of the owner’s source store should occur before finalizing this; real absence is better evidence than the C++ warning.

## 4. Provenance needs three names, not one overloaded `source`

C65 currently calls the adapter the `author`, while `source` simultaneously means plan hash and import path/hash. Those have different meanings and cardinalities.

I would model:

- `written_via`: `cli`, `mcp`, `gui`, `import`.
- `actor`: optional claimed actor/client identity, when one actually exists.
- `applied_plan`: the canonical plan/application hash on the row.
- Receipt-level `origin`: import artifact metadata, including its consistent-snapshot digest and legacy timestamps.

`acq` is not an author; it is a channel. An absolute source path also should not be repeated on thousands of rows.

There is a further C++ trap: the source userstore uses WAL. A SHA-256 of the main `.db` file can omit uncheckpointed WAL content and is not a digest of the logical snapshot being imported. The importer must read under a consistent SQLite snapshot—or import from a backup—and hash that logical/captured artifact. The C++ userstore is also filename-bound to a username, not internally UUID-bound, so C73 must explicitly report the strength of its source-account binding. Multi-account import cannot quietly infer identity from a selected file.

Existing v2 rows will need honest migrated provenance such as `written_via = unknown_legacy`; migration must not manufacture an author.

## 5. C71 needs an explicit fact-drift ruling

Atomic annotation CAS is right, but the packet overclaims that the driver “never re-reads to learn whether the world moved” ([packet](/Users/tom/Development/GitHub/gerwaric/acquisition/spikes/rust-playground/brainstorming-notes/10-pricing-design-packet.md:64)). Facts and annotations are separate SQLite files; the existing refresh snapshot already acknowledges that they cannot share one transaction.

The plan needs to say what its preconditions protect:

- Intent preconditions are binding and checked atomically.
- Fact basis is provenance unless explicitly made a precondition.
- Fact drift does not change the authorized row mutation—but it may change whether that mutation is effective, conflicted, public, or renderable.
- The apply result should therefore return rows-as-written plus enough current-basis information to tell the caller whether a new observation is required.

Also tighten “revision-or-absent.” Tombstones exist specifically to close ABA holes, but the current create path allows recreation over any tombstone when the caller expects absence ([annotations](/Users/tom/Development/GitHub/gerwaric/acquisition/spikes/rust-playground/crates/acquisition-store/src/annotations.rs:286)). Decide whether a price plan cares only about semantic absence or about the tombstone generation as well; “one moved revision refuses” is not true otherwise.

The compiler should also reject duplicate mutations for one target and expose mutation counts—creates, updates, clears, unchanged, refused—as pricing’s operation-specific “cost.” Local intent writes have no wire cost, but they still have blast radius.

## 6. C78 is justified, but it is not “its own inverse”

I agree that pricing fires the history trigger. A receipt stored atomically with the batch is worth doing before the first import.

Amend the wording:

- An applied plan contains sufficient preimage and result data to compile a conditional inverse.
- Revert is a new plan against current revisions.
- If any affected row subsequently moved, revert refuses atomically rather than pretending history can be replayed blindly.
- Call this an intent mutation receipt or audit ledger, not the “effects ledger,” because C34 already uses that term for daemon-owned job effects.
- Specify canonical plan hashing and whether receipts retain the complete reviewed plan, only applied mutations, or both plan and outcome.
- Do not repeatedly store giant no-op import plans without an explicit reason or retention policy.

The plan is not mathematically its own inverse; it is evidence from which an inverse can be compiled.

## 7. Shop rendering needs an eligibility derivation

C74 is right in scope but overcommits to C++ rendering details before validation.

The Rust store contains more item rows than “things safely linkable in a forum shop”: socketed items, several character containers, guardian equipment, possibly locked items, and rows lacking coordinates. The C++ shop iterates its flattened item collection and silently skips missing stash indexes; that behavior should not become the new rule.

Add a derived `renderability`/`publication eligibility` result with named reasons:

- socketed in another item
- unsupported character container
- missing position or inventory address
- removed/orphaned
- locked to character/account
- unknown currency
- conflicting game/manual listing
- public game listing already covers it
- missing tab index or parent relationship

Every omitted item should be counted and inspectable. No silent skip.

Accept the pure-render boundary now. Treat link syntax, post-size measurement, page splitting, and exact public/private behavior as hypotheses validated by the owner’s forum reading. GGG’s historical announcement confirms that forum-linked items were supported, but that is historical evidence, not a permanent current contract. [GGG trade announcement](https://www.pathofexile.com/forum/view-thread/2392556)

Also correct the driver’s-seat description of future publishing: split output means potentially multiple credentialed forum mutations, not “one post.”

## Proposed rulings

| Candidate | Recommendation |
|---|---|
| C64 | Accept, amended to distinguish manual listing intent from observed game pricing and consumer-specific listing state. |
| C65 | Amend to structured writer/actor/plan/origin provenance. |
| C66 | Accept. Clarify that exact round-trip applies to current-schema input; older values may upgrade in memory while raw stored JSON remains untouched. |
| C67 | Do not accept as written. Amend target identity and exact-amount grammar; retire `current_offer` from v1. |
| C68 | Accept the enumerable, versioned input concept. Call it versioned reference data rather than forcing it into account facts; give the dataset its own version/hash and carry that basis in plans/renders. |
| C69 | Replace with separate manual inheritance, game observation, and reconciliation/publication-state rules. Store conflicting intent; neither unconditional refusal nor unconditional shadowing. |
| C70 | Accept parent/child inheritance, amended to typed realm-aware targets and separated from renderability. |
| C71 | Accept with explicit fact-drift semantics, absence-generation semantics, duplicate-target rejection, and operation-specific mutation counts. |
| C72 | Accept. This is one of the strongest lines in the packet. |
| C73 | Amend for source-account ambiguity, WAL-consistent provenance, exact numeric conversion, unknown-target policy, and per-source-row preservation. |
| C74 | Accept the render/publish boundary; keep detailed forum mechanics provisional until validation. |
| C75 | Do not rule the final answer yet. Build operation-specific `PricePlan`, then record what is genuinely shared. “Never a grammar” prejudges the method test it claims to run. |
| C76 | Accept only the one-door architectural direction. Defer listing/freshness/two-cycle semantics to its own slice; four tracer runs not using the old command are weak evidence about one-off user workflows. |
| C77 | Accept that freshness is evaluated at compile time and a later replan may correctly refetch. Do not require the `aging` warning yet: an offline plan lacks a trustworthy cycle estimate and quotes are optional. |
| C78 | Accept an atomic receipt ledger, amended as above; reject “its own inverse.” |
| C79 | Keep as a slice hypothesis under C53, not a new ruling yet. Enumerability belongs in C68; summary/filter/pagination shape should be validated through MCP use. “Complete JSON under the filter” may itself be too expensive at thousands of rows. |

## Answers to the owner questions

1. Neither unconditional shadowing nor refusal. Store the manual assertion and derive a conflict/corroboration state; make publication behavior consumer-specific and public-aware.
2. Yes, render belongs in scope because it is pure and gives pricing its real validating consumer.
3. Rule the freshness-window semantics; park the `aging` warning until quote evidence makes it honest.
4. Yes, import before render—but add a read-only source census before the Buyout schema is frozen.
5. Retire `current_offer` from Buyout v1; preserve imported occurrences as explicit non-actions.
6. Keep the plan-granularity receipt ledger, narrowed to an atomic audit/undo substrate with conditional inverse compilation.
7. Treat C79 as surface design under C53 until the CLI/MCP run validates it; do not promote it prematurely.

My suggested slice order is therefore: targeted annotations audit → read-only census of the real C++ source and current fact shapes → finalize target/amount/publication semantics → annotations v3 and typed intent → effective listing state → PricePlan/apply/receipts → import → render → MCP → only then rule what Plan abstractions and read shapes actually generalized.

That preserves the packet’s best insight—the shared observe/compile/review/apply loop—while leaving the product semantics open to the evidence the new system, rather than the C++ implementation, produces.