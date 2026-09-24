//! A displayed number, and the arithmetic done on it (C92, C94, C95):
//! exact, and the same in any order.
//!
//! **A number is read once, where the body is read.** What a game displays
//! is a short decimal: over the census's store copy (212,233 numbers on
//! 22,721 items, 2026-09-21) none has more than two decimals or ten whole
//! digits. [`reads`] is the rule — at most ten whole digits and four
//! decimals — and the deriver asks it of every number of every line: one
//! it does not read is an unread slot of its line (C93; `derive::Slot`),
//! which is the one failure path evidence has, at the grain the evidence
//! was lost. Nothing downstream meets a number outside the rule, so
//! nothing downstream has a fallback, and no answer depends on the order
//! that a sum happened to overflow in (outside review, 2026-09-21).
//!
//! **Arithmetic is on whole units.** Added as binary floats, 0.1 + 0.2 +
//! 0.3 is 0.6000000000000001 one way round and 0.6 the other, and
//! `sum( … )=0.6` matched one of two items carrying the same lines (outside
//! audit, 2026-09-21). A number within the rule is a whole count of
//! hundred-thousandths — four decimals, and one more for the half a ranged
//! pair's mean may end in — which a float holds exactly; units add as
//! integers, and the total is written out as the decimal it is and read
//! by the parser a typed bound is read by, so the two are one float
//! however large the total (dividing a total past 2^53 as a float rounded
//! twice: outside review, 2026-09-22). A comparison needs none of this: two
//! floats read from the same decimal are the same float.
//!
//! **Units are carried, never reconstructed.** A displayed number, or the
//! mean of a ranged pair, becomes units once ([`Exact::of`]), read off the
//! decimal the float prints as — exact for a number within the rule, whose
//! fifteen significant digits a float holds. A total is [`Exact`] from
//! then on: what an item sums to, what a bucket sums those to, what a sort
//! orders by; it becomes a float only where a number is compared or
//! printed. A total's decimal can be longer than a float holds, so a float
//! made of it does not print it back, and units read from that print are
//! wrong — an outside audit (2026-09-22) met a bucket ending `.99313` where the
//! item said `.9931`, and the first fix, which read units off the float
//! again, left nine cancelling totals and `0.0001` summing to `0.00011`
//! (its review, 1).

/// The most whole digits and decimals a number may be displayed with.
const WHOLE: usize = 10;
const DECIMALS: usize = 4;
/// Hundred-thousandths: [`DECIMALS`] and a half.
const UNITS: f64 = 100_000.0;

/// Whether the search reads this number, as written: digits, a point and
/// digits, a sign before them or none, its thousands commas gone — and
/// nothing else: `1e34` is no number the game displays, and read as one
/// it overflowed the units (outside audit, 2026-09-24).
pub(crate) fn reads(literal: &str) -> bool {
    let digits = literal
        .strip_prefix('+')
        .or_else(|| literal.strip_prefix('-'))
        .unwrap_or(literal);
    let (whole, decimals) = match digits.split_once('.') {
        Some((whole, decimals)) if !decimals.is_empty() => (whole, decimals),
        Some(_) => return false,
        None => (digits, ""),
    };
    let all_digits = |s: &str| s.bytes().all(|b| b.is_ascii_digit());
    !whole.is_empty()
        && all_digits(whole)
        && all_digits(decimals)
        && whole.trim_start_matches('0').len() <= WHOLE
        && decimals.len() <= DECIMALS
}

/// A number in whole units: a displayed number, a mean, or a total of
/// them. Ordered and added as the integer it is.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Exact(i128);

impl Exact {
    /// A displayed number, or a mean of two, in units: the one place a
    /// float becomes units.
    pub fn of(value: f64) -> Exact {
        Exact(units(value))
    }

    pub fn as_f64(self) -> f64 {
        read_back(self.0)
    }

    /// The sum; nothing sums to zero.
    pub fn sum(values: impl IntoIterator<Item = Exact>) -> Exact {
        Exact(values.into_iter().map(|e| e.0).sum())
    }

    /// This many halves of the number: a weight of the totals table (C94)
    /// is a whole number or a half, and units hold the half of a displayed
    /// number, a sum of them or a count exactly — each is a whole count of
    /// ten units — and not the half of a mean, which the table refuses a
    /// half weight on (`totals.rs`).
    pub fn halved(self, halves: i64) -> Exact {
        Exact(self.0 * i128::from(halves) / 2)
    }

    /// The mean of two totals, a ranged total's `avg`: exact while both are
    /// whole counts of ten units, which whole weights on displayed numbers
    /// keep them (`totals.rs`).
    pub fn mid(self, other: Exact) -> Exact {
        Exact((self.0 + other.0) / 2)
    }

    /// The product of two displayed numbers — a weapon's attacks per second
    /// and the mean of its damage (`pseudo.rs`, C101) — where it is a whole
    /// count of units, and none where it is not: a product may need more
    /// decimals than a sum, and one the units cannot hold is unread to what
    /// asked it, never rounded (outside audit, 2026-09-24; the game
    /// displays at most two decimals on either, so no displayed weapon
    /// meets it).
    pub fn times(self, other: Exact) -> Option<Exact> {
        let units = UNITS as i128;
        let product = self.0.checked_mul(other.0)?;
        (product % units == 0).then_some(Exact(product / units))
    }
}

fn units(value: f64) -> i128 {
    let text = value.to_string();
    let (sign, digits) = match text.strip_prefix('-') {
        Some(rest) => (-1, rest),
        None => (1, text.as_str()),
    };
    let (whole, decimals) = digits.split_once('.').unwrap_or((digits, ""));
    let whole: i128 = whole.parse().unwrap_or(0);
    // five places, the sixth rounding up: a half is whole in units
    let mut frac: i128 = 0;
    for (i, digit) in decimals.bytes().take(6).enumerate() {
        let d = i128::from(digit - b'0');
        if i < 5 {
            frac = frac * 10 + d;
        } else if d >= 5 {
            frac += 1;
        }
    }
    let places = decimals.len().min(5);
    frac *= 10i128.pow((5 - places) as u32);
    sign * (whole * (UNITS as i128) + frac)
}

fn read_back(units: i128) -> f64 {
    let sign = if units < 0 { "-" } else { "" };
    let (units, one) = (units.unsigned_abs(), UNITS as u128);
    format!("{sign}{}.{:05}", units / one, units % one)
        .parse()
        .unwrap_or(f64::NAN)
}

/// The sum of displayed numbers, as a float to compare: nothing sums to
/// zero. A total kept for adding again is [`Exact::sum`].
pub(crate) fn sum(values: impl IntoIterator<Item = f64>) -> f64 {
    Exact::sum(values.into_iter().map(Exact::of)).as_f64()
}

/// The mean of a ranged pair.
pub(crate) fn mean(low: f64, high: f64) -> f64 {
    // each is a whole count of tens of units, so the half is whole
    read_back((units(low) + units(high)) / 2)
}

#[cfg(test)]
mod tests {
    use super::{Exact, mean, reads, sum};

    #[test]
    fn a_sum_is_the_decimal_sum_in_any_order() {
        assert_eq!(sum([0.1, 0.2, 0.3]), 0.6);
        assert_eq!(sum([0.3, 0.2, 0.1]), 0.6);
        assert_eq!(sum([]), 0.0);
        assert_eq!(sum([-0.1, 0.1]), 0.0);
        assert_eq!(sum([95.0, -5.0, 0.25]), 90.25);
        assert_eq!(mean(0.1, 0.2), 0.15);
        assert_eq!(mean(3.0, 10.0), 6.5);
        assert_eq!(mean(-0.1, -0.2), -0.15);
        assert_eq!(mean(0.0001, 0.0002), 0.00015);
        // the rule's edge, exactly: ten whole digits and four decimals
        let edge = 9999999999.9999;
        assert_eq!(sum([edge, -edge, 0.0001]), 0.0001);
        assert_eq!(sum([edge, 0.0001, -edge]), 0.0001);
        assert_eq!(sum([edge, edge]), 19999999999.9998);
        // past what a float holds exactly, the total is still read once
        assert_eq!(sum([9999999999.9997; 19]), 189999999999.9943);
        assert_eq!(sum([-9999999999.9997; 19]), -189999999999.9943);
        // a total added again — a bucket's sum of item totals — is exact
        // as units, where a float of it is not (outside audit and review,
        // 2026-09-22): nine means of large pairs, their negatives, and
        // a ten-thousandth, split over two totals or one
        let total = Exact::sum([Exact::of(9999999999.9997); 23]);
        assert_eq!(total.as_f64(), 229999999999.9931);
        assert_eq!(Exact::sum([total, total]).as_f64(), 459999999999.9862);
        let up = Exact::sum([Exact::of(mean(9999999999.9997, 9999999999.9998)); 9]);
        let down = Exact::sum(
            [Exact::of(mean(-9999999999.9997, -9999999999.9998)); 9]
                .into_iter()
                .chain([Exact::of(0.0001)]),
        );
        assert_eq!(Exact::sum([up, down]).as_f64(), 0.0001);
        // a float of `up` cannot print its five decimals back
        assert_ne!(Exact::of(up.as_f64()), up);
        assert_eq!(sum([0.5, 0.25]), 0.75);
        assert_eq!(sum([1e-5]), 0.00001);
        assert_eq!(sum([84.0, 70.0]), 154.0);
    }

    /// A weight is applied in halves and stays exact on what the table
    /// admits a half on; a ranged total's mean and a product are exact in
    /// units where the game's decimals allow, and said so where not.
    #[test]
    fn a_weight_a_mean_and_a_product_are_exact_in_units() {
        assert_eq!(Exact::of(9.0).halved(1).as_f64(), 4.5);
        assert_eq!(Exact::of(9.0).halved(2).as_f64(), 9.0);
        assert_eq!(Exact::of(20.0).halved(6).as_f64(), 60.0);
        assert_eq!(Exact::of(0.0001).halved(1).as_f64(), 0.00005);
        assert_eq!(Exact::of(-9.0).halved(1).as_f64(), -4.5);
        assert_eq!(Exact::of(59.0).mid(Exact::of(88.0)).as_f64(), 73.5);
        assert_eq!(Exact::of(0.1).mid(Exact::of(0.2)).as_f64(), 0.15);
        let times = |a: f64, b: f64| Exact::of(a).times(Exact::of(b)).map(Exact::as_f64);
        assert_eq!(times(73.5, 1.25), Some(91.875));
        assert_eq!(times(225.5, 1.25), Some(281.875));
        assert_eq!(times(0.0001, 0.5), Some(0.00005));
        // six decimals do not fit the units: no product, never a rounding
        // (outside audit, 2026-09-24: 0.000025 read as 0.00003)
        assert_eq!(times(0.0001, 0.25), None);
        assert_eq!(times(0.0001, 0.1), Some(0.00001));
        // the rule's largest number squared is within i128 and exact
        assert_eq!(times(9999999999.9999, 2.0), Some(19999999999.9998));
        assert!(times(9999999999.9999, 9999999999.9999).is_none());
    }

    #[test]
    fn the_rule_is_ten_whole_digits_and_four_decimals() {
        for read in [
            "0",
            "95",
            "+95",
            "-5",
            "0.0001",
            "9999999999.9999",
            "0009999999999",
        ] {
            assert!(reads(read), "{read}");
        }
        for unread in [
            "10000000000",
            "0.00001",
            "+10000000000000000000000000000",
            // no number the game displays: read as one, `1e34` overflowed
            // the units (outside audit, 2026-09-24)
            "1e34",
            "inf",
            "NaN",
            "1.",
            ".5",
            "",
            "+-5",
            "1 000",
        ] {
            assert!(!reads(unread), "{unread}");
        }
    }
}
