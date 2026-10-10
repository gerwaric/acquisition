<!-- hand-written during the run of BRIEF.md; call numbers are rows of mcp-seat-calls.csv -->

# Before any item

Calls 001–008. From `open` (001) and four answers that hold no item I
read (003, 005, 006, 007, 008; 004 printed 20 rows I did not ask for):

- One account, GERWARIC#7694, on the mock provider (001, 004).
- Two realms, and a search must name one: pc holds 22623 live items,
  poe2 holds 98 (004, 007). The store counts 22721 items, 6 removed,
  65 characters, 3444 tabs, 22751 events, 1126 responses, 4 refused
  bodies (008).
- pc: 546 places fetched, 2940 never fetched, 17 folders; leagues named
  are Allflame, Ancestors, Hardcore, Phrecia 2.0, Solo Self-Found,
  Standard, Ziz Rapture HCSSF Class Gauntlet, but items sit in only two:
  Standard 21311, Allflame 1312 (006). So every answer below is over
  what was fetched: most of the listed places were never read.
- pc by rarity: rare 5769, normal 3126, unique 3076, magic 2398, none
  8254 (006). By frame: rare 5768, gem 5705, normal 3464, unique 3071,
  magic 2398, currency 1720, divinationcard 490, supporterfoil 6,
  quest 1 (006). By container: items 21332, equipment 966, inventory
  165, jewels 137, guardian 23 (006).
- poe2: all 98 in Standard; magic 25, rare 16, normal 13, unique 1,
  none 43 (007).
- What may be asked (003, 005): text, name, typeline, base, note,
  rarity, frame, class (82), category (the trade site's ids), ilvl,
  reqlevel, stack, sockets, links, socket colours, price, league (the
  place now — "league of origin is not a fact the search holds"), tab,
  tab.type, character, container, id, `is:` flags, `has:`; lines by
  template with numbered slots, source and flags; computed totals
  (82, the trade site's pseudo stats); counts by any field or by
  `line`, crosses of two, sums.
- Already visible as limits: no league of origin (Q4), no notion of
  "legacy" among fields or flags (Q2, Q6), no date on an item.

# The thirteen

**Q1 — a rare to flesh out resistances or attributes (Titan Gauntlets)**

Answer: yes. You hold 12 rare Titan Gauntlets, all in Standard stash
tabs, none on a character (009); 11 carry a resistance or an attribute
(011). By total resistance (011): Brood Paw 85 (cold 26, chaos 27,
fire+lightning 16 crafted; Dump Tab); Morbid Touch 64 (cold 22,
lightning 42; Lab Runners); Hate Nails 57, +35 Dexterity (Cem Stuff 2);
Rune Knuckle 48 (Special Gear (Remove-only)); Glyph Touch 47 cold
(Market Items (Remove-only)); Corruption Fist 45, +25 Strength (Gear #1
(3.16)); Bramble Vise 36, +50 Strength (Decent Gear); Beast Vise 27,
+24 Dexterity and Kraken Knuckle 22 (both ~price 50 chaos (G));
Soul Grasp 18, +48 Strength fractured, +53 Dexterity (8 chaos
(Remove-only)); Hypnotic Vise +28 Dexterity, no resistance (~price 40
chaos (F)). No Intelligence on any (010).

Calls: 009, 010, 011 (three; 009 was half wasted).
Right? I hold it right for what was fetched. It would be wrong if
gauntlets sit in one of the 2940 places never fetched (011 scope), or
if a resistance reaches the item by a line no total reads.
Where the tool stopped or misled: 009 — I wrote the key
`line:resist,strength,…` as `describe` prints its example
(`line:resist,life`, 005) and the tool took the commas as part of one
text and answered `none: 12`, a clean table that says no such gauntlet
has a resistance. It did not refuse. In 011 a row sorted by a total
prints the total's value only under `sort`; the Strength and Dexterity
totals are not printed as numbers at all, only their lines.

**Q2 — a legacy Ashes of the Stars**

Answer: by your own test — the Reservation Efficiency line the modern
one lacks — yes, four of your ten (013). All four read "20% increased
Reservation Efficiency of Skills" and all are in Standard stash tabs:
one in "3.18 Poison Seismic Sab", one in "Build Gear 2", two in
"Special Gear" (014). The other six lack the line; three of the ten are
worn by I_EXIST, TheAbsenceOfPatience and mrs_flowers, and none of the
worn ones is among the four (013, 014).

Calls: 013, 014.
Right? The count of the line is right. That the line means legacy is
the person's word, not the tool's: nothing in the tool knows what the
modern item rolls. It would be wrong if another legacy trait (a value
range) marks one of the other six.
Where the tool stopped: no field, flag or computed value says legacy
(003). The vocabulary table (`line` over the ten, 013) is what made the
question answerable: the 4-of-10 split showed at once.

**Q3 — gear with six linked sockets**

Answer: 187 items in the PoE1 realm have a six-link, none in PoE2
(015, 016). 132 body armours, 30 bows, 9 two-hand axes, 7 staves, 5
two-hand swords, 3 warstaves, 1 two-hand mace; 101 rare, 76 unique, 7
magic, 3 normal; 181 in Standard and 6 in Allflame; 26 are worn by 21
characters and 161 are in 93 stash tabs, the most in Dump Tab (10)
(015). I would ask which slot the interaction wants before listing.

Calls: 015, 016. I read no row.
Right? Yes for "largest link group is 6" (003). Wrong if the
interaction needs six sockets linked on an item whose body the store
refused (4 refused bodies, 008).
Where the tool stopped: the tab table printed 40 of 93 and says
`left_out: 53`, `left_out_needs: "--limit"` — a terminal's flag in a
JSON answer; the field is `view.counts.limit`.

**A2 — Q1's rares with 60 or more total resistance**

Answer: two — Brood Paw 85 (Dump Tab) and Morbid Touch 64 (Lab
Runners) (012).

Calls: 012, one call: Q1's text in parentheses and
`pseudo.total_res>=60` after it, as `search`'s text says a refinement
is. Right: it agrees with the values 011 printed.
Where the tool stopped: nowhere, but the refinement is by retyping the
text; an answer holds no request for "this, narrowed". The term's own
row prints the value (`pseudo.total_res: 85`), which 011's `has:` rows
did not.

**Q4 — the Crucible staff someone offered real money for**

Answer: I cannot say which one. The tool does not hold where an item
came from ("league of origin is not a fact the search holds", 003) and
knows nothing of an offer. What it can hold is a crucible tree on the
item or a tab you named for the league. Seven staves have one or the
other, all in Standard (018): in "Crucible - Challenges (Remove-only)",
The Annihilating Light (Quarterstaff, tree with spell lightning damage,
cooldown recovery, +0.8% spell crit); in "Crucible - Duds
(Remove-only)", The Geomantic Gyre (Highborn Staff, "Socketed Gems are
Supported by Level 30 Unleash"), a second Annihilating Light, the rares
Vengeance Spire and Tempest Beam, and a normal Quarterstaff and a
normal Highborn Staff with trees. None carries a note or a price
(019). You hold 88 staves in all, 49 Staves and 39 Warstaves (017). If
you remember one line of it I can pick it out.

Calls: 017, 018, 019.
Right? The seven are right as "staff with a crucible tree or in a
Crucible tab". Whether the staff is among them I cannot know: it would
not be if it sits in a place never fetched, or if it left (X1).
Where the tool stopped: no league of origin; nothing dated on an item
in a search row; no memory of a trade. 018's "of the seven, which did
the Crucible tab term match" is not printed: a row lists only the
terms that have lines or values to show, so a row matched by
`tab:crucible` alone would show nothing for it.

**Q5 — my leveling gear**

Answer: two Standard tabs are named for it — "AFK & Leveling Gear
(Remove-only)" with 34 items and "AFK Leveling (Remove-only)" with 12
(020). I read all 46 (022). Low-level pieces there: Honourhome (helmet,
level 12), Astramentis (20), Lochtonial Caress (24), two Lifesprig
wands and a Perandus Signet with no level requirement, The Supreme
Truth (41), Maloney's Mechanism (45). The rest needs level 46 to 70:
Kaom's Heart, Rise of the Phoenix, Chitus' Apex, Ming's Heart, rares,
flasks and level-70 gems — gear for a levelled character that idles,
not for the first acts. Beyond those tabs, 1363 rare or unique weapons,
armours and accessories need level 40 or less, 885 of them unique, 117
in Allflame, spread over 147 tabs (021); I did not read them.

Calls: 020, 021, 022.
Right? Right as "what is in tabs you called Leveling". Whether that is
what you mean by leveling gear I do not know; the 1363 is a ceiling,
not an answer.
Where the tool stopped: "leveling gear" is the person's notion and the
tool has no word for it; the only handle was a tab's name. Sorting by
`reqlevel` put the three items that lack it last with `"status": "no
satisfying occurrence"` — for leveling those are the first, not the
last (022). 022 cost 21328 B for 46 rows with no line asked for.

**Q6 — what the legacy explode chest is worth**

Answer: I can find it and cannot price it. Two rare Vaal Regalia carry
"Enemies you Kill Explode, dealing 3% of their Life as Physical
Damage": Rage Curtain, worn by VVickedWitch, and Rapture Wrap in
"3.13 Golems, BL, Misc (Remove-only)" (023, 024). Rapture Wrap is item
level 85, crusader and hunter, six sockets in one group, 205 energy
shield, +85 crafted life; its price reads `"kind":"none"`, "nothing
applies" (025). Neither has a price of yours on it (023). The tool
holds no market value.

Calls: 023, 024, 025.
Right? The two are the only body armours whose line holds "explode"
(023). That this 3% line is the legacy one is my reading, not the
tool's. "Worth" is not answered.
Where the tool stopped: `price` is the owner's own listing only; no
tool here values an item, and nothing says so until one reads a
`price` and finds "nothing applies". `show` prints `item.lines: []`
beside a full top-level `lines` (025), which reads at first as an item
with no lines.

**Q7 — gear with increased Rarity of Items found**

Answer: yes, a great deal. In PoE1, 481 items carry "#% increased
Rarity of Items found", values 4 to 70 — 406 as an explicit line, 69
as an implicit, 24 as a flask's own line, 3 fractured (026). With the
variants (reduced 15, during Effect 9, per Rampage Kills 4, on Low
Life 3) it is 508 items: 189 rings, 115 helmets, 85 amulets, 33 flasks,
31 gloves, 28 boots, 10 belts, 9 rapiers and a few others; 344 rare,
134 unique, 21 magic, 9 normal; 490 in Standard, 18 in Allflame (026).
In PoE2, two worn rares: Sol Slippers 13% (pilatesinstructor) and
Kraken Ward 7% (flamdomrando) (027).

Calls: 026, 027. No pc row read.
Right? The counts are the tool's. Which of them the announcement
covers — explicit only, implicit too, uniques — is the person's to say;
the table splits them so it can be said.
Where the tool stopped: the sources add to 499 and the templates to
512 against 481 and 508 items, since an item is counted under each; the
answer does not say so beside the numbers.

**A1 — items per tab, per league, per rarity, before any item**

Answer: PoE1 holds 22623 items: Standard 21311, Allflame 1312; rare
5769, normal 3126, unique 3076, magic 2398, no rarity 8254 (028).
Crossed: Standard — none 7693, rare 5615, normal 2850, unique 2819,
magic 2334; Allflame — none 561, normal 276, unique 257, rare 154,
magic 64 (029). Per tab: 297 tabs hold items and 1291 items are on no
tab (on characters); the largest are ULTIMATUMS (Remove-only) 499, Maps
(Remove-only) 488, Gems 462, Flasks 461, Gems (Remove-only) 411,
Divination 385, ~price 4554 chaos 370 (028). PoE2 holds 98, all
Standard (007).

Calls: 028, 029 (and 006, 007 before them). No item read.
Right? Yes, as the tool's counts; the two tables agree with each other.
Where the tool stopped: per tab I have 15 of 297; the table says
`left_out: 282`. Each tab's bucket carries its whole request, so the
15 cost most of 028's 7608 B and all 297 would be a result I chose not
to ask for. A tab table has no compact form (name and count only).

**A3 — why one matched and one did not**

Answer: Brood Paw matched `rarity=rare base="Titan Gauntlets"
pseudo.total_res>=60`: each of the three terms `matched`, the last
with the value 85 and the three lines it was summed from (030). Hate
Nails did not: the first two terms matched and `pseudo.total_res>=60`
`failed` with the value 57 from "+27% to Chaos Resistance" and "+15%
to Fire and Chaos Resistances" (031). The definition says why 57: a
two-resistance line counts 2 × its number, so 27 + 2 × 15 (032).

Calls: 030, 031, 032.
Right? Yes; the arithmetic is the tool's own definition applied to the
lines it printed.
Where the tool stopped: nowhere. `show … against` is the clearest
thing in the tool. It prints the lines summed but not the weight each
took; the 2 × is only in `describe` (032). `describe` took the name
`total_res` though the entry's name is `pseudo.total_res`.

**A4 — one item again by its id**

Answer: Morbid Touch, id 157e6106…521ee, comes back three ways: `search`
with `id=<whole id>` — one row, Lab Runners (033); `show` with the id
(as 030, 031 did for others); `get_item` — the body as GGG gave it,
with what no search row told me: it is corrupted, and it sits at x 4,
y 13 of the tab (035).

Calls: 033, 034, 035.
Right? Yes.
Where the tool stopped: 034 — an id with no realm is refused,
`realm_needed`, though an id names one item and `show` and `get_item`
need no realm. `search` and `show` print an item's place down to the
tab and no further; the position in the tab is only in `get_item`'s
raw body or `show` with `body`.

**A5 — +# to maximum Life at 90 or more, across everything**

Answer: 353 items carry a "+# to maximum Life" line of 90 or more —
337 in Standard, 16 in Allflame, none in PoE2; 315 rare, 35 unique, 3
magic; 81 body armours, 69 helmets, 45 rings, 37 belts, 35 shields, 30
boots, 23 gloves, 19 quivers, 12 amulets, 2 two-hand axes (036). The
tool adds, unasked, that 34 more reach 90 only over several such lines
together (036). In order of value I read the first 50 of 353: Kaom's
Heart +1000 twice (Special Gear in Allflame; AFK Tank), Kaom's Heart
+500 five times, The Bringer of Rain 286, 276, the rare ring Damnation
Whorl 248, Bringer of Rain 212, 209, Wrath Sanctuary 207, Kaom's Roots
194 (worn by VVickedWitch) and 190, Spirit Shroud 185, Lioneye's
Remorse 176 … down to Vortex Trap +140 at row 50 (037, 038).

Calls: 036, 037, 038.
Right? The count is right for the line as displayed. The order is
right for the 50 read. 303 rows I did not read (038 `left_out`).
Where the tool stopped: the size. 25 rows cost 17956 B and 18040 B
(037, 038); the remaining 303 would be thirteen more such calls. A row
cannot be asked for thinner — name, place, value. The sort value I
guessed from the `together` request's spelling,
`line("# to maximum Life").arg1`; it worked, and nothing I had read
said how a line's number is written as a sort.

**X1 — what has left, and when**

Answer: the store records six items gone, all from one character,
TheAbsenceOfPatience in Allflame: the rares Dusk Grasp (Sorcerer
Gloves), Rapture Hunger (Gutting Knife), Grim Hold (Samite Gloves),
Kraken Grasp (Ruby Ring), Tempest Suit (Assassin's Garb) and a
Jeweller's Orb (042). Each is marked `removed_at: 1788810111`; Tempest
Suit was last seen at 1788314433 (044). By my own arithmetic, not the
tool's, those are 2026-09-07 19:41 UTC and 2026-09-02 02:00 UTC: they
left between the two, and the later is when the store looked, not when
they went. Nothing is recorded as having left a stash tab (040, 042).

Calls: 039, 040, 041, 042, 043, 044.
Right? The six are what the store holds as removed: `scope.removed: 6`
(040) and `items_removed: 6` (008) agree. It is not all that left: a
place fetched once cannot show a loss, and 2941 places were never
fetched (040).
Where the tool stopped or misled: `item_events` (039) is the tool
named for this and gave 50 `added` rows of one tab, oldest first, with
no filter by kind — 10666 B and no answer, with 22751 events behind it
(008). `is:removed` is refused (041) and membership takes only `live`
or `all` (043): the removed alone can be had only by noticing that they
lack a container (040). Times are epoch seconds everywhere.

# Faults

1. The instructions say "Begin with search with no arguments, for what
   the store holds" (001). It is refused: `realm_needed` (002). The
   first thing the tool tells an agent to do fails on this store.
   Should say: name a realm, or `{"scope":{"realm":"all"}}`.
2. `--count`'s example is `line:resist,life` (005). As one key in
   `keys` the commas become part of the text and the answer is a clean
   `none: 12` (009) — a wrong "no" where a refusal or a split was due.
   The examples under `counts` are a terminal's comma lists, not JSON
   keys.
3. `item_events` cannot be asked for one kind and starts at the oldest
   (039). For "what left" it returns `added` rows only. Should take a
   kind, or the `search` answer should offer the removed (below).
4. No way to ask for the removed alone: `is:removed` refused (041),
   membership `removed` refused (043). The route is an accident of
   `-has:container` (040, 042). 043's message — "this is no request and
   no route an answer printed" — is about something else; `kind` is
   `tree` and it has no readings or offers, against what `search`'s
   text says a refusal carries. 041 likewise has no `readings`.
5. A search by a whole item id is refused for want of a realm (034).
6. Messages and fields name a terminal's flags: `left_out_needs:
   "--limit"` (015, 017, 021, 028), `--realm pc` (002, 034). `search`'s
   text does map flags to fields, so this is a cost, not a block.
7. Sorting ascending by a value puts items lacking it last under
   `"status": "no satisfying occurrence"` (022); for a required level,
   lacking it is the lowest.
8. `show` prints `item.lines: []` beside the real `lines` (025, 030,
   031, 044).
9. Times are bare epoch integers: `removed_at`, `first_seen`,
   `last_seen`, `list_seen`, `newest_fetch` (042, 044, 004).
10. A row does not say the item is corrupted or carry any flag; I
    learned Morbid Touch is corrupted from `get_item` (035) after
    offering it in Q1 and A2.
11. `price` reads "nothing applies" (025) with no word that the tool
    holds only the owner's own listings and no market value.

# The tool's own words

Right, and used: "a term's or a table's count that is not zero holds
the request that returns its members, sent back as printed" (014, 042);
"a refusal is a result marked isError with readings and offers" (002
gave both offers and I sent one back, 004); "describe whole is over
100 KB" — I never asked for it; "A refinement is the old text in
parentheses and a new term" (012); `view.rows.next` "send it back
whole" (038); `show`'s `against` (030, 031); the flag-to-field map at
the end of `search`'s text; `scope.membership` "all, which adds removed
items, each row marked with when it left" (042).

Wrong: "Begin with search with no arguments" (002). `item_events`
"from recent ingests" reads as the answer to X1 and is not (039).

Missing: that the store may hold several realms and the first call must
name one; how a line's number is written as a sort value; that `keys`
is a JSON list where `describe` shows comma lists; what `pseudo.*`
names exist short of the 100 KB `computed` section — I found them only
because a `line:` table lists them (010); that rows carry no flags; any
word on value or worth; any word on what was never fetched, which
`scope` prints in every answer (`never_fetched: 2940`) and no text
explains; what `folders`, `fetched`, `list_seen` count.

Read and never used: `accounts`, `tabs`, `characters`, `search_items`,
`sync_policy`, `refresh_plan`, `daemon_status`, `list_jobs`,
`job_status`, `job_result`; the four refused tools; `search`'s `count`,
`counted_at`, `context: corpus`, `view.explain`, `view.counts.sum`;
`show`'s `body`; `describe`'s `linked`, `values`, `composition`,
`slots` as their own sections. Much of `open`'s 20090 B (001), by eye and unmeasured, is
the refresh and job tools, which no question touched.

# The answers' size

Read whole and used: every counts answer; the rows of 011, 012, 014,
018, 022, 024, 042; the `show`s.
Never read, though printed: the 20 rows of 004 (10144 B, asked for by
the offer I sent back, which carries `view.rows`); `basis` and `scope`,
repeated in full in every answer; each bucket's `request`, which
repeats the query, account, membership and realm — in 015 the 40 tab
buckets are most of 24223 B, in 021 25 are most of 18698 B; the `terms`
block of the whole store's counts on every refinement; the
continuation `token`, which repeats the request beside it in base64
(037, 038).
Calls whose size cost something: 037 and 038 (17956 B and 18040 B for
25 rows each) are why A5 stops at 50 of 353. 022 (21328 B, 46 rows).
028 is why A1 has 15 of 297 tabs. 039 (10666 B) bought nothing. No
result passed the door's 150,000 B.

# What you would ask of the tool

- A first call that works with no arguments on a store of two realms,
  or instructions that say to name one (002).
- The removed as a membership or a term of its own, and events by kind
  (039, 041, 043).
- A thin row and a thin bucket: id, name, place, value; count and value
  — with the request on demand (028, 037).
- An item's flags, corrupted first, on its row (035).
- Times as times (042, 044).
- The position in the tab beside the tab (035).
- Each summed line's weight beside it where a total is shown (031,
  032).
- `keys` refused, or split, when one key holds a comma list (009).
- A short list of the computed names without the 100 KB (010).
- A search by a whole id without a realm (034).
- A plain word, where `price` is none, that worth is not a thing the
  tool holds (025).
- "Lacking" first, or sayable, in an ascending sort (022).

# Left out

Not tried: `describe` whole or its `computed`, `linked`, `values`,
`composition`, `slots` sections; `view.explain`, `view.counts.sum`,
`context: corpus`, a count sent back with `count` and `counted_at`;
`show` with `body`; `tabs`, `characters`, `accounts`, `search_items`,
`sync_policy`, `refresh_plan`, `daemon_status`, `list_jobs`; any of the
four refused tools, so the door's refusal was never seen; realms xbox
and sony; the PoE2 realm for Q1, Q2, Q4, Q5, Q6; the rows of Q3's 187
and Q7's 508; the 1363 low-level items of Q5; the 34 "together" items
of A5; the other 282 tabs of A1; whether the Q4 staff is among the
removed (it is not: the six in 042 hold no staff).
Files opened that are not the brief: none. This report is the only
file written.
The one thing cut I would most want back: the other 303 rows of A5 in
order — the question asked for the items, and I gave 50.
