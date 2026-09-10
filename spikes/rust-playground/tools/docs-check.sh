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
#      cross, read from `cargo metadata` — the split's edge table
#      (brainstorming-notes/18 §2.1; C1 as amended) for every crate that
#      exists: the protocol crate's purity (serde only), the store's
#      blindness to everything above it, the planner's and the client's
#      independence from the daemon, the daemon's from the client, the
#      planner and any frontend, and the daemon named by no package but
#      itself — each forbidden edge refused at any depth.
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
  | grep -vE '^ACQ_(CONTRACT_REVISION|UPDATE_FIXTURES)$')
readme_knobs=$(grep -oE 'ACQ_[A-Z_]+' README.md | sort -u)
missing_knobs=$(comm -23 <(printf '%s\n' "$code_knobs") <(printf '%s\n' "$readme_knobs") | tr '\n' ' ')
if [[ -n $missing_knobs ]]; then
  printf 'KNOB    %-18s read in the crates, no row in the knob table: %s\n' README.md "$missing_knobs"
  fail=1
else
  echo "ok      knobs         every ACQ_* the crates read has a README row"
fi

# ---- 5. dependency direction ------------------------------------------
# The four-layer rule (C34), the planner's home (C39) and the daemon as
# its own artifact (C1, C13, C82) are enforced by what links what, not by
# discipline: the daemon crate (`acquisition-daemon`, binary `acqd`)
# links the protocol crate and the store — never the client crate, the
# planner or a frontend — and never names the intent API (it writes facts
# through the store and reads nothing else); no package but the daemon
# names the daemon, so a frontend cannot be in-process with it (C13) and
# the client crate cannot embed it; the client crate links the protocol
# crate and tokio (the store from step 5), never the daemon or the
# planner, so a client or locator edit recompiles no daemon; the store
# crate links none of them and no HTTP client, so a store read cannot
# initiate traffic (C41); the protocol crate — the daemon's contract as a
# frontend sees it — links serde and serde_json, nothing else (sha2 at
# build time only), because a dependency there is every frontend's and
# the daemon's both, and the revision it computes must move only when
# the contract does (§2.1); the planner links the protocol crate and the
# store, never the daemon or the client (§2.1: every path it took from
# core was the wire's or the vocabulary's, so a planner that links the
# daemon again has grown a third door).
#
# One graph, from Cargo: the `resolve` section of `cargo metadata
# --all-features` — every package Cargo resolves for this workspace with
# every member feature on and no platform filter, so every optional edge
# a member can activate is present, and an optional edge nothing can
# activate is absent because no build can link it either. The edges are
# the union over targets, on purpose: a target predicate is not followed
# along a path, so `plan → helper` under `cfg(unix)` with `helper → daemon`
# under `cfg(windows)` is refused although no single target links the
# two — the boundary is what the manifests declare, not what one
# platform happens to build (review 2026-09-10, accepted as the
# conservative reading). Vertices are package ids, never names: the same
# name at two versions is two vertices, and a name is used only to state
# a rule and to print. Edges carry their kinds; the closure walks normal
# and build edges at every depth and dev edges from the root only, since
# Cargo links no one else's dev-dependencies. Membership is
# `.workspace_members`, never a guess from the package's source.
#
# Never the manifest's text (review 2026-09-10: a lexical parser missed
# three valid forms); never `cargo tree` (the host's active closure under
# default features missed a `cfg(windows)` edge and an optional one);
# never the declared table of `packages[].dependencies` (an inactive
# optional dependency is declared but has no package entry, so its own
# edges are invisible). Six review rounds on this check found the same
# shape each time: a reader whose empty or partial answer satisfied
# every rule. So the graph is read twice, by two jq programs whose exit
# status is checked directly and whose answers must agree row by row.
# The first proves the graph whole — every member is a node, every edge
# lands on a node, every node has a package — and names, by a set
# fixpoint of its own, every fact the table must hold, by package id,
# sorted. The second walks the graph and emits the table in that order.
# Bash compares the two by identity and by name with builtins alone —
# no awk, grep or comm in a process substitution can answer for it — so
# a missing row, a duplicate in its place, a fabricated row, an extra
# one, or a name that does not belong to its id all refuse; then every
# package the rules are about must have its own row with every direct
# edge inside its closure. The rules read the names so authenticated;
# the path beside a row is printed on a hit and never read. What remains
# is a reader that forges a whole table, its identities and its names to
# match: a consistent lie, accepted. `tools/docs-check-breakers.sh`
# stages every case above and requires the refusal that names it; run
# it whenever an edge is added here.
meta=$(mktemp)
if ! cargo metadata --format-version 1 --all-features --offline >"$meta" 2>/dev/null \
   && ! cargo metadata --format-version 1 --all-features >"$meta"; then
  echo 'EDGE    cargo metadata failed — the dependency graph could not be read'
  rm -f "$meta"; exit 1
fi
# The graph must be whole before it is read, and the first reader states
# what the table must contain: after `whole`, one line per fact, by
# package id, sorted, each carrying the names the rules will read —
# `direct <member id> <kind> <dep id> <member> <name>` and `closure
# <member id> <dep id> <member> <name>` — from a set fixpoint and an
# id-to-name map of its own.
if ! partial=$(jq -r '
    ([.resolve.nodes[]?.id] | unique) as $nodes
    | ([.packages[].id] | unique) as $pkgs
    | ([.resolve.nodes[]? | {key: .id, value: .deps}] | from_entries) as $deps
    | ([.packages[] | {key: .id, value: .name}] | from_entries) as $name
    | def reach($m): reduce range(0; 10000) as $_ (
        { seen: [$m], front: ([$deps[$m][] | .pkg] | unique) };
        if .front == [] then . else
          (.seen + .front) as $seen
          | .front = (([ .front[] | $deps[.][] | select(any(.dep_kinds[]; .kind != "dev")) | .pkg ] | unique) - $seen)
          | .seen = $seen
        end) | .seen;
    if (.workspace_members | length) == 0 then "no workspace member"
      elif ($nodes | length) == 0 then "no resolve graph"
      elif any(.workspace_members[]; . as $m | ($nodes | index($m)) == null) then "a workspace member is not a node"
      elif any(.resolve.nodes[].deps[].pkg; . as $d | ($nodes | index($d)) == null) then "an edge lands on no node"
      elif any($nodes[]; . as $n | ($pkgs | index($n)) == null) then "a node has no package entry"
      else "whole",
        ([.workspace_members[] as $m | $deps[$m][] | .pkg as $d | .dep_kinds[] | [$m, (.kind // "normal"), $d]] | sort | .[] | "direct\t\(.[0])\t\(.[1])\t\(.[2])\t\($name[.[0]])\t\($name[.[2]])"),
        ([.workspace_members[] as $m | reach($m)[] | [$m, .]] | sort | .[] | "closure\t\(.[0])\t\(.[1])\t\($name[.[0]])\t\($name[.[1]])")
      end' "$meta"); then
  echo 'EDGE    jq failed or is not installed — the dependency graph could not be read'
  rm -f "$meta"; exit 1
fi
if [[ $partial != "whole"* ]]; then
  printf 'EDGE    the metadata is partial (%s) — the dependency graph could not be read\n' "$partial"
  rm -f "$meta"; exit 1
fi
# The table, one tab-separated line per fact, in the first reader's order
# (direct rows by member id, kind, dep id; closure rows by member id,
# dep id); ids and names are compared, the path is diagnostic only:
#   direct  <member> <kind> <name> <member id> <dep id>
#   closure <member> <name> <member id> <dep id> <path by names>
if ! table=$(jq -r '
    ([.packages[] | {key: .id, value: .name}] | from_entries) as $name
    | ([.resolve.nodes[] | {key: .id, value: .deps}] | from_entries) as $deps
    | ([.workspace_members[] as $m | $deps[$m][] | .pkg as $d | .dep_kinds[] | [$m, (.kind // "normal"), $d]]
       | sort | .[] | "direct\t\($name[.[0]])\t\(.[1])\t\($name[.[2]])\t\(.[0])\t\(.[2])"),
      ([.workspace_members[] as $m
        | { seen: {($m): $name[$m]}, queue: [ $deps[$m][] | .pkg ], from: {} }
        | .from = ([ $deps[$m][] | {key: .pkg, value: $m} ] | from_entries)
        | until(.queue == [];
            .queue[0] as $x | .queue |= .[1:]
            | if .seen[$x] then . else
                .seen[$x] = (.seen[.from[$x]] + " → " + $name[$x])
                | reduce ($deps[$x][] | select(any(.dep_kinds[]; .kind != "dev")) | .pkg) as $y
                    (.; if .seen[$y] or .from[$y] then . else .from[$y] = $x | .queue += [$y] end)
              end)
        | .seen | to_entries[] | [$m, .key, .value]]
       | sort | .[] | "closure\t\($name[.[0]])\t\($name[.[1]])\t\(.[0])\t\(.[1])\t\(.[2])")' "$meta"); then
  echo 'EDGE    jq failed reading the dependency graph'
  rm -f "$meta"; exit 1
fi
rm -f "$meta"
# The two answers must agree row by row, by identity and by name: the
# table's rows, projected to (member id, kind, dep id, member, name) and
# (member id, dep id, member, name), must be exactly the first reader's
# lines in order — a missing row, a duplicate in its place, a fabricated
# row, an extra one, or a row whose name does not belong to its id all
# refuse here, before any rule is consulted (review 2026-09-10: matching
# counts had passed a duplicate standing in for the dropped row; matching
# ids had passed a renamed dependency the rules then could not find).
# The path column is not compared: it is printed on a hit, never read by
# a rule. Builtins only.
expect=()
IFS=$'\n' read -d '' -ra expect <<<"${partial#whole}" || true
actual=()
while IFS=$'\t' read -r kind a b c d e _; do
  case $kind in
    direct)  actual+=("direct	$d	$b	$e	$a	$c") ;;
    closure) actual+=("closure	$c	$d	$a	$b") ;;
    '')      ;;
    *)       printf 'EDGE    the dependency table has a row of unknown kind (%s) — the table is not the graph\n' "$kind"; exit 1 ;;
  esac
done <<<"$table"
if ((${#expect[@]} == 0)); then
  echo 'EDGE    the first reader named no fact — the graph was not read'; exit 1
fi
if ((${#actual[@]} != ${#expect[@]})); then
  printf 'EDGE    the dependency table has %d rows, the graph has %d — the table is partial\n' "${#actual[@]}" "${#expect[@]}"; exit 1
fi
for ((i = 0; i < ${#expect[@]}; i++)); do
  if [[ ${actual[i]} != "${expect[i]}" ]]; then
    printf 'EDGE    the dependency table differs from the graph at row %d: the graph has "%s", the table "%s" — the table is not the graph\n' \
      "$((i + 1))" "${expect[i]//	/ }" "${actual[i]//	/ }"; exit 1
  fi
done
table=$'\n'"$table"$'\n'
has_row() { [[ $table == *$'\n'"$1"* ]]; }   # has_row <line prefix>: builtins only
named() {  # named <package>: its own rows must be there — itself in its closure, every direct edge in it
  local pkg=$1 kind name
  if ! has_row "closure	$pkg	$pkg	"; then
    printf 'EDGE    the dependency table has no row for %s — the graph was not read\n' "$pkg"
    exit 1
  fi
  while IFS=$'\t' read -r _ _ kind name _; do
    if ! has_row "closure	$pkg	$name	"; then
      printf 'EDGE    %s links %s directly but its closure does not contain it — the graph was not read\n' "$pkg" "$name"
      exit 1
    fi
  done < <(direct_rows "$pkg")
}
direct_rows() {  # direct_rows <package>: the direct rows, builtins only
  local line
  while IFS= read -r line; do
    [[ $line == "direct	$1	"* ]] && printf '%s\n' "$line"
  done <<<"$table"
}
path_to() {  # path_to <package> <name>: how the package reaches the name
  local line k m n mi di path
  while IFS=$'\t' read -r k m n mi di path; do
    [[ $k == closure && $m == "$1" && $n == "$2" ]] && { printf '%s\n' "$path"; return; }
  done <<<"$table"
}
edge_bad=0
forbid() {  # forbid <package> <why> <name>...: refuse any of these names, in any section, at any depth
  local pkg=$1 why=$2; shift 2
  local name direct='' deep='' d
  named "$pkg"
  for name in "$@"; do
    while IFS=$'\t' read -r _ _ _ d _; do [[ $d == "$name" ]] && direct+="$name " && break; done < <(direct_rows "$pkg")
    [[ " $direct" == *" $name "* ]] && continue
    has_row "closure	$pkg	$name	" && deep+="$name "
  done
  if [[ -n $direct ]]; then
    printf 'EDGE    %-22s links %s— %s\n' "$pkg" "$direct" "$why"
    fail=1; edge_bad=1
  fi
  if [[ -n $deep ]]; then
    printf 'EDGE    %-22s links %stransitively — %s\n' "$pkg" "$deep" "$why"
    for name in $deep; do printf '          %s\n' "$(path_to "$pkg" "$name")"; done
    fail=1; edge_bad=1
  fi
}
allow() {  # allow <package> <kind> <name>...: refuse any other direct dependency of that kind
  local pkg=$1 kind=$2; shift 2
  local k name extra='' ok a
  named "$pkg"
  while IFS=$'\t' read -r _ _ k name _; do
    [[ $k == "$kind" ]] || continue
    ok=0; for a in "$@"; do [[ $a == "$name" ]] && ok=1; done
    ((ok)) || extra+="$name "
  done < <(direct_rows "$pkg")
  if [[ -n $extra ]]; then
    printf 'EDGE    %-22s [%s] links %s— the protocol crate is serde-only (§2.1)\n' "$pkg" "$kind" "$extra"
    fail=1; edge_bad=1
  fi
}
only_self() {  # only_self <package> <why>: no other member reaches it, at any depth
  local pkg=$1 why=$2 k m n mi di path bad='' seen=' '
  named "$pkg"
  while IFS=$'\t' read -r k m n mi di path; do
    [[ $k == closure && $n == "$pkg" && $m != "$pkg" && $seen != *" $m "* ]] && { bad+="$m "; seen+="$m "; }
  done <<<"$table"
  if [[ -n $bad ]]; then
    printf 'EDGE    %-22s is linked by %s— %s\n' "$pkg" "$bad" "$why"
    for m in $bad; do printf '          %s\n' "$(path_to "$m" "$pkg")"; done
    fail=1; edge_bad=1
  fi
}
# The frontends are named here; a new one (the GUI, a queue TUI) is added
# to the daemon's list when its package exists — the reverse direction,
# that nothing links the daemon, needs no list.
forbid acquisition-daemon 'the daemon links protocol and store, never the client crate, the planner or a frontend (C1, C39)' \
  acquisition-client acquisition-plan acquisition-cli acquisition-mcp
only_self acquisition-daemon 'no package but acquisition-daemon names the daemon — a frontend is never in-process with it, and the client never embeds it (C1, C13, C82)'
forbid acquisition-client 'the client links protocol, store and tokio, never the daemon or the planner (C1, §2.1)' \
  acquisition-daemon acquisition-plan
forbid acquisition-plan  'the planner links protocol and store, never the daemon or the client (C39, §2.1)' \
  acquisition-daemon acquisition-client
forbid acquisition-store 'the store links no daemon, no client, no protocol and no HTTP client (C41; the split, §2.1)' \
  acquisition-daemon acquisition-client acquisition-protocol acquisition-plan reqwest tokio
allow acquisition-protocol normal serde serde_json
allow acquisition-protocol dev    serde serde_json
allow acquisition-protocol build  sha2
if grep -rqE 'Annotations|annotations_path' crates/acquisition-daemon/src crates/acquisition-protocol/src crates/acquisition-client/src; then
  echo 'EDGE    crates/acquisition-{daemon,protocol,client}/src    names the intent API — the daemon, the wire and the client are permanently blind to intent (C34)'
  fail=1; edge_bad=1
fi
if ((edge_bad == 0)); then
  echo 'ok      dependencies  daemon ∌ client/planner/frontend, nothing ∌ daemon but itself, client ∌ daemon/planner, planner ∌ daemon/client, store ∌ daemon/client/protocol/HTTP, daemon/protocol/client ∌ intent API, protocol = serde only (C1, C34, C39, C41, §2.1)'
fi

exit $fail
