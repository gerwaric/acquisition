# 5 — The forms, v2 (a prop for the walk; not the registry, not note 35)

Frozen before the walk in file 6. Marks: **R** ruled today (`decisions/search.md`, `search/DESIGN.md`); **A** agreed between Astra and Fable in files 1–4, not ruled; **T** admitted for the walk to test, contested or new this round; **U** unavailable — no spelling or no defined response yet, on purpose. Names of fields, totals and classes are illustrative; the `language` context supplies the real ones.

## Two values

```
filter          the conditions: text ⇄ tree (each prints to the other, C104).      R
                Portable: carries no account, realm or view.                        A
bound request   { account, realm | all, membership: live | removed | both }         A
                + filter + view   [+ basis, on a continuation or a drill-down]
                A receiver whose selected account differs refuses; rebinding is
                a deliberate act (--rebind), never a default.                       A
```

## The request at a terminal

```
acq search [--account A] --realm pc|xbox|sony|poe2|all ['<filter>']                 A  (no filter: every item in scope)
   view      rows (default) | --count key,…  | --cross key,key   [--sum value]      A
             [--fields name,…] [--sort value [--desc]] [--limit n] [--next token]   A
   context   --context matches (default) | corpus | language      with --count      A
   explain   --explain <node>        literal substitution, that node                T
             --alternatives <node>   candidate replacement, that node               T
   exchange  --print-request         the bound request as JSON                      A
             --request <file|->      run a bound request                            A
             --rebind                accept a request bound elsewhere, shown        A
   define    --define name           a total's rows / a derived field's definition  T
acq show <id> [--against '<filter>'] [--basis B]      one item; the why-not         R, A
```
`realm:pc` inside the filter: **removed** (A). `in pc: …`: **U**, pending the copy test (3-response).

## The filter

```
COMPOSITION
  t t        and — at whatever level you are in; whitespace never binds across     A
  and  or  not  -t  ( )      words accepted; `-` is not                             A
  a b or c   unparenthesised mix: an error showing both readings                    A (C91)
  holds(q, q, q)>=2    holds(q, q, q)=2..3                                          A
  undecided(thing | term)    true when that value / that truth cannot be
                             established; itself always decided; known absence
                             satisfies neither                                      A

ITEM-LEVEL TERMS
  word  "a phrase"           text: substring, any case, over everything the item
                             displays (name, base, lines as shown). Numbers are
                             characters: "Level 1" also finds "Level 10".            A
  name:kaom  name="Kaom's Heart"      `:` contains, `=` whole value                  A
  rarity=rare  class:ring             closed set: `=` exact; `:` picks among legal
                                      values and the answer lists which              A
  ilvl>=84   ilvl=80..84              = > >= < <= ; a..b inclusive, a side blank     A
  has:x   -has:x                      presence; absence is claimed only when all
                                      that could hold x was readable                 R
  is:corrupted   -is:corrupted        yes / no; silence is any                       A
  league:  tab:  character:  container:       place, as text fields                  R
  id:<handle>                         any id an answer printed                       R
  unknown field / operator / closed value  →  authoring error, near names offered    A
  a bare word is never a name: `priced` is text, `has:priced` the field             A

MEMBERS — conditions that must hold on one member                                   A
  line( <conditions over one occurrence> )      a selector; as a term: exists
  linked( <conditions over one link group> )
  line attributes:    "T" (leading; = template="T")  template:words
                      source=explicit   is:fractured  -is:crafted
                      #1 #2 …  and, on a ranged line, low high avg   with  = > >= < <= a..b
  linked attributes:  red green blue white (counts)   size
  inside a group: and / or / - / ( ) allowed; a bare word is an error               A
  shown:words  (substring over the displayed occurrence)                            U
  a template typed with its numbers — line("+92 to maximum Life"),
  "+92 to maximum Life">=90 — is an error showing two readings:
  any value, or that value                                                          A

VALUES — a selector projected; what a comparison, a sum or a sort consumes          T
  line(P).slot        the slot over the occurrences satisfying P; a one-slot line
                      needs no slot word
  "T">=90   "T".avg>=20           shorthand: lowers at once to line("T" #1>=90),
                                  line("T" avg>=20); no rules of its own            A, T (the dot)
  "T" low>=15 high<=45            error: slot words need a line; offers
                                  line("T" low>=15 high<=45)  and  "T".low>=15 "T".high<=45     A
  a selector where a value is needed, or the reverse: a type error
  sum(v)              the item's sum of a projected value (C92); sum("T") allowed   R
  total-res  total-str            a named total: reviewed rows, one meaning (C94)   R
  dps  defence-pct                a derived field (C101)                            R
  sockets  links  sockets.red     counts over the socket collection                 T
  priced (has:priced)             Boolean. price.amount  price.currency  price.lot  A
                                  cross-currency comparison                         U
  sum(2*total-str, total-dex)>=150    one scalar weighted example                   T
      operands: anything with a numeric value, a definition and a status; literal
      weights; overlapping rows both contribute, contributions printed; C94's
      missing rule; no simplification.  Range-valued weighted sums, negative
      weights on ranges, guarded groups                                             U

SORT (C92)   --sort takes a value. A ranged value with no slot is the slot error;
             an item with no satisfying occurrence sorts last either way.          R
```

## The answer

```
request     the bound request as understood: canonical filter text and tree; every
            node carries a path id (n0.2.1); a client's own node ids are echoed     A
basis       store and account, snapshot, intent revision, reference versions        R
scope       items searched; realm; leagues present; per location: a full handle
            (realm, league, kind, id, path by name), observed-at; never-fetched
            handles; when the location *listing* was last observed                  R + A
terms       per item-level atomic term, over the scope: matched, failed, lacked,
            undecided; the together count where it applies. Unit: items.           R
            each count carries a route: a bound request stating its denominator
            (scope, or under the filter) and the basis it was counted at            A
total       definite matches                                                        R
undecided   root-undecided, with its route; on each item its reason and the
            reason's remedy class: refresh may resolve | reference data |
            pricing                                                                 R + T
view        rows: id, name, base, rarity, place as a path, the lines the filter
            touched with their values; counts: real buckets, none, undecided,
            each with a route (a crossed cell's route carries both keys)            R + A
zero        when total is 0, in place of rows: selectors that resolved to nothing
            (with suggestions — tolerant suggestion, exact execution); the
            root-undecided route; never-fetched handles; and the *offer* of
            --explain / --context corpus, not their output                          A
next        returned, total, a continuation naming basis, filter, view, order.
            Basis changed: refused, naming both bases. A resident holder may
            continue a retained basis and says newer data exists; an expired or
            never-held basis is its own error. show --against obeys the same.       R + A
```

## Explain and alternatives (T)

`--explain n`: the root's definite and undecided counts for the request as given, for `Q[n := true]` and for `Q[n := false]`, each with its transformed filter printed. Labelled literally — "with n forced true" — never "removed", never "contribution". Inside a member group the substitution is per member: it never creates a member, and an unread collection is not a population.

`--alternatives n`: `n` must be a leaf comparing one key to a value. For each value *v* of the key in scope, the root matches of `Q[n := key=v]`, computed as T_*v* + (F − F_*v*) for a single-valued key; items whose key is undecided are counted apart; for a multi-valued key the counts do not partition and the answer says so.

## Invariants (A)

1. Canonicalisation lowers sugar and normalises spelling. It never reorders, flattens, merges, deduplicates or simplifies (`holds(P, P)>=2`; `T − T`).
2. A `:` selector is never replaced by its resolution; the answer shows the binding beside the authored selector. A vocabulary row's selector is always the exact form.
3. A selector's validity never depends on the corpus: an exact template nothing carries is evaluable, matches nothing known, and is undecided where eligible evidence is unread.
4. A count shown has a route to its members, and the route keeps its denominator.
5. Search is a pure read: coverage hands location handles to the client, which may take them to the planner; no item predicate ever supports a claim about an unfetched location.
