//! Sums of displayed numbers (C92, C94, C95): exact, and the same in any
//! order.
//!
//! A line's number is a decimal the game displayed — `0.2% of Damage
//! Leeched as Life` — and a bound is a decimal the author typed. Added as
//! binary floats, 0.1 + 0.2 + 0.3 is 0.6000000000000001 in one order of
//! occurrences and 0.6 in the other, so `sum( … )=0.6` matched one of two
//! items carrying the same three lines (the fifth audit, finding 1). So a
//! sum is taken over the decimals themselves: each number's shortest
//! spelling — which is the text it was read from, for any number a game
//! displays — as an integer and a scale, added as integers, and the total
//! read back as the float nearest to it, which is the float the author's
//! `0.6` is. The mean of a ranged pair (`avg`) is the same sum, halved.
//!
//! A number with more digits than an `i128` adds safely is no displayed
//! number; then the floats are added smallest first, which is at least the
//! same in any order.

/// An integer and how many of its digits follow the point.
fn decimal(value: f64) -> Option<(i128, u32)> {
    if !value.is_finite() {
        return None;
    }
    // Rust prints the shortest digits that read back as this float, never
    // an exponent
    let text = format!("{value}");
    let (whole, fraction) = text.split_once('.').unwrap_or((&text, ""));
    if whole.len() + fraction.len() > 30 {
        return None;
    }
    let mantissa: i128 = format!("{whole}{fraction}").parse().ok()?;
    Some((mantissa, fraction.len() as u32))
}

fn read_back(mantissa: i128, scale: u32) -> Option<f64> {
    let digits = mantissa.unsigned_abs().to_string();
    let scale = scale as usize;
    let digits = format!("{digits:0>width$}", width = scale + 1);
    let (whole, fraction) = digits.split_at(digits.len() - scale);
    let sign = if mantissa < 0 { "-" } else { "" };
    format!("{sign}{whole}.{fraction}0").parse().ok()
}

fn exactly(values: &[f64], halved: bool) -> Option<f64> {
    let parts: Vec<(i128, u32)> = values.iter().map(|v| decimal(*v)).collect::<Option<_>>()?;
    let scale = parts.iter().map(|(_, s)| *s).max().unwrap_or(0);
    let mut total: i128 = 0;
    for (mantissa, s) in parts {
        let raised = mantissa.checked_mul(10i128.checked_pow(scale - s)?)?;
        total = total.checked_add(raised)?;
    }
    if halved {
        // a half is five tenths
        return read_back(total.checked_mul(5)?, scale + 1);
    }
    read_back(total, scale)
}

fn in_one_order(values: &[f64]) -> f64 {
    let mut values = values.to_vec();
    values.sort_by(|a, b| a.abs().total_cmp(&b.abs()).then(a.total_cmp(b)));
    values.iter().sum()
}

/// The sum of displayed numbers; nothing sums to zero.
pub(crate) fn sum(values: impl IntoIterator<Item = f64>) -> f64 {
    let values: Vec<f64> = values.into_iter().collect();
    exactly(&values, false).unwrap_or_else(|| in_one_order(&values))
}

/// The mean of a ranged pair.
pub(crate) fn mean(low: f64, high: f64) -> f64 {
    exactly(&[low, high], true).unwrap_or((low + high) / 2.0)
}

#[cfg(test)]
mod tests {
    use super::{mean, sum};

    #[test]
    fn a_sum_is_the_decimal_sum_in_any_order() {
        assert_eq!(sum([0.1, 0.2, 0.3]), 0.6);
        assert_eq!(sum([0.3, 0.2, 0.1]), 0.6);
        assert_eq!(sum([]), 0.0);
        assert_eq!(sum([-0.1, 0.1]), 0.0);
        assert_eq!(sum([95.0, -5.0, 0.25]), 90.25);
        assert_eq!(sum([1e15, 0.3, -1e15]), 0.3);
        assert_eq!(mean(0.1, 0.2), 0.15);
        assert_eq!(mean(3.0, 10.0), 6.5);
        assert_eq!(mean(-0.1, -0.2), -0.15);
        // no displayed number: the floats, in one order either way
        assert_eq!(sum([1e300, 1.0, -1e300]), sum([-1e300, 1e300, 1.0]));
        assert!(sum([f64::NAN, 1.0]).is_nan());
    }
}
