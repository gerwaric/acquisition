<!-- hand-kept: the owner's words, recorded verbatim in conversation on 2026-09-13; the reading columns are the agent's, confirmed where the owner did not correct them -->

# The owner's questions of the stash

One row per question as the owner said it. *Wants back* and *Today* are the agent's reading of the answer shape and the current route; the owner refined Q1 and did not correct the rest (2026-09-13). *Needs* names the facts and derivations the answer depends on and the track that holds them.

| # | Question, verbatim | Purpose | Wants back | Today | Needs |
| --- | --- | --- | --- | --- | --- |
| Q1 | "Do I have a rare that I can use to flech out the resistances or attribute requirements for a build I'm testing?" | gearing a build | a short list of candidates for one slot, with their resist and attribute lines visible, to pick from | C++ Mods filter with pseudo totals, one slot at a time via Category | pseudo totals over displayed values (cpp-search F3, trade-query F8); class/slot (repoe F3); a specific mod by identity — a fractured mod, a build-enabling mod, a defensive-layer stat (repoe F4, the identity question) |
| Q2 | "Do I have a legacy version of a specific unique item?" | find one remembered item | one item or "no", and where it is | Name filter, then read the mod values by eye to tell legacy from current | name; mod values against the current version's ranges (repoe `mod-stat-index.csv` / unique data, not yet extracted) |
| Q3 | "I just read about a new interaction someone found and I want gear to test it out." | gearing a build | a list, defined by whatever the interaction needs: a mod, a base, socket colours, a unique | Mods filter if it is a mod, Name if a unique | a mod by identity, base, sockets and colours, name |
| Q4 | "Where is that staff from Cruicible league someone tried to pay me real money for?" | find one remembered item | one item and its tab | remembered name, or scrolling tabs by hand; no "from Crucible" filter exists | league of origin — not a field the item carries (item-facts F2); crucible mods (`crucibleMods`, unread by the C++ app); class |
| Q5 | "I want to practice leveling, so I need to find my leveling gear." | gearing a character | a set of items across tabs, by level bracket | by tab name if a leveling tab was kept; otherwise R. Level max and by eye | requirements level (cpp-search F2), tab, name; possibly the uniques known as leveling gear |
| Q6 | "What is that legacy explode chest I have worth?" | price one item | a price — outside the stash search — after finding the item and knowing it is the legacy version | Name filter, then the trade site by hand | as Q2; then a trade query built from the item (trade-query F9, prior-art) |
| Q7 | "Do I have any gear with the modifier GGG just anounced is going away except on Standard?" | a sweep across everything | a list across all tabs and characters, item and tab, to decide keep or sell | Mods filter with the exact template, only if the mod normalises to one template | a mod by identity across every array, every tab, every character (item-facts F3, repoe F4); realm and league coordinates (store) |

## Refinements, verbatim

On Q1 (2026-09-13): "Sometimes a specific item will have a specific stat, like a crafting base with a specific fractured modifier, or a key build-enabling modifier, or a stat that is needed to complete a defensive layer."

"Usually well-crafted endgame gear is already on bases with the right armour or evasion, so I don't worry about those numbers as much."

"I don't play melee builds, but people who do care very much about the modifiers on their weapong, and many builds care about other damage mods--sometimes they are hit-based, sometimes they are spell-based, sometimes they are damage over time, sometimes they are other ailments."

## Pricing and listing, verbatim (prompt 2, 2026-09-13)

"I don't use acquisition for managing sales, so I don't have much feedback. There are a small number of users who depend upon this however, but I don't know their workflows. I have mainly been maintaining the c++ price management features and fixing the occasionally reported bug. I do know those users care about keeping buyouts between version upgrades and being to update forum shops. However, I do not want acquisition updating forum shops directly. Those users can have their agents drive their web browsers, or maybe we re-enable this feature later, but I don't want to support direct updates out of the gate because of the use of POESESSID."

So no pricing question of the owner's own goes in the table; the pricing consumer is other users, whose workflows are unknown, and their two known needs are already rulings (buyouts survive upgrades: C35; no direct forum updates: the parked publishing item in `CONTEXT.md`, now with the POESESSID reason).

## Clearing and organising, verbatim (prompt 3, 2026-09-13)

"I generally don't use acquisition to manage my stash at all, because it cannot perform actions like move items or rename tabs. However, knowing counts and total of different currencies, equipable items, and other non-equipable items such as maps and fragments might be useful, or at least interesting."

So no organising question goes in the table: the app cannot act on the stash (the API is read-only for stashes), and the owner manages it in the game. What survives is one candidate, counts and totals by kind — currencies, equipment, maps, fragments — as a "useful, or at least interesting" derivation, not a need (cf. the parked currency-totals item in `decisions/pricing.md`, "when I asked for it").

## The trade site against the stash, verbatim (prompt 4, 2026-09-13)

"The trade site lets me select yes/no/any for binary flags. It's stat field's autocomplete distinguishes implicit, explicit, pseudo, fractured, and other modifier types. The stat field autocompelte is also almost instantly responsive to keystrokes and lists autocomplete options that makes sense in an order that makes sence. The c++ mod search box is terrible by comparison on both fronts. The c++ does not allow for complex boolean searches such as (A or (B AND C)). The trade site allows that kind of logic, but only for stat modifiers, not for any of the other search fields, so I can't ask for something like "(Armour > 1000) OR (Required Level < 80)"."

## Anything else, verbatim (prompt 5, 2026-09-13)

"Being able to use a trade search query against my stash would be fantastic, especially if there was an integrated way to make this happen--e.g. with a simple browser addon, or even a basic copy/paste. There's already a browser extension called Better Trading that people use to manage trade searches. Integrating with that might be fun, but not a core features."

"Similarly, integrating with Awakened PoE trade to price items from within acquisition somehow (gui? cli? mcp?) would be useful. Some newer players have asked for this."
