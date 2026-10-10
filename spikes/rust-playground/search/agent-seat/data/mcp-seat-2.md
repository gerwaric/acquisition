<!-- hand-written during the run of BRIEF.md; call numbers are rows of mcp-seat-calls.csv -->

## 1. Before any item

O, from calls 1-5.

- `open` (call 1, 20660 B) names the provider (mock), the three item
  tools (search, describe, show), the older store tools, and the first
  two steps to take, with the first step's refusal said in advance.
- `search {}` (call 2, 403 B) was refused as the instructions said it
  would be: the store holds two realms, pc and poe2, and the refusal
  carried one request per realm. From it alone: one account,
  GERWARIC#7694, and two realms.
- The pc offer sent back (call 4, 10144 B): 22623 live items in pc;
  7 leagues (Allflame, Ancestors, Hardcore, Phrecia 2.0, Solo
  Self-Found, Standard, Ziz Rapture HCSSF Class Gauntlet); 546 places
  fetched, 2940 never fetched, 17 folders; fetch times between
  1788810111 and 1789147353. This call is not "without reading a single
  item": the offer's view is rows, so 20 item rows came with the scope
  (most of the 10144 B). The instructions' first step leads there; a
  counts view would have told the same scope with no row.
- `describe ["fields"]` (call 3, 10503 B): what may be asked of an
  item — text, name, typeline, base, note, rarity, frame, class (82),
  category (82 ids), ilvl, reqlevel, stack, sockets, links, priced and
  price.*, league, tab, tab.type, character, container, id, socket
  colours, the `is` flags, `has`. It says outright that league of
  origin is not a fact the search holds (bears on Q4).
- `describe ["counts","operators","values","composition"]` (call 5,
  4486 B): the count keys, the four operators, `line(P).<slot>` and
  `sum`, and and/or/not/holds/undecided.

## 2. The thirteen

**Q1 — a rare Titan Gauntlets for resistances or attributes**

Told: yes. You hold 12 rare Titan Gauntlets, all in Standard stash
tabs, none on a character (call 7); 11 carry a resistance or an
attribute (call 8), by total resistance:

| name | tab | total res | lines |
| --- | --- | --- | --- |
| Brood Paw | Dump Tab | 85 | +26% cold, +27% chaos, +16% fire and lightning (crafted) |
| Morbid Touch | Lab Runners | 64 | +22% cold, +42% lightning |
| Hate Nails | Cem Stuff 2 (Remove-only) | 57 | +27% chaos, +15% fire and chaos (crafted), +35 dex |
| Rune Knuckle | Special Gear (Remove-only) | 48 | +28% cold, +20% chaos |
| Glyph Touch | Market Items (Remove-only) | 47 | +47% cold |
| Corruption Fist | Gear #1 (3.16) | 45 | +34% cold, +11% lightning, +25 str |
| Bramble Vise | Decent Gear | 36 | +16% fire, +10% lightning and chaos (crafted), +50 str |
| Beast Vise | ~price 50 chaos (G) (Remove-only) | 27 | +27% lightning, +24 dex |
| Kraken Knuckle | ~price 50 chaos (G) (Remove-only) | 22 | +22% cold |
| Soul Grasp | 8 chaos (Remove-only) | 18 | +18% lightning, +48 str (fractured), +53 dex |
| Hypnotic Vise | ~price 40 chaos (F) (Remove-only) | none | +28 dex |

No intelligence on any (call 7, `has:pseudo.total_int` 0).

Calls: 7 (23240 B), 8 (12336 B). Two calls; the first was a count I
could have done without.

Right? I hold it right for what was fetched. What would say not: the
scope's `never_fetched` 2940 — places the store has never read could
hold more; and "attribute requirements" I read as attribute lines, not
the gloves' own requirement.

Stopped or misled: call 7's five `line:` keys cost 23240 B, most of it
the same three computed buckets (`pseudo.total_life`, `total_dex`,
`total_str`) printed again under strength, dexterity, intelligence and
attributes — `pseudo.total_life` is offered for the word "strength".
A row's `sort.shows` repeats `matched[].shows` line for line (call 8).

**Q2 — a legacy Ashes of the Stars**

Told: yes, four. You hold 10 Ashes of the Stars (call 9); 4 carry
"20% increased Reservation Efficiency of Skills", the line you gave as
the legacy mark: one in tab "3.18 Poison Seismic Sab", one in "Build
Gear 2", two in "Special Gear", all Standard. The six without it: on
characters I_EXIST and TheAbsenceOfPatience (Allflame) and mrs_flowers
(Standard), and in tabs "Flamewood Totems (Remove-only)", "Ex Trap
Trickster (Remove-only)" and the unique tab under "~price 4444 chaos
(Remove-only)".

Calls: 9 (7820 B). One call.

Right? Only by your definition: the tool has no notion of legacy, and
I found the four by the line you named. What would say not: a legacy
variant marked some other way (a value outside today's range) — the
tool cannot tell me today's range.

Stopped: to see a line on rows that may lack it I wrote
`(line(…) or true())`; there is no way to ask a row to show a line
without making it a condition.

**Q3 — gear with six linked sockets**

Told: 187 items hold a six-link (call 10): 132 body armours (67 rare,
62 unique, 2 normal, 1 magic), 30 bows, 9 two-hand axes, 7 staves,
5 two-hand swords, 3 warstaves, 1 two-hand mace. 181 are in Standard
and 6 in Allflame; 26 are on characters, the rest spread over 93 tabs,
most in "Dump Tab" (10) (call 11). Tell me the slot or the colours the
interaction needs and I will name the items.

Calls: 10 (7867 B), 11 (5619 B). Two calls; no row read.

Right? The count is the tool's, with 0 undecided (call 10). What would
say not: `links` is "the size of the item's largest link group"
(call 3), which is what was asked.

Stopped: nowhere. The question is too open to list 187 rows; each cell
carries the request for its members.

**Q4 — the Crucible staff someone offered real money for**

Told: I cannot tell you which one. The tool does not hold where an
item came from (call 3, `league`: "league of origin is not a fact the
search holds"), and nothing about an offer. The nearest thing it holds
is a staff with a Crucible tree: there are seven (call 12), all in
Standard — "The Annihilating Light" (Quarterstaff) in "Crucible -
Challenges (Remove-only)", and in "Crucible - Duds (Remove-only)" a
second Annihilating Light, The Geomantic Gyre, the rares Vengeance
Spire and Tempest Beam (Highborn Staff), and a normal Quarterstaff and
a normal Highborn Staff. None carries a note or a price (call 13). If
it is the one not filed under Duds, it is the Annihilating Light in
"Crucible - Challenges".

Calls: 12 (7811 B), 13 (2798 B).

Right? The seven are right as "staff with a crucible line"; the pick is
a guess from a tab's name and I say so. What would say not: a Crucible
staff whose tree was removed, or a warstaff — `weapon.staff` holds
`weapon.warstaff` by the dot, so those were asked.

Stopped: the tool has no fact for league of origin and says so before
I asked — the one place its own words saved a call.

**Q5 — my leveling gear**

Told: three tabs in Standard carry the word. "Levling Gear
(Remove-only)" (your spelling) holds 16 items (call 17): Tabula Rasa,
Goldrim, two Wanderlust, Seven-League Step, Lochtonial Caress, Belt of
the Deceiver, Lifesprig, and eight unnamed items — 4 wands, 3 boots,
2 rings, an amulet, a belt, a chest, gloves, a helmet, a support gem, a
flask by class. "AFK & Leveling Gear (Remove-only)" holds 34 and "AFK
Leveling (Remove-only)" 12 (calls 14, 15): Honourhome, Astramentis,
Lochtonial Caress, two Lifesprig, Perandus Signet, The Supreme Truth,
Maloney's Mechanism, then gear requiring level 48 to 70 (Kaom's Heart,
Ming's Heart, Rise of the Phoenix, Chitus' Apex, rares, flasks, gems)
— that tab reads as a set for an AFK character, not a new one. Beyond
the tabs you hold 664 identified unique weapons, armours and
accessories a character of level 30 or less can wear (call 16): 17
Lifesprig, 14 Le Heup of All, 13 Wanderlust, 12 Belt of the Deceiver,
11 Goldrim, 10 Astramentis and 187 more names, most of them in
"~price 4554 chaos" (124) and Allflame's "Uniques 1" (75).

Calls: 14 (5404 B), 15 (21328 B), 16 (26090 B), 17 (5930 B). Four.

Right? In part. The misspelled tab I found by accident: `tab:level`
(call 14) cannot match "Levling", and I saw the name only because call
16's tab table happened to list it at 8. Had it held fewer low uniques
it would have fallen under `left_out` 64 and I would have told you two
tabs. What would say not: other tabs meant for leveling under a name
without the word.

Stopped: no fuzzy match on a tab's name and no list of tab names from
`search` short of a `tab` count over everything. "Leveling gear" is the
person's notion; the tool has only reqlevel.

**Q6 — what the legacy explode chest is worth**

Told: I cannot tell you what it is worth, and the tool says it cannot
(call 1: "No tool says what an item is worth: a price is the owner's
own listing"). What I can tell you: you hold two body armours with
"Enemies you Kill Explode, dealing 3% of their Life as Physical
Damage", both rare Vaal Regalia in Standard (call 18) — "Rage Curtain"
worn by VVickedWitch and "Rapture Wrap" in tab "3.13 Golems, BL, Misc
(Remove-only)". Rapture Wrap is item level 85, crusader and hunter,
six sockets in one group, 205 energy shield, and carries no price of
yours: `price` is none, "nothing applies" (call 19). Neither matched
`has:priced` (call 18).

Calls: 18 (5035 B), 19 (2692 B).

Right? That these two are "the explode chest" is my reading of the
line; that either is legacy I cannot check — the tool has no notion of
it. What would say not: the person meaning a unique or another explode
line; call 18 lists nine explode templates over 36 items, two of them
on body armours.

Stopped: at worth, by design and said up front. `show` printed `price`
as an object with `kind` none; the row gave no sign of price at all,
so "unpriced" on a row is read from a term's absence.

**Q7 — gear with increased Rarity of Items found**

Told: yes, a great deal. 508 items carry a line about Rarity of Items
found (call 20); 481 of them the plain "#% increased Rarity of Items
found", from 4% to 70% — 406 as an explicit line, 69 as an implicit,
24 on a flask's own lines. By class: 189 rings, 115 helmets, 85
amulets, 31 gloves, 28 boots, 26 utility flasks, 10 belts, 9 thrusting
swords, 7 hybrid flasks, 3 shields, 2 body armours, a jewel, a
two-hand axe, a two-hand mace. 344 rare, 134 unique, 21 magic, 9
normal. 490 in Standard, 18 in Allflame. The highest (call 21): three
Andvarius (70%, 67%, 53% explicit, each with a Gold Ring implicit
besides), the rare rings Empyrean Gyre and Empyrean Eye (51%),
Aurseize on VVickedWilliam (50%), Greed's Embrace (48%), the amulet
Dire Torc (41% + 16% implicit), the helmet Tempest Ward (40%), a
Ventor's Gamble (39%).

Calls: 20 (13049 B), 21 (9578 B).

Right? The counts are the tool's with 0 undecided. What would say not:
whether GGG's announcement covers the implicit, the flask and the
"reduced" lines is the person's to say; I gave each its count.

Stopped: the sort `line(…).arg1` orders by the largest single
occurrence, so an Andvarius of 70 + 8 sorts at 70 (call 21, `sort.value`
70 beside two lines); the item's total is `pseudo.increased_rarity`,
which the count named (call 20) and I did not sort by.

**A1 — items per tab, per league, per rarity, before any item**

Answer (call 22, both realms, no row): 22721 live items. Per league:
pc Standard 21311, pc Allflame 1312, poe2 Standard 98. Per rarity:
rare 5785, normal 3139, unique 3077, magic 2423, none 8297 (gems,
currency, cards). Per tab: 297 tabs hold items and I asked for 15 —
ULTIMATUMS (Remove-only) 499, Maps (Remove-only) 488, Gems 462,
Flasks 461, Gems (Remove-only) 411, Divination 385, ~price 4554 chaos
370, Gems 8 (Remove-only) 350, Gems (Leveled) 349, Gems 7
(Remove-only) 337, Fragments 329, Flasks 4 (Remove-only) 322, Maps
(Allflame) 315, Flasks 2 (Remove-only) 302, Flasks 1 (Remove-only)
285 — with 282 tabs left out and 1389 items in no tab (on characters).

Calls: 22 (7906 B). One.

Right? Yes for league and rarity. For "per tab" I hold 15 of 297: the
whole table would be one more call with a limit of 297, which by the
size of a bucket (about 400 B) would run over 100 KB; I did not make it.

Stopped or misled: `scope.leagues` lists eight leagues and the league
table holds items in three; the other five are leagues with nothing
live, which the answer does not say. The league table splits by realm
and the rarity table does not.

**A2 — Q1's rares with 60 or more total resistance**

Answer: two (call 23) — Brood Paw, 85, in "Dump Tab"; Morbid Touch,
64, in "Lab Runners".

Calls: 23 (7179 B). One, as a refinement: call 8's text in parentheses
and `pseudo.total_res>=60` after it, as `query`'s description says.

Right? Yes; it agrees with call 8's sort values (85, 64, then 57).

Stopped: nowhere. A refinement re-sends the whole text; nothing lets
an answer be named and narrowed. The answer repeated all six of the
old terms' counts and requests, so 7179 B carried two rows.

**A3 — why one matched and one did not**

Answer: `show` with `against` (calls 24, 25), on the query
`base="Titan Gauntlets" rarity=rare pseudo.total_res>=60`.
Brood Paw: outcome match; the three nodes matched; the total printed
as 85 beside its three lines (+26 cold, +27 chaos, +16 fire and
lightning). Hate Nails, which I expected because it shows three
resistance words: outcome no_match; base and rarity matched;
`pseudo.total_res>=60` failed with the value 57 beside +27 chaos and
+15 fire and chaos.

Calls: 24 (3043 B), 25 (3147 B).

Right? Yes. What it does not print is the sum: 27 + 15 is 42, and 57
only if the two-element line counts twice; I had to work that out. The
definition is under `describe` `total_res`, which I did not ask for.

Stopped: nowhere; this is the tool at its best. I did not use the
`explain` view, which answers for a node over the whole scope rather
than for an item.

**A4 — one item again by its id**

Answer: three ways, all found Brood Paw. `search` with `id=<whole id>`
and realm pc (call 26, 1952 B): one row. `show` with the id (call 24):
the item whole, no realm asked. `get_item` (call 28, 1560 B): the
stored body, and the only place that printed where in the tab it lies
(`x` 9, `y` 8).

Calls: 26, 27 (refused), 28.

Right? Yes.

Stopped: call 27, `id:bb26c41155` with no scope, was refused for want
of a realm (447 B) though `show` and `get_item` find an id with none.
I did not then send the short id with a realm, so whether a cut item
id matches under `:` I do not know.

**A5 — +# to maximum Life at 90 or more, across everything**

Answer: 353 items, over both realms (call 29): 337 in pc Standard, 16
in pc Allflame, none in poe2. By class: 81 body armours, 69 helmets,
45 rings, 37 belts, 35 shields, 30 boots, 23 gloves, 19 quivers, 12
amulets, 2 two-hand axes. The tool adds, unasked, that 34 more items
reach 90 only by adding two such lines (`together`, call 29). In order
of the value I read the first 60 (calls 30, 31): two Kaom's Heart at
+1000 (Allflame "Special Gear"; Standard "AFK Tank (Remove-only)"),
five Kaom's Heart at +500, The Bringer of Rain at +286, +276, +212,
+209, the rare ring Damnation Whorl at +248, Wrath Sanctuary +207, two
Kaom's Roots +194 and +190, then rares from +185 down to +136 at the
sixtieth row.

Calls: 29 (5396 B), 30 (20887 B), 31 (20925 B).

Right? The count yes. The order I hold for 60 of 353; 293 are left
(call 31, `left_out`). At about 700 B a row the rest is ten more calls
of 30, or one call that would pass the door's 150,000 B; I stopped.
Some values look wrong for the game (a ring at +248, a chest at +207)
and I cannot tell from the tool whether the store or the item is odd.

Stopped: no view gives the ordered items thin — a name, a place and
the value — so an ordered list of 353 costs about 245 KB. `next` sent
back whole worked at the first try (call 31).

**X1 — what has left my stash and my characters, and when**

Told: six items, all from the character TheAbsenceOfPatience in
Allflame, all recorded as gone at 1788810111 (seconds since 1970; by
my own arithmetic 2026-09-07 19:41 UTC, the tool prints no date)
(call 39): the rares Dusk Grasp (Sorcerer Gloves), Grim Hold (Samite
Gloves), Rapture Hunger (Gutting Knife), Kraken Grasp (Ruby Ring),
Tempest Suit (Assassin's Garb), and a Jeweller's Orb. Nothing is
recorded as having left a stash tab. In the last 700 hours nothing was
removed; one talisman, Rune Gorget, moved from a character to a
Standard tab at 1789146342 (call 36).

Calls: 32 (2 B), 33 (2148 B), 34 (2853 B), 35 (333 B, refused), 36
(758 B), 37 (4869 B), 38 (4635 B), 39 (3651 B). Eight, of which two
were a subtraction and one a refused guess.

Right? The six agree with `scope.removed` 6 (calls 34, 39) and with
Allflame's 1318 against 1312 (calls 34, 22). What would say not:
`removed_at` equals the realm's `list_seen` and `oldest_fetch`
(1788810111, call 4), so it reads as when a fetch noticed, not when
the thing left; and 2941 places were never fetched (call 34), so what
left those is unknown. "Something I had is gone" may be none of the six.

Stopped, hard: nothing says how to ask for the removed. `membership:
all` adds them to every answer and prints `removed: 6` with no request
beside it (call 34); `is:removed` is refused with a list of flags and
no reading toward membership (call 35); `item_events` answers `[]` by
default with no word of its window (call 32), comes oldest first, takes
no kind, and so shows added rows until the limit (call 33). I found
the six only because call 37's container table had a `none` bucket of
5 that call 38's lacked — that a removed item lacks a container is
written nowhere I read.

## 3. Faults

1. **The removed cannot be asked for.** `scope.removed: 6` carries no
   request, against the instructions' "a term's or a table's count
   that is not zero holds the request that returns its members" — it
   is a scope count, so the sentence holds to the letter and fails the
   reader (call 34). `is:removed` is refused with no way on (call 35).
   The way that works, `-has:container` under `membership: all`
   (call 39), is an accident of the data. Should have said: under
   `removed`, the request for them; and in the refusal, a reading.
2. **`item_events` says nothing when it finds nothing.** `[]`, 2 B
   (call 32): no window, no count of what lies outside it, no time of
   the newest event. Its rows give a place as an id path and in two
   shapes — `stash/5ba2e1880a` (call 33) and
   `stash/pc/Standard/03cb479c65` (call 36) — and never a tab's name.
   It takes no kind and no order, so a removal behind a wall of added
   rows is out of reach by anything but guessing the window.
3. **The first step the instructions give reads items.** "Begin with
   search with no arguments, for what the store holds" leads, through
   the refusal's offer, to 20 rows and 10144 B (call 4) where the
   scope block was the whole of what was wanted. The offer should be,
   or sit beside, a counts view.
4. **A realm is demanded to find an id.** `search` `id:…` with no
   scope is refused (call 27); `show` and `get_item` find the same id
   with none (calls 24, 28). Learned by being refused.
5. **`line:` tables repeat and overreach.** Under `line:strength`,
   `line:dexterity`, `line:intelligence` and `line:attributes` the
   same computed buckets came four times, `pseudo.total_life` among
   them for the word "strength", and zero-count computed rows are
   printed (call 7, 23240 B).
6. **Every answer carries everything.** `basis`, `scope`, and for each
   term three counts each with a whole request: an answer of two rows
   was 7179 B (call 23). A row's `sort.shows` repeats its
   `matched[].shows` (calls 8, 23, 30).
7. **Where in the tab is only in `get_item`.** `show` is "one item
   whole" and prints no position; `get_item` printed `x` 9, `y` 8
   (calls 24, 28). A person asking where an item is wants it.
8. **`scope.leagues` names leagues that hold nothing.** Eight listed,
   three with items (call 22); the answer does not say the rest are
   empty or unfetched.
9. **A total's arithmetic is not shown.** 57 beside +27 and +15
   (call 25): the lines are printed, the weights are not.
10. **`next` holds a `token` the schema does not list.** `search`'s
    input schema has `request` and no `token`; "send it back whole"
    worked (call 31) but only by trusting the prose over the schema.

## 4. The tool's own words

Right, and used:
- The instructions' forecast of the first refusal and what to do with
  its offers (calls 2, 4).
- "describe whole is over 100 KB … name what is needed": I asked three
  times by section (calls 3, 5, 6) and never paid for `computed`.
- "No tool says what an item is worth" — answered Q6 before a call.
- `league`'s "league of origin is not a fact the search holds" —
  answered the hard half of Q4 before a call.
- `query`'s "A refinement is the old text in parentheses and a new
  term" — A2 in one call.
- `view`'s account of a sort's value, and `line(P).<slot>`'s "the
  largest that satisfies P" — exact, as call 21 showed.
- `show`'s `against` — exact (calls 24, 25).
- The `together` count on a line term (call 29): unasked, and the
  right thing to be told.

Wrong or missing:
- Nothing on how to reach removed items (fault 1); `scope`'s
  "each row marked with when it left" is true and is not a way to
  find them.
- `item_events`: "from recent ingests" does not say the default
  window is 24 hours measured from now, nor the order (fault 2).
- "a count-first look is {"view": {"counts": …}}" is in `search`'s
  text but the instructions' first step does not use it (fault 3).
- `show`: "its place" without saying position is left out (fault 7).
- Nothing says a time printed is to be converted by the reader; every
  time is bare seconds.
- Nothing says which of `search_items`/`get_item` and `search`/`show`
  an agent should prefer; the instructions list both sets as store
  tools.

Read and never used: the `explain` view; `holds( … )`; `undecided`;
`linked( … )` and socket colours; `--sum`; `context: corpus`; `count`
and `counted_at` with `before`; `query.tree`; `show`'s `body`; the
whole refresh, job and daemon paragraph of the instructions and the
tools `accounts`, `tabs`, `characters`, `store_status`, `search_items`,
`sync_policy`, `refresh_plan`, `daemon_status`, `list_jobs`,
`job_status`, `job_result`.

## 5. The answers' size

Every result was shown whole; none reached the door's limit. The
largest: call 16 (26090 B), call 7 (23240 B), call 15 (21328 B), calls
30 and 31 (20887 B, 20925 B), call 1 (20660 B).

What I read: the rows, the `total`, the tables' buckets, `resolved`
and `together` under terms, `scope` once and then only where it
changed (call 22, call 34).

What I never read past the first time: `basis` (in all 28 answered
search and show results), the `request` under each term's `failed`,
`lacked` and `matched`, the `token` strings, `query.tree`. I sent back
none of the per-term requests; I used a printed request three times
(calls 4, 31, and call 17's id from call 16's bucket).

Calls whose size cost something: call 7 — five `line:` keys at once,
most of it repetition (fault 5); call 16 — two 25-bucket tables whose
every bucket repeats a 120-character query text, to learn one tab's
name; calls 30 and 31 — 30 rows each at about 700 B a row, the reason
A5's order stops at 60 of 353.

## 6. What you would ask of the tool

- A request beside `scope.removed`, and a field or flag that selects
  the removed (calls 34, 35).
- `item_events` to take a kind and an order, and to say its window and
  the newest event when it returns nothing (calls 32, 33).
- A scope-and-counts answer as the first offer, not rows (call 4).
- A thin row: id, name, place and the sort value, no `matched`, no
  term block — for ordered lists (calls 30, 31).
- Leave `basis` and the per-term requests out unless asked (call 23).
- A row to show a named line without the line being a condition
  (call 9's `or true()`).
- Tab names by pattern or near-match, or a `tab` vocabulary like
  `line:` (call 14 missed "Levling Gear", call 16 found it).
- Position in the tab on `show` and on a row's place (call 28).
- `id` terms and `show`-like lookups to need no realm (call 27).
- The weights of a total beside its lines in `against` (call 25).
- A date beside each time (call 39).
- Computed buckets once per answer, and only where the count is not
  zero (call 7).
- For "legacy": nothing — the tool is right to hold no such notion;
  but a line's known range today would let an agent see an outlier
  (call 9, call 30's +248 ring).

## 7. Left out

Not tried: the poe2 realm's 98 items beyond counting them (call 22);
rows 61 to 353 of A5; the full per-tab table of A1 (15 of 297 tabs);
any row of Q3's 187; `describe` of `computed` or of `total_res`; the
`explain` view; a short item id with a realm named; `show` on one of
Q2's six amulets without the line, to see what they carry instead; the
second explode chest whole; the tools `tabs`, `characters`,
`accounts`, `store_status`, `search_items`, `sync_policy`,
`refresh_plan`, `daemon_status`, `list_jobs`, `job_status`,
`job_result`; the four the door refuses were never called, so I did
not see its refusal.

Files opened that are not the brief: none. I ran
`search/agent-seat/scripts/seat.py` without reading it and wrote this
file, appending to it between calls.

The one cut I would most want back: a call to `tabs`, to read every
tab's name for Q5. My answer there rests on a tab I met by chance, and
the list of names is the only thing that would say whether a fourth
leveling tab exists under a name with no such word in it.
