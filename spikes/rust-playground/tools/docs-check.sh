#!/usr/bin/env bash
# docs-check.sh — the documentation half of the quality gate.
#
# Five checks, all mechanical (P5, CONTEXT.md "Working style": a lint
# where mechanical, a recorded property where stakes are real):
#
#   1. Byte budgets on the always-loaded documents. Every session reads
#      these before acting; growth past the budget is the signal that a
#      narrative landed where a ruling belongs (AGENTS.md, "Routing").
#      Moving text to its home is compliance, not gaming. Past 90% the
#      check says so without failing, so routing happens at a session
#      close and never as a side quest in the middle of a slice.
#   2. The decision registry: one bullet per decision under a length
#      limit, a capped count of always-loaded ones, every cited id real,
#      the uncited ones reported.
#   3. Stale identifiers. A backticked code identifier in a control
#      document that no longer exists in the workspace is a parallel
#      description that has rotted. Checked: `Type::path` items,
#      CamelCase types, snake_case names with an underscore, ACQ_* knobs,
#      and *.rs / *.sh / *.py / *.sql file names.
#   4. The README's form: one line per verb in the tour, a row per knob.
#   5. Dependency direction: the layer rules as edges the crates cannot
#      cross, read from `cargo metadata` — the daemon and the store as
#      before, and since the daemon split's step 1 the protocol crate's
#      purity (serde only) and the store's blindness to it; the rest of
#      the split's edge table (brainstorming-notes/18 §2.1) lands with
#      the crates it names.
#
# Exit 1 on any failure; the report names each offender.
set -euo pipefail
cd "$(dirname "$0")/.."

fail=0

# ---- 1. budgets (bytes) ---------------------------------------------------
budget() {
  local file=$1 limit=$2 size
  size=$(wc -c <"$file" | tr -d ' ')
  if ((size > limit)); then
    printf 'BUDGET  %-18s %7d > %7d bytes\n' "$file" "$size" "$limit"
    fail=1
  elif ((size * 10 > limit * 9)); then
    printf 'near    %-18s %7d / %7d bytes — over 90%%: route at session close, before a slice trips it\n' "$file" "$size" "$limit"
  else
    printf 'ok      %-18s %7d / %7d bytes\n' "$file" "$size" "$limit"
  fi
}
budget AGENTS.md        8000
budget CONTEXT.md      20000
budget README.md       15000
budget LIVE-TESTING.md 15000
# RUN-LEDGER.md has no budget: one row per live run, append-only by
# construction, read by its tail. Its rows cite decision ids (scanned
# below) but are history, so the stale-identifier scan skips it.

# ---- 2. the decision registry ------------------------------------------
# Every decision is one bullet under a length limit (a narrative cannot fit,
# so the mechanism goes to the code); every `C<n>` cited anywhere exists in
# the registry; a decision nothing cites is reported (enforced by nothing is
# either a lint, a test, or a smell).
ENTRY_LIMIT=800
reg=$(mktemp)
# entries are single lines in the registry (one bullet, no continuation),
# spread over CONTEXT.md (cross-cutting only) and decisions/*.md (per area)
grep -hE '^- \*\*C[0-9]+ ' CONTEXT.md decisions/*.md >"$reg"
dups=$(grep -oE '^- \*\*C[0-9]+' "$reg" | sort | uniq -d | sed 's/^- \*\*//' | tr '\n' ' ')
if [[ -n $dups ]]; then echo "DUPLICATE decision id across registry files: $dups"; fail=1; fi
CROSS_LIMIT=15
cross=$(grep -cE '^- \*\*C[0-9]+ ' CONTEXT.md)
if ((cross > CROSS_LIMIT)); then
  printf 'CROSS   %-18s %5d always-loaded decisions > %d — move area rulings to decisions/<area>.md\n' CONTEXT.md "$cross" "$CROSS_LIMIT"; fail=1
else
  printf 'ok      %-18s %5d always-loaded decisions (limit %d)\n' CONTEXT.md "$cross" "$CROSS_LIMIT"
fi
over=0
while IFS= read -r line; do
  n=$(printf '%s' "$line" | wc -c | tr -d ' ')
  if ((n > ENTRY_LIMIT)); then
    printf 'ENTRY   %-18s %5d > %d bytes  %s\n' CONTEXT.md "$n" "$ENTRY_LIMIT" "$(printf '%s' "$line" | cut -c1-40)"
    over=$((over+1))
  fi
done <"$reg"
ids=$(grep -oE '^- \*\*C[0-9]+' "$reg" | sed 's/^- \*\*//' | sort -u)
count=$(printf '%s\n' "$ids" | grep -c .)
if ((over > 0)); then fail=1; else printf 'ok      %-18s %5d decisions, every entry within %d bytes\n' registry "$count" "$ENTRY_LIMIT"; fi
cited=$(grep -rhoE '\bC[0-9]+\b' crates tools README.md LIVE-TESTING.md RUN-LEDGER.md TESTING-NOTES.md REFRESH-SLICE.md AGENTS.md .claude 2>/dev/null \
  --include='*.rs' --include='*.sh' --include='*.py' --include='*.md' | sort -u)
unknown=$(comm -13 <(printf '%s\n' "$ids") <(printf '%s\n' "$cited") | grep . || true)
if [[ -n $unknown ]]; then
  printf 'UNKNOWN decision id cited outside the registry: %s\n' "$unknown" | tr '\n' ' '; echo
  fail=1
fi
uncited=$(comm -23 <(printf '%s\n' "$ids") <(printf '%s\n' "$cited") | tr '\n' ' ')
[[ -n $uncited ]] && printf 'note    uncited decisions (no test, doc, or tool names them): %s\n' "$uncited"
rm -f "$reg"

# ---- 3. stale identifiers -----------------------------------------------
# The haystack is the code and its schemas; docs never vouch for docs.
hay=$(mktemp)
trap 'rm -f "$hay"' EXIT
find crates tools -type f \( -name '*.rs' -o -name '*.sh' -o -name '*.py' -o -name '*.sql' -o -name '*.toml' \) \
  -not -path '*/target/*' -print0 | xargs -0 cat >"$hay"
ls -R crates tools >>"$hay"

missing=0
for doc in AGENTS.md CONTEXT.md README.md LIVE-TESTING.md; do
  # backticked tokens without spaces, then the shapes worth checking
  grep -o '`[^` ]\{3,\}`' "$doc" | tr -d '`' | sort -u | while read -r tok; do
    case "$tok" in
      *::*)                      needle=${tok##*::} ;;                # Store::record → record
      *.rs|*.sh|*.py|*.sql)      needle=$(basename "$tok") ;;
      ACQ_*)                     needle=${tok%%=*} ;;
      *_*)                       [[ $tok =~ ^[A-Za-z_][A-Za-z0-9_]*$ ]] || continue; needle=$tok ;;
      *)                         [[ $tok =~ ^[A-Z][a-z]+[A-Z][A-Za-z0-9]*$ ]] || continue; needle=$tok ;;
    esac
    needle=${needle%\(\)}
    # observed GGG data values quoted in rulings, not code
    case "$needle" in BodyArmour|DefaultAttackSkills|SkillSlots) continue ;; esac
    [[ -n $needle ]] || continue
    if ! grep -qF -- "$needle" "$hay"; then
      printf 'STALE   %-18s `%s`\n' "$doc" "$tok"
      echo x >>"$hay.miss"
    fi
  done
done
if [[ -f "$hay.miss" ]]; then
  missing=$(wc -l <"$hay.miss" | tr -d ' ')
  rm -f "$hay.miss"
fi
if ((missing > 0)); then
  echo "$missing stale identifier(s)"
  fail=1
else
  echo "ok      identifiers   every checked identifier exists in the workspace"
fi

# ---- 4. the README's form -----------------------------------------------
# The README is the index to what exists and how to reach it
# (brainstorming-notes/13): the tour is one line per verb — a
# comment-only line is a group header after a blank line, never a
# continuation of a verb line (44 of those had accreted by 2026-09-07) —
# and the knob table is complete: every ACQ_* the crates read outside
# tests has a row. The tour's verbs and flags are pinned by
# acquisition-cli/tests/readme_tour.rs against the binary.
tour_bad=$(awk '/^```sh/{f=!f; next} f{ if ($0 ~ /^[[:space:]]*#/ && prev !~ /^[[:space:]]*$/) print "        line "NR": "$0; prev=$0 }' README.md)
if [[ -n $tour_bad ]]; then
  printf 'TOUR    %-18s a comment line continues a verb line — one line per verb, the rest is its --help:\n%s\n' README.md "$tour_bad"
  fail=1
else
  echo "ok      tour          one line per verb"
fi
code_knobs=$(grep -rhoE '"ACQ_[A-Z_]+"' crates --include='*.rs' --exclude-dir=tests | tr -d '"' | sort -u \
  | grep -vE '^ACQ_(RUNTIME_REVISION|UPDATE_FIXTURES)$')
readme_knobs=$(grep -oE 'ACQ_[A-Z_]+' README.md | sort -u)
missing_knobs=$(comm -23 <(printf '%s\n' "$code_knobs") <(printf '%s\n' "$readme_knobs") | tr '\n' ' ')
if [[ -n $missing_knobs ]]; then
  printf 'KNOB    %-18s read in the crates, no row in the knob table: %s\n' README.md "$missing_knobs"
  fail=1
else
  echo "ok      knobs         every ACQ_* the crates read has a README row"
fi

# ---- 5. dependency direction ------------------------------------------
# The four-layer rule (C34) and the planner's home (C39) are enforced by
# what links what, not by discipline: the daemon crate never links the
# planner and never names the intent API (it writes facts through the
# store and reads nothing else); the store crate links neither the daemon
# nor an HTTP client, so a store read cannot initiate traffic (C41); the
# protocol crate — the daemon's contract as a frontend sees it — links
# serde and serde_json, nothing else (sha2 at build time only), because a
# dependency there is every frontend's and the daemon's both, and the
# revision it computes must move only when the contract does (§2.1).
# The edges are read from `cargo metadata`, i.e. from what Cargo itself
# resolves — every section, every spelling, target-specific tables and
# `[dependencies.x]` tables included — never from the manifest's text
# (review finding 2026-09-10: a lexical parser missed three valid forms).
# The metadata is read once, into a flat `package kind name` table, by a
# jq command whose exit status is checked directly — a jq inside a
# process substitution fails without failing the script (review finding
# 2026-09-10: a missing jq passed every edge with an empty set). The
# table must name every crate the edges are about, so an empty or
# partial answer refuses too instead of satisfying each allowlist
# vacuously.
meta=$(mktemp)
if ! cargo metadata --format-version 1 --no-deps --offline >"$meta" 2>/dev/null \
   && ! cargo metadata --format-version 1 --no-deps >"$meta"; then
  echo 'EDGE    cargo metadata failed — the dependency edges could not be read'
  rm -f "$meta"; exit 1
fi
if ! edges=$(jq -r '.packages[] | .name as $p | .dependencies[]
                    | "\($p) \(.kind // "normal") \(.name)"' "$meta"); then
  echo 'EDGE    jq failed or is not installed — the dependency edges could not be read'
  rm -f "$meta"; exit 1
fi
rm -f "$meta"
for pkg in acquisition-core acquisition-store acquisition-protocol; do
  if ! grep -q "^$pkg " <<<"$edges"; then
    printf 'EDGE    the dependency table names no dependency of %s — the metadata was not read\n' "$pkg"
    exit 1
  fi
done
deps_of() {  # deps_of <package> <kind: normal|dev|build>: dependency names of that kind, any target
  awk -v p="$1" -v k="$2" '$1 == p && $2 == k {print $3}' <<<"$edges" | sort -u
}
all_deps_of() { awk -v p="$1" '$1 == p {print $3}' <<<"$edges" | sort -u; }
edge_bad=0
forbid() {  # forbid <package> <why> <name>...: refuse any of these names, in any section
  local pkg=$1 why=$2; shift 2
  local hit
  hit=$(comm -12 <(all_deps_of "$pkg") <(printf '%s\n' "$@" | sort -u) | tr '\n' ' ')
  if [[ -n $hit ]]; then
    printf 'EDGE    %-22s links %s— %s\n' "$pkg" "$hit" "$why"
    fail=1; edge_bad=1
  fi
}
allow() {  # allow <package> <kind> <name>...: refuse any other dependency of that kind
  local pkg=$1 kind=$2; shift 2
  local extra
  extra=$(comm -23 <(deps_of "$pkg" "$kind") <(printf '%s\n' "$@" | sort -u) | tr '\n' ' ')
  if [[ -n $extra ]]; then
    printf 'EDGE    %-22s [%s] links %s— the protocol crate is serde-only (§2.1)\n' "$pkg" "$kind" "$extra"
    fail=1; edge_bad=1
  fi
}
forbid acquisition-core  'the daemon never links the planner (C39)' acquisition-plan
forbid acquisition-store 'the store links no daemon, no protocol and no HTTP client (C41; the split, §2.1)' \
  acquisition-core acquisition-protocol acquisition-plan reqwest tokio
allow acquisition-protocol normal serde serde_json
allow acquisition-protocol dev    serde serde_json
allow acquisition-protocol build  sha2
if grep -rqE 'Annotations|annotations_path' crates/acquisition-core/src crates/acquisition-protocol/src; then
  echo 'EDGE    crates/acquisition-{core,protocol}/src    names the intent API — the daemon and the wire are permanently blind to intent (C34)'
  fail=1; edge_bad=1
fi
if ((edge_bad == 0)); then
  echo 'ok      dependencies  daemon ∌ planner, daemon/protocol ∌ intent API, store ∌ daemon/protocol/HTTP, protocol = serde only (C34, C39, C41, §2.1)'
fi

exit $fail
