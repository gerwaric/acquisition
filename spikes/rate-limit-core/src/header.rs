use std::time::Duration;

use http::{HeaderMap, HeaderName};

const POLICY_HEADER: HeaderName = HeaderName::from_static("x-rate-limit-policy");
const RULES_HEADER: HeaderName = HeaderName::from_static("x-rate-limit-rules");

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
    InvalidRuleName {
        raw: String,
    },
    MalformedTriplet {
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
    let name = required_header(headers, &POLICY_HEADER)?.to_owned();
    let rule_names = required_header(headers, &RULES_HEADER)?;
    let mut rules = Vec::new();

    for raw_rule in rule_names.split(',') {
        let rule = raw_rule.trim();
        if rule.is_empty() {
            return Err(PolicyParseError::InvalidRuleName {
                raw: raw_rule.to_owned(),
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
            raw: rule.to_owned(),
        }
    })
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
    parse: fn([u32; 3]) -> T,
) -> Result<Vec<T>, PolicyParseError> {
    required_header(headers, name)?
        .split(',')
        .map(|raw| {
            let malformed = || PolicyParseError::MalformedTriplet {
                header: name.clone(),
                raw: raw.to_owned(),
            };
            let fields = raw
                .trim()
                .split(':')
                .map(parse_strict_u32)
                .collect::<Option<Vec<_>>>()
                .ok_or_else(malformed)?;
            let fields: [u32; 3] = fields.try_into().map_err(|_| malformed())?;
            Ok(parse(fields))
        })
        .collect()
}

fn parse_window([max_hits, period, restriction]: [u32; 3]) -> Window {
    Window::new(
        max_hits,
        Duration::from_secs(period.into()),
        Duration::from_secs(restriction.into()),
    )
}

fn parse_window_state([current_hits, period, restriction_active]: [u32; 3]) -> WindowState {
    WindowState {
        current_hits,
        period: Duration::from_secs(period.into()),
        restriction_active: Duration::from_secs(restriction_active.into()),
    }
}
