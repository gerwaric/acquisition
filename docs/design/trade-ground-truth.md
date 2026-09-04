# Trade Ground Truth

**Status: living research document, begun September 4, 2026.** This is
the companion to `network-ground-truth.md` for a different subject: not
how the Path of Exile API limits requests, but how items get *listed for
sale* — the trade site's two channels, the forum shop mechanism, the
in-game price dialog, and what the sanctioned API does and does not
report about any of it. The Rust spike's pricing rulings
(`spikes/rust-playground/decisions/pricing.md`: C67 the price value, C68
the currency table, C69 the listing state, C74 the shop render) rest on
these claims. When a claim here falls, every ruling that cites it falls
with it; that is the point.

Rules of this document (the same as `network-ground-truth.md`):

- **Claims are numbered T1, T2, … and never renumbered.** Designs,
  rulings and reference tables cite them by number.
- Every claim carries a **source tag** and a **confidence**. When better
  evidence lands, the claim is upgraded in place with the new citation —
  history stays in git.
- A claim states what the evidence shows and no more. An endpoint that
  returned no `forum_note` for one item is a claim about that endpoint
  and that item, not about the public-stash stream.
- Hypotheses and open questions live here too, clearly marked. An open
  question is one hand experiment or one read away; it is not acted on
  as if answered.
- GGG's own text is quoted verbatim. The owner's observations are quoted
  verbatim and marked as the owner's. The saved web pages the SITE and
  COMMUNITY claims were read from are kept locally by the owner and are
  never committed (the spike's `pricing-info/`, gitignored).

Source tags:

| Tag | Meaning |
|-----|---------|
| DOC | The official API reference (`https://www.pathofexile.com/developer/docs/reference`), a sanctioned surface; re-read September 4, 2026 |
| SITE | A dated browser read of the trade site (`https://www.pathofexile.com/trade`) by the owner, saved as a page; the saved copy is the observation. Under the spike's C79 the site is a governed surface: a human reads it, tooling never fetches it |
| OWNER | The owner's own observation in game or on the website, verbatim, dated |
| RUN | The spike's live run against the sanctioned API: the `LIVE-TESTING.md` run-ledger row plus the facts it landed, dated. Nothing here was fetched from a surface GGG does not sanction |
| CODE | What the C++ application on `master` and the third-party tool Procurement do — evidence of what worked once, never of what the forum requires |
| COMMUNITY | The PoE Wiki (`poewiki.net`), a third party's reading of GGG's surfaces; dated, corroboration only |
| HYP | Hypothesis — plausible, load-bearing, unverified |

Confidence: **Confirmed** (GGG's own text, or a direct observation of
the API's response), **High** (strong secondary evidence), **Provisional**
(acting on it, would like better evidence — a single in-game observation
by the owner, or a 2022 community page).

Dates: the price-notes run and the API re-read fall on September 4, 2026
UTC, which is the evening of September 3 in the owner's time zone. The
saved pages and the owner's in-game observations are dated September 3.

---

## Claims ledger

### The trade site

**T1. The trade site indexes two listing channels and no third: the
shop forums, and public premium stash tabs.** [SITE — Confirmed; About
page, saved September 3, 2026]
The About panel's first sentence, verbatim:

> "This website allows you to search through items listed for sale in
> Shop Forums as well as those listed using Public Premium Stash Tabs
> in-game."

Nothing on the page names another channel. The API reference's
`/public-stash-tabs` stream (T5) is how the second channel reaches
third parties; the forum is the only route for an item the stash
channel cannot see — a non-public tab, or a character's inventory (T8).

**T2. A bulk-exchange listing is written as a ratio,
`~price <wanted>/<lot> <currency>`, on the item offered, "either
in-game or on the forum"; the denominator is a lot size, not a
divisor.** [SITE — Confirmed; About page, saved September 3, 2026]
The "Bulk Item Exchange" section, verbatim:

> "Certain items may be listed for bulk sale (listed below). Your
> eligible items will be grouped together under one price, with the
> total amount of items presented as "stock". In order to list exchange
> entries, either in-game or on the forum, use the ratio format on the
> item you wish to offer for the type of item you would like in
> return."

The two worked examples, verbatim (each shows a stack of the offered
item with its stack size, then the note):

> For example: [a Chaos Orb stack of 10] `~price 3/1 alch` will list
> your intent to buy 3 Orbs of Alchemy for 1 Chaos Orb.
>
> Alternatively: [an Orb of Transmutation stack of 20]
> `~price 2/35 chaos` will list your intent to buy 2 Orbs of Chaos for
> 35 Orbs of Transmutation . Though this is technically equivalent to
> selling 17.5 Orbs of Transmutation per Chaos Orb, pricing this way is
> more appropriate for bulk sets of items.

So the grammar is `~price <wanted>/<lot of this item> <currency
wanted>`: the numerator counts the currency asked for, the denominator
counts the offered item per lot. GGG's own text says `2/35` is
deliberately not `1/17.5`; by the same token `3/1` is not `3`. A
consumer that reduces the pair or compares it as a rational loses what
the seller wrote (the spike's C67 keeps the pair unreduced).

The eligible items ("listed below" — the "Item Tags – *group*" panels)
are rendered at runtime from a list the saved page does not contain;
the capture holds the template only. Which items are bulk-eligible is
therefore **not established** here (Q10). Whether a ratio on an item
outside that list is ignored, listed singly, or grouped is open (Q6).

**T3. "Stock" counts every publicly listed item of that type, priced or
not.** [SITE — Confirmed; About page, saved September 3, 2026]
Verbatim: "Please note: Stock includes all of the items of that type
that are publicly listed (priced or not)." So an unpriced item in a
public tab is still *listed* for the purpose of stock; whether an
unpriced forum link is indexed at all is a separate question (Q5).

**T4. The trade site's realms are `pc`, `xbox` and `sony` (labelled
"PoE 1 PC", "PoE 1 Xbox", "PoE 1 Sony"); on September 3, 2026 each
realm carried the same eight leagues.** [SITE — Confirmed for that
date; the page's embedded options, saved September 3, 2026]
The saved page's option data lists three realms and, for each, the
leagues `Allflame`, `Hardcore Allflame`, `Ruthless Allflame`,
`HC Ruthless Allflame`, `Standard`, `Hardcore`, `Ruthless`,
`Hardcore Ruthless`. The site read was of the PoE 1 site; no `poe2`
realm appears on it. This matches N42's finding that the API reports
`poe2` as a realm the documented `pc | xbox | sony` enumeration lacks;
whether PoE2 has a trade site of its own was not read. The league list
is a snapshot: it changes every league.

### The API

**T5. `Item` carries both `note` and `forum_note`, each "user-generated
text"; `Item` is the object `/stash`, `/character` and
`/public-stash-tabs` all return; `StashTab.index` is an optional
unsigned integer; `StashTab.metadata.public` is "always true if
present".** [DOC — Confirmed; reference re-read September 4, 2026]
The reference's `Item` object lists, in this order and wording:
`note ?string user-generated text`, `forum_note ?string user-generated
text`. `StashTab` lists `index ?uint` and, under `metadata`, `public
?bool always true if present` and `folder ?bool always true if present`.
`PublicStashChange` (the `/public-stash-tabs` stream's element) carries
`public bool if false then optional properties will be null`,
`accountName ?string`, `stash ?string the name of the stash`,
`stashType`, `league`, and `items array of Item`. The stream is
documented as "all public stashes in all leagues for the given realm",
PoE1 only, under scope `service:psapi`.

What this does *not* say: which endpoints ever populate `forum_note`.
The field's existence in the shared `Item` schema is the only
documented trace of the forum channel in the API. See T6 and Q9.

**T6. `GET /character/{name}` does not report a forum listing: on
September 4, 2026 it returned the forum-listed, forum-priced,
site-listed body armour with neither `note` nor `forum_note`.** [RUN —
Confirmed for `/character` and this item; ledger row 2026-09-04,
response 543]
The run (`LIVE-TESTING.md` ledger row dated 2026-09-04; daemon pid
31313, 00:51–00:52 UTC) sent `GET /character/I_EXIST` and received 200
with 53 items. The owner had, on September 3, linked and priced that
character's body armour (Foulborn Skin of the Lords, `inventoryId`
`BodyArmour`, x 0, y 0) in a forum shop post (T7), and the trade site
lists it (T8). The item came back with no `note` and no `forum_note`;
no item in the run's three responses, and none in the spike's store,
carries `forum_note`.

The claim is about `/character` and one item. Whether
`/public-stash-tabs` carries `forum_note` for a forum-listed stash item
is untested and, for the spike, moot — it never reads that stream (Q9).
Consequence for the spike: a forum listing is intent it holds and text
it renders, never a fact it observes; "did my post index?" is answered
on the trade site by a human. The forum is write-only from the tool's
side.

### The forum shop mechanism

**T7. The website's item-link button emits a `linkItem` code that
carries `realm=`; once the post is saved, the forum has replaced the
code with `[item post="<post id>" index="<ordinal>"]` — the link is
resolved at post time into an item bound to the post.** [OWNER —
Confirmed for the shape the site emitted on September 3, 2026; one
post]
The owner, September 3, 2026: the website's link button for the
character item emitted, verbatim,

```
[linkItem realm="pc" location="BodyArmour" character="I_Exist" x="0" y="0"]
```

and once the post was saved, the post's edit view showed, verbatim,

```
[item post="26816146" index="1"]
```

So `realm=` is part of what the site emits today (the C++ application
was right to include it; the 2022 wiki and Procurement, T14 and T15,
omit it). And the text a poster submits is not what the forum stores:
the forum resolves each link when the post is saved, and the stored
form names the post and an ordinal, not a location. Consequences: a
tool cannot compare its rendered text with the forum's content;
"unchanged since last post" can only be a comparison with the tool's
own previous output (the C++ app's page hash). Whether the forum
re-resolves a linked item that later moves is open (Q8). The character
attribute was emitted with the character's display capitalization
(`I_Exist`) while the run's request path (T6) used I_EXIST; whether
the forum's match is case-sensitive is not established.

**T8. A character-inventory item is listable through the forum: the
item linked in T7 appears on the trade site as a listing.** [OWNER —
Confirmed for one item, September 3, 2026]
The owner, verbatim: the body armour on the character in the Allflame
league "shows up on the official site as a trade listing". Together
with T1 this is the forum channel doing the one thing the stash channel
cannot: listing an item that is not in a public premium tab. (The C++
user documentation makes the same claim in general terms; this is the
first time it was observed on this account with a dated post.)

### The in-game price dialog

**T9. The in-game price dialog offers 39 currencies; for each, the word
the game writes into `note` is fixed and is the site's spelling, not
the community abbreviation: `exalted`, `chrome`, `jewellers`, `fusing`
where the C++ table wrote `exa`, `chrom`, `jew`, `fuse`; `echor` is the
game's spelling. `chisel`, `coin` and `silver` are not offered.**
[RUN + OWNER — Confirmed for the words (the game wrote them); the
display names and the "not offered" list are the owner's reading of
the dialog on September 4, 2026]
The run fetched the owner's `ACQUISITION-PRICE-TEST` tab (`GET
/stash/Standard/03cb479c65`, response 542: a public premium tab with no
price in its name, 80 items, 50 with notes) after the owner had priced
one item in every currency the dialog offers. The 39 distinct words
that GGG's client wrote, and the dialog entry the owner chose for each
(one to one, checked against the notes; the spike's committed fixture
`crates/acquisition-plan/reference/price-notes-2026-09-04.txt` is the
verbatim corpus):

| word | display name | word | display name |
|------|--------------|------|--------------|
| `chaos` | Chaos Orb | `scrap` | Armourer's Scrap |
| `divine` | Divine Orb | `whetstone` | Blacksmith's Whetstone |
| `alch` | Orb of Alchemy | `gcp` | Gemcutter's Prism |
| `exalted` | Exalted Orb | `bauble` | Glassblower's Bauble |
| `alt` | Orb of Alteration | `offer` | Offering to the Goddess |
| `mirror` | Mirror of Kalandra | `offer-dedication` | Dedication to the Goddess |
| `chrome` | Chromatic Orb | `offer-gift` | Gift to the Goddess |
| `blessed` | Blessed Orb | `offer-tribute` | Tribute to the Goddess |
| `fusing` | Orb of Fusing | `lesser-ember` | Lesser Eldritch Ember |
| `jewellers` | Jeweller's Orb | `greater-ember` | Greater Eldritch Ember |
| `regal` | Regal Orb | `grand-ember` | Grand Eldritch Ember |
| `vaal` | Vaal Orb | `excep-ember` | Exceptional Eldritch Ember |
| `chance` | Orb of Chance | `lesser-echor` | Lesser Eldritch Ichor |
| `annul` | Orb of Annulment | `greater-echor` | Greater Eldritch Ichor |
| `aug` | Orb of Augmentation | `grand-echor` | Grand Eldritch Ichor |
| `regret` | Orb of Regret | `excep-echor` | Exceptional Eldritch Ichor |
| `scour` | Orb of Scouring | `facetors` | Facetor's Lens |
| `transmute` | Orb of Transmutation | `engineers` | Engineer's Orb |
| `wisdom` | Scroll of Wisdom | `infused-engineers-orb` | Infused Engineer's Orb |
| `portal` | Portal Scroll | | |

Against the C++ application's 19 tags (T15): twelve identical (`chaos`,
`chance`, `divine`, `alch`, `alt`, `scour`, `regret`, `mirror`,
`blessed`, `regal`, `gcp`, `vaal`); four spelled differently by the game
(`exa`→`exalted`, `chrom`→`chrome`, `jew`→`jewellers`, `fuse`→`fusing`);
three the dialog does not offer (`chisel`, `coin`, `silver` — the owner,
September 4: no longer in the game); 23 words the C++ table lacks. The
owner also reports having seen forum posts with `mir` for Mirror, so the
indexer's matching is looser than the dialog's output; how loose is not
established, and the spike does not model it (C68).

**T10. The shapes the dialog writes: `~price <amount> <word>` (exact),
`~b/o <amount> <word>` (negotiable), amounts as integers or decimals to
at least four places and to at least five digits, `~skip ` for "Do not
index", and `~price  <word>` — an empty amount, two spaces — left after
an invalid entry.** [RUN + OWNER — Confirmed for the shapes observed;
the meaning of `~skip` and of the empty amount is the owner's word,
September 4, 2026]
From the same tab, verbatim notes (the corpus is the fixture in T9):
`~price 999 <word>` (the usual form, 39 times, once per currency),
`~b/o 999 chaos`, `~b/o 1.5 divine`, `~price 1.4 divine`,
`~price 999.1 chaos`, `~price 999.12 chaos`, `~price 999.123 chaos`,
`~price 999.1234 chaos` (twice), `~price 12345 chaos`, `~skip ` (with a
trailing space), `~price  chaos` (empty amount, two spaces). Thirty of
the 80 items carry no `note`. No ratio note is present: the owner
reports the dialog does not offer the ratio on incubators (T12), and
the empty-amount note is what it left behind after one was attempted.

The owner, September 4, 2026: the empty amount "is what the game leaves
after an invalid entry such as a ratio on a non-bulk item"; `~skip` is
the game's "Do not index" choice; the five-digit and four-decimal
amounts are the game's own (whether a fifth decimal is truncated or
refused is not established). The prefix, one space, the amount, one
space, the word — that is the whole grammar the game wrote. A parser
that treats an empty amount as "no price" rather than "invalid" reads
the game's residue as a decision (the spike's C69 reports `invalid`).

### Tab prices and what beats what

**T11. A tab is priced only by renaming it in game; a public tab whose
name is a valid price lists every item in it at that price; a ratio in
a tab name is invalid and lists nothing.** [OWNER — Provisional; the
owner's in-game observations, September 3, 2026]
The owner, verbatim: "tab prices can only be set in-game by renaming
the tab"; and, on the tab named with a ratio: "The in-game tab prices
may be invisible, but every item in a priced tab shows up in the trade
listing unless the tab pricing is invalid. It looks like tab prices do
not support the 'X/Y' syntax." So the tab name is a price source with
two outcomes for the whole tab — valid, applying to each item, or
invalid, unlisting all of them — with no per-item residue.

The owner's own priced tabs (RUN, the same account's listing of
September 4: 13 tabs in pc/Standard, none of them public) carry
trailing text after the price, verbatim: `~price 30 chaos (C)`,
`~price 20 chaos (Remove-only)`, `~price 20 chaos (A) (Remove-only)`,
and so on — the `(Remove-only)` suffix is the game's own marking of a
remove-only tab. None of those tabs is public, so whether the site
tolerates the trailing text is not established here; the C++ parser
tolerated it. Whether an item's own `note` overrides its tab's name
price on the site is open (Q2).

**T12. A forum shop lists individual items, never whole tabs; the
in-game dialog offers the ratio format only on bulk-tradeable items; a
forum price takes precedence over a tab's in-game price.** [OWNER —
Provisional; the owner's in-game and website observations, September 3,
2026]
The owner, verbatim: "only individual items can be listed in forum
shops, not entire tabs"; "only bulk-tradeable items can use the 'X/Y'
format for price"; "items priced in forums take precedence over the tab
if the tab has an in-game price". The first agrees with the link-code
grammar (T7, T14: a link names one item by position); the second agrees
with T2's "eligible items"; the third is the first evidence about a
*relation* between the two channels and points one way only — a forum
price against an item *note* is untested (Q2), as is a forum price
against a game `~skip` (Q7).

### Item addressing

**T13. Every stash item's `inventoryId` is the literal `Stash1`,
whatever tab or substash it is in; a socketed item has no position and
no `inventoryId`; a character item's `inventoryId` is its slot; `index`
was present on every one of the 402 listed tabs.** [RUN — Confirmed for
this account; the spike's facts, listing of September 3, 2026 11:36
UTC]
The spike's census (`PRICING-SLICE.md`): 792 of 792 stash items carry
`inventoryId` `Stash1`; socketed items (24 in stash, 531 on characters)
carry neither `x`/`y` nor `inventoryId`; character items carry slot
names (`MainInventory`, `Weapon2`, `PassiveJewels`, `BodyArmour`, …);
`index` is present on all 402 tabs listed in pc/Standard (16 of them
folders, 82 of them substashes). The documented `index` is optional
(T5), so "present on all" is an observation, not a guarantee.

Consequence: the `Stash<n>` in a forum link code (T7, T14, T15) cannot
be read off the item; it has to be derived from the tab, and the only
candidate in the data is the tab's `index` (the C++ app uses
`index + 1`). Which tab `Stash<n>` names when folders and substashes
occupy indices is open (Q1), as is how a substash item is addressed at
all (Q3). Socketed items have no address and cannot be linked.

### Community evidence

**T14. Per the PoE Wiki's 2022 guide: the indexer reads the public
stash API and the trading subforums "a few times every minute", has a
thread limit (hence bumping), threads live in per-league shop
subforums, the thread title does not matter, the link code is obtained
by clicking an item in the website's stash view, and the price is
written after the code.** [COMMUNITY — Provisional; "Last updated on
March 7, 2022", page last edited December 1, 2024; saved September 3,
2026]
Verbatim, from the guide:

> "Pathofexile.com/trade gets its information through the Public stash
> tab API, as well as from the forums (particularly the trading
> subforums). The forums have a system to link items from your game
> stash to your posts, which helps verify if the item is still
> available, in the same way the stash tab indexer does. The items can
> then be priced by appending the correct syntax after the code that
> links the item."
>
> "A few times every minute, pathofexile.com/trade look at any posts on
> the forum, and updates the index with any listed items. The indexer
> has a limit to how many threads it searches, which is why it is good
> practice to regularly 'bump" the thread, or post a comment to have it
> appear higher in the subforum."
>
> "Go to the Standard League - Shops subforum. Create a thread. The
> title of the post does not matter."
>
> "Click only one time on the item you want to sell. A code, looking
> like this one, will be copied to the text of your forum post:
> `[linkItem location="Stash2" league="Standard" x="5" y="0"]`. If you
> preview your post, you'll see the item picture and a proper link in
> your post."
>
> "After the code, you can tell the indexer what price you want to sell
> the item for. For example, the syntax for a buyout (b/o) of 3 Chaos
> Orb … would be `~b/o 3 chaos`."

The 2022 link code has no `realm=`; the site emits one today (T7). The
polling cadence, the thread limit and the bumping practice are the
wiki's account, not GGG's, and are dated; nothing in this file rests on
them beyond noting that publishing would have to own bumping.

### What the code did

**T15. The C++ application and Procurement emit the same two link
shapes — `[linkItem location="Stash<index+1>" league="<L>" x="<x>"
y="<y>"]` and `[linkItem location="<inventoryId>" character="<name>"
x="<x>" y="<y>"]` — the C++ app with `realm="<r>"` appended and
Procurement without; the C++ app groups items under a spoiler whose
title is the price, Procurement writes the price on the line after the
code; the C++ post limit is a 50,000-character constant; the C++ app
never posted a game-sourced price.** [CODE — what worked once; C++
`master` at 33928a87, Procurement at its last commit (December 22,
2022), read September 3–4, 2026]
The C++ app (`src/itemlocation.cpp`, `GetForumCode`): the stash form
takes the tab index and writes `Stash<index + 1>`; the character form
writes the item's `inventoryId` as `location` and the character's
name; both end with `realm="<r>"`, the realm from settings.
`src/shop.cpp`: items sorted by buyout, each run of equal buyouts
wrapped as `[spoiler=" ~b/o 3 chaos"]…[/spoiler]` (the prefix map
carries a leading space: `" ~b/o "`, `" ~price "`, `" ~c/o "`; the
value is formatted with up to 15 significant digits; a no-price row
gets an empty title); pages cut at `kMaxCharactersInPost = 50000`,
each page substituted for the one `[items]` token of the shop template
and wrapped in a plain `[spoiler]`; the page set is hashed (MD5) to
skip an unchanged post. `src/buyout.cpp`: `IsPostable` is `source !=
game && (priced || no_price)` — a price read from a tab name or an item
note was never written to the forum; `src/currency.cpp` holds the 19
tags (T9 compares them). The C++ user documentation states the forum's
purpose as listing "items in remove-only tabs and character
inventories, which the site does not index from your stash directly".

Procurement (`Procurement/ViewModel/ForumExportVisitors/VisitorBase.cs`):
the same two shapes without `realm=`; the `location` is the item's own
`inventoryId` as the legacy website stash endpoint reported it (under
T13 the OAuth API's value, `Stash1` for everything, would not serve);
the price on the following line as `~b/o <v>`, `~c/o <v>` or
`~price <v>`; tab-wide buyouts keyed by tab **name**; a currency map of
16 abbreviations (`chrom alt jew chance chisel fuse alch scour blessed
chaos regret regal gcp divine exa vaal`), a strict subset of the C++
19 (no `coin`, `mirror`, `silver`). Its last release is 1.29.2 and its
last commit December 22, 2022 (GitHub, read September 4, 2026).

None of this is evidence of what the forum requires: two tools agreeing
on a shape shows the shape was accepted at the time each was written.
The 50,000 figure is a constant, not a measured limit (Q4). The spoiler
form and the next-line form are both folklore about where the indexer
looks for the price; the wiki's "after the code" (T14) and the site's
own bulk examples (T2, a note on the item) are the only GGG-adjacent
statements.

---

## Open questions

Each is one hand experiment by the owner or one read; the render's
policy table (C74) reports the cell as blocked-and-counted until it is
answered.

- **Q1. `Stash<n>` numbering under folders and substashes.** Is `n`
  the tab's `index + 1` when folders and substash children occupy
  indices (T13)? Experiment: link one item from a tab past a folder,
  preview the post, and see which item picture appears. Decides whether
  the first rendered page is trustworthy.
- **Q2. Item note versus tab name in game.** Rename the test tab to a
  valid price and see whether the noted items keep their own price on
  the site (T11). The spike's note-then-tab order rests on the C++ code
  until then. Also: a forum price against an item note (T12 covers
  forum versus tab only).
- **Q3. The link code for a substash item** (a child of a map or unique
  tab). Experiment: click one in the website's stash view and record
  what it emits.
- **Q4. The real post size limit.** 50,000 is the C++ constant (T15);
  the forum's actual limit was never measured.
- **Q5. Is an unpriced forum link indexed?** T3 says an unpriced item
  in a public tab counts as listed; the forum case is untested.
  Experiment: one link with no price, then the seller-account search.
- **Q6. A ratio on a non-bulk item posted to the forum: ignored,
  listed singly, or grouped?** And `~b/o a/b` against `~price a/b` (T2
  shows only `~price`). In game the dialog refuses the first case (T10,
  T12); the forum is a free text field.
- **Q7. A game `~skip` against a forum price.** Does the forum price
  list an item the game marked "Do not index" (T10, T12)?
- **Q8. Does the forum re-resolve a linked item that moves after
  posting?** T7 shows the link is resolved at post time into
  `[item post= index=]`; whether that binding follows the item or
  breaks is unknown. Experiment: move a linked item, then check the
  post's preview and the site.
- **Q9. Does `/public-stash-tabs` carry `forum_note`?** T6 rules it
  out for `/character`; the stream is the one endpoint left. Moot for
  the spike (it never reads the stream); answerable by anyone with the
  `service:psapi` scope.
- **Q10. The bulk-eligible item list** (T2's "listed below"). A browser
  read of the expanded "Item Tags" groups; parked in the spike until a
  ratio on a non-currency item appears.
- **Q11. Does the seller-account search show items from non-public
  priced tabs?** The owner's 13 priced tabs are all non-public (T11);
  T1 says the site cannot see them from the stash. The search, run in a
  browser for this account and league, is the oracle for the listing
  state as a whole.

---

## Appendix — Related registers

- `docs/design/network-ground-truth.md` — the N-claims (rate limiting,
  endpoint policies, realm and character shapes). T4 and T13 lean on
  N42 and N43.
- `spikes/rust-playground/decisions/pricing.md` — the rulings that cite
  these claims: C67 (the price value and its lot ratio: T2, T10), C68
  (the currency table's `emit` column: T9), C69 (the game side's four
  outcomes: T10, T11), C74 (the render's link code, channels and policy
  rows: T1, T6, T7, T8, T11, T12, T13, T15).
- `spikes/rust-playground/SURFACES.md` — the register of governed
  surfaces under C79: the trade site, the forum, the wiki and
  Procurement, each with the access method its claims here were read
  under.
- `spikes/rust-playground/crates/acquisition-plan/reference/price-notes-2026-09-04.txt`
  — the verbatim note corpus behind T9 and T10, committed as a parser
  fixture.
- `spikes/rust-playground/LIVE-TESTING.md` — the run ledger; the row
  dated 2026-09-04 is T6, T9 and T10's run.
