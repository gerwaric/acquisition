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
        let p = &self.params;
        let s = |k: &str| p.get(k).and_then(Value::as_str);
        match self.kind.as_str() {
            "stash" => match (s("league"), s("id"), s("sub")) {
                (Some(l), Some(id), Some(sub)) => format!("{l}/{id}/{sub}"),
                (Some(l), Some(id), None) => format!("{l}/{id}"),
                _ => String::new(),
            },
            "stashes" => s("league").unwrap_or("").to_string(),
            "refresh" => {
                let l = s("league").unwrap_or("");
                if p.get("all").and_then(Value::as_bool).unwrap_or(false) {
                    format!("{l} (all)")
                } else {
                    format!(
                        "{l} ({} tabs)",
                        p.get("tabs").and_then(Value::as_array).map_or(0, Vec::len)
                    )
                }
            }
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
