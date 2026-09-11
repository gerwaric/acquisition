#!/bin/bash
# preflight-breakers.sh — stages every refusal of provenance_matches_journal
# (tools/preflight.sh, C84) and requires it: a journal that passes, then a
# header removed, a send whose pid has no header, no header at all, a
# header naming another acqd, another contract, a pid the OS reused for
# the successor (a second lifetime: passes), and a provenance.json
# without the hashes. Each case runs the
# function in a subshell (it exits 2 on refusal) and names the property it
# expects in the refusal. Run by hand after touching the function; ~1 s.
set -uo pipefail
here=$(cd "$(dirname "$0")/.." && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
RUN_DIR="$work"; MODE=mock; ACQ=/dev/null; ACQD=/dev/null; SOCK=none
. "$here/tools/preflight.sh"

good() {
    cat <<'J'
{"event":"open","pid":1,"contract":"cccccccccccc","daemon":"dddd","clock":"system","ts":"t"}
{"pid":1,"method":"POST","route":"oauth-token","status":200}
{"pid":1,"method":"GET","route":"profile@A#1","status":200}
{"event":"open","pid":2,"contract":"cccccccccccc","daemon":"dddd","clock":"system","ts":"t"}
{"pid":2,"method":"POST","route":"oauth-token","status":200}
{"pid":2,"method":"HEAD","route":"stash@A#1","status":204}
{"pid":2,"method":"GET","route":"stash@A#1","status":200}
J
}
echo '{"contract":"cccccccccccc","acqd_sha256":"dddd"}' >"$work/provenance.json"

fails=0
expect() { # <name> <pass|fail> <needle in the refusal> <journal file>
    local name=$1 want=$2 needle=$3 journal=$4 out rc
    out=$( (provenance_matches_journal "$journal") 2>&1 ); rc=$?
    if [ "$want" = pass ] && [ $rc -eq 0 ]; then echo "ok   $name: passed"; return; fi
    if [ "$want" = fail ] && [ $rc -ne 0 ] && [[ $out == *"$needle"* ]]; then echo "ok   $name: refused, naming: $needle"; return; fi
    echo "FAIL $name: rc $rc, wanted $want${needle:+ naming \"$needle\"}:"; echo "$out" | sed 's/^/     /'; fails=$((fails+1))
}

good >"$work/good.jsonl"
expect "a whole journal" pass "" "$work/good.jsonl"
good | grep -v '"open","pid":2' >"$work/header-removed.jsonl"
expect "the second lifetime's header removed" fail "pid 2: POST oauth-token under pid 1's header, not its own" "$work/header-removed.jsonl"
good | sed -n '2,$p' >"$work/first-header-removed.jsonl"
expect "the first lifetime's header removed" fail "pid 1: POST oauth-token with no open header before it" "$work/first-header-removed.jsonl"
good | sed 's/{"pid":2,"method":"GET"/{"pid":3,"method":"GET"/' >"$work/unmatched.jsonl"
expect "a send whose pid has no header" fail "pid 3: GET stash@A#1 under pid 2's header, not its own" "$work/unmatched.jsonl"
good | grep -v '"event":"open"' >"$work/no-headers.jsonl"
expect "no header at all" fail "no open header in the journal at all" "$work/no-headers.jsonl"
good | sed '4s/"daemon":"dddd"/"daemon":"eeee"/' >"$work/other-daemon.jsonl"
expect "a header naming another acqd" fail "pid 2: header names contract cccccccccccc daemon eeee" "$work/other-daemon.jsonl"
good | sed '1s/"contract":"cccccccccccc"/"contract":"bbbbbbbbbbbb"/' >"$work/other-contract.jsonl"
expect "a header naming another contract" fail "pid 1: header names contract bbbbbbbbbbbb" "$work/other-contract.jsonl"
good | sed 's/"pid":2/"pid":1/' >"$work/reused.jsonl"
expect "a pid the OS reused for the successor lifetime" pass "" "$work/reused.jsonl"
: >"$work/empty.jsonl"
expect "an empty journal" fail "no open header in the journal at all" "$work/empty.jsonl"
echo '{"contract":"cccccccccccc"}' >"$work/provenance.json"
expect "a provenance.json without the acqd hash" fail "names no contract or acqd hash" "$work/good.jsonl"

if [ $fails -eq 0 ]; then echo "breakers green"; else echo "breakers RED ($fails)"; exit 1; fi
