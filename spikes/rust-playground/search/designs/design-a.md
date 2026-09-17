# Design A — Search as a question with its evidence

## 1–2. The model and grammar — one page

**Ask about items; get an answer that can show its work.** A question is
an explicit scope, a condition, and the information wanted back. The
answer contains the same question, the number of matches, and compact
items with their locations and the facts that satisfied the condition.
Refinement adds a condition to that question. Nothing depends on what
the server remembers or what rows an agent has already downloaded.

An item has ordinary fields and repeated evidence: displayed lines,
socket groups, and other collections. A line keeps its source, flags,
words and separate numbers. “Has this line with this number” is one
condition; several constraints inside it refer to the **same occurrence**.
An item can have several matching occurrences. They are never collapsed.
An unfamiliar line is still searchable as text; search need not identify
the hidden game mod to find the observed line.

The vocabulary read is the same model with descriptions and counts:
what can I ask, what values are here, and what would selecting one mean?
It returns ready-to-use predicates. Named totals are ordinary derived
fields with inspectable formulas, not a second class of query. Facts the
system cannot establish remain unknown; unknown candidates are counted
beside matches, including when there are no matches.

One typed query tree is the contract. The CLI prints/parses this text;
MCP accepts its structured JSON; a GUI edits it. These are encodings,
not competing evaluators. Text is a proposed notation, not existing CLI.

```text
Q := in S [where P] [select E [as name], ...]
     [summarize A, ... [by E, ...]] [facets [E, ...]]
     [order E asc|desc, ...] [inspect id]
P := true | false | E op E | P and P | P or P | not P | (P)
   | at_least(N, P, ...) | any(collection, P) | present(E) | unknown(E)
E := field | literal | [E, ...] | (E) | E + E | E - E | E * E | E / E | if(P,E,E)
   | count(collection where P) | sum|min|max(collection where P, E)
A := count() | sum(E) | min(E) | max(E)
op := = | != | < | <= | > | >= | in | contains
line(P) := any(lines, P)
```

Arithmetic has ordinary precedence; comparisons bind before `not`, then
`and`, then `or`. Literals include numbers, quoted strings and booleans;
enums are unquoted. In `order`, a select alias resolves to its expression.
`$name` marks an example substitution (value or predicate), not executable
syntax. `Q.refine(P)` denotes resubmitting Q with `where (old) and P`.
`select` chooses item fields; `summarize` replaces rows with grouped
aggregates; `facets`
adds independent count tables. Defaults are `where true`, decision fields,
and identity order. `page_size` and continuation are transport options.
`inspect` explains one id instead of listing matches. Example:

```text
in pc/Standard where base = "Spiked Gloves"
  and line(template = "# to maximum Life" and n1 >= 90)
  and (armour > 1000 or required.level < 80)
```

Scope notation abbreviates a document naming provider, account, realm,
league and live/removed membership; no implicit “last league”. An
unambiguous account can be filled by the existing account selection.
Discovery lists fields, collections, enums and templates before rows.
This page is a model synopsis, not a standalone implementation contract
(the missing definitions are a gap recorded in §4.7).

## 3. The derivation this requires

This is a proposal, not a new registry ruling. Its engine choice is my
judgment informed by S144–S153: **a store-owned, transactionally maintained
projection, scanned in long-lived frontends and queried through the same
Rust evaluator in short-lived ones. No search service.** Query time was
cheap; loading and decoding was expensive. Persisting the small facts
used by search removes repeated JSON decoding without inventing another
process, IPC surface or synchronization protocol.
All behavioral rules here are proposed choices; citations identify the
evidence motivating them, not a claim that a track mandated this design.

The projection carries the item's full identity; canonical location,
including named character, tab ancestry and socket parent; raw item body
by reference; typed item fields; repeated line occurrences and numeric
slots; socket groups; other observed collections; extraction diagnostics;
and freshness/membership evidence. The proposed route to documented
item fields without a convenience name is a published `fact["API.path"]` accessor and collection navigation.
**Reach gap:** no catalogue maps S42's 59 private-item filters to these
fields or recipes; raw access alone does not demonstrate that floor. That
accessor means the retained API body, never a SQLite column. Malformed
values produce diagnostics, not coercion to zero. Raw paths are an
escape from incomplete naming, not a separate query language or a way
around scope. Discovery states their types and presence coverage.

Convenience fields include rarity separate from frame family, item and
required levels, flags, dimensions, stack count, socket/link/colour
facts, displayed defences and damage, and arithmetic such as displayed
physical DPS. Explicitly named alternatives distinguish displayed
defence from base defence or a quality-normalised value (S26, S90).
Nothing labels all three “armour”. Map tier can be parsed from the
observed base name; absence of the old property is not tier zero (S10).
PoE2 property 109 is a class source (S9). PoE1 classes use a reviewed,
bundled base-to-class table (S69, S93); unmatched bases remain unclassed
(the base-join coverage is S70).
`equipable` follows that taxonomy and shares its coverage, not a guess
from an icon or a missing requirement. No runtime reference fetch.

Each fact mutation updates affected item projections **in the same store
transaction** and advances a facts revision. Character league and
location metadata remain authoritative joins rather than copied stamps
(S131). A projection version mismatch is rebuilt before use or derived
directly from the current snapshot for that read; there is no period
where old derived values can answer as current. An update that the
extractor cannot understand commits the body with explicit unknowns,
not the previous projection. The daemon merely calls the fact mutation
surface; it gains no search, fact-reading or intent-reading behavior
(C2, C34).

Every request opens a fresh store snapshot. A frontend's resident arrays
are reusable only if their facts revision and derivation version match
that snapshot exactly. Otherwise it loads the current projection before
answering; initial implementation reloads, rather than inventing a delta
bus. MCP/GUI hold arrays for their process lifetime; CLI exits and pays
the projection load again. S151 measured 38 ms at 36,139 and 1,142 ms at
1.08 million for a narrower projection: these are evidence, **not this
design's performance promise**. Full fields, reference rules and intent
will cost more. **Measurement gap:** the specified projection has no
load measurement at either 36,139 or 1,084,170 items. **CLI gap:** a
short-lived scan still pays that load on every invocation. S152 finds
SQLite faster for that lifetime; the cheaper load in S151 does not
establish that this choice competes with S147's per-query SQLite times.
At a million items CLI cold reads may take seconds; this proposal does
not establish acceptable cold-response performance. The GUI may finish an already-started read of revision R, labelled R, but
may not reuse it to answer a new query against R+1. A rebuild is a visible
loading state. Cancel obsolete keystroke requests; do not call old rows
the new answer. Measure cold open, refresh reload, sorting and allocation
before adding indexes or incremental loading (S37, S157).

An answer names a consistent pair of facts and annotation snapshots,
plus reference and derivation versions. Establish the facts snapshot,
then the annotation snapshot, then verify that the facts revision has
not changed using a fresh read; retry if it has. The two held snapshots
then overlap at the annotation read. Revisions are monotonic. This
avoids pretending two unrelated database reads were atomic. Continuous
writes can cause a bounded `snapshot_busy` error with retry guidance;
they cannot produce an unlabeled mixed answer. No operation holds a
long writer lock while searching.

Search remains within the store read boundary, with shared semantics
in Rust (C12, C46). Existing effective-listing resolution must be reused,
not copied. Its dependency location may need adjustment to make that
possible without a store-to-plan cycle; the public search API still
belongs to the store. That is a build task, not permission for a third
frontend channel. Reference inputs follow C68/C79: bundled, versioned,
reviewed extracts, with human review before adoption. The local-source
permissions in `SURFACES.md` are not permission for automatic updates.

**Published contract:** Query v1, field descriptors, line normalization
v1, result/diagnostic types, and SQL views of precisely this read model
(with template conventions and location names documented, S200):
`items`, `locations`, `lines`, `line_values`, socket collections and
read-only intent/listing fields. Internal layout stays private. SQL
runs on an isolated read-only representation of the same snapshots;
no `ATTACH`, file functions, writes or facts-schema escape. It may use
the familiar SQL language fully, but gains no additional facts. Query
evaluation and SQL obtain values from the same derivation, including
unknown reasons and line slot meanings. SQL nulls represent unavailable
values with companion status columns; explain the consequence of SQL
three-valued filtering in its schema help. SQL reports still carry scope,
basis and pagination. It is available on both CLI and MCP as required.

Additive fields do not change old query meanings. A change to normalization
or a formula's meaning creates a new version; a saved query must explicitly
upgrade or receive `query_version_unsupported`, never reinterpret itself.
Old reference meanings are retained for saved queries while supported;
current data may still leave them unbound. A saved search is the query
document as user-scoped intent through the store, not retained results.
Persistence adds no macro language, server handle or hidden parameters.

## 4. The seven design questions

### 1. Identity belongs to observable evidence

A line occurrence is `(item identity, source array, ordinal)` within a
facts snapshot. Its searchable family is `(realm, normalized template,
source, decoded flags)`. The family describes observations, **not a
hidden mod identity**. A predicate mentioning only the template
deliberately ranges across sources and flags; the vocabulary presents
those distinctions together and supplies a predicate for the selected
one. “Explicit” denotes the physical source array; crafted/fractured
are proposed decoded flags, not mutually exclusive buckets. S50 shows
optional flags on private lines but does not establish their values;
S51's crafted/fractured decoding is evidence about trade fetch only.
**Gap:** private-line decoding is unverified, so `flag.fractured` and
`flag.crafted` remain unknown wherever no reviewed private mapping exists.
A compound condition within `line(...)` binds one occurrence; separate `line(...)` conditions may bind different
ones. Repeated lines survive and may satisfy both predicates; requiring
two occurrences uses `count(lines where ...) >= 2` (S27–S28).

Normalization v1 unwraps `[Tag|Display]` to Display, normalizes whitespace
and case for matching, and replaces each signed decimal number with `#`,
recording its signed value in ordered `n1`, `n2`, … slots. Thus `+90 to
maximum Life` has template `# to maximum Life`, `n1=90`; `Adds 17 to 30`
has two slots, never their mean (S68). Original bytes and source
are retained. Numeric literals, such as a granted skill's level, also
become slots: their position is not a claim that they are a random roll.
Selecting an exact literal simply constrains its slot, or the raw text.
A numberless line has a template and zero slots. Unknown numeric syntax
stays text with a diagnostic; a veiled placeholder has no value (S12).
Recognized range separators are not negative signs: `10-20` has values
10 and 20, while `-10 to -5` has -10 and -5. Ambiguous syntax is unknown.
Do not delete punctuation or invert “reduced” into “increased”.

`n1` is intentionally a position, not a guessed meaning. Discovery shows
a representative verbatim line with each capture highlighted. Optional
reviewed slot labels can say “lower damage” and “upper damage”; the
query still has separate values. This costs a little ergonomics in
exchange for not teaching a wrong meaning across arbitrary new lines.

Local/global twins remain observationally indistinguishable unless
the item's context and a reviewed rule resolve them (S44, S72). Such
semantic aliases are optional metadata, with candidates and provenance,
never replacements for the raw family. No hash reconstruction is needed
to search. A renamed line becomes a new family; old saved predicates
are retained, but an unbound template produces a named diagnostic even
when another OR branch matches. Offer lexical candidates for a human to
inspect, never substitute them in evaluation (S107, S110–S111). If the
old wording still exists on some items, the app cannot detect that the
user intended a broader renamed family: this is an additional limit.

### 2. One question, Boolean throughout

Fields, a line predicate, collection predicates and arithmetic all enter
the same Boolean tree (S169, S172). `at_least` counts satisfied child
conditions, not lines. `not`, nesting and optional-presence conditions
express the site's useful composition across every field. For example,
`not present(required.level) or required.level < 80` explicitly permits
known absence; it does not silently grant level zero (S158). Boolean
flags are `= true`, `= false`, or unconstrained; unknown is neither.

Atoms evaluate true, false or unknown. `false and unknown` is false;
`true or unknown` is true; `not unknown` remains unknown. At-least-N is
true if N children are true, false if even all unknowns cannot reach N,
otherwise unknown. A missing known optional field is absent, an
unparseable/unsupported one unknown; ordinary comparisons to either
are unknown. `present` distinguishes known absence from presence and
itself stays unknown for undecodable input. `unknown` explicitly selects
unavailable values. Collection existence is false only when the relevant
collection is known complete and no occurrence qualifies.

Scalar aggregates consume occurrences, not distinct templates. An empty
complete sum is zero; an unavailable contributing value makes the sum
unknown, as does an unknown selection condition that could change it.
Min/max of an empty collection are absent. `if` takes only its selected
branch; an unknown condition yields unknown. Division by zero is
unknown with a diagnostic. Sorting puts absent/unknown last in either
direction and appends full identity as a tie-break. Text `contains` is
case-insensitive substring over the named display field; `=` is exact
after that field's documented normalization. There is no accidental
match against an icon URL or unrelated body text.

Result aggregation follows the same rules. `summarize count(),
sum(stack.count) by base` gives item and unit totals without downloading
rows; unknown values are reported, never silently omitted from a claimed
total. Facets are the shorthand for several independent `count() by E`
summaries, with absent and unknown buckets. Their denominators are all
true matches, before pagination; unknown candidates stay separately
visible in the answer envelope.
League keys include realm, and location keys include kind and full
identity; duplicate display names never merge distinct containers.

Named computed fields are small reviewed recipes over this same evidence.
`total.fire_resistance` sums qualifying fire, all-elemental and relevant
hybrid contributions once per line occurrence: a proposed recipe,
motivated by the trade observation S49, not validated on private lines.
S49 establishes neither private contributor coverage nor equivalence
with the site. Elemental total sums fire, cold and lightning, and intentionally counts an all-elemental
contribution three times. Attributes follow the analogous published
recipe. Recipes explain their supported sources, applicability and
excluded meanings; they do not simulate character effects or conditional
uptime. Unknown applicable lines that cannot be ruled out as contributors
make a semantic total incomplete, not a deceptively exact lower number.
The result exposes its known subtotal and uncovered lines as evidence;
the predicate on the total remains unknown. Explicit user arithmetic over
selected templates is exact about that selection, but never silently
promoted to a complete game stat. This conservative completeness policy
leaves an **everyday-search gap**: OQ1/AQ2 may not yield the requested
answer. No private-corpus coverage trial establishes how often this
happens. Writing finer line tests does not repair the promised easy case.

For MCP, the common predicate has a small literal shape:

```json
{"all":[{"field":"rarity","eq":"rare"},
        {"line":{"template":"# to maximum Life","n1":{"gte":90}}}]}
```

`all`/`any`/`not`/`at_least` and general typed expression nodes are the
JSON encoding of the grammar above; the line object desugars to a
same-occurrence conjunction. Discovery supplies these nodes ready to
splice in. CLI completion inserts the corresponding text. The adapter
canonicalizes shorthand before evaluation; there is one semantic tree.

### 3. Discovery is a useful first answer

`search_describe` (CLI `items search describe`) accepts a scope plus
optional terms/fields and returns descriptors, applicable value sets,
coverage counts and ready-to-use predicate fragments together. A first
call can ask for `ring`, `fire resistance`, and `dexterity` at once. It
returns field types and units, exact template/source/flag alternatives,
slot examples, named recipe definitions, supported/unknown counts, and
location names beside ids. It does not require a sample item (S181).

Rank exact names first, then token matches, then frequency within scope;
stable lexical ordering breaks ties. Include zero-occurrence referenced
entries as such, never silently remove a saved choice. Bound the response
and provide continuation and total vocabulary counts. Discovery is not
fuzzy query execution. GUI autocomplete calls this same read, with
cancellation; the MCP asks a few terms at once. On a cold open it may
load; once loaded it scans compact vocabulary, not whole item bodies.
“Instant” is an acceptance measurement on the implementation, not an
assertion borrowed from a narrower benchmark (S174, S190).

### 4. Ownership and restart

Section 3 is the commitment: persistent derivation owned by the store,
short-lived snapshots, optional frontend-resident arrays, no service.
The benchmark justifies avoiding repeated body decoding, not a particular
index or a guarantee about reload latency. This preserves the same
interface for a third Rust frontend and keeps PoE2 as realm-scoped
vocabulary and extraction rules. December's unfamiliar lines are text
and numbers on day one; new meanings and aliases wait for evidence.

### 5. Trade is translation with a remainder

Accept pasted query JSON or decode a self-contained query URL locally
(S56); bound compressed and decoded size. Never fetch a short URL or
contact the site. Translation returns a proposed query plus a per-clause
report: exact, ambiguous, unsupported, or listing-only. A nonempty
remainder prevents automatic execution as an equivalent query. The user
can explicitly choose an edited local query, whose report names the
dropped conditions. Read-only search does not need a generic approval
ceremony; only the change in meaning must be explicit.

Trade numbers map through reviewed reference data to possible local
line/context predicates (S61–S67). Several possible hidden identities
are not solved by arbitrarily choosing the first or OR-ing everything:
OR is equivalent only when the mapping proves that union is the intended
question. Averaged trade range bounds translate to arithmetic over both
slots; do not change the core's separate-value model. Weighted groups
become arithmetic over contributions, optional conditions Boolean
expressions, counts `at_least`. A group whose semantics have not been
verified remains unsupported; the digest names eight types but does not
fully specify all eight (S41). No claim of complete URL compatibility.

In reverse, only the subset with proved mappings can become a trade
query. Local locations, observation times, annotations, unknown families
and arbitrary field composition may have no equivalent. Return those
as a remainder; never emit a broad market search labelled equivalent.
The translation contract is part of this design; building the bridge
is separate scope under note 22. The core's item reach does not wait for
trade identifiers or a browser integration (S155).

### 6. An answer is compact and accountable

Every answer carries resolved scope, canonical query, snapshot basis,
exact matched/false/unknown item counts, returned count, truncation and
continuation, plus coverage warnings. Unknown candidates have their
own selectable diagnostic view; they are not silently returned as
matches. Body parse failures and unknown league membership count as
coverage gaps, even if scope prevents placing them among candidates.
An unknown scope token is a structured error listing valid choices,
not zero items (S199). “Zero matches; 18 unknown candidates; 3 listed
tabs never observed” is a different answer from “zero matches in all
observed items”. Fresh local truth is never a promise of fresh server
truth.

Default rows carry full usable identity, display name/base/rarity, realm,
league, named location and parent chain, observation age, and bounded
positive witnesses. Counts of omitted witnesses and an audit read prevent
silent truncation. The audit view adds all predicate outcomes, verbatim
source lines, numeric slots, recipe contributors and extraction/reference
versions. JSON is authoritative; text follows C53, listing ten or fewer
entities and otherwise counting with a continuation action. Bodies are
opt-in, not the price of obtaining twenty candidates (S183, S193).

Pagination is bound to query, ordering and basis. A continuation against
changed facts/intent returns `basis_changed` and the restart query; it
never skips or duplicates items under the old cursor. This avoids a
cross-process snapshot-retention service, but a rapidly changing stash
can force repeated restarts: an explicit additional limit.

Listing fields distinguish manual assertions, game observations and the
effective resolution, including cause and annotation revision (C64,
C69); `priced` is only a documented alias for a specified resolution.
No automatic market price. A future legacy annotation is readable as
the user's assertion, not reclassified as a fact; this proposal does
not introduce such a kind merely to answer OQ2. Socketed children are
searchable items with their parent's location, not duplicated top-level
gear. Counts clearly distinguish item rows from stack totals.

### 7. Limits are part of the output contract

The digest's entire limits register is inherited:

| Limit | Output |
| --- | --- |
| S12 | Veiled placeholder shown; no numeric value match. |
| S14 | `map.area: unknown — not supplied`; no icon inference. |
| S52 | Displayed line and value known; generating mods unknown. |
| S53 | Derived class with source/coverage, or unclassed. |
| S67 | All unresolved trade candidates, translation blocked as equivalent. |
| S107 | Unrecognized semantic line still shown and text-searchable; no guessed alias. |
| S111 | Changed wording becomes an unbound saved selector or new family; built-in recipe dependencies fail validation loudly. |
| S177 | Origin unknown; first-seen/current league explicitly named as different facts. |
| S178 | Legacy criterion supplied by user; no claim about present game availability. |

Additional limits: a corpus may be old or incompletely refreshed; numeric
position is not numeric meaning; some raw paths have no convenient alias;
unrecognized future mod-array shapes cannot silently be counted as fully
covered by `lines`; recipes and taxonomy have declared incomplete
coverage; unchanged old wording cannot reveal intended renamed variants;
snapshot establishment and pagination may require retry; full-projection
load and CLI cold-response performance are unproven; URL conversion is
partial. Private crafted/fractured flag decoding and the complete S42
filter mapping are unverified; named totals can leave everyday OQ1/AQ2
unanswered. AQ1 cannot restrict just its tab facet in one query. Data
coverage failures have diagnostic fields; the unmeasured performance and
unmapped reach remain proposal gaps. In particular, a new unknown
array remains queryable through its raw path and marks line coverage
incomplete until extraction understands it.

**One-page contract gap:** §1–2 alone cannot specify an interoperable
implementation. Normalization, three-valued logic, aggregate/empty/error
rules, ordering, text comparison, field catalogue, decoded flags,
collection schemas, raw paths, recipe coverage, select targets, scope
tokens, answer envelope, continuation, and JSON encoding require §3 and
§4 or remain unspecified there. The page is a synopsis; this repair does
not claim to fit those fifteen definitions into it.

## 5. Acceptance appendix — written first

These are specifications, not executed searches. Examples supply missing
user parameters explicitly; they do not claim those are the owner's
actual build requirements. `S` below is the selected account/provider's
live items, in realm `pc`, league `Standard`, across tabs, characters and
socketed children. It is an explicit scope value, not a server session.

| Question | One query, or the exact gap |
| --- | --- |
| OQ1: a rare to complete resistances or attributes | **One query**, given slot and deficits: `in S where rarity = rare and class = Ring and (total.fire_resistance >= 40 or total.dexterity >= 30) select identity, location, total.fire_resistance, total.dexterity, evidence`. These totals mean the published displayed-line recipes, not a build simulation. **Gap:** unclassified possible contributors leave the predicate unknown; the everyday answer is not assured without the missing coverage trial. A stricter supplied need is simply `and`, not `or`. The fractured-base refinement is `base = "Spiked Gloves" and line(template = "#% chance to Suppress Spell Damage" and flag.fractured = true and n1 >= 14)`; this illustrates the notation, not a claim that this base/roll exists or that private fractured flags have been decoded. That decoding is a separate gap (§4.1). |
| OQ2: a legacy version of a named unique | **One query after the observed criterion is supplied**: `in S where name = "Ashes of the Stars" and line(template = $legacy_template and $legacy_value_test) select identity, location, evidence`. S165 names the variant but does not give a verbatim item line. If an observed line literally reads `10-20% increased Reservation Efficiency of Skills`, its predicate is `template = "#-#% increased Reservation Efficiency of Skills" and n1 = 10 and n2 = 20`; a confirmed single-roll line instead uses one slot and the user's bounds. Neither shape is established by S165. **Gap:** without the observed template and user-supplied value test this is not an executable legacy search; “legacy” alone is not a criterion, and the query cannot certify present game availability. |
| OQ3: gear for a newly described interaction | **One query given the interaction**: `in S where base = "Spiked Gloves" and line(template = $interaction_line and n1 >= $minimum) and any(socket_groups, red >= 2 and blue >= 1) select identity, location, evidence`. All three colours must be in one group. A unique or an alternative base can be another Boolean branch. **Gap:** discovering the interaction or deciding whether a build works is not search. |
| OQ4: the valuable staff from Crucible | **Gap in the question as asked:** neither league of origin nor the remembered offer is a stored fact. `in S where class in [Staff, Warstaff] and line(source = crucible) select identity, location, evidence` is a **candidate query only**, not an answer to the original question. Once the owner supplies its id or distinguishing lines, one query locates it. No “Crucible item” inference is silently substituted for origin. |
| OQ5: leveling gear across tabs | **One query**, with a supplied bracket: `in S where equipable = true and required.level >= 1 and required.level <= 28 select identity, location, required.level order required.level asc`. **Gap:** an absent or undecodable requirement is unknown, not zero; such items are counted separately. The query finds eligible candidates, not a complete recommended equipment set. |
| OQ6: the value of the legacy explode chest | **Gap:** market valuation is outside search. Given a user-supplied exact legacy criterion: `in S where class = BodyArmour and line(template = $legacy_explode_line and $legacy_value_test) select identity, location, evidence` is **one query for the prerequisite identification**, not the requested price. A broad `line(text contains "explode")` is discovery, never a legacy test. |
| OQ7: gear with a retiring modifier | **One query given its displayed variants**: `in S where line(template in $announced_templates and $announced_value_test) select identity, location, evidence`. `$announced_templates` is an explicit finite list supplied by the user or selected from discovery; a source/flag restriction is added only if the announcement requires it. **Gap:** a hidden generating-mod id or an unprovided new wording cannot be reconstructed from private lines (S52). |
| AQ1: counts per tab, league, rarity before rows | **Gap in the exact three-table answer:** `in account where true facets [location, league, rarity]` yields per-location, per-league and per-rarity counts, including character locations. `account` explicitly spans its realms and leagues. Facets share one predicate; restricting it to tabs also changes the league/rarity denominators. A second tab-only query is needed to return the requested tab table while preserving account-wide league/rarity counts. No item bodies; the location table is not claimed as the tab table. |
| AQ2: previous Q1 rares, total resistance at least 60 | **One query**: `Q1.refine(total.elemental_resistance >= 60)`. `refine` is the adapter's local operation of appending a predicate to Q1's returned query document and resubmitting it. It carries the original scope, projection and ordering. No handle, replayed rows or remembered server context. “Total resistance” is resolved here to elemental, explicitly excluding chaos; discovery offers both. **Gap:** incomplete recipe coverage can make this predicate unknown, so refinement does not assure the requested shorter list. |
| AQ3: why an item matched or did not | **One query**: `in S where $previous_predicate inspect $item_id`. Returns that item's predicate tree with true/false/unknown leaves, raw supporting lines and scope membership. Inspecting an out-of-scope id says so rather than widening S. Ordinary queries already return positive witnesses; empty results already report scope and unknown counts. |
| AQ4: find an id again | **One query**: `in account where id = $full_id select identity, location, evidence`. Every returned identity includes the provider/account/realm needed to construct this scope; adapters pass it through. Default live-only scope reports a matching removed id as excluded when inspected, never silently revives it. |
| AQ5: whole-corpus maximum-life line at least 90 | **One query**: `in account where line(template = "# to maximum Life" and n1 >= 90) select identity, location, evidence, max(lines where template = "# to maximum Life", n1) as life order life desc`. The line selector spans every recognized mod array and preserves occurrences. The count is the exact number of matching items, not matching lines. |

Discovery and refinement are separate from these acceptance queries.
Discovering a template does not answer OQ7; looking up a remembered
item's possible candidates does not answer OQ4. In particular, the three
gaps about origin, legacy knowledge and market value survive the appendix,
as do unverified flags, total coverage and AQ1's per-tab answer.

## 6. Cold start: help and tool descriptions only

The tool description itself shows the Query v1 skeleton, basic Boolean
operators, default scope resolution, decision fields, and the discovery
call. No one must call an unrelated command to learn that discovery
exists. `acq --help` locates `items search`; `acq items search --help`
gives that skeleton and names the discovery command. Neither supplies
the account's realm/league vocabulary: that needs a discovery read
(the cold-start need is S181). Help documents `--scope account` and
`--fields` for this use of the existing scoped discovery operation.
Calls below count every invocation and identify data reads separately;
ordinary user clarification is identified separately. They assume an already selected, populated account.
If several accounts need selection, that is one additional account read.

**OQ1, terminal stranger.** The original question lacks a slot and build
deficits; no truthful search can invent them. Suppose the person supplies
“Standard, a rare ring, 40 fire resistance or 30 dexterity.”

1. `acq --help` locates the search verb.
2. `acq items search --help` supplies the query and discovery shapes.
3. `acq items search describe --scope account --fields realm,league`
   resolves `pc` and verifies `Standard` before narrowing the scope.
4. `acq items search describe --realm pc --league Standard --terms 'ring,fire resistance,dexterity'`
   returns the Ring enum predicate, the rare enum, both recipe predicates
   with definitions/coverage, and a complete example of combining them.
   This is one batched discovery read, with no item row pulled.
5. Run OQ1's query. The decision view includes location, requested totals,
   matching lines and completeness; exact totals and unknown counts
   make the answer usable without `show` for each candidate.

**Five terminal invocations: two help calls and three search reads.** A deliberate candidate-audit read adds a sixth
invocation; it is not required to discover what the result means.
If the named total has unknown contributors, the answer is a bounded
candidate set plus a stated gap; more tool calls do not magically turn
unknown into false. This count reaches an answer envelope, not a
guaranteed resolution of OQ1; falling back to line tests would expose
the everyday-search gap rather than close it.

**AQ5, MCP stranger.** The tool list says “Use `search_describe` to get
typed fields and line predicates; `search_items` accepts Query v1;
line numbers are signed, positional, and never averaged.”

1. `search_describe` with scope `account` and terms `maximum Life` returns
   `# to maximum Life`, its one numeric slot, source/flag alternatives,
   a verbatim example, and the exact predicate fragment. It also gives
   the aggregate/order expression shape for that slot.
2. `search_items` submits the AQ5 query object, requesting compact rows,
   exact total and descending maximum matching line value.

**Two calls, no sample-body detour.** Selecting a particular source would
be a local edit of the provided fragment. Refining the result is one
further call with its returned query plus the new condition, without
re-describing the schema. AQ1's broader location facets would be one
call from the tool description alone: its field names and the facets shape appear there. Its exact
per-tab answer remains the two-query gap recorded in §5. Descriptions
must be held to these concrete cold-start walks in validation, rather
than declaring victory because a trained agent eventually succeeds.

The agent benefit I would defend is **accumulating an explicit question,
not accumulating a local copy of a stash**. Each result is enough to
refine, reproduce, explain or hand to a person. A later interface can
render that same question without inheriting my conversation history.
This is my design judgment, not a finding measured by the tracks.

## 7. Concept inventory and deletion test

This inventory includes costs hidden from the everyday text syntax.

| Concept | Why retain it after trying to remove it? |
| --- | --- |
| Query document | Removing it makes refinement and saved intent depend on adapter history. `refine` is its local predicate-append operation, not a server handle. |
| Explicit scope | Removing it makes “none” ambiguous across accounts, realms, locations and removed items. |
| Typed field, including derived field | Removing it leaves string comparisons and invisible units/absence rules. Recipes are definitions of fields, not a new query mode. This includes typed raw-body access via `fact["API.path"]`; manual, observed and effective listing fields with the `priced` alias; and named locations with socket parents/parent chains. Without those distinctions raw facts, pricing causes and child locations become inaccessible or misleading. |
| Repeated occurrence and same-occurrence condition | Removing it merges duplicate lines or lets two unrelated lines jointly satisfy one range. Collections also include `socket_groups` with `red`/`blue` fields: removing their shared binding loses OQ3's linked-colour condition. |
| Template plus ordered captures | Removing it forces literal text searches for precise rolls; preserving separate captures obeys S68. The line family combines realm, template, source and flags; removing that grouping merges distinct vocabulary choices. Optional reviewed slot labels explain positional meanings without changing the captures. |
| Source and flags | Removing them loses fractured/crafted distinctions; they are field values, not separate engines. |
| Boolean composition, including at-least-N | Removing it fails ordinary alternatives and the trade floor. At-least-N could expand to Boolean combinations, but that exports needless work to the caller. |
| Scalar arithmetic and aggregation | Removing it hard-codes every weighted or computed question into the application. Conditional `if(P,E,E)` chooses one contribution without requiring a new named recipe. |
| Selection, order and summaries | Removing them makes the caller download and sort/count bodies; facets are independent count summaries. |
| Unknown/absent with reasons | Removing them makes missing data masquerade as zero or a failed predicate. |
| Vocabulary descriptors with predicate fragments | Removing them recreates the sample-row/schema-discovery loop. Optional semantic aliases carry candidates and provenance; without those, an offered alias would conceal ambiguity. |
| Evidence/inspection | Removing it makes a match or empty answer uncheckable; it is an answer view, not a second query model. |
| Basis and continuation | Removing them lets pagination mix different stashes without saying so. Snapshot-establishment retry and `snapshot_busy` expose failure to obtain the consistent basis. Bundled reviewed reference extracts and their version belong to that basis; omitting it conceals changes to class/recipe meanings. |
| Saved query and semantic version | Removing persistence loses user intent; removing the version silently changes that intent after upgrades. No saved-result object. |
| Translation remainder | Removing it makes partial trade compatibility claim equivalence it cannot establish. Peripheral to the core, but indispensable if the bridge ships. |
| Published SQL view | Removing it violates note 22's settled SQL floor, which proposes amending C48; using the internal schema instead violates C48's retained privacy boundary. |

The twelve do not exercise at-least-N, general arithmetic, `summarize`,
continuation, saved-query versions, translation remainders or SQL, nor
do they require repeated occurrences or a second numeric slot (OQ2's
conditional two-slot illustration is not observed evidence). Their
justifications above rest on the broader floor, S27/S68 and design
judgment; these acceptance examples do not validate their necessity.

This is not a four-concept design disguised by grouping headings. The
largest learning cost is collection aggregation; **it is the first
concept I would delete if forced**, but then weighted questions and
arbitrary line combinations become application features or listed gaps.
Named recipes keep ordinary searches short without foreclosing those
questions. No query handles, sessions, macros, user-defined executable
functions, semantic mod-id spine, query planner DSL or search daemon.

## 8. What this design refuses

- An app-maintained account of what the game can currently generate,
  automatic legacy classification, build simulation or market valuation
  (S164, S178, S180).
- Fuzzy execution, implicit typo correction, a missing number treated
  as zero, a line's several numbers averaged by default, or a successful
  partial trade import masquerading as the original query.
- Runtime access to governed sources, hidden generating-mod recovery,
  item movement or shop publishing.
- A speed promise from synthetic cloned corpora (S141–S142), an always-on
  corpus service, and a warm array reused without a current basis check.
- A second semantic implementation in MCP, the CLI, SQL projection or
  the eventual webview. SQL has its own syntax and null behavior, not
  its own definitions of item values.

Three consequences are worth accepting explicitly. First, conservative
totals may be less convenient than an optimistic sum; they are inspectable
and fixable through reviewed recipe coverage rather than silent misses.
Second, a saved literal question follows the observed words, not presumed
game identity; old-wording survival is an honest boundary, not something
an update agent can solve by intuition. Third, maintaining the projection
transactionally spends work at ingest so every interface can rely on
one freshness rule. All three choices follow from wanting to trust an
empty answer a year after this discussion has been forgotten.

## 9. Decisions left to the owner

I am not leaving grammar, engine, identity or unknown semantics for the
owner to choose: the sections above make my choices. Acceptance would
need the normal boundary rulings for the store search read surface,
query/SQL contract and saved-search intent; this file grants none.

The owner alone supplies game knowledge in a particular legacy/build
question and decides whether to fund the separate trade bridge now or
later. My recommendation is **core query, discovery, evidence and saved
queries first; trade translation later**, with the remainder contract
already designed. Reference rows still need the human review C68 requires;
that is not a request to relax C79 or fetch new material during this run.
Whether conservative computed totals are tolerable in actual daily use
needs both seats' verdict, not a speculative relaxation here. I would
test known positives and negatives alongside the number of unknown
candidates, particularly after a patch, before defending those totals
as an everyday shortcut.

I left out an exhaustive field/recipe catalogue, detailed SQL DDL,
parser error productions, GUI layouts, complete trade-group translations,
and measured cold/reload timings for this larger derivation. None was
measured by this run; no undigested finding is used. **The one cut I
most want reversed is a worked coverage trial of resistance totals on
real line families**, recording exactly which unknown lines prevent a
complete total. It could show that my most important honesty rule makes
my most useful shortcut too weak, and that is the first thing I would
want to learn before implementation hardens it.
