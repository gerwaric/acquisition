#!/bin/sh
# Rung 11 helper: run `acq` against one of two isolated live daemons, A or B.
#   tools/acq-as.sh A auth --no-browser
#   tools/acq-as.sh B characters --json
# Each label has its own socket (so its own limiter, tripwire and store),
# its own journal, and NO keyring: the stored refresh token of the real
# account is never read or overwritten by this experiment.
# Rails are the ladder's: ACQ_GGG=1 ACQ_TRIPWIRE=1, ceiling 8 sends.
set -eu
label=${1:?A or B}; shift
case "$label" in A|B) ;; *) echo "label must be A or B" >&2; exit 2;; esac
here=$(cd "$(dirname "$0")/.." && pwd)
run=${ACQ_RUN_DIR:-$here/runs/$(date -u +%F)-r11}
mkdir -p "$run/store-$label"
ACQ_GGG=1 ACQ_TRIPWIRE=1 ACQ_NO_KEYRING=1 ACQ_MAX_SENDS=${ACQ_MAX_SENDS:-8} \
ACQ_IDLE_SHUTDOWN=3600 \
ACQ_SOCKET="/tmp/acq-r11-$label.sock" \
ACQ_STORE_DIR="$run/store-$label" \
ACQ_JOURNAL="$run/$label.jsonl" \
exec "$here/target/debug/acq" "$@"
