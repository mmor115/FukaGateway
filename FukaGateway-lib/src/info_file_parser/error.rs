use crate::info_file_parser::error::InfoFileParserError::*;
use crate::info_file_parser::lexer::InfoFileToken;
use crate::info_file_parser::LineCol;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InfoFileParserError {
    #[error("Unexpected end of file at {0}")]   
    UnexpectedEof(LineCol),
    #[error("Unexpected character {0} at {1}")]  
    UnexpectedChar(char, LineCol),
    #[error("Unexpected token {0} at {pos}", pos = .0.interval.begin)]
    UnexpectedToken(InfoFileToken),
    #[error("Illegal numerical value {0} at {pos}: {1:?}", pos = .0.interval.begin)]
    IllegalNumericalValue(InfoFileToken, std::num::ParseFloatError)
}

impl InfoFileParserError {
    pub fn unexpected_char(c: Option<char>, lc: LineCol) -> Self {
        match c {
            None => UnexpectedEof(lc),
            Some(c) => UnexpectedChar(c, lc)
        }
    }
}
