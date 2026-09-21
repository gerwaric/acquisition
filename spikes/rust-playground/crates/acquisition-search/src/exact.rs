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
//! that a sum happened to overflow in (the fifth audit's follow-up, 1).
//!
//! **Arithmetic is on whole units.** Added as binary floats, 0.1 + 0.2 +
//! 0.3 is 0.6000000000000001 one way round and 0.6 the other, and
//! `sum( … )=0.6` matched one of two items carrying the same lines (the
//! fifth audit, finding 1). A number within the rule is a whole count of
//! hundred-thousandths — four decimals, and one more for the half a ranged
//! pair's mean may end in — which a float holds exactly; units add as
//! integers, and the total is written out as the decimal it is and read
//! by the parser a typed bound is read by, so the two are one float
//! however large the total (dividing a total past 2^53 as a float rounded
//! twice: the audit's third review, 2). A comparison needs none of this: two
//! floats read from the same decimal are the same float.

/// The most whole digits and decimals a number may be displayed with.
const WHOLE: usize = 10;
const DECIMALS: usize = 4;
/// Hundred-thousandths: [`DECIMALS`] and a half.
const UNITS: f64 = 100_000.0;

/// Whether the search reads this number, as written: digits, a point and
/// digits, a sign before them or none, its thousands commas gone.
pub(crate) fn reads(literal: &str) -> bool {
    let digits = literal.trim_start_matches(['+', '-']);
    let (whole, decimals) = digits.split_once('.').unwrap_or((digits, ""));
    whole.trim_start_matches('0').len() <= WHOLE && decimals.len() <= DECIMALS
}

fn units(value: f64) -> i128 {
    (value * UNITS).round() as i128
}

fn read_back(units: i128) -> f64 {
    let sign = if units < 0 { "-" } else { "" };
    let (units, one) = (units.unsigned_abs(), UNITS as u128);
    format!("{sign}{}.{:05}", units / one, units % one)
        .parse()
        .unwrap_or(f64::NAN)
}

/// The sum of displayed numbers; nothing sums to zero.
pub(crate) fn sum(values: impl IntoIterator<Item = f64>) -> f64 {
    read_back(values.into_iter().map(units).sum())
}

/// The mean of a ranged pair.
pub(crate) fn mean(low: f64, high: f64) -> f64 {
    // each is a whole count of tens of units, so the half is whole
    read_back((units(low) + units(high)) / 2)
}

#[cfg(test)]
mod tests {
    use super::{mean, reads, sum};

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
        for unread in ["10000000000", "0.00001", "+10000000000000000000000000000"] {
            assert!(!reads(unread), "{unread}");
        }
    }
}
