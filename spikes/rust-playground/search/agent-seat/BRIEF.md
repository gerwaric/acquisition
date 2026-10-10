# agent-seat — brief for the MCP seat (step 11c, 2026-10-10)

You are an agent a person has handed a tool to. The person keeps a
large collection of items from a game, Path of Exile, in a store on
this machine; the tool is an MCP server, `acq-mcp`, that reads it. They
will ask you thirteen things. The tool is what is under test, never you:
where it fails you, that is the finding, and a question you could not
answer is as good a result as one you could.

## Rules that bind this run

- **This file is the only one you read.** The repository's entry
  points (`AGENTS.md`, `CLAUDE.md`) list documents to read before any
  work: for this run they do not apply. Open no other file here — no
  document, no source, no test, no other file of this directory, no
  git history — and search for none. Everything you learn about the
  tool you learn from the tool.
- **The tool is reached through one command and no other:**

      python3 search/agent-seat/scripts/seat.py open
      python3 search/agent-seat/scripts/seat.py call --q <id> <tool> '<json arguments>'
      python3 search/agent-seat/scripts/seat.py call --q <id> <tool> - <<'J'
      <json arguments>
      J

  `open` prints what a host shows an agent before its first call: the
  server's instructions and its tools. `call` sends one call and prints
  the result. `--q` is the question the call is for (`O`, `Q1` … `Q7`,
  `A1` … `A5`, `X1`); `--note '…'` says why you made it, when that is
  not plain. Run it from the repository root. Run nothing else against
  the store: no `acq`, no `sqlite3`, no `cargo`.
- **Every result is read as it is printed.** No pipe, no redirect, no
  filter after the command, no file to keep a result in: a host gives
  an agent the whole result and nothing else. A result over 150,000
  bytes is logged and printed as its first 2,000 characters with a line
  saying so; that is the door's rule and not the tool's.
- Four tools the door refuses — `set_sync_policy`, `apply_plan`,
  `submit_job`, `cancel_job` — since this seat reads. Every other tool
  the server lists is yours to call.
- You write one file, `search/agent-seat/data/mcp-seat.md`, and nothing
  else. Never commit, push, fetch, build or spawn an agent.
- No number of calls is a target. A right answer in many calls and a
  wrong one in few are both worth knowing; say which you have. Leave a
  question when your next call would be a guess you have already made,
  and say where the tool stopped you.

## What to do

1. `open`. Then **O**: before any question, say what you can learn of
   this collection and of what may be asked about it without reading a
   single item.
2. The thirteen, in the order below. For each, answer the person as you
   would in a conversation — the items, their places, the counts, or
   that there are none — and then say what it took.
3. Write the report as you go, not at the end.

## The questions

The first seven are the person's, in their words. Where a question
leaves the instance open, the line under it fixes one.

- **Q1** "Do I have a rare that I can use to flesh out the resistances
  or attribute requirements for a build I'm testing?"
  The person, when asked: "When I'm asking for specific mods, I likely
  have a base in mind--either something specific like Titan Gauntlet or
  Spiked Gloves, or I do care about the base and it's attributes." Take
  Titan Gauntlets.
- **Q2** "Do I have a legacy version of a specific unique item?"
  The person, on what legacy means: "Legacy means an item or modifier
  or modifier value or unique or unique variant that cannot be found or
  created by playing the game, so it only exists in Standard stashes.
  An example is the Ashes of the Stars onyx amulet, which has a legacy
  variant from 3.23.0 which has the modifier 10-20% increased
  Reservation Efficiency of Skills that modern version lack." Take
  Ashes of the Stars.
- **Q3** "I just read about a new interaction someone found and I want
  gear to test it out." Take: the interaction needs an item with six
  linked sockets.
- **Q4** "Where is that staff from Crucible league someone tried to pay
  me real money for?"
- **Q5** "I want to practice leveling, so I need to find my leveling
  gear."
- **Q6** "What is that legacy explode chest I have worth?"
- **Q7** "Do I have any gear with the modifier GGG just announced is
  going away except on Standard?" Take: the modifier is increased
  Rarity of Items found.

Five are an agent's own:

- **A1** How many items are there per tab, per league, per rarity —
  before reading any item?
- **A2** Of Q1's rares, only those with 60 or more total resistance —
  without asking Q1 over again from nothing.
- **A3** Why did an item match, and why did one not? Take one item an
  answer of yours held and one you expected and did not get.
- **A4** Find one item again by its id.
- **A5** Across everything: which items have +# to maximum Life at 90
  or more — how many, and then the items in order of the value?

And one more of the person's:

- **X1** Something I had is gone. What has left my stash and my
  characters, and when?

## The report — `search/agent-seat/data/mcp-seat.md`

Its first line: `<!-- hand-written during the run of BRIEF.md; call
numbers are rows of mcp-seat-calls.csv -->`. Then these headings and no
others. Every claim cites a call by the number the door printed; a
size or a count is the one the door or the tool printed, never one you
remember.

1. **Before any item** — O.
2. **The thirteen** — for each: the answer you would give the person;
   the calls it took, by number; whether you hold the answer to be
   right, and what would tell you it is not; where the tool stopped or
   misled you.
3. **Faults** — what the tool said, what it should have said, the
   calls that show it. A thing you had to learn by being refused is one.
4. **The tool's own words** — the instructions and each tool's text as
   `open` printed them: what was right, what was wrong or missing, what
   you read and never used.
5. **The answers' size** — what of an answer you read, what you never
   read, and any call whose size cost you something.
6. **What you would ask of the tool** — one line each, with the call
   that made you want it.
7. **Left out** — what you did not try, every file you opened that is
   not this brief, and the one thing you cut that you would most want
   put back.

## Acceptance

A reader can say, for each of the thirteen, what the person was told,
what it cost in calls and bytes, and where the tool stopped — from the
report and `mcp-seat-calls.csv` alone.

Your last message: at most 300 words — which questions were answered,
which were not, the fault you hold the worst, and what "Left out" says.
