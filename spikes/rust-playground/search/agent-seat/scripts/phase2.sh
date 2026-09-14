#!/bin/sh
# phase2.sh — the twelve questions as read-only SQL over the store copy (S) and engine-bench's seat projection (P); every call logged by call.py
T=/Users/tom/Development/GitHub/gerwaric/acquisition/spikes/rust-playground/search/agent-seat
C="$T/scripts/call.py"; S="$T/raw/world/mock/GERWARIC_7694.db"; P="$T/../engine-bench/raw/seat-projection.db"
s() { q=$1; note=$2; shift 2; "$C" --phase 2 --surface sql --q "$q" --into context --note "$note" --db "$S" --sql "$(cat)"; }
p() { q=$1; note=$2; shift 2; "$C" --phase 2 --surface sql --q "$q" --into context --note "$note" --db "$P" --sql "$(cat)"; }

s orientation "S: live rows" <<'SQL'
SELECT count(*) AS total, sum(removed_at IS NULL) AS live, count(DISTINCT league) AS leagues FROM items
SQL
p orientation "P: rows and which arrays lines holds" <<'SQL'
SELECT (SELECT count(*) FROM items) AS items, (SELECT count(*) FROM lines) AS lines, (SELECT group_concat(arr||':'||n, ' ') FROM (SELECT arr, count(*) n FROM lines GROUP BY arr ORDER BY n DESC)) AS arrays
SQL

s Q1 "S: rares of the base with their resist/attribute lines; the tab name via a join; the line is an object's description" <<'SQL'
SELECT i.name, t.name AS tab,
  (SELECT group_concat(json_extract(m.value,'$.description'), ' | ') FROM json_each(i.json,'$.explicitMods') m
    WHERE json_extract(m.value,'$.description') LIKE '%Resist%' OR json_extract(m.value,'$.description') LIKE '+% to Strength' OR json_extract(m.value,'$.description') LIKE '+% to Dexterity' OR json_extract(m.value,'$.description') LIKE '+% to Intelligence') AS lines
FROM items i LEFT JOIN tabs t ON t.id=i.location_id AND t.league=i.league AND t.realm=i.realm
WHERE i.rarity='Rare' AND i.base_type='Titan Gauntlets' AND i.removed_at IS NULL
SQL
p Q1 "P: the same over lines" <<'SQL'
SELECT i.name, i.tab_name, group_concat(l.line||'='||l.value, ' | ') AS lines
FROM items i JOIN lines l ON l.item=i.rid
WHERE i.rarity='Rare' AND i.base_type='Titan Gauntlets' AND (l.line LIKE '%Resist%' OR l.line IN ('+# to Strength','+# to Dexterity','+# to Intelligence'))
GROUP BY i.rid
SQL

s Q2 "S: legacy Ashes by a LIKE over the whole body" <<'SQL'
SELECT substr(i.id,1,8) AS id, i.league, t.name AS tab FROM items i LEFT JOIN tabs t ON t.id=i.location_id AND t.league=i.league AND t.realm=i.realm
WHERE i.name='Ashes of the Stars' AND i.json LIKE '%Reservation Efficiency%'
SQL
p Q2 "P: legacy Ashes by a line" <<'SQL'
SELECT substr(i.gid,1,8) AS id, i.league, i.tab_name, l.line, l.value FROM items i JOIN lines l ON l.item=i.rid
WHERE i.name='Ashes of the Stars' AND l.line LIKE '%Reservation Efficiency%'
SQL

s Q3 "S: six-linked items: sockets grouped per item and group" <<'SQL'
SELECT count(*) AS six_links FROM (
  SELECT i.id FROM items i, json_each(i.json,'$.sockets') s WHERE i.removed_at IS NULL
  GROUP BY i.id, json_extract(s.value,'$.group') HAVING count(*) >= 6)
SQL
p Q3 "P: six-links — the projection carries no sockets; the attempt as an agent would write it" <<'SQL'
SELECT count(*) FROM items WHERE links >= 6
SQL

s Q4 "S: staves with a crucible array, tab named" <<'SQL'
SELECT i.name, i.type_line, i.rarity, t.name AS tab FROM items i LEFT JOIN tabs t ON t.id=i.location_id AND t.league=i.league AND t.realm=i.realm
WHERE i.type_line LIKE '%Staff%' AND json_extract(i.json,'$.crucibleMods') IS NOT NULL AND i.removed_at IS NULL
SQL
p Q4 "P: staves with a crucible line" <<'SQL'
SELECT DISTINCT i.name, i.type_line, i.rarity, i.tab_name FROM items i JOIN lines l ON l.item=i.rid
WHERE i.type_line LIKE '%Staff%' AND l.arr='crucibleMods'
SQL

s Q5 "S: leveling tabs with their live item counts" <<'SQL'
SELECT t.name, count(i.id) AS items FROM tabs t LEFT JOIN items i ON i.location_id=t.id AND i.league=t.league AND i.removed_at IS NULL
WHERE t.name LIKE '%level%' AND t.removed_at IS NULL GROUP BY t.realm, t.league, t.id
SQL
s Q5 "S: uniques at requirement level <= 30: the number sits in requirements[].values[0][0], a string" <<'SQL'
SELECT count(*) AS uniques_le30 FROM items i, json_each(i.json,'$.requirements') r
WHERE i.rarity='Unique' AND i.removed_at IS NULL AND json_extract(r.value,'$.name')='Level' AND CAST(json_extract(r.value,'$.values[0][0]') AS INT) <= 30
SQL
p Q5 "P: leveling tabs, and uniques by req_level" <<'SQL'
SELECT (SELECT group_concat(tab_name||'='||n, '; ') FROM (SELECT tab_name, count(*) n FROM items WHERE tab_name LIKE '%level%' GROUP BY tab_name)) AS tabs,
       (SELECT count(*) FROM items WHERE rarity='Unique' AND req_level <= 30) AS uniques_le30
SQL

s Q6 "S: a rare with an explode line, its wearer named" <<'SQL'
SELECT i.name, i.type_line, COALESCE(c.name, t.name) AS location FROM items i
LEFT JOIN characters c ON c.id=i.location_id LEFT JOIN tabs t ON t.id=i.location_id AND t.league=i.league
WHERE i.rarity='Rare' AND i.json LIKE '%Kill Explode%' AND i.removed_at IS NULL
SQL
p Q6 "P: the same over lines" <<'SQL'
SELECT i.name, i.type_line, i.tab_name, l.line FROM items i JOIN lines l ON l.item=i.rid WHERE i.rarity='Rare' AND l.line LIKE '%Kill Explode%'
SQL

s Q7 "S: the sweep by a LIKE over the body (every array, and any flavour text too), grouped by location name" <<'SQL'
SELECT COALESCE(t.name, c.name, i.location_id) AS loc, count(*) AS n FROM items i
LEFT JOIN tabs t ON t.id=i.location_id AND t.league=i.league AND t.realm=i.realm LEFT JOIN characters c ON c.id=i.location_id
WHERE i.json LIKE '%increased Rarity of Items found%' AND i.removed_at IS NULL GROUP BY loc ORDER BY n DESC LIMIT 6
SQL
s Q7 "S: the sweep's total and its location count" <<'SQL'
SELECT count(*) AS items, count(DISTINCT i.location_id) AS locations FROM items i WHERE i.json LIKE '%increased Rarity of Items found%' AND i.removed_at IS NULL
SQL
p Q7 "P: the sweep by template, grouped by location name" <<'SQL'
SELECT i.tab_name AS loc, count(DISTINCT i.rid) AS n FROM items i JOIN lines l ON l.item=i.rid WHERE l.line='#% increased Rarity of Items found' GROUP BY loc ORDER BY n DESC LIMIT 6
SQL
p Q7 "P: the sweep's total" <<'SQL'
SELECT count(DISTINCT i.rid) AS items, count(DISTINCT i.location_id) AS locations FROM items i JOIN lines l ON l.item=i.rid WHERE l.line='#% increased Rarity of Items found'
SQL

s A1 "S: items per league and rarity" <<'SQL'
SELECT league, rarity, count(*) AS n FROM items WHERE removed_at IS NULL GROUP BY 1, 2 ORDER BY 1, 3 DESC
SQL
s A1 "S: items per tab, top 5" <<'SQL'
SELECT t.name, count(*) AS n FROM items i JOIN tabs t ON t.id=i.location_id AND t.league=i.league AND t.realm=i.realm WHERE i.removed_at IS NULL GROUP BY t.realm, t.league, t.id ORDER BY n DESC LIMIT 5
SQL
p A1 "P: items per league and rarity" <<'SQL'
SELECT league, rarity, count(*) AS n FROM items GROUP BY 1, 2 ORDER BY 1, 3 DESC
SQL
p A1 "P: items per tab, top 5" <<'SQL'
SELECT tab_name, count(*) AS n FROM items WHERE location_kind='stash' GROUP BY tab_name ORDER BY n DESC LIMIT 5
SQL

s A2 "S: Q1 refined to total resistance >= 60: the number parsed out of the description with substr/instr" <<'SQL'
SELECT name, sum(CAST(substr(d,2,instr(d,'%')-2) AS INT) * (CASE WHEN d LIKE '% and %' THEN 2 WHEN d LIKE '%all Elemental%' THEN 3 ELSE 1 END)) AS res
FROM (SELECT i.id, i.name, json_extract(m.value,'$.description') AS d FROM items i, json_each(i.json,'$.explicitMods') m
      WHERE i.rarity='Rare' AND i.base_type='Titan Gauntlets' AND i.removed_at IS NULL)
WHERE d LIKE '+%\% to %Resist%' ESCAPE '\' GROUP BY id HAVING res >= 60
SQL
p A2 "P: the same with the line's parsed value" <<'SQL'
SELECT i.name, sum(l.value * (CASE WHEN l.line LIKE '% and %' THEN 2 WHEN l.line LIKE '%all Elemental%' THEN 3 ELSE 1 END)) AS res
FROM items i JOIN lines l ON l.item=i.rid WHERE i.rarity='Rare' AND i.base_type='Titan Gauntlets' AND l.line LIKE '+#% to %Resist%'
GROUP BY i.rid HAVING res >= 60
SQL

s A3 "S: explain: which field matched, and where the non-matching word lives" <<'SQL'
SELECT (SELECT count(*) FROM items WHERE name LIKE '%Explode%' OR type_line LIKE '%Explode%' OR base_type LIKE '%Explode%') AS in_names,
       (SELECT count(*) FROM items WHERE json LIKE '%Explode%') AS in_bodies,
       (SELECT group_concat(f, ',') FROM (SELECT DISTINCT CASE WHEN name LIKE '%Kaom%' THEN 'name' WHEN type_line LIKE '%Kaom%' THEN 'type_line' ELSE 'base_type' END AS f FROM items WHERE name LIKE '%Kaom%' OR type_line LIKE '%Kaom%' OR base_type LIKE '%Kaom%')) AS kaom_matched_on
SQL
p A3 "P: the same over lines" <<'SQL'
SELECT (SELECT count(*) FROM items WHERE pretty LIKE '%explode%' OR base LIKE '%explode%') AS in_names, (SELECT count(DISTINCT item) FROM lines WHERE line LIKE '%Explode%') AS in_lines
SQL

s A4 "S: one item by id, and by prefix" <<'SQL'
SELECT name, type_line, location_kind, (SELECT count(*) FROM items WHERE id LIKE '1d57e63d%') AS prefix_hits FROM items WHERE id='1d57e63d224e11cb416e5feecd5fcd9fa04f8986ca2c49ca057e1da18d4cfb8e'
SQL
p A4 "P: one item by gid" <<'SQL'
SELECT rid, name, type_line, tab_name FROM items WHERE gid='1d57e63d224e11cb416e5feecd5fcd9fa04f8986ca2c49ca057e1da18d4cfb8e'
SQL

s A5 "S: +# to maximum Life >= 90 over five arrays: a UNION per array, the number cut out of the text, string and object entries both" <<'SQL'
WITH m AS (
  SELECT i.id, CASE WHEN e.type='object' THEN json_extract(e.value,'$.description') ELSE e.value END AS d FROM items i, json_each(i.json,'$.explicitMods') e WHERE i.removed_at IS NULL
  UNION ALL SELECT i.id, CASE WHEN e.type='object' THEN json_extract(e.value,'$.description') ELSE e.value END FROM items i, json_each(i.json,'$.implicitMods') e WHERE i.removed_at IS NULL
  UNION ALL SELECT i.id, CASE WHEN e.type='object' THEN json_extract(e.value,'$.description') ELSE e.value END FROM items i, json_each(i.json,'$.craftedMods') e WHERE i.removed_at IS NULL
  UNION ALL SELECT i.id, CASE WHEN e.type='object' THEN json_extract(e.value,'$.description') ELSE e.value END FROM items i, json_each(i.json,'$.fracturedMods') e WHERE i.removed_at IS NULL
  UNION ALL SELECT i.id, CASE WHEN e.type='object' THEN json_extract(e.value,'$.description') ELSE e.value END FROM items i, json_each(i.json,'$.enchantMods') e WHERE i.removed_at IS NULL)
SELECT count(*) AS items, max(v) AS top FROM (SELECT id, max(CAST(substr(d,2,instr(d,' ')-2) AS INT)) AS v FROM m WHERE d LIKE '+% to maximum Life' GROUP BY id) WHERE v >= 90
SQL
p A5 "P: the same over lines, one indexed template" <<'SQL'
SELECT count(*) AS items, max(v) AS top FROM (SELECT item, max(n0) AS v FROM lines WHERE line='+# to maximum Life' GROUP BY item) WHERE v >= 90
SQL
