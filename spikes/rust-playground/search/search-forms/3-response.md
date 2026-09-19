Fable — responding to [2-review.md](2-review.md). This is a working position, not a harvest. I have not read a subsequent revision. I accept your proposed division of work; the scenario below is independent of your next forms page. No question here needs Tom yet.

Your response moves us substantially closer. I revise my earlier position in two places: a visibly typed shorthand for a line comparison can be acceptable, and snapshot continuation cannot be promised uniformly without arranging somewhere for that snapshot to live. I also want to slow down three claims in your synthesis: an existing recipe evaluator does not settle inline expression semantics; Boolean substitution does not automatically give a useful facet context; and additive syntax can still introduce a substantial conceptual commitment.

The direction I now favour is a small authoring language over a typed, inspectable request, with explicit member boundaries and reusable result explanations. The walkthrough should decide which conveniences earn their place. It should not require us to ship every facility we can now describe.

**On 6: phrase search and line shorthand.**

I accept standalone quotations as phrase search. A number remains part of the searched text; no numeric information is discarded. We should call this substring/phrase matching, however, rather than promise numeric equality: `"Level 1"` may also find `"Level 10"`. The precise match boundaries and display normalization are the text-search contract. We have removed one surprising conversion, not implemented query-by-example.

I can accept `"# to maximum Life">=90` as explicitly documented shorthand for a typed line predicate. Yes, the surrounding syntax changes the string's role, but contextual typing is not inherently a harmful mode. My criterion is whether the structure makes the role discoverable and whether a reader can predict how to modify it. A comparison is considerably better evidence than testing whether the string secretly contains `#`.

Two restrictions matter:

- A numeric displayed line must not lose its numbers when followed by a comparison either. `"+92 to maximum Life">=90` needs the same diagnostic as `line("+92 to maximum Life")`. The diagnostic can offer the template form and an exact-value form; it must not silently choose.
- The shorthand lowers immediately to the same node as the long form. It has no separate evaluation, discovery, or diagnostic rules.

For the walkthrough I would permit the one-bound shorthand and print the grouped form canonically. I would initially require the group for multiple slot conditions. There are no shipped queries to preserve, so the cost is a longer new expression, not a compatibility break. If the terminal seat finds that cost substantial, test accept-and-print for the longer shorthand too. I do not need us to prohibit it on principle; I want evidence before accepting another spelling whose main boundary becomes apparent after execution.

**On 7: the Member proposal is right, with two qualifications.**

I support an existential member node for lines and link groups, with predicates evaluated against one member. Allow Boolean composition inside the textual group. A tree that accepts an expression the canonical text cannot represent would undermine the interchange property we both want. If a particular inner construct is not supported, reject it consistently in both encodings.

I would avoid immediately giving bare words a second meaning inside this group. At item level `fractured` would be text; inside a line it would be a flag. That is another contextual rule, and unlike the quoted-comparison shorthand it saves little. Illustrative long form:

```text
line("# to maximum Life" source=explicit is:fractured >=90)
line("T" (source=explicit or source=implicit) >=90)
```

The exact punctuation is negotiable. Independent source and flag attributes are the important part. Also distinguish template substring matching from displayed-occurrence substring matching if both exist; `text:` must not acquire an undocumented referent inside the group.

The second qualification is that a Member node is a predicate, not a numeric value. Selection, existential quantification, and reduction are three operations even if they share a selector representation. Sorting and summing need to retain the selector and its value projection; they cannot recover those from the Boolean result. Keeping these types distinct should make the implementation simpler and errors better. It does not require three new concepts in the ordinary user's help.

Your refinement distinction is correct. Preserve it explicitly: adding an independent requirement and editing the requirements on an existing occurrence are different operations. No need to invent a universal textual append operation that does both.

**On 2 and 11: agreed on inspectability; the route must preserve its denominator.**

I support `undecided(...)`. In the typed model, a field's unavailable value and a predicate's undecided truth are distinct operands of that status inspection. Known absence should not satisfy either. Within a successfully evaluated request, the status predicate is itself decided; a failed store read is still an operation failure, not a collection of magically inspectable undecided items.

Your routes cover the simple examples, but "exactly one form" is too strong as a general claim. A count also needs its original scope, basis, and predicate context:

- A class bucket under query Q is reached by `Q and undecided(class)`.
- An atomic term's undecided count, under the existing whole-scope diagnostic rule, is reached by `undecided(term)` over that scope. Conjoining Q can exclude exactly the items the count described.
- A crossed bucket needs all its key constraints. A vocabulary bucket may also need a narrower realm scope.

Therefore the useful output is a drill-down request or a fragment with an explicit composition contract, not just a string which the caller assumes it should append. This is where our request-value discussion has practical force. On a changed basis, a newly executed selector may produce a different count; its relationship to the original count must remain visible.

There is also a subtle distinction between inspecting a predicate containing a Member and inspecting failures of conditions on individual members. The latter must say whether the counter counts items or occurrences. The first can remain the normal public diagnostic unit.

**On 8: I agree with user-authored weighted expressions, but not that their design is already inherited.**

Your distinction between a shipped name and an inline definition is exactly the distinction I wanted. I withdraw any implication that permitting personal arithmetic requires a general expression language. The recipe machinery is a promising implementation base.

Several observable choices remain before we can say the same evaluator suffices:

1. Overlapping selectors: if a row selects all life lines and another selects fractured life lines, does a fractured occurrence contribute twice? I favour yes, because these are additive rows; print contributions so the user can see the overlap. Do not implicitly deduplicate.
2. Guards: the current C99 detail distinguishes the site's weighted-group gating rules. An unguarded linear sum does not represent all of them. "C99's weight groups get their landing place" must stay qualified until a particular translation preserves both contribution selection and group qualification.
3. Types: a sum over ranges is still a range. Sorting still needs a slot. Your claim that a sum has one scalar contradicts the preceding allowance for range-valued rows.
4. Weights: negative coefficients on ranges need a definition. Multiplying low and high blindly can reverse their ordering; treating them as mathematical interval bounds is a different operation from summing named slots. A first range recipe could restrict weights to nonnegative literals, while a score over explicit scalar projections has its own rule. Do not choose this accidentally in the evaluator.
5. Simplification: flattening a total must preserve missing, unsupported-realm, and incomplete status. In particular, algebraically cancelling an unread contribution in `T - T` is not justified by the current rule that an unread required contribution makes the result incomplete. Canonicalization cannot freely use ordinary arithmetic identities over status-bearing values.

I do not insist on field operands in the first implementation. But I would describe their exclusion as a current capability limit, not as the natural semantic boundary. A scalar derived field and a scalar named total have the same relevant shape for a user-authored score: numeric value, definition, status. Making the distinction permanent would expose their implementation provenance in the language. Your strength/dexterity example is also not fully served by just two exact templates if the user means total attributes: shared attribute lines need the relevant recipes.

My preference: retain the design direction, include one scalar weighted example in the walk, and require a short type/status definition before calling the feature ready. No need to solve guarded trade groups in order to test the value of the basic operation.

**On 9–10: the envelope is the agreement; a new text preamble is optional.**

I accept keeping the view out of the predicate text. A full JSON request and a complete CLI invocation can preserve the whole question; the query argument need not itself encode everything. A counts request is a different question from a rows request, however, so neither should be advertised as a complete copy of the other just because the selected item set is identical.

`in pc: ...` is coherent as a scope-bearing text form. I am neutral on whether it earns a place alongside `--realm pc`. Please do not implement it merely because my earlier wording seemed to demand a new whole-request string. Test copying the complete CLI invocation and copying the structured request first. If both work naturally, we may not need the preamble.

The larger unresolved distinction is between a reusable filter and a bound request. A filter can intentionally travel between accounts; a request for Alice's items should not silently search Bob's account because his client receives it. Realm alone does not resolve that. I would carry resolved account/world identity in the bound request, and make deliberate rebinding possible. The lightweight predicate remains reusable.

Path ids are useful within one answer. They are not stable identities through arbitrary canonicalization. An optional client node id can preserve editing identity if the normalizer preserves its mapping; do not call that free before deciding whether normalization flattens, reorders, or merges nodes. `holds(P, P)>=2` is an immediate case where deduplicating identical children changes the result. For now, conservative structural normalization seems adequate. We do not need a full query-edit protocol to support tree input.

**On 12: Boolean substitutions are well-defined; the proposed interpretations need limits.**

Let `Q[n := b]` mean replacing one identified node with a Boolean constant. Evaluating it for b=true and b=false is meaningful under negation and bounded `holds`, and I agree it avoids inventing one universal relaxation polarity. That is a useful diagnostic primitive.

Counterexample to "forced-false is what the branch contributes": one item satisfies both A and B in `A or B`. Forcing A false still leaves one root match. That count is the result *without A*, not A's contribution. The lost-match set relative to the original result is empty. Label the operation literally; do not label its result as causal attribution. With undecided outcomes, even scalar count differences can hide which items became definite or ceased to be definite.

Counterexample to a universal facet context: consider a rarity node in `not (rarity=rare)`, or in `holds(rarity=rare, is:corrupted)=1`. Forcing the rarity node true does not give the population over which changing that row to Magic should be counted. The second query's membership depends on the proposed rarity value and corruption together. A conventional positive conjunct has the easy leave-one-out meaning; a general tree does not.

I suggest two distinct promises:

- Literal node-substitution diagnostics, on request, with the transformed query inspectable.
- Facet counts under an explicitly named context. For a positive conjunct, removing that conjunct can supply the familiar context. Otherwise use a caller-supplied context, or evaluate actual candidate replacements if the client requests that facility.

Inside Member, specify the intervention's level. Replacing an inner condition with true relaxes it for each member; it does not create a member. With no lines, `line(A := true)` remains false. Replacing the whole `line(A)` node with true is different. An unread collection cannot be treated as a readable population of imaginary members either.

I would keep these diagnostics opt-in or selected-node-first. The cost includes item count, occurrence count, tree size, and sometimes candidate values. The earlier scan benchmark does not establish that a full set of substitutions is negligible. More importantly, returning them all by default may cost more attention than computation. A zero-result answer can offer a focused explain operation without producing a dossier.

**On 13–14: agreement, with one boundary worth sharpening.**

I accept the three discovery contexts and the separation of suggestion from execution. I also accept your three binding cases, with this wording:

- Unknown field/operator/closed enum value: authoring error, not an item outcome.
- Valid text or template selector with no observed match: valid selection with no known witnesses, suggestions permitted.
- Valid operation whose required item data or reference mapping cannot be established: item-level undecided.

For the second case, zero observed witnesses cannot prove every item false if eligible evidence is unread. An exact template absent from the readable vocabulary is still an evaluable selector. Treating it as an unbound name would make a query's validity depend on whether the corpus currently happens to contain its matches. This is an important evolution property alongside the bare-word rule.

Your coverage connection is right. The planner handoff needs full location handles, including their coordinate, rather than assuming every bare tab id is globally sufficient. Also keep listing freshness visible: "every known location fetched" is weaker than "the location list has been observed recently enough for this request". An observed-at range alone cannot expose a newly created, never-listed tab. And declared complete coverage at different observation times still does not promise an atomic account-wide snapshot.

**On 15: I concede the lifecycle constraint and prefer an honest refusal to a mixed enumeration.**

For a stateless CLI, default refusal is reasonable. For a resident client with the old result available, continuing that labelled result is desirable. Neither requires promising the other capability. Expiry or process restart must produce a specific unavailable-basis error, and a why-not/show operation against an old result needs the same basis discipline.

I am not persuaded that mixed keyset continuation should be our next feature. "Labelled mixed" is truthful, but it asks the reader to reason about duplicate and missing rows while they are trying to browse. A human GUI can often keep the visible result stable and offer "new data available" instead. An agent enumerating candidates should strongly prefer restart over potentially missing one. My own default is restart; retaining a basis is the improvement I would pursue first where a process can support it.

There is also a separation from effects: even an enumeration consistent at basis B is not current authorization at basis B+1. Passing explicit selected ids to a plan preserves what was selected; the effect's own preconditions decide whether it can still act. We should not imply that search snapshot consistency alone makes a later action current.

**On 16–18: use the compatibility test, but do not let it classify prerequisites as optional.**

I agree that many features can be added later without changing the meaning of an existing query. That does not make all of them optional for a promised client experience. Structured field descriptions may be additive syntax-wise but are part of making a GUI author possible. Drill-down may be additive yet necessary for a visible diagnostic to be useful. Separate compatibility risk from the smallest coherent workflow we intend to validate.

I accept the proposed walk. We can run it on draft notation; we need only refuse to invent new syntax mid-task and record where we wished we could. No need to settle every issue above first. I propose that your v2 marks an operation unavailable when it has no defined spelling or response. That is useful evidence, not a failed performance by the author.

Here is the scenario script. It defines synthetic facts and expected distinctions, not new claims about GGG or new repository tests. Templates and totals used below are stipulated to be admitted reference definitions for the exercise. The client's visible description must supply their actual names; the participant should not be required to know them from this script.

**Scenario: a ring search which changes while we investigate it.**

Use one account with pc and poe2 data, and a second account available to the receiving client. The task starts in pc/Standard. There are two fetched locations, Rings and Dump, with different observation times, and one listed location, Unfetched, whose contents have never been read. The initial basis is B0. Item labels below stand for stable item handles.

| Item | Class / rarity | Life occurrences | Defined total resistance | Distinguishing fact |
| --- | --- | --- | --- | --- |
| r1 | ring / rare | explicit 95 | 65 | ordinary qualifying candidate |
| r2 | ring / rare | implicit 20, explicit 75 | 60 | qualifies by life sum, not by one life occurrence |
| r3 | ring / rare | explicit 100 | 55 | insufficient resistance |
| r4 | ring / magic | explicit 95 | 70 | wrong rarity |
| r5 | class undecided / rare | explicit 95 | 65 | readable body, missing class mapping |
| r6 | ring / rare | readable explicit 95; implicit array unread | incomplete known subtotal 65 | life occurrence can be witnessed, resistance total cannot be established |
| r7 | ring / rare | known absent | 65 | no life line, not an unread one |

All data are readable except where explicitly stated. A duplicate-looking item exists in poe2 and must not leak into pc scope. Each fetched row has a complete named location path; put one in a nested stash location to exercise more than a tab label. Other than r6, the displayed resistance contributors are complete and their sums equal the table.

1. Start from only the advertised tool/help surface. Establish the intended account and realm, discover how to ask for rare rings with one life occurrence at least 90 and total resistance at least 60, and request compact evidence-bearing rows. At B0 the definite set is `{r1}`, root-undecided is `{r5, r6}`, and the other four fail. Distinguish these outcomes from the unfetched location. Do not fetch anything automatically.
2. Introduce a misspelled field name. Recover using the authoring error. Separately introduce a valid tab substring with a typo that matches no known tab. Recover through corpus discovery after current matches have become empty. Record whether the participant has to guess how to escape the empty discovery context.
3. Inspect why r2 failed. Then change the question to permit summed life of at least 90. The definite set becomes `{r1, r2}`. Preserve the original occurrence query as a reusable value; this is an intentional semantic edit, not correction of the original interpretation.
4. On a separate all-items view at B0, count by class. The undecided-class bucket contains r5 even though every row is a definite match to the empty predicate. Navigate to exactly r5 through the bucket. Then navigate to an atomic term's failures and verify its denominator against the original diagnostic, rather than assuming every route appends to the original filter.
5. Restore the occurrence query and raise the life threshold to 100. There are no definite matches; r6 remains potentially qualifying through its unread evidence. Ask what changing one selected constraint would do. Explain what can be learned locally, what a refresh might resolve, and why refreshing r5's readable body is not a justified remedy for an absent reference mapping.
6. Request independent class/rarity facets, then a crossed class-by-rarity table, then rows again. Transfer the complete request between terminal and structured client. The receiving client initially has the other account selected. Preserve the source account or demand deliberate rebinding; do not silently substitute its default. Also preserve which of the three views was requested.
7. Use the summed-life query with a one-row page and descending resistance. After inspecting page one at B0, publish B1: r1's total becomes 55, r3's becomes 65, and the previously unfetched location supplies a new qualifying r8. Continue and inspect one old row against the old query. The client must reveal whether it retained B0, restarted at B1, or cannot serve the old basis. It must not present old rows with new explanations as one observation.
8. Derive refresh work from coverage, keeping the item predicate out of any claim that unfetched locations cannot match. Include the case where the location listing itself is old. No request count is stipulated here: the planner's actual prerequisites determine it. This is a dry workflow description, not authorization for a live run.

Three short probes accompany that walk:

- Paste `"+92 to maximum Life"`, then try it as a line selector and as a numeric comparison. Observe whether the 92 survives or a specific ambiguity is reported. Repeat with a quoted item name and a bare word that is also a field name.
- An item has two cold-damage occurrences, 10–40 and 20–50. Asking for one occurrence with low >=15 and high <=45 must fail. Asking for two independent existential requirements may pass. Move between text and tree, and add the second bound through the intended editing interaction.
- For diagnostic interpretation, use `A or B` with an item satisfying both, and `holds(A, B)=1`. Ask for a selected node's substitution counts and for alternative values of that node. The answer must not call all these operations "remove this filter".

Record the submitted request, the displayed interpretation, the returned evidence/next step, and the first assumption the participant had to invent. Count calls and unnecessary output, but do not optimize the walk into one call by concealing a discovery dependency. These are paper or prototype exercises until a surface exists; neither of us should report imagined outputs as an executed user study.

For your next response, the useful disagreements to settle are narrow: whether multi-slot shorthand earns admission in v2; whether our explicit member attributes are readable enough; and whether the diagnostic/substitution distinction above changes your fourth discovery context. You can revise the page and mark the remaining capabilities unavailable without another permission round. I am ready to walk that revision; the weighted-expression and retained-basis questions can remain visibly open while we do.
