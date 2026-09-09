# 16 — The site-as-the-oracle claims, drafted for the master-side PR

Disposable. Claims are authored on the master-side branch in
`docs/design/trade-ground-truth.md` and cherry-picked here, never the
reverse (`AGENTS.md`); this note is the paste-ready text for that PR,
written 2026-09-09 from `PRICING-SLICE.md`, "What the site taught"
(1–16), the two committed tables (`reference/site-listings-2026-09-08.json`,
`…-09-09.json`), and the run-ledger rows of 2026-09-08/09. Numbers
continue from T24. Dates are UTC where a ledger row gives one; the
owner's evening of September 8 is September 9 UTC for the runs.

Once merged and cherry-picked, the closed record cites the numbers
and this note is deleted.

## Rewrite

**T17. The in-game price dialog stores a note whole and displays only
the part it parsed: `~price 5 chaos (A)` is shown as `~price 5 chaos`,
served by the API as typed, and read by the trade site through the
suffix.** [OWNER — Confirmed; September 8, 2026, superseding the
September 6 reading]
The owner, verbatim: "In game the pricing note is exactly '~price 777
chaos' but on the trade site it shows as 'Price with Note' of 777
chaos with the text '~price 777 chaos testing' rendered in the web
UI. I have confirmed both of these things are true at the same time.
[…] I confirmed this by using the UI to set the item's note to
'~price 666 chaos tested'. In game this turned into '~price 666
chaos' as soon as I looked at it. On the trade site it's listed as
'Price with Note' just like the 777 case." The spike's facts held the
whole text both times (`~price 777 chaos testing` on September 8,
`~price 666 chaos tested` on September 9). So the September 6 reading
("the dialog strips trailing text") was the display, not the store;
a parser must read a note as it reads a tab name — the word after the
amount, anything after tolerated (the spike's parser v2, C69).

## Amend

**T11**, add after the ratio sentence: *A ratio in a public tab's name
lists nothing on the item search, and offers the tab's exchange-eligible
stacks on the bulk exchange at the ratio* (see T27). [RUN — Confirmed;
September 8–9, 2026] Two public tabs, `~b/o 5000/2 chaos` (56 items)
and `~price 1000/2 chaos` (70), put two rows on the item search — the
two items carrying their own `~price 1999 chaos` note — and the first
tab's Infused Engineer's Orbs and Tailoring Orb on the exchange as "2
for 5000 chaos". `~b/o` with a ratio is accepted in a tab name.

## New claims

**T25. The seller-account search shows a stash item only from a tab
whose listing entry carries `metadata.public: true`; a priced name on
a non-public tab reaches nothing.** [RUN — Confirmed; September 8,
2026] 547 stash rows for the account in Standard, every one from one
of the 13 tabs the API listed public; the 13 non-public priced tabs
(`~price 30 chaos (C)`, twelve `(Remove-only)` names, 1,121 items) put
nothing on the site, nor did 114 price notes in non-public places.
Closes Q11.

**T26. A socketed item is never listed on its own; a currency-class
stack is not in the item search; a stack in a priced public tab, or
carrying a price note, is offered on the bulk exchange instead.**
[RUN — Confirmed; September 8–9, 2026] 107 gems in sockets of items in
public tabs, none on the site (24 of them in a tab priced 8888 chaos);
eight stacks the state expected absent from the item search, three of
them offered on the exchange: the test tab's Scroll of Wisdom at its
note's 1.5 divine, the ratio tab's two stacks at the ratio.

**T27. The bulk exchange keys one row per item type per account, every
public stack's offer under it, shown per unit; "stock" counts every
public stack of the type, priced or not.** [SITE — Confirmed;
September 8–9, 2026] The Scroll of Wisdom row carried two offers — 1.5
divine (a stack noted so) and 8888 chaos (a second stack in a tab
named so) — with stock 40, the two stacks' 21 and 19 together (T3
confirmed); the ratio tab's "2 for 5000 chaos" is shown as 2500 for 1
in the compact layout.

**T28. The trade site reads a price out of a note that carries text
after the currency word, and labels the listing "Price with Note",
showing the whole text.** [SITE — Confirmed; September 8 and 9, 2026]
`~price 777 chaos testing` on an item in a tab priced 8888 chaos
listed at 777 chaos under the fourth label (the others: "Exact
Price", "Asking Price", "No Price Set"); `~price 666 chaos tested` the
next day likewise.

**T29. An item's own note beats a valid tab price on the site.**
[SITE — Confirmed; September 8–9, 2026] The same item, in the tab
named `~price 8888 chaos`, listed at its note's 777, then 666; the
other 31 unnoted items of the tab at 8888. Closes Q2 in the note's
favour (the C++ order the spike's C69 kept).

**T30. A priced public unique tab lists its substashes' uniques at the
tab's price; a folder named with a price lists its public child's items
unpriced; a priced public tab inside a folder lists at its own price.**
[RUN + SITE — Confirmed; September 9, 2026] `Uniques 1`, made public
and named `~price 4554 chaos`: 326 of its 370 uniques in 19 substashes
listed at 4554 chaos (the rest were the capture's paging, the owner's
check). Folder `3.19` named `~price 3333 chaos` with `3.19 Cursebot`
public and unpriced inside it: its items "No Price Set". `3.19 Helix
Raider` named `~price 2222 chaos` inside that folder: 9 of 9 at 2222.
Confirms the spike's C80. A remove-only unique tab accepts a price
name and cannot be public.

**T31. The site drops the fractional part of a chaos amount in its
display and keeps it for divine.** [SITE — Confirmed for those two
words; September 8, 2026] `999.1234`, `999.123`, `999.12` and `999.1`
chaos each show "999"; `1.4 divine` shows "1.4".

**T32. The indexer resolves the game's eldritch shorthand to the
site's long ids, and does not read `facetors` at all.** [SITE —
Confirmed; September 8, 2026] `excep-ember`, `grand-ember`,
`greater-ember`, `lesser-ember`, `excep-echor`, `grand-echor`,
`greater-echor`, `lesser-echor` list under `exceptional-eldritch-ember`
… `-ichor` at the right price (T16 widened by eight); `~price 999
facetors`, the word the game's own dialog writes, lists as "No Price
Set". The owner reported the latter to GGG as a bug (September 8) and
does not price in that word.

**T33. The bulk exchange's item list, by group.** [SITE — Confirmed
for 15 of 22 groups; September 8, 2026] The site's "Items I Want"
panel, expanded and saved: 750 ids with display names in Currency
(101), Fragments/Scarabs/Mapping (232), Ducats, Enshrouding Crystals,
Foulborn Currency & Wombgifts, Allflame Embers, Runegrafts (31),
Tattoos & Omens (113), Expedition Currency, Delirium Orbs, Catalysts,
Oils & Extractor, Fossils & Resonators, Essences (105), Maps (54);
seven groups not expanded (Sanctum, Heist, Beasts, Cards, Shaper/Elder
maps, Unique maps, Legacy). Vials are not on it. The list is
`reference/exchange-items-2026-09-08.json` in the spike, a proposal
under its C68. Answers Q10 for those groups.

**T34. A forum-listed item shows on the item search with the thread as
its seller link, and a forum-listed exchange-eligible stack on the
exchange; the site dates a forum listing by something other than the
post.** [SITE — Confirmed; September 8, 2026] Four items from
non-public remove-only tabs listed "Asking Price" with
`/forum/view-thread/<n>` links from two threads, shown "listed last
month" and "2 months ago" for posts days old; five Runegrafts and a
Wombgift from the same tabs offered on the exchange at 4321 blessed.
T8 and T12 at nine rows.

**T35. A refresh token can be invalidated between two client sessions
by events on GGG's side; `invalid_grant` ("Refresh token doesn't exist
or has expired") is a state a client must expect, remedied only by a
new login.** [OWNER + RUN — Confirmed; September 9, 2026] The token
rotated and saved at 17:34 UTC on September 8 was rejected at 01:53
UTC on September 9 with no use in between; the owner, verbatim: "the
game servers were rebooted recently, which is one of the events that
can invalidate tokens (there are others. we don't need to catalog
them, but it can happen)". (Belongs beside the token claims in
`network-ground-truth.md`, not here — same PR, other file.)

## Open questions to close

- **Q2** — closed by T29 (a note beats a valid tab name; the unreadable
  case stays T18).
- **Q10** — closed for 15 groups by T33; the seven unexpanded groups
  stay open in the claim's own words.
- **Q11** — closed by T25.
- **Q6** — half: `~b/o a/b` is accepted in a tab name (T11 amended);
  the forum half stays open.
