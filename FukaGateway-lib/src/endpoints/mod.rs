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