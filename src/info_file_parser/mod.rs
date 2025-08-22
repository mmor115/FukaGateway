use derive_more::Display;

pub(crate) mod lexer;
pub(crate) mod error;
pub(crate) mod parser;

type Result<T> = core::result::Result<T, error::InfoFileParserError>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Display)]
#[display("{line}:{col}")]
pub (crate) struct LineCol {
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
struct Interval<T> {
    begin: T,
    end: T
}