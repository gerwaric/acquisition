# Pricing design review — accepted for harvest

**Disposition: accept the current design packet as written and harvest it. No
further packet revision is requested by this review.**

On 2026-09-03 the owner stated that the packet had received its final revision
and was ready for harvest. The external review concurs. The packet's candidate
lines, slice order, parking lot, and discovery method now form a coherent
system, and the deliberately open questions have evidence steps and landing
places rather than hidden assumptions.

The two findings from the prior review are resolved in the packet's current
text and are not outstanding work:

- C79 attaches permission to the **access method**. The trade site is registered
  for browser observation with no automated fetch; tooling may fetch only an
  official export or a source for which explicit permission is recorded.
- C73 classifies zero overlap with current facts as `unverified`, requiring
  acknowledgement, while reserving `contradicted` for affirmative evidence of
  another identity.

Earlier review arguments and packet revisions remain available in git history.
Future readers should not revive them from an older commit.

## Why the packet is ready

The design now composes cleanly across the stack:

- facts keep game observations verbatim;
- intent keeps only explicit manual assertions and their provenance;
- versioned reference data is a reviewed input with stable semantic identity;
- inheritance, parsing, reconciliation, eligibility, and rendering are
  reproducible derivations;
- effects remain outside this slice;
- `PricePlan` applies intent atomically through revisions and receipts;
- CLI and MCP remain adapters over shared Rust semantics.

This separation contains uncertainty in replaceable knowledge. Evidence from
the legacy census or forum matrix can change claims, reference rows, parsers,
and publication rules without migrating the user's intent or weakening the
compile → review → apply boundary. C++ remains evidence of behavior, not an
authority on the new system's shape.

The durability and safety boundaries are also explicit before the first real
price lands: typed strict values, stable targets, tombstone-aware
preconditions, logical-snapshot import, evidence-graded account binding,
atomic receipts, conditional revert, retained import plans, named omission
reasons, and permission-scoped external observations.

## Review closure

There are no remaining review findings to resolve before harvest. The amount
grammar's emitted forms, currency evidence, forum behavior, exact renderer
policy, storage performance, and the shared shape of the two Plan consumers
are intentionally learned during the slice. Those are development findings,
not omissions in the packet.

After harvest, reopen a ruling only if implementation or real use produces
evidence against a boundary: identity, durable value meaning, mutation
authority, atomicity, reference-tag identity, external-source permission, or
the render/publish separation. Mechanism choices should be resolved in code,
tests, and the slice findings register without returning to this review.
