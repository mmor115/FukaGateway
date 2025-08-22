use crate::info_file_parser::error::InfoFileParserError::{UnexpectedChar, UnexpectedEof, UnexpectedToken};
use crate::info_file_parser::LineCol;
use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use thiserror::Error;
use crate::info_file_parser::lexer::InfoFileToken;

#[derive(Error, Debug)]
pub enum InfoFileParserError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Unexpected end of file at {0}")]   
    UnexpectedEof(LineCol),
    #[error("Unexpected character {0} at {1}")]  
    UnexpectedChar(char, LineCol),
    #[error("Unexpected token {0} at {pos}", pos = .0.interval.begin)]
    UnexpectedToken(InfoFileToken)
}

impl InfoFileParserError {
    pub fn unexpected_char(c: Option<char>, lc: LineCol) -> Self {
        match c {
            None => UnexpectedEof(lc),
            Some(c) => UnexpectedChar(c, lc)
        }
    }
}

impl actix_web::error::ResponseError for InfoFileParserError {
    fn status_code(&self) -> StatusCode {
        match self {
            InfoFileParserError::IoError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            UnexpectedEof(_) => StatusCode::BAD_REQUEST,
            UnexpectedChar(_, _) => StatusCode::BAD_REQUEST,
            UnexpectedToken(_) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
                     .insert_header(ContentType::html())
                     .body(format!("Error while parsing info file: {}", self))
    }
}
