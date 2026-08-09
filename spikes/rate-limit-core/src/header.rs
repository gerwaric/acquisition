use std::time::Duration;

use http::{HeaderMap, HeaderName};

const POLICY_HEADER: HeaderName = HeaderName::from_static("x-rate-limit-policy");
const RULES_HEADER: HeaderName = HeaderName::from_static("x-rate-limit-rules");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicySnapshot {
    name: String,
    rules: Vec<RuleSnapshot>,
}

impl PolicySnapshot {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn rules(&self) -> &[RuleSnapshot] {
        &self.rules
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleSnapshot {
    pub name: String,
    pub pair: RulePair,
    pub state: RuleStatePair,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window {
    max_hits: u32,
    period: Duration,
    restriction: Duration,
}

impl Window {
    pub fn max_hits(&self) -> u32 {
        self.max_hits
    }
    pub fn period(&self) -> Duration {
        self.period
    }
    pub fn restriction(&self) -> Duration {
        self.restriction
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RulePair {
    burst: Window,
    sustained: Window,
}

impl RulePair {
    pub fn burst(&self) -> &Window {
        &self.burst
    }
    pub fn sustained(&self) -> &Window {
        &self.sustained
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowState {
    current_hits: u32,
    period: Duration,
    restriction_active: Duration,
}

impl WindowState {
    pub fn current_hits(&self) -> u32 {
        self.current_hits
    }
    pub fn period(&self) -> Duration {
        self.period
    }
    pub fn restriction_active(&self) -> Duration {
        self.restriction_active
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleStatePair {
    burst: WindowState,
    sustained: WindowState,
}

impl RuleStatePair {
    pub fn burst(&self) -> &WindowState {
        &self.burst
    }
    pub fn sustained(&self) -> &WindowState {
        &self.sustained
    }
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
        if burst.period >= sustained.period {
            return Err(PolicyParseError::NonIncreasingPeriods {
                rule: rule.to_owned(),
                burst_period: burst.period,
                sustained_period: sustained.period,
            });
        }

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
        if burst_state.period != burst.period || sustained_state.period != sustained.period {
            return Err(PolicyParseError::StatePeriodsMismatch {
                rule: rule.to_owned(),
            });
        }

        rules.push(RuleSnapshot {
            name: rule.to_owned(),
            pair: RulePair { burst, sustained },
            state: RuleStatePair {
                burst: burst_state,
                sustained: sustained_state,
            },
        });
    }
    if rules.is_empty() {
        return Err(PolicyParseError::InvalidRuleName { raw: String::new() });
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

fn parse_triplets<T>(
    headers: &HeaderMap,
    name: &HeaderName,
    parse: fn([u32; 3]) -> T,
) -> Result<Vec<T>, PolicyParseError> {
    required_header(headers, name)?
        .split(',')
        .map(|raw| {
            let fields = raw
                .trim()
                .split(':')
                .map(str::parse::<u32>)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| PolicyParseError::MalformedTriplet {
                    header: name.clone(),
                    raw: raw.to_owned(),
                })?;
            let fields: [u32; 3] =
                fields
                    .try_into()
                    .map_err(|_| PolicyParseError::MalformedTriplet {
                        header: name.clone(),
                        raw: raw.to_owned(),
                    })?;
            Ok(parse(fields))
        })
        .collect()
}

fn parse_window([max_hits, period, restriction]: [u32; 3]) -> Window {
    Window {
        max_hits,
        period: Duration::from_secs(period.into()),
        restriction: Duration::from_secs(restriction.into()),
    }
}

fn parse_window_state([current_hits, period, restriction_active]: [u32; 3]) -> WindowState {
    WindowState {
        current_hits,
        period: Duration::from_secs(period.into()),
        restriction_active: Duration::from_secs(restriction_active.into()),
    }
}
