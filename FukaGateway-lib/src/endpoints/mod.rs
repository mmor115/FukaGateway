use serde::Serialize;
use uuid::Uuid;

pub mod error;

#[derive(Serialize, Debug)]
#[serde(tag = "job_status")]
pub enum SubmitInfoFileResponse {
    Submitted { job_id: Uuid },
    AlreadySubmitted { job_id: Uuid },
    AlreadyRunning { job_id: Uuid },
    AlreadyFinished { job_id: Uuid },
    AlreadyFailed { job_id: Uuid }
}

#[derive(Serialize, Debug)]
#[serde(tag = "job_status")]
pub enum JobPollResponse {
    NotFound,
    Pending,
    Running,
    Finished,
    Failed
}