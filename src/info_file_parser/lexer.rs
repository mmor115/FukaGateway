use derive_more::Display;
use InfoFileParserError::UnexpectedChar;
use crate::info_file_parser::{Interval, LineCol, Result};
use crate::info_file_parser::error::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum InfoFileTokenKind {
    Ident,
    Num,
    LCurly,
    RCurly
}

#[derive(Debug, Clone, PartialEq, Eq, Display)]
#[display("{kind}({text})")]
pub struct InfoFileToken {
    pub kind: InfoFileTokenKind,
    pub text: String,
    pub interval: Interval<LineCol>
}

pub struct InfoFileLexer {
    text: Vec<char>,
    pos: usize,
    line_col_pos: LineCol
}

impl InfoFileLexer {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.chars().collect(),
            pos: 0,
            line_col_pos: LineCol::new(1, 1)
        }
    }

    fn advance(&mut self) -> Option<char> {
        if self.pos >= self.text.len() {
            return None;
        }

        let c = self.text[self.pos];
        self.pos += 1;

        self.line_col_pos = if c == '\n' {
            self.line_col_pos.next_line()
        } else {
            self.line_col_pos.next_col()
        };

        Some(c)
    }

    #[must_use = "This method does not advance the lexer position. Use advance() to advance the lexer position."]
    fn peek(&self) -> Option<char> {
        if self.pos >= self.text.len() {
            return None;
        }

        Some(self.text[self.pos])
    }

    pub fn lex(&mut self) -> Result<Vec<InfoFileToken>> {
        let mut res = Vec::new();

        while let Some(c) = self.peek() {
            match c {
                '\n' | '\r' | '\t' | ' ' => {
                    self.advance();
                },
                'a'..='z' | 'A'..='Z' | '_' => {
                    res.push(self.lex_identifier()?);
                },
                '0'..='9' | '-' => {
                    res.push(self.lex_number()?);
                },
                '{' | '}' => {
                    let (pos, pos_lc) = (self.pos, self.line_col_pos);
                    let kind = if let Some('{') = self.advance() {
                        InfoFileTokenKind::LCurly
                    } else {
                        InfoFileTokenKind::RCurly
                    };

                    res.push(InfoFileToken {
                        kind,
                        text: self.begin_to_owned_string(pos),
                        interval: self.begin_to_interval(pos_lc),
                    });
                }
                _ => {
                    return Err(UnexpectedChar(c, self.line_col_pos));
                }
            }
        }

        Ok(res)
    }

    fn lex_identifier(&mut self) -> Result<InfoFileToken> {
        let (begin, begin_lc) = (self.pos, self.line_col_pos);
        
        while let Some(c) = self.peek() {
            match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '.' => {
                    self.advance();
                },
                _ => {
                    break;
                }
            }
        }

        Ok(InfoFileToken {
            kind: InfoFileTokenKind::Ident,
            text: self.begin_to_owned_string(begin),
            interval: self.begin_to_interval(begin_lc)
        })
    }

    fn lex_number(&mut self) -> Result<InfoFileToken> {
        let (begin, begin_lc) = (self.pos, self.line_col_pos);

        if let Some('-') = self.peek() {
            self.advance();
        }

        let mut n_int_digits = 0;
        while let Some(c) = self.peek() {
            match c {
                '0'..='9' => {
                    self.advance();
                    n_int_digits += 1;
                },
                _ => {
                    break;
                }
            }
        }

        if n_int_digits == 0 {
            return Err(InfoFileParserError::unexpected_char(self.peek(), self.line_col_pos));
        }

        if self.peek() == Some('.') {
            self.advance();

            let mut n_frac_digits = 0;
            while let Some(c) = self.peek() {
                match c {
                    '0'..='9' => {
                        self.advance();
                        n_frac_digits += 1;
                    },
                    _ => {
                        break;
                    }
                }
            }

            if n_frac_digits == 0 {
                return Err(InfoFileParserError::unexpected_char(self.peek(), self.line_col_pos));
            }
        }

        if self.peek() == Some('e') || self.peek() == Some('E') {
            self.advance();

            if self.peek() == Some('-') || self.peek() == Some('+') {
                self.advance();
            }

            let mut n_exp_digits = 0;
            while let Some(c) = self.peek() {
                match c {
                    '0'..='9' => {
                        self.advance();
                        n_exp_digits += 1;
                    },
                    _ => {
                        break;
                    }
                }
            }

            if n_exp_digits == 0 {
                return Err(InfoFileParserError::unexpected_char(self.peek(), self.line_col_pos));
            }
        }

        Ok(InfoFileToken {
            kind: InfoFileTokenKind::Num,
            text: self.begin_to_owned_string(begin),
            interval: self.begin_to_interval(begin_lc)
        })
    }

    fn begin_to_owned_string(&self, begin: usize) -> String {
        self.text[begin..self.pos].iter().collect::<String>()
    }

    fn range_to_owned_string(&self, begin: usize, end: usize) -> String {
        self.text[begin..end].iter().collect::<String>()
    }

    fn begin_to_interval(&self, begin: LineCol) -> Interval<LineCol> {
        Interval {
            begin,
            end: self.line_col_pos,
        }
    }
}