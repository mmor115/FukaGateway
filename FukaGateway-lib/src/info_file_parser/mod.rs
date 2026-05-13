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

use derive_more::Display;

pub mod lexer;
pub mod error;
pub mod parser;
pub mod flat_property_map;

type Result<T> = core::result::Result<T, error::InfoFileParserError>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Display)]
#[display("{line}:{col}")]
pub struct LineCol {
    line: u64,
    col: u64
}

impl LineCol {
    pub fn new(line: u64, col: u64) -> Self {
        Self {
            line,
            col
        }
    }

    pub fn next_line(&self) -> Self {
        Self::new(self.line + 1, 1)
    }

    pub fn next_col(&self) -> Self {
        Self::new(self.line, self.col + 1)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Display)]
#[display("[{begin}..{end}]")]
pub struct Interval<T> {
    begin: T,
    end: T
}