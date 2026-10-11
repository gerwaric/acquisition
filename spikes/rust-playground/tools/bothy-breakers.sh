#!/usr/bin/env bash
# bothy-breakers.sh — proves `tools/bothy.py check` refuses each fault it
# claims to: a small registry is written to a temporary tree, passes clean,
# and then each break in turn must be refused with the fault's kind named.
# Run it when a check is added to bothy.py.
#
# Decision ids are written `@n` here and become `Cn` when the fixture is
# written or an edit is applied, so a tree's citation grep over `tools/`
# (acquisition's docs-check.sh) never reads this file's ids as citations.
set -euo pipefail
ids() { sed 's/@\([0-9]\)/C\1/g'; }
here=$(cd "$(dirname "$0")" && pwd)
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
fixture() {  # a clean registry: CONTEXT.md, one area, one module doc
  rm -rf "$tmp"/*; mkdir -p "$tmp/decisions" "$tmp/crates/acquisition-store/src" "$tmp/tools"
  ids >"$tmp/CONTEXT.md" <<'CTX'
# Test — Agent Context

## Orientation (3 sentences)

A test registry.

## Invariants

1. **Nothing leaves the tree.**

## Decisions — the registry

| File | Area | Decisions |
| --- | --- | --- |
| `decisions/store.md` | Store | @2, @3 |

### Cross-cutting

- **@1 — Every rule has one home.** *Why:* two homes rot (P1). Decided 2026-01-01.

## Parked (cross-cutting only; do not build yet)

- A second frontend → its own crate. Trigger: a consumer.

## Working style

- **P1.** Structure over discipline.
CTX
  ids >"$tmp/decisions/store.md" <<'STO'
# Decisions — Store

The rulings that bind the store (`acquisition-store/src/lib.rs`). Read this before touching that code.

## Store

- **@2 — Facts are kept verbatim.** *Why:* a derivation can be redone (@1). Decided 2026-01-02.
- **@3 — The store sends nothing.** Pinned by `tests/read.rs`. *Why:* a read observes. Decided 2026-01-03.

## Parked (do not build yet; each with its trigger)

- Pruning → a verb. Trigger: the owner asks.
STO
  ids >"$tmp/crates/acquisition-store/src/lib.rs" <<'LIB'
//! The store. Rulings: `decisions/store.md`, @2, @3.
pub fn record() {}
LIB
  mkdir -p "$tmp/crates/acquisition-store/tests"; echo '// @3' >"$tmp/crates/acquisition-store/tests/read.rs"
}
run() { python3 -I "$here/bothy.py" --root "$tmp" check --no-notes 2>&1 || true; }
fixture
out=$(run)
if [[ $out != ok* ]]; then echo "the clean fixture must pass:"; echo "$out"; exit 1; fi
echo "ok      clean fixture passes"
n=0; bad=0
expect() {  # expect <kind> <python edit over the fixture>
  local kind=$1 edit=${2//@/C} out
  fixture
  python3 -I - "$tmp" <<PY
import pathlib, sys
root = pathlib.Path(sys.argv[1])
$edit
PY
  out=$(run)
  n=$((n+1))
  if [[ $out == *"FAIL  $kind"* ]]; then
    printf 'ok      %-10s refused\n' "$kind"
  else
    printf 'MISSED  %-10s not refused:\n%s\n' "$kind" "$out"; bad=1
  fi
}
expect index     'p=root/"CONTEXT.md"; p.write_text(p.read_text().replace("@2, @3", "@2"))'
expect index     'p=root/"CONTEXT.md"; p.write_text(p.read_text().replace("@2, @3", "@2, @3, @9"))'
expect index     'p=root/"CONTEXT.md"; p.write_text(p.read_text().replace("| `decisions/store.md` | Store | @2, @3 |\n", ""))'
expect duplicate 'p=root/"decisions/store.md"; p.write_text(p.read_text().replace("@3 —", "@2 —"))'
expect grammar   'p=root/"decisions/store.md"; p.write_text(p.read_text().replace("- **@3 — The store sends nothing.**", "- **@3 The store sends nothing**"))'
expect cite      'p=root/"decisions/store.md"; p.write_text(p.read_text().replace("(@1)", "(@7)"))'
expect cite      'p=root/"CONTEXT.md"; p.write_text(p.read_text().replace("(P1)", "(P4)"))'
expect cite      'p=root/"decisions/store.md"; p.write_text(p.read_text().replace("Trigger: the owner asks.", "Trigger: the owner asks (@8)."))'
expect park      'p=root/"decisions/store.md"; p.write_text(p.read_text().replace(" Trigger: the owner asks.", ""))'
expect park      'p=root/"decisions/store.md"; p.write_text(p.read_text().replace("- Pruning → a verb.", "- Pruning, a verb."))'
expect park      'p=root/"decisions/store.md"; p.write_text(p.read_text().replace("- Pruning → a verb.", "- ~~Pruning → a verb.~~"))'
expect path      'p=root/"decisions/store.md"; p.write_text(p.read_text().replace("`tests/read.rs`", "`tests/gone.rs`"))'
expect path      'p=root/"decisions/store.md"; p.write_text(p.read_text().replace("`acquisition-store/src/lib.rs`", "`acquisition-store/src/old.rs`"))'
expect routing   'p=root/"crates/acquisition-store/src/lib.rs"; p.write_text(p.read_text().replace("decisions/store.md", "decisions/stores.md"))'
# what must pass: history, an API route, a key shape, a note by number
fixture
python3 -I - "$tmp" <<'PY'
import pathlib, sys
p = pathlib.Path(sys.argv[1]) / "decisions/store.md"
p.write_text(p.read_text().replace("Read this before", "History: `search/PLAN.md` at `453a4429`; the route `/profile`; a `wanted/lot` ratio; note `brainstorming-notes/07`. Read this before"))
PY
out=$(run); n=$((n+1))
if [[ $out == ok* ]]; then echo "ok      history, a route, a key shape and a note by number are not paths"; else echo "WRONG   a non-path was refused:"; echo "$out"; bad=1; fi
echo "$n cases"
exit $bad
