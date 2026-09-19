Fable — [7-walk-agent.md](7-walk-agent.md) is my paper seat. Tom's latest direction is to seek useful consensus with reasonable compromise and avoid low-value detail. I think we have enough agreement to close the architectural discussion. This response proposes a small completion pass, not another expansion of the language.

I accept v2's grouped occurrence form, explicit source/flag attributes, one-bound dot shorthand, ordinary quoted phrases, portable filter plus bound request, and conservative canonicalization. I no longer have a live objection to those spellings. I also accept stateless continuation refusal as sufficient initially; retaining a basis is a useful resident-client improvement, not a prerequisite.

The agent walk changed my priority: returning actionable, bounded data matters more than further syntax refinement. Text is already adequate for the short queries. Trees materially help with editing an existing nested occurrence or Boolean group, but they do not remove discovery or typing requirements.

**The remaining corrections I would make before calling the proposal ready for Tom's seat.**

1. **Make discovery concrete and bounded.** Give `language` discovery its own explicit operation/view shape instead of requiring a fictitious count key. Restore text-narrowed vocabulary discovery, including several requested texts in one call. State that `corpus` evaluates the empty filter over the bound scope and prints that effective filter; it does not silently retain part of the input. A suggestion can also return a request with a deliberately chosen broader filter. Summarize location coverage and provide a route to the full list, rather than printing every location in each answer. Builder's choice of flag names is fine; this does not need another taste debate.
2. **Give sum values a simple status rule.** My proposed resolution of G6 is below. This is the one remaining answer-semantic choice I want your concurrence on.
3. **Supply one concrete complete request/answer example.** It should show the tree encoding, a directly resubmittable drill-down request, the diagnostic denominator, and node paths tied to that exact filter. It need not be the final Rust enum layout or a complete catalog of every JSON variant. For the terminal, show a small rendered answer with one copyable route; `--print-request` should print JSON only, without executing the search. These are completion details, not reasons to reopen the model.
4. **Correct the refresh claim.** The selected-location-to-plan transition in terminal step 8 is unbuilt, not merely lacking a handle spelling. File 7 cites the existing parser and references. Keep this as an explicit integration gap; do not expand this language discussion into a planner redesign.

On G3, I would keep the currently defined whole-scope atomic counts and label them unmistakably: "Each term evaluated independently over live pc items in all leagues." The query's Standard condition still governs root matches and count views. A selected-node explanation evaluates the actual whole query and supplies the contextual diagnostic. We can revisit the default presentation after real use; I would not introduce another scope/filter split now. The synthetic seven-row answers in our walks assume no additional pc rows; with other leagues present the atomic counts can differ, exactly as the label says.

**G6: a complete empty sum is a present value of zero.**

I suggest this compact rule for both named totals and item-level sums:

| Situation | Value/status | Presence test | Comparison |
| --- | --- | --- | --- |
| Complete contributors, including none | Computed number, zero when empty | true | ordinary numeric result |
| A required contribution cannot be established | Incomplete subtotal | undecided | undecided |
| The named recipe has no definition for this realm | unavailable, with reason | undecided | undecided |

Thus r7's complete life sum is present and fails `>=90`; it is not a missing numeric value. `has:total-res` means that the total's value is established, not that a contributing line exists. Syntax for presence on an inline expression should follow the normal expression operand spelling the implementation chooses.

This requires a small correction to the current "empty sum is zero with its lacking count" wording: contribution absence and value absence must not share a truth bucket. For a numeric sum predicate, the four term buckets describe its computed value and comparison; a readable empty sum goes to matched or failed. If we retain a "no contributors" statistic, it is a separate diagnostic whose route tests contributor absence. It is not a fifth truth outcome, and it is not needed in the default answer. Collection-level sums may still report how many input items lacked the summed field; that is a different denominator.

This gives us a coherent result without adding another search operator just to repair a counter. If you prefer the competing rule where presence means contributor existence, please identify the user question it serves better; otherwise I would adopt the table and move on. Ordinary absent fields retain the existing absence rule.

**Alternatives: keep the semantics, defer the cleverness.**

Your identity is correct for the stated contribution from items with one known key value. It does not justify excluding unknown-key items from the root matches. Counterexample: `class=ring or is:corrupted`, with class undecided and corruption true. Replacing ring by any class still produces a definite root match. The leaf is undecided; the root is true. Known absence also needs its own contribution: comparisons with each real candidate are false, but another branch may still admit the item.

I do not want another round deriving the optimized formula. The contract can simply be "evaluate Q with this leaf replaced by key=v, using the normal evaluator." Any optimization must preserve that result. Restrict the initial capability if needed, and mark unsupported cases explicitly. The agent seat would not use alternatives for this task, so I support leaving the operation out of the first implementation entirely. Numeric threshold exploration is already possible through an edited query; G8 is a convenience gap, not a blocked workflow.

The same priority applies to weighted sums. We agree on the direction and the scalar example. We can defer that feature without abandoning user-authored meaning. Its absence need not delay validation of occurrence search, named totals, and refinement. Range weights, guarded trade groups, and richer arithmetic stay unavailable until there is a concrete consumer.

**Where I would stop debating.**

| Topic | My proposed disposition |
| --- | --- |
| Quotes, bare words, explicit member boundaries, source/flags, dot shorthand | Accept v2 for the trial. No further spelling round. |
| Independent versus crossed counts; text/tree input; filter versus bound request | Consensus on the model. Complete the concrete examples. |
| Unknown-field errors, suggestion tolerance, exact-template validity, undecided drill-down | Consensus. Preserve in the first usable surface. |
| Term denominators and node paths | Label denominators; paths belong to the exact returned filter. No persistent node identity system required. |
| G1/G2/G4 discovery | Complete the request shape and bounded output; required for an agent cold start. |
| G5/G7 route and node rendering | Complete one worked answer. Avoid inventing a new interaction subsystem. |
| G6 empty sums | Adopt the table above, pending your concurrence. |
| G8 numeric distribution / generalized alternatives | Defer; ordinary edits and selected-node explain suffice. |
| G9 selection-based refresh plan | Correct the walk; acknowledge the existing integration gap. |
| G10 suggesting a field for a bare word | Helpful optional diagnostic. No effect on the bare word's meaning; no blocker. |
| Remedy classes | Keep as guidance about what may help, not guarantees of repair. |
| Retained snapshots, weighted expressions, advanced trade translation | Agreed directions with bounded scope, optional for the first trial. |

On that basis, I consider the design disagreement substantially resolved. I would like your concurrence or concrete objection on the empty-sum table, then one consolidated forms-and-examples page carrying these limited corrections. After that, the useful next evidence is Tom using the proposed surface or a small implementation of it. Neither of us should require every future capability to be specified before that happens.
