Fable — the agent seat against [5-forms-v2.md](5-forms-v2.md), using the synthetic scenario in [3-response.md](3-response.md). This is a paper walk. No search implementation, MCP call, or live request was exercised. I read your terminal walk first, so this is a second perspective, not an independent blind trial.

One limitation applies throughout: v2 promises tree input but supplies no serialized node schema or complete example request. The existing MCP reference documents the older substring search. I cannot honestly produce executable proposed MCP calls from these inputs. Below I use semantic tree notation only, and mark where a documented request shape is needed. That is sufficient to inspect the edits; it does not validate the wire interface.

The initial request, in that notation, is:

```text
bound account A, realm pc, live items; view rows
All(
  Equal(league, Standard),
  Equal(class, ring),
  Equal(rarity, rare),
  Exists(lines, All(TemplateEquals("# to maximum Life"), Compare(slot1, >=, 90))),
  Compare(total-res, >=, 60)
)
```

Call the life subtree L. That is my local reference to the returned node, not a proposed permanent wire id.

| Scenario step | Agent operation | Expected consequence from v2 | Friction or missing definition |
| --- | --- | --- | --- |
| 1: cold discovery | Resolve A and pc; ask for relevant fields/totals and observed life/resistance templates; construct the request above. | Definite `{r1}`; root-undecided `{r5,r6}`; unread/unfetched locations reported separately. The readable 95 witnesses r6's life condition. | Language discovery has no concrete request shape, and template discovery cannot be text-narrowed in v2. These are the same substantive gaps as terminal G1/G2. Tree input adds no automatic knowledge of the vocabulary. |
| 2: authoring error | Misspell the rarity field, inspect the structured error, and replace that field. Separately use a valid tab selector with a misspelled value and recover through corpus discovery. | First operation fails validation; the second is a valid query with no matches and suggestions. | A tree prevents punctuation mistakes, not unknown-field mistakes. Corpus discovery must explicitly say which filter it evaluated after removing the failed restriction. |
| 3: change occurrence to total | Copy the returned request; replace L with `Compare(Sum(Project(SelectLines(template=T), slot1)), >=, 90)`; retain the original request. | Definite `{r1,r2}`; r6 is now undecided on life sum as well as resistance. r7 has complete life sum zero. | This is a useful structural edit: exactly one subtree changes. It still requires knowing the sum/projection schema. For this short filter, replacing the textual term would be equally reasonable. |
| 4: inspect counts | Request class counts under an empty filter; submit the returned undecided-bucket route unchanged. Separately follow the original life term's failed route. | Class route yields r5. Atomic failure route yields r2 in the stipulated seven-item domain; it must not append the original Q1 conditions. | Bound drill-down requests save reasoning and calls. The route must be directly acceptable as input, not an explanatory object that I have to translate. Complete-zero totals still need a presence/diagnostic rule (G6). |
| 5: refine a bound | Restore the occurrence request; change L's bound from 90 to 100; ask for the life node's explanation. Then, if considering 95, copy that request and change 100 to 95. | At 100: no definite matches, r6 root-undecided. At 95: definite r1, root-undecided r5/r6. | This is one numeric edit and one ordinary search per proposed threshold. I do not need a numeric distribution feature to complete this task. It could later reduce exploration cost. |
| 6: change view and transfer | Keep the filter; set independent class/rarity facets, then crossed counts, then rows. Transfer the returned complete bound request to the other client. | View and account binding survive. The receiver cannot silently substitute B for A. | The filter/view separation is useful. The exact JSON union shapes remain unspecified; a documented example is needed before this can become a callable walk. |
| 7: concurrent refresh | Under the summed-life request, read one row ordered by resistance descending; then continue after B1. | A retained B0 may continue as B0, otherwise the continuation refuses. A new search or new-basis why-not is explicit. | Default refusal is usable for this first version. My workflow restarts and keeps any previously inspected ids for comparison. I would not accept a silently mixed enumeration. |
| 8: coverage to fetching | Read the full coordinates of the uncovered locations; inspect available planner capabilities. | Report the work needed without asserting that unfetched locations do or do not contain matches. | The current planner cannot take this selection directly. The terminal walk's proposed command is invalid; details below. This is an integration gap, not a missing display label. |

For step 5's explanation, the expected counts are those in your terminal walk: original 0 definite / 1 undecided; life node forced true 1 definite / 2 undecided; forced false 0 definite / 0 undecided. The remedy for r5 is investigation of the reference mapping, while refreshing r6 may supply readable evidence. A remedy class is a useful hint, not a guarantee that the action will resolve the condition.

**Where tree editing helps.**

The clearest benefit is keeping two bounds on the same line occurrence. Given an existential line node with `low>=15`, I add `high<=45` to its inner conjunction. For the 10–40 and 20–50 example this stays false. Appending a second existential node to the outer conjunction instead passes. The tree makes the intended location of the edit explicit. The canonical grouped text does so too, which means the two encodings reinforce each other.

Changing 90 to 100 is not evidence that an agent needs trees: text is shorter and clear here. Nor is tree input a reason to define a separate remote patch API. Returning a documented tree and accepting its edited copy suffices. I would use text for short fresh questions and the returned tree when preserving or changing nested structure. Node paths should be treated as local to the exact returned filter; a structural edit means using the paths from the next answer.

**Would I call alternatives?**

Occasionally, for a categorical question such as "what if this rarity selection were different?" It is not needed anywhere in the main ring task. Numeric exploration here is cheap enough to perform through an explicit bound edit and another search. I would not delay the first usable search to build an alternatives engine or a distribution view.

I also would not infer actual call counts or response byte sizes from this paper walk. V2's machine discovery and node schema are incomplete, so any precise cold-start number would depend on invented responses.

**Two material findings beyond the terminal walk.**

First, compactness must cover the whole answer. V2's scope block describes a per-location handle and observed-at value, plus every never-fetched handle. Enumerating thousands of locations would recreate the large discovery dump even if the result view returns one row. Return coverage summaries and bounded examples by default, with an explicit route to the complete location set. Apply the same rule to vocabulary tables and diagnostic members. For an agent, this matters more than reducing a short filter by a few characters.

Second, terminal step 8 does not describe an existing command:

```text
acq refresh --tabs <id> --league Standard --realm pc --plan
```

`--plan` conflicts with `--tabs`, `--all`, and `--deep` in [main.rs](../../crates/acquisition-cli/src/main.rs), both in the clap declaration and the test `refresh_plan_conflicts_with_the_execute_selectors`. [CLI-REFERENCE.md](../../CLI-REFERENCE.md) says `--plan` compiles stored policy. The MCP `refresh_plan` likewise takes account/realm/league, not a location selection. C76 in [decisions/plans.md](../../decisions/plans.md) describes the future selected-location plan path; its build is still parked.

The existing policy plan can help when policy covers the location, but that is conditional and may include other work. Removing `--plan` changes the command into execution. Replacing policy would mutate durable intent. Neither is an invisible repair for this read-only planning step. Mark a plan directly from the uncovered-location selection unavailable until that integration exists. Search can still provide useful, honest coverage information meanwhile.

My seat verdict: the core authoring and refinement model is workable on paper. The high-value remaining work is bounded discovery, a concrete request/route shape, complete-zero sum semantics, and an honest planner handoff. I do not need more syntax variants, generalized alternatives, or retained snapshot infrastructure to begin a real trial.
