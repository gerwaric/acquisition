#!/bin/bash
# preflight-breakers.sh — stages every refusal of provenance_matches_journal
# (tools/preflight.sh, C84) and of the legacy-endpoint probe
# (preflight_refuse_legacy_endpoints, C83) and requires each: a journal that passes, then a
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
RUN_DIR="$work"; MODE=mock; ACQ=/dev/null; ACQD=/dev/null
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

# ---- the legacy-endpoint probe (C83, the split's step 6) ---------------------
# Each case runs preflight_refuse_legacy_endpoints in a subshell with
# TMPDIR pointed into the scratch, so the fixed default endpoint is the
# staged one; the historical /tmp endpoints are read as they stand (an
# actual daemon on one of them refuses every case, which is the point).
expect_legacy() { # <name> <pass|fail> <needle> [<PATH prefix dir>]
    local name=$1 want=$2 needle=$3 shim=${4:-} out rc
    out=$( (export TMPDIR="$work/t"; [ -n "$shim" ] && export PATH="$shim:$PATH"; preflight_refuse_legacy_endpoints) 2>&1 ); rc=$?
    if [ "$want" = pass ] && [ $rc -eq 0 ]; then echo "ok   $name: passed"; return; fi
    if [ "$want" = fail ] && [ $rc -ne 0 ] && [[ $out == *"$needle"* ]]; then echo "ok   $name: refused, naming: $needle"; return; fi
    echo "FAIL $name: rc $rc, wanted $want${needle:+ naming \"$needle\"}:"; echo "$out" | sed 's/^/     /'; fails=$((fails+1))
}
mkdir -p "$work/t"
legacy="$work/t/acquisition-playground.sock"
expect_legacy "no legacy socket file" pass ""
# A stale file: bound once and closed, so the path stands and nothing answers.
python3 -c 'import socket,sys; s=socket.socket(socket.AF_UNIX); s.bind(sys.argv[1]); s.close()' "$legacy"
expect_legacy "a stale legacy socket file" pass ""
# A listener that never answers a handshake still answers a connect: refused as listening.
rm -f "$legacy"
python3 -c 'import socket,sys,time
s=socket.socket(socket.AF_UNIX); s.bind(sys.argv[1]); s.listen(1)
while True: time.sleep(1)' "$legacy" & listener=$!
sleep 0.3
expect_legacy "a daemon listening on the legacy socket" fail "listening on $legacy"
kill $listener 2>/dev/null; wait $listener 2>/dev/null; rm -f "$legacy"
# The probe itself failing, with a socket file standing: refused, never passed.
python3 -c 'import socket,sys; s=socket.socket(socket.AF_UNIX); s.bind(sys.argv[1]); s.close()' "$legacy"
mkdir -p "$work/shim-missing" "$work/shim-broken" "$work/shim-hang" "$work/shim-silent"
printf '#!/bin/sh\nexit 127\n' >"$work/shim-missing/python3"
printf '#!/bin/sh\necho "ImportError: staged" >&2\nexit 1\n' >"$work/shim-broken/python3"
printf '#!/bin/sh\nexit 3\n' >"$work/shim-hang/python3"
printf '#!/bin/sh\nexit 0\n' >"$work/shim-silent/python3"
chmod +x "$work"/shim-*/python3
expect_legacy "python3 missing" fail "could not probe $legacy (the probe exited 127)" "$work/shim-missing"
expect_legacy "the probe failing to run (exit 1, an ImportError's)" fail "could not probe $legacy (the probe exited 1)" "$work/shim-broken"
expect_legacy "the probe timing out or erroring" fail "could not probe $legacy (the probe exited 3)" "$work/shim-hang"
expect_legacy "a python3 that ran nothing (exit 0)" fail "could not probe $legacy (the probe exited 0)" "$work/shim-silent"
rm -f "$legacy"

if [ $fails -eq 0 ]; then echo "breakers green"; else echo "breakers RED ($fails)"; exit 1; fi
