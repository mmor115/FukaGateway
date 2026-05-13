// Copyright (C) 2026 Max Morris.
// 
// This file is part of FukaGateway.
// 
// FukaGateway is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
// 
// FukaGateway is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
// 
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use async_process::Command;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::Path;
use thiserror::Error;
use uuid::Uuid;
use crate::job::JobStatus::NotStarted;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum JobStatus {
    NotStarted,
    Pending {
        slurm_job_id: usize
    },
    Running {
        slurm_job_id: usize
    },
    Finished,
    Failed
}

#[derive(Error, Debug)]
pub enum JobError {
    #[error("Problem communicating with slurm: {0}")]
    SlurmCommunication(String),
    #[error("Problem enqueueing the job with slurm: {0}")]
    SlurmEnqueue(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error)
}

pub async fn get_job_status(job_id: Uuid) -> Result<Option<JobStatus>, JobError> {
    if !Path::exists(get_job_dir(job_id).as_ref()) {
        return Ok(None);
    }

    if Path::exists(get_result_file_path(job_id).as_ref()) {
        return Ok(Some(JobStatus::Finished));
    }

    let output =
        Command::new("squeue")
                .arg("--noheader")
                .arg("--array")
                .arg("--Format")
                .arg("jobid:##,state")
                .arg("--name")
                .arg(job_id.to_string())
                .output()
                .await?;

    if !output.status.success() {
        return Err(JobError::SlurmCommunication(String::from_utf8_lossy(&output.stderr).to_string()));
    }

    let output = String::from_utf8_lossy(&output.stdout);

    if output.is_empty() {
        return Ok(Some(NotStarted));
    }

    let mut parts = output.splitn(2, "##");

    let slurm_job_id =
        parts.next()
             .and_then(|s| s.trim().parse::<usize>().ok())
             .ok_or_else(|| JobError::SlurmCommunication(String::from("Could not parse slurm job id")))?;

    match parts.next().map(|s| s.trim().to_ascii_uppercase()).as_deref() {
        Some("COMPLETED") => {
            Ok(Some(JobStatus::Finished))
        },
        Some("BOOT_FAIL") | Some("CANCELLED") | Some("DEADLINE") | Some("FAILED") | Some("NODE_FAIL") | Some("OUT_OF_MEMORY") | Some("PREEMPTED") | Some("TIMEOUT") => {
            Ok(Some(JobStatus::Failed))
        },
        Some("PENDING") | Some("CONFIGURING") | Some("RESIZING") | Some("REQUEUED") => {
            Ok(Some(JobStatus::Pending { slurm_job_id }))
        },
        Some("RUNNING") | Some("COMPLETING") => {
            Ok(Some(JobStatus::Running { slurm_job_id }))
        },
        Some(s) => {
            Err(JobError::SlurmCommunication(format!("Could not parse slurm job state: {}", s)))
        },
        None => {
            Err(JobError::SlurmCommunication(String::from("Could not parse slurm job state")))
        }
    }
}

pub async fn submit_job(job_id: Uuid) -> Result<usize, JobError> {
    let output =
        Command::new("sbatch")
                .arg("-D")
                .arg(get_job_dir(job_id))
                .arg("--job-name")
                .arg(format!("{job_id}"))
                .arg(get_sbatch_script_path())
                .output()
                .await?;

    if !output.status.success() {
        return Err(JobError::SlurmEnqueue(String::from_utf8_lossy(&output.stderr).to_string()));
    }

    let output = String::from_utf8_lossy(&output.stdout);

    if output.is_empty() {
        return Err(JobError::SlurmEnqueue(String::from("Slurm rejected the job")));
    }

    let mut parts = output.splitn(4, " ");

    if !matches!(parts.next(), Some("Submitted")) {
        return Err(JobError::SlurmEnqueue(String::from("Slurm rejected the job")));
    }

    if !matches!(parts.next(), Some("batch")) {
        return Err(JobError::SlurmEnqueue(String::from("Slurm rejected the job")));
    }

    if !matches!(parts.next(), Some("job")) {
        return Err(JobError::SlurmEnqueue(String::from("Slurm rejected the job")));
    }

    let slurm_job_id =
        parts.next()
             .and_then(|s| s.trim().parse::<usize>().ok())
             .ok_or_else(|| JobError::SlurmEnqueue(String::from("Could not parse slurm job id")))?;

    Ok(slurm_job_id)
}

pub fn get_jobs_dir() -> String {
    env::var("FUKAGATEWAY_JOBS_DIR").expect("FUKAGATEWAY_JOBS_DIR must be set")
}

pub fn get_scripts_dir() -> String {
    env::var("FUKAGATEWAY_SCRIPTS_DIR").expect("FUKAGATEWAY_SCRIPTS_DIR must be set")
}

pub fn get_job_dir(job_id: Uuid) -> String {
    Path::new(&get_jobs_dir()).join(job_id.to_string()).to_str().unwrap().to_string()
}

pub fn get_sbatch_script_path() -> String {
    Path::new(&get_scripts_dir()).join("kadath.sh").to_str().unwrap().to_string()
}

pub fn get_info_file_path(job_id: Uuid) -> String {
    Path::new(&get_jobs_dir()).join(job_id.to_string()).join("initial_bns.info").to_str().unwrap().to_string()
}

pub fn get_result_file_path(job_id: Uuid) -> String {
    Path::new(&get_jobs_dir()).join(job_id.to_string()).join("data.tar.gz").to_str().unwrap().to_string()
}

