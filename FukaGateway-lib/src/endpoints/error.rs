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

use actix_web::http::header::ContentType;
use crate::info_file_parser::error::InfoFileParserError;
use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use serde_json::json;
use thiserror::Error;
use crate::database;
use crate::job;

#[derive(Error, Debug)]
pub enum EndpointError {
    #[error("Error parsing info file: {0}")]
    InfoFileParserError(#[from] InfoFileParserError),
    #[error("Database error: {0}")]
    DatabaseError(#[from] database::error::DatabaseError),
    #[error("Thread pool is shut down")]
    BlockingError(#[from] actix_web::error::BlockingError),
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Job management error: {0}")]   
    JobError(#[from] job::JobError),
    #[error("Shim error: {0}")]
    ShimError(#[from] reqwest::Error),
    #[error("No such resource")]
    NoSuchResource
}

impl actix_web::error::ResponseError for EndpointError {
    fn status_code(&self) -> StatusCode {
        match self {
            EndpointError::InfoFileParserError(_) => StatusCode::BAD_REQUEST,
            EndpointError::NoSuchResource => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR
        }
    }

    fn error_response(&self) -> HttpResponse {
        let err_body = json!({
            "error": format!("{self}")
        });
        
        HttpResponse::build(self.status_code())
                     .insert_header(ContentType::json())
                     .body(err_body.to_string())
    }
}