use crate::info_file_parser::error::InfoFileParserError::UnexpectedToken;
use crate::info_file_parser::lexer::{InfoFileToken, InfoFileTokenKind};
use crate::info_file_parser::Result;

pub mod ast;

pub struct InfoFileParser {
    tokens: Vec<InfoFileToken>,
    pos: usize
}

impl InfoFileParser {
    pub fn new(tokens: Vec<InfoFileToken>) -> Self {
        Self {
            tokens,
            pos: 0
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
        self.pos += 1;

        Some(t)
    }

    fn look_ahead(&self, n: usize) -> Option<&InfoFileToken> {
        self.tokens.get(self.pos + n)
    }

    fn expect(&mut self, t: &InfoFileToken) -> Option<&InfoFileToken> {
        if self.peek() == Some(t) {
            self.advance()
        } else {
            None
        }
    }
    
    pub fn parse(&mut self) -> Result<ast::Root> {
        let mut children = Vec::new();

        while let Some(t) = self.peek() {
            match t.kind {
                InfoFileTokenKind::Ident => {
                    children.push(self.parse_group()?);
                }
                _ => {
                    return Err(UnexpectedToken(t.clone()))
                }
            }
        }
        
        Ok(ast::Root {
            children
        })
    }

    fn parse_group_member(&mut self) -> Result<ast::GroupMember> {
        unimplemented!()
    }

    fn parse_group(&mut self) -> Result<ast::Group> {
        unimplemented!()
    }

    fn parse_kv_pair(&mut self) -> Result<ast::KvPair> {
        unimplemented!()
    }

    fn parse_value(&mut self) -> Result<ast::Value> {
        unimplemented!()
    }
}