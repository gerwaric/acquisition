use std::time::Duration;

use http::{HeaderMap, HeaderName};

const POLICY_HEADER: HeaderName = HeaderName::from_static("x-rate-limit-policy");
const RULES_HEADER: HeaderName = HeaderName::from_static("x-rate-limit-rules");

// Absolute wire bounds (external review finding 4): once bootstrap seeding
// makes configuration wire-derived, parse-time ceilings are the only
// non-circular bound on downstream allocation. Values sit far above anything
// the API has ever sent (observed: 2 rules, 2 triplets, max_hits <= 45,
// periods <= 300 s) while keeping worst-case synthesis small.
pub const MAX_RULES_PER_POLICY: usize = 8;
pub const MAX_TRIPLETS_PER_RULE: usize = 8;
pub const MAX_HITS_CEILING: u32 = 10_000;
pub const MAX_PERIOD_SECS: u32 = 3_600;

// Byte ceilings (follow-up review 2026-08-10): names are copied into
// snapshots and formatted into header names, and diagnostics quote raw wire
// text — the remaining wire-sized copies after the count/numeric ceilings.
// Observed names: policies <= 28 bytes, rules <= 7 ("Account").
pub const MAX_POLICY_NAME_BYTES: usize = 256;
pub const MAX_RULE_NAME_BYTES: usize = 64;
pub const MAX_DIAGNOSTIC_BYTES: usize = 64;

// Plain-data types carry public fields: there is no invariant for a getter
// to defend, so accessors would only be noise. `RulePair` is the deliberate
// exception — its constructor enforces the shape invariant, so its fields
// stay private.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicySnapshot {
    pub name: String,
    pub rules: Vec<RuleSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleSnapshot {
    pub name: String,
    pub pair: RulePair,
    pub state: RuleStatePair,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window {
    pub max_hits: u32,
    pub period: Duration,
    pub restriction: Duration,
}

impl Window {
    pub fn new(max_hits: u32, period: Duration, restriction: Duration) -> Self {
        Self {
            max_hits,
            period,
            restriction,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RulePair {
    burst: Window,
    sustained: Window,
}

impl RulePair {
    pub fn new(burst: Window, sustained: Window) -> Result<Self, RulePairShapeError> {
        if burst.period >= sustained.period {
            return Err(RulePairShapeError::NonIncreasingPeriods {
                burst_period: burst.period,
                sustained_period: sustained.period,
            });
        }
        Ok(Self { burst, sustained })
    }

    pub fn burst(&self) -> &Window {
        &self.burst
    }
    pub fn sustained(&self) -> &Window {
        &self.sustained
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RulePairShapeError {
    NonIncreasingPeriods {
        burst_period: Duration,
        sustained_period: Duration,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowState {
    pub current_hits: u32,
    pub period: Duration,
    pub restriction_active: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleStatePair {
    pub burst: WindowState,
    pub sustained: WindowState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyParseError {
    MissingHeader {
        name: HeaderName,
    },
    InvalidHeaderValue {
        name: HeaderName,
    },
    // D8 Full grammar: an empty or whitespace policy name is not a policy.
    EmptyPolicyName,
    PolicyNameTooLong {
        limit: usize,
    },
    InvalidRuleName {
        raw: String,
    },
    RuleNameTooLong {
        limit: usize,
    },
    TooManyRules {
        limit: usize,
    },
    TooManyTriplets {
        header: HeaderName,
    },
    MalformedTriplet {
        header: HeaderName,
        raw: String,
    },
    // D8 Full grammar plus absolute ceilings: limit hits must be positive,
    // every period positive, and both below the wire bounds.
    OutOfRangeTriplet {
        header: HeaderName,
        raw: String,
    },
    UnexpectedPolicyShape {
        rule: String,
        triplet_count: usize,
    },
    NonIncreasingPeriods {
        rule: String,
        burst_period: Duration,
        sustained_period: Duration,
    },
    StatePeriodsMismatch {
        rule: String,
    },
}

pub fn parse_policy(headers: &HeaderMap) -> Result<PolicySnapshot, PolicyParseError> {
    let name = required_header(headers, &POLICY_HEADER)?;
    if name.trim().is_empty() {
        return Err(PolicyParseError::EmptyPolicyName);
    }
    // Length-checked before the copy: wire data must not size an allocation.
    if name.len() > MAX_POLICY_NAME_BYTES {
        return Err(PolicyParseError::PolicyNameTooLong {
            limit: MAX_POLICY_NAME_BYTES,
        });
    }
    let name = name.to_owned();
    let rule_names = required_header(headers, &RULES_HEADER)?;
    // Bounded before any per-rule work; take() short-circuits the count.
    if rule_names.split(',').take(MAX_RULES_PER_POLICY + 1).count() > MAX_RULES_PER_POLICY {
        return Err(PolicyParseError::TooManyRules {
            limit: MAX_RULES_PER_POLICY,
        });
    }
    let mut rules = Vec::new();

    for raw_rule in rule_names.split(',') {
        let rule = raw_rule.trim();
        if rule.is_empty() {
            return Err(PolicyParseError::InvalidRuleName {
                raw: truncate_raw(raw_rule),
            });
        }
        if rule.len() > MAX_RULE_NAME_BYTES {
            return Err(PolicyParseError::RuleNameTooLong {
                limit: MAX_RULE_NAME_BYTES,
            });
        }
        let limit_header = rule_header(rule, false)?;
        let state_header = rule_header(rule, true)?;
        let windows = parse_triplets(headers, &limit_header, parse_window)?;
        if windows.len() != 2 {
            return Err(PolicyParseError::UnexpectedPolicyShape {
                rule: rule.to_owned(),
                triplet_count: windows.len(),
            });
        }
        let mut windows = windows.into_iter();
        let burst = windows.next().expect("length checked above");
        let sustained = windows.next().expect("length checked above");
        // The shape invariant lives in RulePair::new; this only retags the
        // error with the rule it came from.
        let pair = RulePair::new(burst, sustained).map_err(
            |RulePairShapeError::NonIncreasingPeriods {
                 burst_period,
                 sustained_period,
             }| PolicyParseError::NonIncreasingPeriods {
                rule: rule.to_owned(),
                burst_period,
                sustained_period,
            },
        )?;

        let states = parse_triplets(headers, &state_header, parse_window_state)?;
        if states.len() != 2 {
            return Err(PolicyParseError::UnexpectedPolicyShape {
                rule: rule.to_owned(),
                triplet_count: states.len(),
            });
        }
        let mut states = states.into_iter();
        let burst_state = states.next().expect("length checked above");
        let sustained_state = states.next().expect("length checked above");
        if burst_state.period != pair.burst().period
            || sustained_state.period != pair.sustained().period
        {
            return Err(PolicyParseError::StatePeriodsMismatch {
                rule: rule.to_owned(),
            });
        }

        rules.push(RuleSnapshot {
            name: rule.to_owned(),
            pair,
            state: RuleStatePair {
                burst: burst_state,
                sustained: sustained_state,
            },
        });
    }
    Ok(PolicySnapshot { name, rules })
}

fn required_header<'a>(
    headers: &'a HeaderMap,
    name: &HeaderName,
) -> Result<&'a str, PolicyParseError> {
    headers
        .get(name)
        .ok_or_else(|| PolicyParseError::MissingHeader { name: name.clone() })?
        .to_str()
        .map_err(|_| PolicyParseError::InvalidHeaderValue { name: name.clone() })
}

fn rule_header(rule: &str, state: bool) -> Result<HeaderName, PolicyParseError> {
    let suffix = if state { "-state" } else { "" };
    HeaderName::from_bytes(format!("x-rate-limit-{rule}{suffix}").as_bytes()).map_err(|_| {
        PolicyParseError::InvalidRuleName {
            raw: truncate_raw(rule),
        }
    })
}

/// Diagnostics keep at most `MAX_DIAGNOSTIC_BYTES` of raw wire text — an
/// error payload is an allocation the wire must not size, same as any other.
fn truncate_raw(raw: &str) -> String {
    let mut end = raw.len().min(MAX_DIAGNOSTIC_BYTES);
    while !raw.is_char_boundary(end) {
        end -= 1;
    }
    raw[..end].to_owned()
}

/// Numeric wire fields are bare ASCII digits. `str::parse` alone would also
/// accept a leading `+`, which no observed header carries — reject it.
pub(crate) fn ascii_digits_only(raw: &str) -> bool {
    !raw.is_empty() && raw.bytes().all(|byte| byte.is_ascii_digit())
}

fn parse_strict_u32(raw: &str) -> Option<u32> {
    if !ascii_digits_only(raw) {
        return None;
    }
    raw.parse().ok()
}

fn parse_triplets<T>(
    headers: &HeaderMap,
    name: &HeaderName,
    parse: fn([u32; 3]) -> Option<T>,
) -> Result<Vec<T>, PolicyParseError> {
    let raw_list = required_header(headers, name)?;
    let mut triplets = raw_list.split(',');
    let parsed = triplets
        .by_ref()
        .take(MAX_TRIPLETS_PER_RULE)
        .map(|raw| {
            let malformed = || PolicyParseError::MalformedTriplet {
                header: name.clone(),
                raw: truncate_raw(raw),
            };
            // take(4): a fourth field is already malformed, so nothing sized
            // by the wire is ever collected beyond it.
            let fields = raw
                .trim()
                .split(':')
                .take(4)
                .map(parse_strict_u32)
                .collect::<Option<Vec<_>>>()
                .ok_or_else(malformed)?;
            let fields: [u32; 3] = fields.try_into().map_err(|_| malformed())?;
            parse(fields).ok_or_else(|| PolicyParseError::OutOfRangeTriplet {
                header: name.clone(),
                raw: truncate_raw(raw),
            })
        })
        .collect::<Result<Vec<T>, _>>()?;
    if triplets.next().is_some() {
        return Err(PolicyParseError::TooManyTriplets {
            header: name.clone(),
        });
    }
    Ok(parsed)
}

// D8 Full grammar for a limit triplet: hits > 0 (a zero-hit lookback is
// meaningless and was a live divide-by-zero in the C++ client), period > 0,
// restriction >= 0 — plus the absolute wire ceilings.
fn parse_window([max_hits, period, restriction]: [u32; 3]) -> Option<Window> {
    if max_hits == 0 || max_hits > MAX_HITS_CEILING || period == 0 || period > MAX_PERIOD_SECS {
        return None;
    }
    Some(Window::new(
        max_hits,
        Duration::from_secs(period.into()),
        Duration::from_secs(restriction.into()),
    ))
}

// D8 Full grammar for a state triplet: hits >= 0 (counters legitimately
// start at zero, N24; synthesis is separately capped), period > 0 and
// bounded, restriction >= 0.
fn parse_window_state([current_hits, period, restriction_active]: [u32; 3]) -> Option<WindowState> {
    if period == 0 || period > MAX_PERIOD_SECS {
        return None;
    }
    Some(WindowState {
        current_hits,
        period: Duration::from_secs(period.into()),
        restriction_active: Duration::from_secs(restriction_active.into()),
    })
}
