# 35 — The search forms table (stage-5 audit, session C, step 1)

Provenance: Fable (`claude-fable-5-1`), in session with the owner,
2026-09-18. Read: the audit's finding 5 and its Fates; `decisions/search.md`
whole; `search/DESIGN.md` whole; note 28 §1 and §4; note 24 §1–2; note 25
§1–2; note 31 rows 2, 2d, 8 and its list of the modes both reviewers
tripped on ("What the users said").

**What this is.** The language's surface on one page, so the owner rules
the spelling *before* a page of worked examples is built on it. Writing
the examples is fixed-input work once this is ruled (a subagent, under
`search/BRIEF.md`, rewritten then); choosing the spelling is design, so
a session does it and the owner rules it. This is a draft for verdicts,
not an authority: what is accepted lands in `search/DESIGN.md` as the top
of the language reference — the text the help and the MCP tool
description are later cut from — and this note is deleted at the close.

**Marks.** **R** — fixed by a ruling or the owner's words (cited).
**K** — kept from note 24 §2, untouched by any ruling. **N*n*** — chosen
here; choice *n* in §2 has the options and a blank verdict. **G*n*** — a
gap in meaning, §3. Everything K and N is provisional until the owner's
first seat (note 23, decision 15). Field and total *names* are
illustrative throughout: the closed lists are reference data's and the
help's (C94, C97), not this page's.

## 1. The forms

```
THE REQUEST — what is stated outside the conditions (C96, C100)
  acq search [--realm pc|poe2|all] '<query>'                          N1
      [--count name,…] [--cross name,name] [--sum thing]              N2
      [--sort term [--desc]] [--fields name,…]                        K, N3
      [--define name] [--next <token>]                                N4
  acq show <id> [--against '<query>']     one item; the why-not       R C100
  realm:pc  at the front of a plain query spells --realm pc           R C96

THE QUERY — composition (C91)
  term term              and                                          K
  -term                  not                                          K
  term or term           or                                           K
  ( query )                                                           K
  a b or c               and beside or, unparenthesised: an error
                         that shows both readings                     N5
  (old query) term       a refinement                                 R C91
  holds(q, q, q)>=2      at least 2 of these hold                     K
  holds(q, q, q)=2..3    both bounds                                  N6

A TERM is a thing and, optionally, conditions
  ilvl>=84               a number field:  =  >  >=  <  <=             K
  ilvl=80..84            a range, inclusive, either side blank        N6
  name:kaom              text: `:` is contains, any case              K
  name="Kaom's Heart"    text: `=` is the whole value                 N7
  rarity:rare            a closed value set: `:` picks among the
                         legal values, the answer lists which; none
                         is an error that lists them                  K
  has:sockets            presence                                     R C93
  -has:reqlevel          absence (undecided while anything that
                         could hold it is unread)                     R C93
  is:corrupted  -is:corrupted     a flag: yes, no; silence is any     K
  league:  tab:  character:  container:    place, like any text       R C96
  id:<x>                 any id an answer printed                     R C100

LINES
  "# to maximum Life"              one whole template — short for
                                   line="…"; holds when an occurrence
                                   exists                             N8
  "# to maximum Life">=90          holds on one occurrence            R C92
  "Adds # to # Cold Damage" low>=12 high<=28
                                   several conditions, one occurrence R C92, owner
  "Adds # to # Cold Damage" avg>=20        the mean, asked by word    R C92
  "Players have #% less Armour per #% Alert Level" #1>=20
                                   any line's numbers, by position    R C92
  "Adds # to # Cold Damage">=20    no slot named: an error listing
                                   low, high, avg                     R C92
  line:suppress                    contains: every template it
                                   resolves to, listed in the answer  N8
  explicit="…"   fractured:suppress     a kind in place of `line`:
                                   a source array, or a flag          N9
  explode                          a bare word that is no name is
                                   text:explode, and prints back so   N10
  sum("# to maximum Life")>=90     the item's sum of the line         R C92
  total-res>=60                    a named total                      R C94
  total-cold-damage avg>=30        a ranged total takes the slot words R C94
  dps>=400                         a derived field                    R C101
  priced                           the effective price                R C100, G2

SOCKETS                                                               N11
  red>=2 blue>=1                   colours over the whole item
  linked(red>=2 blue>=1)           met together by one link group
  sockets>=5   links>=5            how many; the largest group

SMALL RULES                                                           N12
  `low high avg #1 #2 …` are reserved: they follow a line, a sum or a
  total and are never field names. Spaces around an operator are allowed and
  the canonical text has none. Text is matched in any case. A value
  with a space is quoted; `\"` is a quote inside quotes. Typing a line
  as displayed (`"+92 to maximum Life"`) names its template.
```

## 2. Choices — one verdict each

**N1 — the verb and the scope flag.** `acq search`, `--realm pc|poe2|all`.
`acq items search` exists today (a substring read `DESIGN.md` marks
deletable); the MCP carries the same names as keys. The realm words
follow `DESIGN.md`'s own `realm:pc`. *Alternative:* keep `acq items
search` and grow it. **Verdict:**

**N2 — the two count spellings (C95: "two spellings, never one redefined
as the other").** Note 24's `--count a,b` gave two tables where a reviewer
expected one crossed table (note 31: "`--count league,class`
returns two tables and no error"). Here: `--count tab,league,rarity` is
independent facets, one table each (AQ1); `--cross league,class` is one
crossed table; `--sum stack` adds the one summed thing to either;
`--count line:resist,line:life` is the vocabulary read, several texts
in one call (C97).
*Alternative:* `--count a,b` crosses and facets repeat the flag
(`--count a --count b`). I prefer two words: neither reads as the other.
**Verdict:**

**N3 — sort takes a term.** `--sort '"Adds # to # Cold Damage" avg'`,
ascending, `--desc` reverses; the slot rule is the comparison's (C92),
so a ranged line with no slot is the same error. *Alternative:*
descending by default, since a sort on a value usually wants the best
first. **Verdict:**

**N4 — a definition on request (C94: "prints a definition on request").**
`--define total-res` prints the rows of the total, or a derived field's
definition and source version (C106), in place of a view. *Alternative:*
only in the help. **Verdict:**

**N5 — and beside or.** Note 24: `or` binds looser than adjacency, so
`class:ring life or mana` is `(class:ring life) or mana`; note 31 lists
it among the silent modes met in use. Here it is C91's ambiguity error: mixing the
two at one level without parentheses shows both readings and runs
neither; the canonical text always parenthesises. *Alternatives:* keep
note 24's rule; or bind `or` tighter than adjacency. **Verdict:**

**N6 — `=` with a range.** `ilvl=80..84`, `low=10..12`,
`holds(a, b, c)=2..3` (C91's upper bound; C99's `count` min and max
land here). Note 24 wrote ranges `thing:a..b`; with `:` meaning
contains, `=` keeps `:` to one meaning. **Verdict:**

**N7 — text: `:` contains, `=` whole value.** Note 24 had no exact text
match. One rule each, on every text thing — fields, place, lines.
**Verdict:**

**N8 — a quoted string standing as a thing is always one whole
template.** (After `:` or `=`, quotes only hold a value with a space.) The rulings do not say
what a text without `#` means. Note 24: with `#`, a template; without,
a substring over all displayed text, and under a comparison "any
template it resolves to" — the `#`-inside-quotes mode note 31 lists.
Here: a quoted line is one whole template, always (typed as a template
or as displayed), short for `line="…"` under N7's rule; one that is
none answers with the
templates sharing its words (`DESIGN.md`, C90); loose matching is asked
for with `:` — `line:suppress`, `fractured:suppress` — and the terms
block lists what it resolved to; a comparison across resolved templates
with different slots is C92's error. The vocabulary read hands out the
exact term, so exactness costs a paste, not a memory. *Alternative:*
note 24's rule, the mode stated. **Verdict:**

**N9 — a kind takes the place of `line`.** Sources (the arrays the
corpus shows: `implicit explicit enchant crucible scourge utility
ultimatum veiled …`, with `property` and `requirement`) and flags
(`crafted fractured mutated`, S17) are two closed lists, each
selectable (C97): `explicit="# to maximum Life">=90`,
`fractured:suppress`. Both at once on one occurrence is
`explicit.fractured:suppress` — the spelling I am least sure of; in the
corpus flags sit only on explicit lines, so it is rarely needed.
**Verdict:**

**N10 — the bare word.** Note 24: a bare word is a name when one exists
and text otherwise — the "secretly a name" mode. Here: a bare word that
is no name is `text:word` (name, base, every line — the stash-box
case, S186), and the canonical text prints it so; a bare word that *is*
a name, alone, is an error showing the readings (`has:sockets`,
`text:sockets`). *Alternative:* a bare word is always text, and names
appear only with an operator or a prefix. **Verdict:**

**N11 — sockets (C101, S59).** Note 24's `colors:rrg` was whole-item
only and its `:` was a third meaning. Here colours are number fields
over the item (`red>=2 blue>=1`), and `linked(…)` binds its conditions
to one link group, any qualifying group — the same idea as a line's one
occurrence, for the one other collection (no general collection syntax,
C101). *Alternative:* letters, as players write them — `colors=rrb`
over the item, `linked=rrb` within a group: shorter, but "at least
these" hides in an `=`. **Verdict:**

**N12 — the small rules**, as listed. **Verdict:**

## 3. Gaps in meaning — not spelling, so not mine to choose

**G1 — may a query sum several lines, or weigh them?** C92 gives `sum`
of *the line*; C94 says nothing but the reference table defines a
total; C99 has a site weight group translate "only where that guard has
a representation". Note 24's `sum(a, 2*b)` would be that
representation, and is also an unreviewed total by another door.
Readings: (a) `sum` takes one line, and a weight group sits in the
translation's remainder; (b) `sum(a, 2*b)` exists, unnamed and
unshipped. **Verdict:**

**G2 — a comparison on `priced` needs a unit.** `priced` alone is
presence; `priced>=50` compares in what — the currency table's chaos
equivalent (`decisions/pricing.md`), or the stated currency only
(`priced>=50chaos`)? Until ruled, the reference shows `priced` and
`has:priced` and no comparison. **Verdict:**

**G3 — do the `none` and `undecided` buckets carry a selecting term
(`DESIGN.md`, C105, left open for this page)?** `none` already has one
under the rulings: known absence is what `-has:class` asks, so the row
can carry it at no cost. `undecided` has none: a term true exactly when
another is undecided (`undecided:class`) is new — it is note 25's
`unknown(E)`, which the reconciliation did not keep. Readings: (a)
`none` carries `-has:<key>`, `undecided` carries nothing and its items
are reached through the answer's undecided count; (b) add
`undecided:<thing>`. **Verdict:**
