//! The sockets (C101; the build plan, step 8): what `sockets`, `links`,
//! `sockets.<colour>` and a link group's `red green blue white size` count
//! on one item, and what leaves each open. Arithmetic over the collection
//! the deriver read (`derive::Item::sockets`), and nothing of the query
//! tree: what `linked( … )` means is `group.rs`'s, which reads the counts
//! here.
//!
//! The words are `search/DESIGN.md`'s reference (*Members*, *Values*);
//! the rules that decide a count are this doc's (C101; taken from the
//! contract detail 2026-09-24).
//!
//! # Decisions as recorded
//!
//! - **C101 — the sockets.** Socket colours are asked two ways: over the
//!   whole item — `sockets.red>=2`, a count over every socket the item
//!   has — and within a link group — `linked(red>=3 green>=1)`, where
//!   every colour and count named must be met together by one group
//!   (S59, the owner: "the filter matches the item if any of the link
//!   groups contains at least as many sockets of each color that were
//!   specified by the filter"). `sockets` is how many sockets the item
//!   has, `links` the size of its largest link group, a link group's
//!   `size` its own count. Every socket counts in `sockets`, whatever its
//!   colour: an abyssal socket (GGG's `A`) and a resonator's (`DV`) are
//!   sockets, each in a group of its own on the census's copy; the
//!   colours a query names are the reference's four, so an abyssal
//!   socket is asked for by the line every such item displays (`"Has #
//!   Abyssal Sockets"`) and a resonator by its base (K1; owner,
//!   2026-09-24: "agreed").
//!
//! # As built
//!
//! - **A colour word is GGG's letter** ([`COLOURS`]): `red` is `R`,
//!   `green` `G`, `blue` `B`, `white` `W`, matched in any case as every
//!   closed word is (B1); a socket's colour is compared as GGG spelled
//!   it. A word outside the four is an authoring error offering them.
//! - **A count is an interval** ([`Counted`]): what was read is its floor,
//!   and each socket that may or may not add to it — an element that was
//!   no socket, a socket whose colour or group could not be read — widens
//!   its ceiling by one (rule 8 of the plan, in the unit of one socket).
//!   A comparison on it is decided when the whole interval agrees and
//!   undecided otherwise (`Counted::truth`); the value is established
//!   when the interval is one number, which is when it sorts and sums as
//!   a value and `undecided( … )` of it is false (`eval::scalar`).
//!   `sockets` counts every element, so only an element that was no
//!   socket leaves it open; `sockets.<colour>` is open by that and by a
//!   socket whose colour is unread; `links` and a group's counts by that
//!   and by a socket whose group is unread, which may sit in any group or
//!   make one of its own ([`Groups`]). A socket whose group is unread is
//!   a socket still: it adds to its own colour's ceiling and to no other,
//!   it makes a link group exist, and alone — no group known, no element
//!   unread — it is a group of one, exactly (an outside review,
//!   2026-09-24, 2 and 3). A colour a socket's group could turn is a
//!   reason a `linked( … )` is open (its 1).
//! - **Present and absent** (C93): an item whose body carries no `sockets`
//!   lacks every count, so `sockets>=1` is false on a ring and `-has:sockets`
//!   finds it. An item whose collection is present holds `sockets` and
//!   every `sockets.<colour>` as a number — zero on GGG's `[]`, a poe2
//!   weapon that grants a skill (N44) — and `links` only while it has a
//!   link group: the largest of no groups is nothing, so such an item
//!   lacks `links`, `-has:links` finds it, and a `linked( … )` asked of it
//!   is lacked as a `line( … )` is of an item with no such occurrence.
//!   A poe2 socket has no colour, known: `sockets.red` is 0 on it.
//! - **The layout** ([`layout`]) is the collection as the game shows it:
//!   groups in GGG's order, the sockets of a group joined by `-`, groups
//!   set apart by a space — `R-R-G B W` — a socket with no colour shown
//!   by its `type` or as `?`, one whose colour is unread as `?`, and one
//!   whose group is unread set apart last, marked `(group unread)`. It
//!   is what `show` and a row print, made here once (rule 10).

use crate::bind::NumTest;
use crate::derive::{Item, Part, Socket};
use crate::group::Truth;
use crate::tree::Op;

/// The colour words of the language and the letter GGG spells each with.
pub const COLOURS: &[(&str, &str)] = &[("red", "R"), ("green", "G"), ("blue", "B"), ("white", "W")];

/// The words a query names a colour by.
pub fn colour_words() -> Vec<&'static str> {
    COLOURS.iter().map(|(word, _)| *word).collect()
}

/// GGG's letter for a colour word, in any case.
pub fn letter(word: &str) -> Option<&'static str> {
    COLOURS
        .iter()
        .find(|(w, _)| w.eq_ignore_ascii_case(word))
        .map(|(_, letter)| *letter)
}

/// A count over the sockets, as far as it was read (the module doc).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Counted {
    /// The item has no such count: no collection, or — for `links` — no
    /// link group.
    Absent,
    /// At least `low` and at most `high`; established when the two are one.
    Range { low: usize, high: usize },
}

impl Counted {
    /// The number, where it is established.
    pub fn value(self) -> Option<usize> {
        match self {
            Counted::Range { low, high } if low == high => Some(low),
            _ => None,
        }
    }

    /// What was read of it, where it is not established: the floor.
    pub fn floor(self) -> Option<usize> {
        match self {
            Counted::Range { low, .. } => Some(low),
            Counted::Absent => None,
        }
    }

    /// A comparison over the interval: true when every count in it
    /// satisfies the test, false when none does, undecided otherwise.
    /// Absent is false (C93).
    pub(crate) fn truth(self, test: &NumTest) -> Truth {
        let Counted::Range { low, high } = self else {
            return Truth::False;
        };
        let holds = |n: usize| test.holds(n as f64);
        let (at_low, at_high) = (holds(low), holds(high));
        match test {
            // monotone in n: the ends decide
            NumTest::Cmp(Op::Gt | Op::Ge, _) => {
                if at_low {
                    Truth::True
                } else if at_high {
                    Truth::Undecided
                } else {
                    Truth::False
                }
            }
            NumTest::Cmp(Op::Lt | Op::Le, _) => {
                if at_high {
                    Truth::True
                } else if at_low {
                    Truth::Undecided
                } else {
                    Truth::False
                }
            }
            // `=` and a range hold on an interval of counts: all of it, or
            // none of it, or some
            _ => {
                if (low..=high).all(holds) {
                    Truth::True
                } else if (low..=high).any(holds) {
                    Truth::Undecided
                } else {
                    Truth::False
                }
            }
        }
    }
}

/// The sockets of one item, read for counting: the collection, how many
/// elements were no socket, and the sockets by group.
pub struct Groups<'a> {
    sockets: &'a [Socket],
    /// Elements that were no socket: each may be a socket of any colour
    /// in any group.
    junk: usize,
    /// Sockets whose group could not be read: each may sit in any group,
    /// or make one of its own.
    unplaced: usize,
    /// The groups GGG numbered, in order of their number: (group, the
    /// sockets in it by their place in the collection).
    groups: Vec<(i64, Vec<usize>)>,
}

/// One link group's counts, as far as they were read: the floor is what
/// its own sockets say, the ceiling adds every socket that may belong to
/// it (the module doc).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GroupCounts {
    /// The sockets of the group, by their place in the collection; empty
    /// for the group that unplaced sockets may make of their own.
    pub members: Vec<usize>,
    pub size: Counted,
    /// By GGG's letter, the colours the group's sockets carry, and the
    /// four of the language each with a count even when zero.
    colours: Vec<(String, Counted)>,
}

impl GroupCounts {
    /// How many sockets of GGG's colour `letter` the group holds.
    pub fn colour(&self, letter: &str) -> Counted {
        self.colours
            .iter()
            .find(|(l, _)| l == letter)
            .map_or(Counted::Absent, |(_, n)| *n)
    }
}

/// The collection of an item, or none where the body carries no
/// `sockets` — the array itself being unread is the same absence, and
/// what says so is the deriver's unread part.
pub fn groups(item: &Item) -> Option<Groups<'_>> {
    let sockets = item.sockets.as_deref()?;
    let junk = item
        .unread
        .iter()
        .filter(|u| u.part == Part::Sockets)
        .count();
    let mut groups: Vec<(i64, Vec<usize>)> = Vec::new();
    let mut unplaced = 0;
    for (i, socket) in sockets.iter().enumerate() {
        match socket.group {
            None => unplaced += 1,
            Some(group) => match groups.iter_mut().find(|(g, _)| *g == group) {
                Some((_, members)) => members.push(i),
                None => groups.push((group, vec![i])),
            },
        }
    }
    groups.sort_by_key(|(group, _)| *group);
    Some(Groups {
        sockets,
        junk,
        unplaced,
        groups,
    })
}

impl Groups<'_> {
    /// `sockets`: how many sockets the item has.
    pub fn count(&self) -> Counted {
        Counted::Range {
            low: self.sockets.len(),
            high: self.sockets.len() + self.junk,
        }
    }

    /// `sockets.<colour>`: how many of GGG's colour `letter` over the item.
    pub fn colour(&self, letter: &str) -> Counted {
        let known = self
            .sockets
            .iter()
            .filter(|s| s.colour.as_deref() == Some(letter))
            .count();
        let unread = self.sockets.iter().filter(|s| s.colour_unread).count();
        Counted::Range {
            low: known,
            high: known + unread + self.junk,
        }
    }

    /// Whether the item is known to have a link group: one GGG numbered,
    /// or a socket whose group is unread, which sits in one whatever it is.
    pub fn has_group(&self) -> bool {
        !self.groups.is_empty() || self.unplaced > 0
    }

    /// Whether the item may have a link group and is not known to: an
    /// element that may be a socket, and nothing that is one.
    pub fn may_have_group(&self) -> bool {
        !self.has_group() && self.junk > 0
    }

    /// The sockets whose group is unread that may add to a colour's count:
    /// those of that colour, and those whose colour is unread too.
    fn unplaced_of(&self, letter: &str) -> usize {
        self.sockets
            .iter()
            .filter(|s| {
                s.group.is_none() && (s.colour.as_deref() == Some(letter) || s.colour_unread)
            })
            .count()
    }

    /// `links`: the size of the largest link group. Absent while no socket
    /// is known and no element may be one; at least one while a socket is.
    pub fn links(&self) -> Counted {
        let may_join = self.unplaced + self.junk;
        match self.groups.iter().map(|(_, members)| members.len()).max() {
            Some(largest) => Counted::Range {
                low: largest,
                high: largest + may_join,
            },
            None if self.unplaced > 0 => Counted::Range {
                low: 1,
                high: may_join,
            },
            None if self.junk > 0 => Counted::Range {
                low: 0,
                high: self.junk,
            },
            None => Counted::Absent,
        }
    }

    /// Every link group the item may have, each with its counts: the
    /// groups GGG numbered, a socket whose group is unread on its own
    /// where it is the whole collection (a group of one, exactly), and —
    /// where a socket may sit in any group — one more, of the sockets that
    /// may make a group of their own, whose members are none since it may
    /// not exist.
    pub(crate) fn each(&self) -> Vec<GroupCounts> {
        let may_join = self.unplaced + self.junk;
        let letters = |of: &[&Socket]| {
            let mut letters: Vec<String> = COLOURS.iter().map(|(_, l)| l.to_string()).collect();
            for socket in of {
                if let Some(colour) = &socket.colour
                    && !letters.contains(colour)
                {
                    letters.push(colour.clone());
                }
            }
            letters
        };
        let counts = |members: &[usize], others: usize| {
            let of: Vec<&Socket> = members.iter().map(|i| &self.sockets[*i]).collect();
            let unread = of.iter().filter(|s| s.colour_unread).count();
            let colours = letters(&of)
                .into_iter()
                .map(|letter| {
                    let known = of
                        .iter()
                        .filter(|s| s.colour.as_deref() == Some(&*letter))
                        .count();
                    let high = known + unread + self.unplaced_of(&letter) + self.junk;
                    (letter, Counted::Range { low: known, high })
                })
                .collect();
            GroupCounts {
                members: members.to_vec(),
                size: Counted::Range {
                    low: of.len(),
                    high: of.len() + others,
                },
                colours,
            }
        };
        if self.groups.is_empty() && self.unplaced == 1 && self.junk == 0 {
            let alone = self
                .sockets
                .iter()
                .position(|s| s.group.is_none())
                .unwrap_or(0);
            return vec![counts(&[alone], 0)];
        }
        let mut out: Vec<GroupCounts> = self
            .groups
            .iter()
            .map(|(_, members)| counts(members, may_join))
            .collect();
        if may_join > 0 {
            out.push(GroupCounts {
                members: Vec::new(),
                size: Counted::Range {
                    low: 1,
                    high: may_join,
                },
                colours: COLOURS
                    .iter()
                    .map(|(_, l)| {
                        let high = self.unplaced_of(l) + self.junk;
                        (l.to_string(), Counted::Range { low: 0, high })
                    })
                    .collect(),
            });
        }
        out
    }

    /// One group's sockets as the game shows them: `R-R-G`.
    pub fn group_text(&self, members: &[usize]) -> String {
        members
            .iter()
            .map(|i| socket_text(&self.sockets[*i]))
            .collect::<Vec<_>>()
            .join("-")
    }

    /// The collection as the game shows it (the module doc, "The layout").
    pub fn layout(&self) -> String {
        let mut parts: Vec<String> = self
            .groups
            .iter()
            .map(|(_, members)| self.group_text(members))
            .collect();
        let unplaced: Vec<String> = self
            .sockets
            .iter()
            .filter(|s| s.group.is_none())
            .map(socket_text)
            .collect();
        if !unplaced.is_empty() {
            parts.push(format!("{} (group unread)", unplaced.join(" ")));
        }
        if self.junk > 0 {
            parts.push(format!(
                "{} {} unread",
                self.junk,
                if self.junk == 1 {
                    "element"
                } else {
                    "elements"
                }
            ));
        }
        parts.join(" ")
    }
}

fn socket_text(socket: &Socket) -> String {
    match (&socket.colour, socket.colour_unread, &socket.kind) {
        (Some(colour), _, _) => colour.clone(),
        (None, true, _) => "?".to_string(),
        (None, false, Some(kind)) => kind.clone(),
        (None, false, None) => "?".to_string(),
    }
}

/// The collection as the game shows it, for an item; none where the body
/// carries no `sockets`.
pub fn layout(item: &Item) -> Option<String> {
    groups(item).map(|g| g.layout())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_comparison_over_an_interval_is_decided_where_the_whole_interval_agrees() {
        let cmp = |op: Op, n: i64| NumTest::Cmp(op, n as f64);
        let two_to_four = Counted::Range { low: 2, high: 4 };
        assert_eq!(two_to_four.truth(&cmp(Op::Ge, 2)), Truth::True);
        assert_eq!(two_to_four.truth(&cmp(Op::Ge, 3)), Truth::Undecided);
        assert_eq!(two_to_four.truth(&cmp(Op::Ge, 5)), Truth::False);
        assert_eq!(two_to_four.truth(&cmp(Op::Le, 4)), Truth::True);
        assert_eq!(two_to_four.truth(&cmp(Op::Lt, 4)), Truth::Undecided);
        assert_eq!(two_to_four.truth(&cmp(Op::Lt, 2)), Truth::False);
        assert_eq!(two_to_four.truth(&cmp(Op::Eq, 3)), Truth::Undecided);
        assert_eq!(two_to_four.truth(&cmp(Op::Eq, 7)), Truth::False);
        assert_eq!(
            Counted::Range { low: 3, high: 3 }.truth(&cmp(Op::Eq, 3)),
            Truth::True
        );
        let range = |from: i64, to: i64| NumTest::Range(Some(from as f64), Some(to as f64));
        assert_eq!(two_to_four.truth(&range(1, 6)), Truth::True);
        assert_eq!(two_to_four.truth(&range(3, 6)), Truth::Undecided);
        assert_eq!(two_to_four.truth(&range(5, 6)), Truth::False);
        assert_eq!(Counted::Absent.truth(&cmp(Op::Le, 9)), Truth::False);
    }
}
