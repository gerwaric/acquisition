#!/usr/bin/env bash
# docs-check-breakers.sh — proves the dependency guard in docs-check.sh
# fails closed, by breaking the code and the tool and watching it refuse.
#
# Six review rounds on the guard (DAEMON-SPLIT-SLICE.md, rounds 5–11)
# found the same shape each time: a reader whose empty, partial or
# renamed answer satisfied every rule. Each case here is one of those
# findings, or a route a future crate could take, and states what the
# guard must say about it — a refusal is evidence only when it names
# the property, because twice a malformed breaker printed a refusal
# that was not the property (a shell quote closed early; BSD grep
# without -P). So an edit that does not apply, or leaves the tool
# unparseable, fails the suite instead of passing as a refusal.
#
# Run it when an edge is added to docs-check.sh (the split's step 3 added
# the client and daemon crates and their edges), not in the gate: it
# runs the check once per case. Every case restores the files it touched from copies
# and compares them byte for byte — never `git checkout`, which
# discards uncommitted work (the record's observation) — and a trap
# restores on any exit. Needs python3 and perl beside bash and jq.
set -uo pipefail
cd "$(dirname "$0")/.."

scratch=$(mktemp -d "${TMPDIR:-/tmp}/acq-breakers.XXXXXX")
files=(Cargo.toml Cargo.lock crates/acquisition-plan/Cargo.toml crates/acquisition-store/Cargo.toml crates/acquisition-cli/Cargo.toml crates/acquisition-mcp/Cargo.toml crates/acquisition-client/Cargo.toml crates/acquisition-daemon/Cargo.toml tools/docs-check.sh)
for f in "${files[@]}"; do mkdir -p "$scratch/keep/$(dirname "$f")"; cp "$f" "$scratch/keep/$f"; done
restore() {
  local f
  for f in "${files[@]}"; do
    cp "$scratch/keep/$f" "$f"
    cmp -s "$scratch/keep/$f" "$f" || { echo "RESTORE FAILED: $f"; exit 2; }
  done
  rm -rf crates/acquisition-helper
  rm -f crates/acquisition-client/src/zz_breaker.rs
}
trap 'restore; rm -rf "$scratch"' EXIT

# ---- scratch crates outside the tree -----------------------------------
root=$PWD
crate() {  # crate <dir> <name> <version> <manifest tail>
  mkdir -p "$scratch/x/$1/src"; printf 'pub fn f() {}\n' >"$scratch/x/$1/src/lib.rs"
  printf '[package]\nname = "%s"\nversion = "%s"\nedition = "2024"\n%b' "$2" "$3" "$4" >"$scratch/x/$1/Cargo.toml"
}
crate bridge  external-bridge  0.1.0 "\n[dependencies]\nacquisition-daemon = { path = \"$root/crates/acquisition-daemon\" }\n"
crate dup1    duplicate-helper 0.1.0 ""
crate dup2    duplicate-helper 0.2.0 "\n[dependencies]\nacquisition-protocol = { path = \"$root/crates/acquisition-protocol\" }\n"
crate devonly devonly          0.1.0 ""
crate outside outside          0.1.0 "\n[dev-dependencies]\ndevonly = { path = \"$scratch/x/devonly\" }\n"

# ---- edits ---------------------------------------------------------------
adddep() {  # adddep <manifest> <line> [<table, default [dependencies]>]
  python3 - "$1" "$2" "${3:-[dependencies]}" <<'PY'
import pathlib, sys
p = pathlib.Path(sys.argv[1]); s = p.read_text(); a = sys.argv[3] + '\n'
if s.count(a) == 0: s += '\n' + a
assert s.count(a) == 1, sys.argv[3]
p.write_text(s.replace(a, a + sys.argv[2] + '\n', 1))
PY
}
helper() {  # helper <manifest tail>: a workspace member the planner links
  mkdir -p crates/acquisition-helper/src; printf 'pub fn helper() {}\n' >crates/acquisition-helper/src/lib.rs
  printf '[package]\nname = "acquisition-helper"\nversion.workspace = true\nedition.workspace = true\n\n%b' "$1" >crates/acquisition-helper/Cargo.toml
  python3 - <<'PY'
import pathlib
p = pathlib.Path('Cargo.toml'); s = p.read_text(); a = '    "crates/acquisition-plan",\n'; assert s.count(a) == 1
p.write_text(s.replace(a, a + '    "crates/acquisition-helper",\n', 1))
PY
}
member_links_helper() { helper "$2"; adddep "crates/$1/Cargo.toml" 'acquisition-helper = { path = "../acquisition-helper" }' "${3:-[dependencies]}"; }   # <member> <helper manifest tail> [table]
plan_links_helper() { member_links_helper acquisition-plan "$1" "${2:-[dependencies]}"; }
tool() {  # tool <python replace old> <new>: an exact-once substitution in docs-check.sh
  python3 - "$1" "$2" <<'PY'
import pathlib, sys
p = pathlib.Path('tools/docs-check.sh'); s = p.read_text()
assert s.count(sys.argv[1]) == 1, 'edit anchor not found once: ' + sys.argv[1][:60]
p.write_text(s.replace(sys.argv[1], sys.argv[2]))
PY
}
TABLE_END='\t\(.[0])\t\(.[1])\t\(.[2])")'"'"' "$meta"); then'
shadow_table() { tool "$TABLE_END" "${TABLE_END/\"\$meta\"); then/\"\$meta\" | $1); then}"; }   # a filter on the table reader's answer
META_CMD='if ! cargo metadata --format-version 1 --all-features --offline >"$meta" 2>/dev/null \
   && ! cargo metadata --format-version 1 --all-features >"$meta"; then'
shadow_meta() { tool "$META_CMD" "if ! cat \"$1\" >\"\$meta\"; then"; }   # a file in place of cargo metadata

# ---- the harness -----------------------------------------------------------
pass=0; failed=0
run_case() {  # run_case <name> <refuse|pass> <pattern the output must contain> <setup...>
  local name=$1 want=$2 pattern=$3; shift 3
  local out status verdict
  if ! "$@" 2>"$scratch/setup.err"; then
    printf 'FAIL    %-52s the breaker did not apply: %s\n' "$name" "$(head -c 200 "$scratch/setup.err")"; failed=$((failed+1)); restore; return
  fi
  if ! bash -n tools/docs-check.sh 2>/dev/null; then
    printf 'FAIL    %-52s the breaker left the tool unparseable\n' "$name"; failed=$((failed+1)); restore; return
  fi
  out=$(tools/docs-check.sh 2>&1); status=$?
  if [[ $want == refuse && $status -ne 0 && $out == *"$pattern"* ]] \
     || [[ $want == pass && $status -eq 0 && $out == *"$pattern"* ]]; then
    printf 'ok      %-52s %s: %s\n' "$name" "$want" "$(grep -m1 -F "$pattern" <<<"$out" | cut -c1-90)"; pass=$((pass+1))
  else
    printf 'FAIL    %-52s wanted %s with "%s"; exit %d, got:\n%s\n' "$name" "$want" "$pattern" "$status" "$(grep -E 'EDGE|dependencies' <<<"$out" | sed 's/^/          /')"
    failed=$((failed+1))
  fi
  restore
}
none() { :; }
P='acquisition-plan → acquisition-helper → acquisition-daemon'
TR='links acquisition-daemon transitively'

echo '== the code: edges the planner must not have'
run_case 'clean tree'                                            pass   'ok      dependencies' none
run_case 'plan links core directly'                              refuse 'links acquisition-daemon —' adddep crates/acquisition-plan/Cargo.toml 'acquisition-daemon = { path = "../acquisition-daemon" }'
run_case 'plan → helper → core'                                  refuse "$P" plan_links_helper '[dependencies]\nacquisition-daemon = { path = "../acquisition-daemon" }\n'
run_case "helper → core under cfg(windows)"                      refuse "$P" plan_links_helper "[target.'cfg(windows)'.dependencies]\nacquisition-daemon = { path = \"../acquisition-daemon\" }\n"
run_case 'helper → core optional behind a feature'               refuse "$P" plan_links_helper '[dependencies]\nacquisition-daemon = { path = "../acquisition-daemon", optional = true }\n\n[features]\ndaemon = ["dep:acquisition-daemon"]\n'
run_case 'helper → external-bridge (optional, no member) → core' refuse 'acquisition-helper → external-bridge → acquisition-daemon' plan_links_helper "[dependencies]\nexternal-bridge = { path = \"$scratch/x/bridge\", optional = true }\n"
run_case 'union over targets: cfg(unix) then cfg(windows)'       refuse "$P" plan_links_helper "[target.'cfg(windows)'.dependencies]\nacquisition-daemon = { path = \"../acquisition-daemon\" }\n" "[target.'cfg(unix)'.dependencies]"

echo '== the code: the edges of the split'"'"'s step 3 (C1 as amended; §2.1)'
DP='acquisition-daemon = { path = "../acquisition-daemon" }'
CP='acquisition-client = { path = "../acquisition-client" }'
run_case 'client links daemon directly'                          refuse 'acquisition-client     links acquisition-daemon —' adddep crates/acquisition-client/Cargo.toml "$DP"
run_case 'client links plan directly'                            refuse 'acquisition-client     links acquisition-plan —' adddep crates/acquisition-client/Cargo.toml 'acquisition-plan = { path = "../acquisition-plan" }'
run_case 'client → helper → daemon'                              refuse 'acquisition-client → acquisition-helper → acquisition-daemon' member_links_helper acquisition-client "[dependencies]\n$DP\n"
run_case 'daemon links client directly'                          refuse 'acquisition-daemon     links acquisition-client —' adddep crates/acquisition-daemon/Cargo.toml "$CP"
run_case 'daemon links client as a dev-dependency'               refuse 'acquisition-daemon     links acquisition-client —' adddep crates/acquisition-daemon/Cargo.toml "$CP" '[dev-dependencies]'
run_case 'daemon links plan directly'                            refuse 'acquisition-daemon     links acquisition-plan —' adddep crates/acquisition-daemon/Cargo.toml 'acquisition-plan = { path = "../acquisition-plan" }'
run_case 'daemon → helper → client'                              refuse 'acquisition-daemon → acquisition-helper → acquisition-client' member_links_helper acquisition-daemon "[dependencies]\n$CP\n"
run_case 'the CLI links the daemon directly'                     refuse 'is linked by acquisition-cli' adddep crates/acquisition-cli/Cargo.toml "$DP"
run_case 'the MCP links the daemon directly'                     refuse 'is linked by acquisition-mcp' adddep crates/acquisition-mcp/Cargo.toml "$DP"
run_case 'the CLI → helper → daemon'                             refuse 'acquisition-cli → acquisition-helper → acquisition-daemon' member_links_helper acquisition-cli "[dependencies]\n$DP\n"
run_case 'the CLI → helper → daemon under cfg(windows)'          refuse 'acquisition-cli → acquisition-helper → acquisition-daemon' member_links_helper acquisition-cli "[target.'cfg(windows)'.dependencies]\n$DP\n"
run_case 'plan links client directly'                            refuse 'acquisition-plan       links acquisition-client —' adddep crates/acquisition-plan/Cargo.toml "$CP"
# store → daemon and store → client are cycles (the daemon and, since the
# split's step 5, the client link the store), and Cargo refuses a cycle
# before any guard reads the graph: the refusal is Cargo's, named as
# such — the edge is unreachable by construction, not by the rule.
run_case 'store links client directly (a cycle: Cargo refuses it first)' refuse 'cargo metadata failed' adddep crates/acquisition-store/Cargo.toml "$CP"
run_case 'store links daemon directly (a cycle: Cargo refuses it first)' refuse 'cargo metadata failed' adddep crates/acquisition-store/Cargo.toml "$DP"
run_case 'the store links tokio through a [dependencies.tokio] table' refuse 'acquisition-store      links tokio' adddep crates/acquisition-store/Cargo.toml 'workspace = true' '[dependencies.tokio]'
intent_in_client() { printf '// breaker: a client that reads intent\npub fn f() -> Option<u32> { let annotations_path = 1; Some(annotations_path) }\n' >crates/acquisition-client/src/zz_breaker.rs; }
run_case 'the intent API named in client/src'                    refuse 'names the intent API' intent_in_client

echo '== the code: shapes that must pass'
dup() { adddep crates/acquisition-store/Cargo.toml "duplicate-helper = { path = \"$scratch/x/dup1\" }"; adddep crates/acquisition-cli/Cargo.toml "duplicate-helper = { path = \"$scratch/x/dup2\", version = \"0.2.0\" }"; }
run_case 'same name, two versions: store v1, cli v2 → protocol'  pass   'ok      dependencies' dup
run_case 'an excluded path crate with a dev-only path dependency' pass  'ok      dependencies' adddep crates/acquisition-plan/Cargo.toml "outside = { path = \"$scratch/x/outside\" }"
# A frontend is a bin-only package: Cargo drops a dependency on it as
# invalid ("missing a lib target") and resolves no edge, so nothing can
# link a frontend and the daemon's rule against one cannot be staged
# today. The case documents the fact; it fails the day a frontend gains a
# library target (the Tauri GUI's shape), when the rule earns a breaker.
run_case 'the daemon declares a bin-only frontend: Cargo drops it, nothing links' pass 'ok      dependencies' adddep crates/acquisition-daemon/Cargo.toml 'acquisition-cli = { path = "../acquisition-cli" }'

echo '== the tool: the table reader answers wrongly, the staged plan → helper → core path standing'
staged() { plan_links_helper '[dependencies]\nacquisition-daemon = { path = "../acquisition-daemon" }\n'; }
with_staged() { staged && "$@"; }
run_case 'the plan→core row replaced by a duplicate row'         refuse 'differs from the graph at row' with_staged shadow_table "perl -ne 'next if /^closure\\tacquisition-plan\\tacquisition-daemon\\t/; print; print if /^closure\\tacquisition-plan\\tacquisition-helper\\t/'"
run_case 'the row replaced by a fabricated row'                  refuse 'differs from the graph at row' with_staged shadow_table "perl -ne 'next if /^closure\\tacquisition-plan\\tacquisition-daemon\\t/; print; END { print \"closure\\tx\\ty\\tz\\tw\\tv\\n\" }'"
run_case 'two rows swapped, nothing lost'                        refuse 'differs from the graph at row' with_staged shadow_table "perl -e '@l=<>; (\$l[100],\$l[101])=(\$l[101],\$l[100]); print @l'"
run_case 'the dependency name of the row renamed'                refuse 'differs from the graph at row' with_staged shadow_table "perl -pe 's/^closure\\tacquisition-plan\\tacquisition-daemon\\t/closure\\tacquisition-plan\\tmasked-core\\t/'"
run_case 'the member name of the row renamed'                    refuse 'differs from the graph at row' with_staged shadow_table "perl -pe 's/^closure\\tacquisition-plan\\tacquisition-daemon\\t/closure\\tmasked-plan\\tacquisition-daemon\\t/'"
run_case 'a direct row'"'"'s dependency name renamed'             refuse 'differs from the graph at row' with_staged shadow_table "perl -pe 's/^direct\\tacquisition-plan\\tnormal\\tacquisition-store\\t/direct\\tacquisition-plan\\tnormal\\tmasked-store\\t/'"
run_case 'only the path column altered (diagnostic, unread)'     refuse "$TR" with_staged shadow_table "perl -pe 's/(^closure\\tacquisition-plan\\tacquisition-daemon\\t.*\\t).*$/\\1masked path/'"
run_case 'the self rows only'                                    refuse 'rows, the graph has' with_staged shadow_table "perl -ne 'print if /^closure\\t(acquisition-[a-z]+)\\t\\1\\t/'"
run_case 'every row two or more steps deep dropped'              refuse 'rows, the graph has' with_staged shadow_table "grep -v ' → .* → '"
run_case 'the direct rows dropped'                               refuse 'rows, the graph has' with_staged shadow_table "grep -v '^direct'"
run_case 'the table empty, exit 0'                               refuse 'rows, the graph has' with_staged shadow_table 'sed d'

echo '== the tool: the metadata partial, or the tools absent'
capture() { staged && { cargo metadata --format-version 1 --all-features --offline >"$scratch/staged.json" 2>/dev/null || cargo metadata --format-version 1 --all-features >"$scratch/staged.json"; }; }
no_node() { capture && jq '.resolve.nodes |= map(select(.id | test("acquisition-helper") | not))' "$scratch/staged.json" >"$scratch/no-node.json" && shadow_meta "$scratch/no-node.json"; }
no_pkg()  { capture && jq '.packages |= map(select(.name != "acquisition-helper"))' "$scratch/staged.json" >"$scratch/no-pkg.json" && shadow_meta "$scratch/no-pkg.json"; }
run_case 'a member node missing from the resolve'                refuse 'a workspace member is not a node' no_node
run_case 'a package entry missing'                               refuse 'a node has no package entry' no_pkg
run_case 'the table jq answers nothing'                          refuse 'rows, the graph has' with_staged tool '([.packages[] | {key: .id, value: .name}] | from_entries) as $name
    | ([.resolve.nodes[] | {key: .id, value: .deps}] | from_entries) as $deps
    | ([.workspace_members[] as $m' 'empty | ([.packages[] | {key: .id, value: .name}] | from_entries) as $name
    | ([.resolve.nodes[] | {key: .id, value: .deps}] | from_entries) as $deps
    | ([.workspace_members[] as $m'
nojq() { mkdir -p "$scratch/nojq"; printf '#!/bin/sh\nexit 127\n' >"$scratch/nojq/jq"; chmod +x "$scratch/nojq/jq"; tool 'cd "$(dirname "$0")/.."' "cd \"\$(dirname \"\$0\")/..\"; export PATH=\"$scratch/nojq:\$PATH\""; }
run_case 'jq absent'                                             refuse 'jq failed or is not installed' with_staged nojq
run_case 'cargo metadata misspelled, both calls'                 refuse 'cargo metadata failed' with_staged tool "$META_CMD" "${META_CMD//cargo metadata/cargo metadataa}"

echo
printf '%d ok, %d failed\n' "$pass" "$failed"
((failed == 0))
