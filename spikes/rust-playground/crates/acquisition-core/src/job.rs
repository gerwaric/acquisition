use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub type JobId = u64;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobState {
    Waiting,
    Running,
    Done,
    Failed,
    Cancelled,
}

impl JobState {
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            JobState::Done | JobState::Failed | JobState::Cancelled
        )
    }
}

impl fmt::Display for JobState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            JobState::Waiting => "waiting",
            JobState::Running => "running",
            JobState::Done => "done",
            JobState::Failed => "failed",
            JobState::Cancelled => "cancelled",
        };
        f.write_str(s)
    }
}

/// Higher priority runs sooner; ties break by submission order.
pub type Priority = u8;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JobInfo {
    pub id: JobId,
    pub kind: String,
    pub state: JobState,
    pub priority: Priority,
    pub submitted_by: String,
    /// Predicted seconds until this job starts running. Only meaningful for
    /// waiting jobs; computed from limiter state + same-route queue depth.
    pub eta_seconds: Option<u64>,
    /// The job that submitted this one (a refresh's tabs, a deep tab's
    /// substashes). A parent finishes when its last descendant does.
    pub parent: Option<JobId>,
    /// Times this job has been put back in the queue after a 429.
    pub retries: u32,
    /// The GGG account this job runs as (`name#discriminator`), fixed at
    /// submit: it selects the token it sends with and the store file its
    /// response lands in. `None` only for jobs that need no account
    /// (`sleep`, the mock's `fetch`).
    #[serde(default)]
    pub account: Option<String>,
    /// What the job was submitted with, verbatim. Public: every connected
    /// client sees it, so a job's params must never carry a secret (tokens
    /// are fetched inside the daemon, never passed in). This is what makes
    /// a queued job identifiable to a person — the prerequisite for any
    /// queue-management UI — and what lets a client label a failed child.
    pub params: Value,
}

impl JobInfo {
    /// A short human label for what the job targets, derived from its
    /// params: `Standard/cur1`, `Standard/maps/m003`, `Standard (all)`.
    /// Rendering is the client's business; this is the one shared default.
    pub fn target(&self) -> String {
        target_of(&self.kind, &self.params)
    }
}

/// `JobInfo::target` for a kind and params without a `JobInfo`.
/// A realm other than pc leads the label (`xbox/Standard/cur1`,
/// `poe2/Exile`); pc is silent, as on the wire.
pub fn target_of(kind: &str, p: &Value) -> String {
    {
        let s = |k: &str| p.get(k).and_then(Value::as_str);
        // `pc` and an absent or unparseable realm all show nothing: the
        // label is for a person, and admission already refused bad values.
        let r = match s("realm") {
            Some(realm) if realm != "pc" => format!("{realm}/"),
            _ => String::new(),
        };
        match kind {
            "stash" => match (s("league"), s("id"), s("sub")) {
                (Some(l), Some(id), Some(sub)) => format!("{r}{l}/{id}/{sub}"),
                (Some(l), Some(id), None) => format!("{r}{l}/{id}"),
                _ => String::new(),
            },
            "stashes" => format!("{r}{}", s("league").unwrap_or("")),
            "characters" => r.trim_end_matches('/').to_string(),
            "character" => format!("{r}{}", s("name").unwrap_or("")),
            "refresh" => {
                let l = s("league").unwrap_or("");
                if p.get("all").and_then(Value::as_bool).unwrap_or(false) {
                    format!("{r}{l} (all)")
                } else {
                    format!(
                        "{r}{l} ({} tabs)",
                        p.get("tabs").and_then(Value::as_array).map_or(0, Vec::len)
                    )
                }
            }
            "apply" => format!(
                "plan ({} requests)",
                p.get("jobs").and_then(Value::as_array).map_or(0, Vec::len)
            ),
            "probe" => s("route").unwrap_or("").to_string(),
            _ => String::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum Outcome {
    Success { payload: Value },
    Failure { error: String },
    Cancelled,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// pc is silent in a job's label, as on the wire; any other realm
    /// leads it.
    #[test]
    fn target_labels_lead_with_a_non_pc_realm_only() {
        assert_eq!(
            target_of(
                "stash",
                &json!({ "league": "Standard", "id": "t1", "sub": "s1" })
            ),
            "Standard/t1/s1"
        );
        assert_eq!(
            target_of(
                "stash",
                &json!({ "realm": "pc", "league": "Standard", "id": "t1" })
            ),
            "Standard/t1"
        );
        assert_eq!(
            target_of(
                "stash",
                &json!({ "realm": "xbox", "league": "Standard", "id": "t1" })
            ),
            "xbox/Standard/t1"
        );
        assert_eq!(
            target_of("stashes", &json!({ "realm": "sony", "league": "Standard" })),
            "sony/Standard"
        );
        assert_eq!(target_of("characters", &json!({})), "");
        assert_eq!(target_of("characters", &json!({ "realm": "poe2" })), "poe2");
        assert_eq!(target_of("character", &json!({ "name": "Exile" })), "Exile");
        assert_eq!(
            target_of("character", &json!({ "realm": "poe2", "name": "Exile" })),
            "poe2/Exile"
        );
        assert_eq!(
            target_of(
                "refresh",
                &json!({ "realm": "xbox", "league": "Standard", "all": true })
            ),
            "xbox/Standard (all)"
        );
    }
}
