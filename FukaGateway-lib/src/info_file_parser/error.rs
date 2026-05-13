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
