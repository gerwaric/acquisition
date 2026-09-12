#!/bin/bash
# sign-acqd-breakers.sh — stages what tools/sign-acqd.sh must refuse and
# what it must accept, on scratch copies of the built daemon (nothing under
# target/ is touched): a nonexistent identity exits 2 and leaves the code
# hash in place — the audit finding of 2026-09-12, when it exited 0 —; a
# missing daemon exits 2; no identity is a no-op exit 0 and the ad-hoc
# signature stays; and, when ACQ_CODESIGN_IDENTITY is set in the shell,
# the real identity exits 0 with the fixed identifier in the requirement.
# Run by hand after touching the script; ~1 s. Needs a cargo build first.
set -uo pipefail
here=$(cd "$(dirname "$0")/.." && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
acqd=$here/target/debug/acqd
[ -x "$acqd" ] || { echo "no daemon at $acqd — cargo build --workspace first" >&2; exit 2; }
fails=0
expect() { # <name> <rc wanted> <needle in stdout+stderr> <env...> -- <args...>
    local name=$1 want=$2 needle=$3 out rc; shift 3
    local envs=(); while [ "$1" != -- ]; do envs+=("$1"); shift; done; shift
    out=$(env "${envs[@]}" "$here/tools/sign-acqd.sh" "$@" 2>&1); rc=$?
    if [ $rc -eq "$want" ] && [[ $out == *"$needle"* ]]; then echo "ok   $name: rc $rc, saying: $needle"; return; fi
    echo "FAIL $name: rc $rc, wanted $want saying \"$needle\":"; echo "$out" | sed 's/^/     /'; fails=$((fails+1))
}
mkdir -p "$work/deps"
cp "$acqd" "$work/acqd"
# Start from what the linker leaves: an ad-hoc signature (the built daemon
# may already carry the real identity from an earlier signing).
codesign -s - -f "$work/acqd" 2>/dev/null
before=$(codesign -d -r- "$work/acqd" 2>&1)
[[ $before == *cdhash* ]] || { echo "FAIL the scratch copy is not ad-hoc signed to begin with: $before"; exit 2; }

expect "a nonexistent identity" 2 "codesign failed" ACQ_CODESIGN_IDENTITY=__no_such_identity__ -- "$work/acqd"
after=$(codesign -d -r- "$work/acqd" 2>&1)
if [ "$after" = "$before" ]; then echo "ok   … and the file's requirement is untouched (still the code hash)"
else echo "FAIL … the file's requirement changed under a failed signing: $after"; fails=$((fails+1)); fi
expect "no identity in the environment" 0 "ACQ_CODESIGN_IDENTITY unset" -u ACQ_CODESIGN_IDENTITY -- "$work/acqd"
expect "no daemon" 2 "no daemon at" ACQ_CODESIGN_IDENTITY=x -- "$work/missing"
if [ -n "${ACQ_CODESIGN_IDENTITY:-}" ]; then
    expect "the real identity, no twin" 0 'identifier "com.gerwaric.acqd"' "ACQ_CODESIGN_IDENTITY=$ACQ_CODESIGN_IDENTITY" -- "$work/acqd"
    cp "$acqd" "$work/deps/acqd-twin"; cp "$acqd" "$work/acqd"
    expect "the real identity, twin found" 0 'identifier "com.gerwaric.acqd"' "ACQ_CODESIGN_IDENTITY=$ACQ_CODESIGN_IDENTITY" -- "$work/acqd"
    cmp -s "$work/deps/acqd-twin" "$work/acqd" && echo "ok   … and the twin and the copy are the same signed bytes" || { echo "FAIL … the twin and the copy differ"; fails=$((fails+1)); }
else
    echo "skip the real-identity cases: ACQ_CODESIGN_IDENTITY is not set in this shell"
fi
[ $fails -eq 0 ] && echo "all breakers hold" || { echo "$fails breaker(s) failed"; exit 1; }
