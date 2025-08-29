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