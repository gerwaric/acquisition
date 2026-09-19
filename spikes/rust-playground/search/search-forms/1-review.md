My assessment is that **the search has a strong semantic foundation, but its interaction model is less mature than its rules for evaluating items**. Note 35 improves the proposed language substantially. I would keep much of it, but I would not treat the remaining work as choosing spellings and then writing examples. Several choices still affect how people form, revise, and trust a question.

I read the audit, [note 35](/Users/tom/Development/GitHub/gerwaric/acquisition/spikes/rust-playground/brainstorming-notes/35-search-forms-table.md), the current contract, the research digest, and the human and agent findings. I also checked GGG’s published API reference. The assessment below is my design judgment, independent of whether a change fits the current project rulings.

What I particularly like:

- **One occurrence and an item’s total are distinct.** This preserves information people actually care about. A fractured modifier, an individual damage range, and total resistance are different questions.
- **An answer carries its interpretation and evidence.** Canonical queries, matching lines, locations, and a why-not operation are excellent foundations for trust.
- **Absence and inability to determine are distinct.** This fixes a class of misleading answers that otherwise becomes almost impossible for a client to detect.
- **Text and structured queries describe the same semantics.** This creates the possibility of moving naturally between conversation, terminal, and GUI.
- **Search remains usable offline, with explicit fetching.** That is a sound basis for a responsive application and predictable agent behavior.

Those are the parts I would protect most strongly.

**I would design the experience around an evolving question.**

Consider me answering: “Do I have a rare ring with enough resistance for this build?”

I need to establish the account and league, understand what “enough” means, discover the available resistance vocabulary, inspect candidates, and perhaps fetch missing locations. After finding nothing, I might need to relax one requirement or establish that the answer is incomplete.

The expression is one part of that activity. My ideal search interface makes each transition inexpensive:

> Discover → express → inspect → refine → assess coverage → fetch if needed → reconsider.

The current design supports several steps individually. The connections deserve more attention.

For example, the scope block already reports unfetched locations and fetch age. That is valuable, but I would want actionable coverage information:

> No matches among the fetched items. Some locations in this league have never been fetched. Here are those locations and the refresh work needed to include them.

An important limitation follows: an item predicate generally cannot tell us which unfetched tabs contain its matches. Fetching only tabs that *previously* contained rings cannot establish that there are no rings elsewhere. Refresh advice must distinguish a useful heuristic from a coverage guarantee.

The existing planner provides much of the foundation. I would connect search coverage to that planner through the client workflow. Search itself can remain a pure read.

Also, a consistent store snapshot is not a simultaneous observation of the account. Its locations may have been fetched at different times. I want the product to make three things independently understandable: **what was searched, when it was observed, and what could be evaluated**.

**Discovery needs to work especially well when the current query is wrong.**

The corpus vocabulary is one of the best ideas here. It grounds suggestions in items I actually own and can supply exact predicates without requiring me to remember templates.

But “count lines under the current query” cannot by itself serve every discovery need.

Suppose I have added a mistaken condition and reached zero results. Vocabulary restricted to those results is now empty too. The mechanism that should help me recover has lost its evidence.

I would distinguish:

- Supported fields, operators, totals, and their definitions.
- Templates observed anywhere in the selected corpus.
- Templates and values occurring among the current matches.

These can share machinery and even a tool. They need distinct meanings.

A GUI introduces another case: when editing a selected rarity filter, I often want the other rarity options under the *remaining* conditions. Otherwise selecting “rare” makes every alternative disappear. An agent benefits from the same capability when asking, “Which constraint is eliminating my candidates?”

The existing per-term diagnostics are useful but do not fully answer that question. Counts calculated across the whole scope can tell me that many items have life and many are rings without explaining why no item satisfies the combination. An optional “matches if this condition is removed” count would be more useful during refinement.

I would also separate suggestion tolerance from matching tolerance. Typo suggestions and word-based template discovery can be forgiving while execution remains exact. There is no need to make the evaluator guess in order to help the author.

**I would make the complete request the thing clients exchange.**

The conditions alone do not preserve what someone asked. Realm, account, ordering, and selected view matter too.

Conceptually, I want:

```text
Request: scope + conditions + view
Answer: interpreted request + results + evidence + coverage + continuation
```

A GUI should be able to hand that request to an agent, which refines it and returns something the GUI can display and edit. A copied query should not quietly acquire another client’s default realm.

This is why I would drop the special `realm:pc` prefix inside the query string. The distinction between scope and conditions is sensible; making one scope setting look like a condition introduces an exception. It also complicates the otherwise appealing instruction to refine with `(old query) new-term`.

For agents, I would accept both text and a documented structured representation. I would ordinarily write text for short queries. When modifying a complicated existing query, I would often prefer to change its tree. The earlier reviewers choosing text does not establish that text is best for both activities.

For a GUI, the tree needs enough identity to associate an error, matching witness, or edit with a particular condition. It also needs descriptions of types, allowed operators, units, and slots. A GUI should not have to extract that information from prose help.

**Several of note 35’s conveniences would make me less confident as a user.**

The clearest example is:

```text
"+92 to maximum Life"
```

Under the proposal, this names the template and discards the value constraint. An item with 12 life could match.

That is mechanically explainable, but I think it violates the expectation created by pasting a concrete line. Printing the interpretation afterward helps, yet a user can easily overlook the loss.

I would make “use this line’s template” and “use this line’s value” explicit choices when inserting a displayed line. In textual syntax, I would strongly prefer an explicit line selector over assigning this special meaning to every standalone quotation.

Likewise, ordinary quoted phrases should be easy to search. Someone typing `"Kaom's Heart"` is very likely searching for the item. Requiring them to learn that standalone quotations always mean modifier templates is a substantial cost.

My other strong preferences are:

- **Bare words should always mean text.** N10’s alternative is better. Adding a field named `sockets` should not cause an old text search for `sockets` to become an error.
- **Unknown fields should be authoring errors.** An unrecognized field name is different from a recognized field whose value cannot be determined on an item.
- **Exact matches and substring matches should remain visibly distinct.** N7 is good. I would use exact values by default for enum selections generated by a GUI.
- **Keep bounded ranges and explicit Boolean grouping.** N5 and N6 seem reasonable. I would also accept explicit `and` and `not`; agents and humans already know those words.
- **Keep `linked(…)`.** It communicates an important relationship. I would test whether names such as `socket.red` make standalone colour fields clearer.

I would not require users to master every available spelling. There should be a small, unsurprising authoring path, with shortcuts added where they demonstrably help.

**The occurrence boundary deserves more visibility than the current typography gives it.**

This expression has important semantics:

```text
"Adds # to # Cold Damage" low>=15 high<=45
```

Both conditions bind to one occurrence. Consider an item with occurrences `10–40` and `20–50`: neither qualifies, although separate searches for the lower and upper conditions could each find a witness.

That distinction is correct. My concern is that it is carried by adjacency—the same visual device that elsewhere means conjunction between independent terms.

I would explore explicit grouping around an occurrence’s conditions. I am less concerned about the final punctuation than about making the boundary visible. A GUI can express this naturally as one modifier row with two bounds.

This also exposes a limitation of “refinement is the old query plus another term.” Refinement sometimes means adding an independent requirement; sometimes it means adding another condition to an existing occurrence. The client model should support both.

Source and flags belong inside that same occurrence selection. `explicit.fractured:suppress` is compact, but it disguises two independent dimensions as a compound category. I would keep those dimensions explicit in the tree and consider a clearer long form for text.

More generally, lines and link groups reveal a recurring concept: **several conditions must hold on the same member of a collection**. I would give that concept one internal representation, even if the public language exposes only a few domain-specific forms. That avoids accumulating unrelated special cases without introducing raw JSON queries or a general programming language.

**I would make every diagnostic bucket inspectable.**

I strongly favour G3’s option to select undecided values, or an equivalent structured drill-down.

There is a concrete problem with the alternative described in the note. Search all items and count by class. An item with an undecidable class still definitely matches “all items.” It belongs in the class table’s undecided bucket, but it does **not** belong in the query’s root-undecided results.

Consequently, those rows cannot always be reached through the answer’s general undecided count.

My broader rule would be: **a count shown to the user should have a route to its members**. That applies to missing values, undecided classifications, unreadable fields, and the diagnostic for lines that meet a threshold only when added together.

This is valuable for all three clients. A human clicks the count; an agent follows its selector; a GUI opens the corresponding rows.

**I would allow more user-authored meaning than G1 currently seems comfortable with.**

A shipped total such as total resistance deserves a stable, reviewed definition. But a personal weighted expression has a different purpose.

If I say, “For this comparison, count strength twice and dexterity once,” I have supplied the judgment. Acquisition can evaluate that arithmetic without endorsing it as a universal definition of item quality.

I would permit a bounded, inspectable weighted expression over supported values, subject to clear missing-value and occurrence rules. It could remain unnamed and need not imply scripting, arbitrary functions, or general aggregation.

This is where my vision diverges most from the current “add a field or a reviewed total” direction. That approach works well for missing objective facts. It can become restrictive for personal combinations of facts—the part of searching where users express their own priorities.

I would not add this merely to claim trade-search parity. I would add it because the principle “judgment belongs to the user” becomes more useful when the user can express that judgment.

G2 needs a different treatment. The existing [currency table](/Users/tom/Development/GitHub/gerwaric/acquisition/spikes/rust-playground/crates/acquisition-plan/src/currency.rs:70) contains identifiers, names, aliases, and applicability; it does not contain chaos exchange rates. “The table’s chaos equivalent” is therefore not an existing option.

I would keep `priced` Boolean and expose price amount, currency, and lot/unit meaning separately. Cross-currency comparison would require an explicit additional valuation input.

**Two kinds of change need a clearer user-facing story.**

First, a query and its current interpretation are distinct.

For example, `line:suppress` expresses a continuing substring search. Its resolved template list can change after a refresh. An exact template selector expresses something narrower. A saved query should preserve which of those intentions the user chose; it should not accidentally freeze discovery results or silently broaden an exact selection.

Returning the canonical query and resolved templates is a good start. I would explicitly distinguish the authored selector from its binding against this corpus.

Second, I would reconsider invalidating every continuation whenever the basis changes.

The current rule in [the search contract](/Users/tom/Development/GitHub/gerwaric/acquisition/spikes/rust-playground/decisions/search.md) is safe, but could be frustrating during an active refresh: page one succeeds, page two requires starting over, and subsequent refreshes repeat the problem.

I would prefer the ability to continue an explicitly labelled snapshot, with a visible indication that newer data exists. A new search uses current data; continuing an existing result finishes reading that result. This has storage and lifetime costs, so it deserves measurement, but it is a legitimate product choice rather than a correctness failure.

Finally, I would make item placement more prominent in the experience. GGG’s objects include nested socketed items and hierarchical stash tabs; a flat result needs enough context to tell someone how to reach the item. “In this tab” may still leave “inside which item or subcontainer?” unanswered. [GGG API reference](https://www.pathofexile.com/developer/docs/reference#type-Item)

What drives my thinking is the cost of recovering from an imperfect question. As an agent, I can learn an unusual grammar. What hurts is an answer that leaves me uncertain which assumption failed, requires a large discovery response, or forces me to rebuild a request. For a human, those costs become hesitation and distrust. For a GUI, they become awkward controls and missing drill-downs.

I would carry note 35 forward as a useful draft, while revisiting its quoted-line behavior, bare-word rule, discovery contexts, occurrence grouping, and diagnostic navigation. Then I would walk one ordinary task through all three clients—including a typo, zero matches, incomplete coverage, and a refresh during inspection. Those transitions would tell us more about whether the search is understandable than another round of isolated syntax verdicts.