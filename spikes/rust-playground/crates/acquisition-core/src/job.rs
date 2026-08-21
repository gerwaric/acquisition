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
        matches!(self, JobState::Done | JobState::Failed | JobState::Cancelled)
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
    #[serde(default)]
    pub parent: Option<JobId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum Outcome {
    Success { payload: Value },
    Failure { error: String },
    Cancelled,
}
