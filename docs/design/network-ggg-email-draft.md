# GGG Support Email — Bucket Questions (Draft)

**Status: draft, July 19, 2026 — transient by design.** This is the
consolidated ask-GGG email the network redesign leaves open: Q4's
positional-classification hypothesis, the legacy policy's bucket
resolutions (N21b), and the `RETRY_BUCKET_PAD_SECS = 60` ceiling
assumption (`network-redesign.md`, D3). It uses the channel GGG
explicitly offered (N14: "feel free to ask about other policies and
we can track down what resolution they use for you"). When answers
arrive, they are transcribed into `network-ground-truth.md` as
upgraded claims — verbatim, per that document's rules — and this
draft is deleted from the tree (git history keeps it). Tom sends it
on the existing support thread; nothing below is private (all quoted
policy definitions are already recorded in the ground-truth ledger
from acquisition's own captured traffic).

---

**Subject:** Rate limit bucket questions for acquisition (follow-up)

Hi,

Following up on our earlier thread about rate limit timing buckets —
you kindly offered that I could ask about other policies and you
could track down what resolution they use. I'm redesigning
acquisition's rate limiting and have three questions that would let
me pin the remaining assumptions to fact instead of calibration.

**1. Within a rule, which limit is "initial" and which is
"sustained"?**

You previously confirmed that the four API policies acquisition uses
are bucketed at 5-second intervals for the initial limit and
1-minute intervals for the sustained limit. My client currently
guesses which limit is which from the period length (75 seconds or
less means "initial"). Is the real rule positional instead — when a
rule carries multiple `hits:period:restriction` triplets, is the
first always the initial limit and the second always the sustained
one?

I ask because `stash-list-request-limit` currently reports
`10:15:60, 30:60:300`. That second limit has a 60-second period,
which my heuristic classifies as initial (so I pad it for 5-second
buckets). If it is actually the sustained limit (1-minute buckets),
I have been under-padding it by 55 seconds — and it would be a
plausible source of the intermittent violations I was never able to
explain.

**2. What are the bucket resolutions for the legacy stash-index
policy?**

The forum-shop stash-index call
(`www.pathofexile.com/character-window/get-stash-items`) reports
policy `backend-item-request-limit`, with rules Account
`30:60:60, 100:1800:600` and Ip `45:60:120, 180:1800:600`. Your
earlier bucket answer covered the four `api.pathofexile.com`
policies; what resolutions apply to this one?

**3. Is 60 seconds the largest bucket anywhere acquisition
touches?**

For retrying after a 429, I plan to wait `Retry-After` plus a fixed
60-second pad to cover bucket quantization on the retry. Is
60 seconds the largest bucket resolution used by any policy
acquisition touches — including the legacy one above — so that pad
is always sufficient?

Thank you — the earlier answers (the bucket sizes, and that HEAD
probing at startup is intended to work) are load-bearing in
acquisition's design, and they're documented in the project so
future maintainers know the provenance.

Tom
