#!/bin/bash
# sign-acqd.sh — give the debug daemon a code identity that survives a
# rebuild, so macOS Keychain stops asking. A keychain item's ACL names a
# trusted app by its designated requirement; the linker's ad-hoc signature
# makes that `cdhash H"…"`, new on every link, so an unsigned acqd is asked
# twice per login after every rebuild (the read at start, the modify on
# save). Signed with a real identity the requirement is the identifier, the
# Apple anchor and the certificate's name and team — the same after every
# build, and after the certificate renews under the same name — so the
# item's "Always Allow" holds (B6, 2026-09-12; auth.rs, "keyring").
#
#   tools/sign-acqd.sh [path-to-acqd]     # default target/debug/acqd
#
# Signs only on macOS and only when ACQ_CODESIGN_IDENTITY names an identity
# (`security find-identity -v -p codesigning`: the SHA-1 is unambiguous, a
# name substring works). Otherwise says so and exits 0: the identity is a
# fact about the developer's machine, never the repo, and other platforms
# have no per-app keychain ACL to satisfy. Preflight runs this right after
# its build and before provenance.json hashes the file (C84: the daemon
# hashes its own file at start, so client and daemon see the same signed
# bytes); the live-run skill's build step runs it by hand. The identifier
# is fixed here: the linker's default embeds cargo's metadata hash, which
# changes with features or dependencies and would re-prompt. `-f` replaces
# the ad-hoc signature; re-signing a signed file is idempotent.
#
# Cargo copies target/debug/acqd from its twin in target/debug/deps on EVERY
# build, even one that compiles nothing (a copy, not a hard link — cargo
# avoids hard-linking executables on macOS), so a signature on the copy
# alone is gone after the next `cargo build` (seen 2026-09-12: undone by a
# 0.08 s no-op). Signing the twin does not dirty the unit, and every later
# copy carries the signature — so the twin is signed and copied into place.
# The twin is found by content, not name: a `-p` build and a `--workspace`
# build are different units with different twins, and the copy matches
# whichever built last. A build that actually re-links the daemon writes a
# fresh ad-hoc twin, which is why this runs after the last build before a
# run, never before it.
set -euo pipefail
here=$(cd "$(dirname "$0")/.." && pwd)
acqd=${1:-$here/target/debug/acqd}
[ -x "$acqd" ] || { echo "sign-acqd: no daemon at $acqd — cargo build --workspace first" >&2; exit 2; }
if [ "$(uname -s)" != Darwin ]; then
    echo "sign-acqd: not macOS, nothing to sign"; exit 0
fi
if [ -z "${ACQ_CODESIGN_IDENTITY:-}" ]; then
    echo "sign-acqd: ACQ_CODESIGN_IDENTITY unset — $acqd keeps its ad-hoc signature (Keychain will prompt after each rebuild)"; exit 0
fi
twin=
for f in "$(dirname "$acqd")"/deps/acqd-*; do
    case $f in *.d|*.dSYM) continue ;; esac
    if cmp -s "$f" "$acqd"; then twin=$f; break; fi
done
target=${twin:-$acqd}
codesign -s "$ACQ_CODESIGN_IDENTITY" --identifier com.gerwaric.acqd -f "$target" 2>&1 | grep -v 'replacing existing signature' || true
if [ -n "$twin" ]; then cp -f "$twin" "$acqd"; else echo "sign-acqd: no deps twin with $acqd's bytes — signed the copy alone; the next cargo build will undo it" >&2; fi
codesign --verify "$acqd"
echo "sign-acqd: $(codesign -d -r- "$acqd" 2>&1 | sed -n 's/^designated => //p')"
