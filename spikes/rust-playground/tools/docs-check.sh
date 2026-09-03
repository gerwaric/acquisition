#!/usr/bin/env bash
# docs-check.sh — the documentation half of the quality gate.
#
# Two checks, both mechanical (brainstorming-notes/09, "the ladder": a
# lint where mechanical, a recorded property where stakes are real):
#
#   1. Byte budgets on the always-loaded documents. Every session reads
#      these before acting; growth past the budget is the signal that a
#      narrative landed where a ruling belongs (AGENTS.md, "Routing").
#      Moving text to its home is compliance, not gaming. Past 90% the
#      check says so without failing, so routing happens at a session
#      close and never as a side quest in the middle of a slice.
#   2. Stale identifiers. A backticked code identifier in a control
#      document that no longer exists in the workspace is a parallel
#      description that has rotted. Checked: `Type::path` items,
#      CamelCase types, snake_case names with an underscore, ACQ_* knobs,
#      and *.rs / *.sh / *.py / *.sql file names.
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
budget README.md       30000
budget LIVE-TESTING.md 60000

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
cited=$(grep -rhoE '\bC[0-9]+\b' crates tools README.md LIVE-TESTING.md TESTING-NOTES.md REFRESH-SLICE.md AGENTS.md .claude 2>/dev/null \
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

exit $fail
