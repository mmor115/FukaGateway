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

use std::str::FromStr;
use crate::info_file_parser::error::InfoFileParserError::{IllegalNumericalValue, UnexpectedEof, UnexpectedToken};
use crate::info_file_parser::lexer::{InfoFileToken, InfoFileTokenKind};
use crate::info_file_parser::{LineCol, Result};

pub mod ast;

pub struct InfoFileParser {
    tokens: Vec<InfoFileToken>,
    pos: usize,
    most_recent_token_pos: Option<usize>
}

impl InfoFileParser {
    pub fn new(tokens: Vec<InfoFileToken>) -> Self {
        Self {
            tokens,
            pos: 0,
            most_recent_token_pos: None
        }
    }

    fn most_recent_line_col(&self) -> LineCol {
        match self.most_recent_token_pos {
            None => LineCol::new(0, 0),
            Some(t) => self.tokens[t].interval.begin
        }
    }

    #[must_use = "This method does not advance the parser position. Use advance() to advance the parser position."]
    fn peek(&self) -> Option<&InfoFileToken> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<&InfoFileToken> {
        if self.pos >= self.tokens.len() {
            return None;
        }

        let t = &self.tokens[self.pos];
        self.most_recent_token_pos = Some(self.pos);
        self.pos += 1;

        Some(t)
    }

    fn look_ahead(&self, n: usize) -> Option<&InfoFileToken> {
        self.tokens.get(self.pos + n)
    }

    fn expect(&mut self, k: InfoFileTokenKind) -> Result<&InfoFileToken> {
        match self.tokens.get(self.pos) {
            Some(t) if t.kind == k => {
                self.most_recent_token_pos = Some(self.pos);
                self.pos += 1;
                Ok(t)
            },
            Some(t) => {
                Err(UnexpectedToken(t.clone()))
            },
            None => {
                Err(UnexpectedEof(self.most_recent_line_col()))
            }
        }
    }
    
    pub fn parse(&mut self) -> Result<ast::Root> {
        let mut children = Vec::new();

        while self.peek().is_some() {
            children.push(self.parse_group()?);
        }
        
        Ok(ast::Root {
            children
        })
    }

    fn parse_group_member(&mut self) -> Result<ast::GroupMember> {
        Ok(match self.look_ahead(1) {
            Some(InfoFileToken { kind: InfoFileTokenKind::LCurly, .. }) => {
                ast::GroupMember::Group(self.parse_group()?)
            },
            _ => {
                ast::GroupMember::KvPair(self.parse_kv_pair()?)
            }
        })
    }

    fn parse_group(&mut self) -> Result<ast::Group> {
        let mut children = Vec::new();

        let group_name = self.expect(InfoFileTokenKind::Ident)?.text.clone();

        self.expect(InfoFileTokenKind::LCurly)?;

        while let Some(t) = self.peek() {
            match t.kind {
                InfoFileTokenKind::Ident => {
                    children.push(self.parse_group_member()?);
                },
                InfoFileTokenKind::RCurly => {
                    self.advance();
                    return Ok(ast::Group {
                        group_name,
                        children,
                    });
                },
                _ => {
                    return Err(UnexpectedToken((*t).clone()));
                }
            }
        }

        Err(UnexpectedEof(self.most_recent_line_col()))
    }

    fn parse_kv_pair(&mut self) -> Result<ast::KvPair> {
        let property_name = self.expect(InfoFileTokenKind::Ident)?.text.clone();
        let property_value = self.parse_value()?;

        Ok(ast::KvPair {
            key: property_name,
            value: property_value,
        })
    }

    fn parse_value(&mut self) -> Result<ast::Value> {
        match self.advance() {
            Some(t) => {
                match t.kind {
                    InfoFileTokenKind::Ident if t.text == "on" => Ok(ast::Value::Bool(true)),
                    InfoFileTokenKind::Ident if t.text == "off" => Ok(ast::Value::Bool(false)),
                    InfoFileTokenKind::Ident => Ok(ast::Value::String(t.text.clone())),
                    InfoFileTokenKind::Num => Ok(ast::Value::Number(f64::from_str(&t.text).map_err(|e| IllegalNumericalValue(t.clone(), e))?)),
                    _ => Err(UnexpectedToken(t.clone()))
                }
            },
            None => Err(UnexpectedEof(self.most_recent_line_col()))
        }
    }
}