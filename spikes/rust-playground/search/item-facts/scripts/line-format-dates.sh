#!/bin/sh
# When did the private stash API's explicitMods lines become objects? Reads every C++ userstore
# the app has ever backed up under its data directory (read-only, in place, never copied) and
# prints (fetch day, line kind, items) — the day is the C++ app's local timestamp (US Central).
# Regenerate: sh scripts/line-format-dates.sh   (macOS path; the backups die with the machine)
set -eu
DIR="$HOME/Library/Application Support/Acquisition"
find "$DIR" -name 'userstore-GERWARIC#7694.db' | while IFS= read -r f; do
  uri="file:$(printf '%s' "$f" | sed 's/#/%23/g; s/ /%20/g')?mode=ro"
  sqlite3 "$uri" "SELECT substr(json_fetched_at, 1, 10), json_type(i.value, '\$.explicitMods[0]'), count(*)
                  FROM stashes s, json_each(s.json_data, '\$.items') i
                  WHERE s.json_data IS NOT NULL AND json_type(i.value, '\$.explicitMods') IS NOT NULL
                  GROUP BY 1, 2" 2>/dev/null
done | sort -u | awk -F'|' '{ k[$1"|"$2] = ($1"|"$2 in k && k[$1"|"$2] > $3) ? k[$1"|"$2] : $3 }
                            END { for (x in k) print x "|" k[x] }' | sort
