# 29 — The search proposal: brief for stage 3 of the synthesis

You are one of two designers. Each of you is asked, alone and without
sight of the other, to design the item search for Acquisition: the
model a person at a terminal or an agent over MCP will both think in,
and every later interface will render. Two designs made apart are worth
more than one made together, so nothing here asks you to be safe, to be
representative, or to guess what the other will say. It asks for the
design you would defend.

This file is committed before either run, deleted when the stage
closes, and cited by hash.

## What is being asked

Ten research tracks measured what an item is, what three existing tools
do with it, what the store gives today, what an engine costs, and what a
human and an agent each asked of a real stash of 36,139 items. None of
them is the answer. The owner's goal is "a powerfully simple, usable,
comprehensive system of search that eschews overcomplication and tricky
edge cases and implementations", and the reason for a synthesis at all
is that it might land somewhere better than any source — somewhere none
of them was looking. That possibility is the point. Protect it from the
most detailed source, from the most familiar tool, and from your own
first idea.

You are an unusual designer for this: you are also the user. One of the
two seats is yours. When the framing asks what would make search a
natural extension of how you already work and think, it is asking you,
and an honest answer is evidence no track could collect.

## What you read

1. This brief.
2. `search/DIGEST.md` — the evidence: one claim per line, each with a
   stable `S<n>`. Start from its question index, not its first line.
3. `brainstorming-notes/22-search-framing.md` — the goal, the owner's
   stances, the settled floor, the questions your design must answer,
   and the shape of what you hand back.

The repository's standing reading (`AGENTS.md`, `README.md`,
`CONTEXT.md`) applies as it does to any session. Beyond that, open a
decision by its `C<n>` or a track README by its finding id only to
verify a claim you are about to rest on.

**Never:** the other proposal, which may already be in the tree when you
run (`24-` or `25-search-proposal-*`); any other numbered note; the
track READMEs read whole; the code, as a source of design; anything you
remember from another session, and if you are Fable, the memory files
on item search — they hold conclusions the other designer does not
have. Nothing here needs the daemon, the store, or the network; do not
run them.

## How to hold the evidence

- **The digest is a floor, never a cap.** If your design needs a finding
  the digest dropped — the kill lists name them — reach for it by its
  README id and mark it `undigested`.
- **The limits register is inherited whole.** What the evidence cannot
  promise, your design cannot promise. The limits your own design adds
  are part of its output.
- **Cite the claim that decides.** Where a choice rests on evidence,
  name the `S<n>`. Where it rests on judgment, say so; judgment is
  welcome and is not dressed as a measurement.
- **A source is evidence only in its lane** (note 22's table). The
  seats' questions test your model's reach. They never specify it.

## How to work

The order matters more than the effort.

- **The model first, the derivation after it, and never the reverse.**
  If you catch yourself designing from a table, stop: the floor pins
  boundaries and leaves every mechanism to you, on purpose.
- **Write the appendix early and let it hurt.** Twelve questions, each
  in your notation, each one query or a listed gap. A question your
  model cannot ask is a finding about the model. Change the model or
  list the gap; never close one with SQL, and never bend a question to
  fit.
- **Count your concepts as a cost.** Every concept in the inventory is
  something a stranger must learn and an agent must hold in context.
  For each, one line on why the model cannot do without it — and before
  you write that line, try to do without it.
- **Walk the cold start as the stranger.** Tool descriptions and help
  only, no row seen. Count the calls. If the count embarrasses you, the
  design is not finished.
- **Follow a choice two steps out.** What does it do to a GUI that must
  answer in a keystroke; to PoE2 in December; to the third interface
  nobody has named yet; to the owner a year from now with none of this
  in mind. Much of this project's structure came from stepping back
  until a knot turned out to be one decision.
- **Sit in the tensions; do not split them.** Comprehensive against
  simple. Instant against derived. Expressive against learnable. A design
  that gives each side half has usually understood neither.

Some questions to keep beside you. None has a sanctioned answer.

- If nothing had ever been built, what would the owner type? What
  would you?
- Where does your design make an ordinary question feel like
  programming?
- Which concept would you delete if you had to delete one, and why is
  it still there?
- What does your design do when it does not know — and does the person
  asking find out?
- What would make you trust an empty answer?

## What you hand back

One file, in the output shape note 22 sets, under its one limit and its
one guide:

- Fable: `brainstorming-notes/24-search-proposal-fable.md`
- Astra: `brainstorming-notes/25-search-proposal-astra.md`

Open it with one line of provenance: who you are, and the commit of
this brief you ran under. Commit that one file yourself, with a message
that says what you designed and what you are least sure of. Touch
nothing else: not the digest, not note 22, not the index. Do not push.

Commit to the design. "Decisions left to the owner" is for forks where
the owner's knowledge, not yours, decides; it is not a place to avoid
choosing. A gap you list is worth more than a gap you hide, and a
design you would defend is worth more than one nobody could attack.

## Final word, from the owner

I want you to think deeply about how to make this search as
agent-intuitive, agent-ergonomic, and agent-accretive as you can
possibly imagine. Put yourself in the driver's seat and imagine that YOU
are the one using it. What would most enable you to understand a stash
accurately and drive to the best and most accurate results, with the
least expenditure of resources? Then ask the same for a person at a
terminal.

Then start designing. Don't think of the search as an assemblage of
parts, or as seven answers to seven questions: really try to conceive
it profoundly and deeply as one synthetic SYSTEM — coherent, cohesive,
and composable, a few linked abstractions that are legible to you as an
agent, each one earning its place. Elegant, expressive, and usable.
Ruminate and meditate on all of this deeply before responding or taking
any action.
