# Pricing design review — convergence reassessment

## Verdict

The packet is converged in substance. I recommend **one final, narrow revision
and then acceptance**, without another general design round.

C64–C78 now form a coherent system. The revision has resolved every earlier
architectural objection: the layers are separate, uncertainty lands in
replaceable knowledge rather than intent, the stored amount is broad enough for
the experiment, the experiment precedes and is independent of production
rendering, reference-data evolution is semantic, legacy identity is
evidence-graded, and detailed import non-actions survive in the reviewed plan.

Only two clauses still cross the threshold from “development detail” into
“boundary decision”:

1. C79 must say that permission attaches to an **access method**, not merely to
   a registered surface or to being off the runtime path.
2. C73 must not treat lack of overlap as proof that the source belongs to a
   different account.

Make those two edits, harvest the packet, and let the slice learn the rest.

## What is now ready to accept

The packet reads as one stacked design rather than a collection of pricing
features:

- **Facts** retain game observations verbatim and remain re-derivable under new
  parsers.
- **Intent** stores only explicit manual assertions, with typed values,
  revisions, provenance, and receipts.
- **Reference data** is reviewed, enumerable, cited, and compatible by stable
  semantic identity rather than by an unnecessarily global digest lock.
- **Derivations** own inheritance, game-note interpretation, the relation
  between game and manual prices, eligibility, and rendering.
- **Effects** remain absent from this slice; manual forum use is an experiment,
  while automated publishing still requires its own boundary design.
- **Plans** preserve the shared observe → compile → review → apply loop without
  forcing refresh and pricing into one action grammar.
- **Frontends** remain adapters over the Rust semantics rather than alternate
  implementations.

This arrangement is resilient to the specific risk posed by the C++ evidence.
If the census or forum matrix disproves an inherited assumption, the affected
parser, claim, reference row, or derivation changes. The user's intent and the
plan/apply safety model do not.

The following previous concerns are fully resolved and should not be reopened
during harvest:

- separate manual and game resolutions with an explicit relation;
- typed, realm-aware targets;
- exact rational storage with emitted and accepted syntax learned separately;
- malformed-note precedence and a format-drift tripwire;
- structured writer, actor, plan, and receipt provenance;
- tombstone generations and atomic intent preconditions;
- fact drift reported across the separate files rather than falsely made
  transactional;
- a consistent logical snapshot of the WAL-backed legacy database;
- a conditional inverse compiled from an atomic receipt;
- explicit eligibility and omission reasons;
- an experimental fixture before the production renderer;
- a separate trade/forum ground-truth file;
- retaining the reviewed import plan because receipt counts intentionally do
  not preserve every non-action.

## The one material revision

### C79: govern the access method, not just the surface

The new cross-cutting boundary is justified. The API choke point does not cover
browser observations, forum experiments, or future third-party feeds, and the
system needs an explicit home for governing them.

The current wording nevertheless grants too much merely because a surface is
registered, human-run, and outside the runtime path. Those properties control
operational coupling; they do not establish permission. GGG's developer policy
says only documented APIs and data exports are supported and specifically
warns against reverse-engineering internal website endpoints. At the same
time, manually reading the public trade site and manually using a forum are
ordinary intended uses. The important distinction is therefore the method of
access, not whether “the trade site” as a whole is sanctioned:

- <https://www.pathofexile.com/developer/docs/index>
- <https://www.pathofexile.com/developer/docs/reference>
- <https://www.pathofexile.com/forum/view-thread/3444007>

I recommend replacing C79 with this boundary, or its concise equivalent:

> **C79 — External non-API surfaces are registered, permission-scoped inputs,
> never runtime dependencies.** No store read, plan compile, or apply depends
> on one. Each registration names the operator, permitted access method, terms
> exposure, and cadence; Acquisition consults it only by a method the operator
> permits. Human observations and permitted human-run tools land as claims or
> reviewed reference data, source cited. Unsupported internal endpoints are
> not automated or reverse-engineered without explicit permission. Using a
> surface as an effect requires its own boundary session first.

C68 then needs only a corresponding phrase: a tool may propose rows **when the
registered access method permits tooling**. Otherwise the evidence is gathered
by a human browser reading. A human commit is the authority for the reference
table, but human review does not retroactively permit an impermissible fetch.

This is not a pricing-model objection. It is the minimum correction needed to
make C79 protect the GGG relationship rather than accidentally authorize a way
around its API rules.

### C73: absence of overlap is not contradiction

Accept the four-state binding model, with one correction to its mechanism.
Positive evidence that a source name maps to another known UUID can establish
`contradicted`. A large source overlapping none of the current facts cannot:
the source may be old, the relevant realm may not be refreshed completely, or
the account's holdings may have changed.

Classify zero overlap as `unverified`, perhaps with a prominent warning. It then
requires the explicit acknowledgement C73 already defines. Reserve
`contradicted` for affirmative evidence of another identity. This keeps the
import conservative without converting incomplete, replaceable facts into an
identity oracle.

## Disposition of the packet

| Lines | Recommendation |
|---|---|
| C64–C67 | Accept as written. |
| C68 | Accept, with the C79 access-method qualification on tooling. |
| C69–C72 | Accept as written. |
| C73 | Accept after zero overlap is classified as `unverified`, not `contradicted`. |
| C74–C78 | Accept as written. |
| C79 | Accept after permission is made access-method-specific as above. |

The slice order is also ready. In particular, broad rational storage removes
the grammar dependency from the schema sequence, and the step-8 instrument →
claims → policy → production-render order removes the evidentiary circle. I
would not reorder it again.

## Answers to the current owner questions

1. Accept C79 as a cross-cutting line after the access-method amendment. The
   trade site can be its first registered surface with two distinct methods:
   human browser observation permitted; unsupported internal API automation
   not permitted absent explicit authorization.
2. Accept C68's evidence-cited v1 table drafted from the C++ list, census, and
   dated browser observations. Tool output is eligible only from a method C79
   records as permitted.
3. Accept the exact rational stored type and learn emitted/accepted forms from
   claims. This is the clean separation between durable meaning and volatile
   syntax.
4. Accept the binding model after changing no-overlap from `contradicted` to
   `unverified`; acknowledgement is the correct gate for uncertainty.
5. Accept the step-8 split exactly as proposed.
6. Accept mandatory pre-apply retention of import plans and receipt linkage.
   Filename layout, relative versus absolute paths, pruning, and relocation are
   implementation findings unless real use shows they affect recoverability.

## What should be left to development

Do not hold harvest for exact integer representation, digit limits, parser
types, SQLite indexes, busy-timeout tuning, receipt compaction, plan-file
directory layout, command spelling, output grouping thresholds, or the numeric
threshold for `corroborated`. The audit, census, tests, and two validation
readings exist precisely to settle those matters.

Reopen a ruling only if development produces evidence against a boundary:
target identity, durable value meaning, mutation authority, atomicity,
reference-tag identity, source permission, or the render/publish separation.
Everything else is an implementation finding. That is the stopping rule that
keeps this design from spiraling after convergence.
