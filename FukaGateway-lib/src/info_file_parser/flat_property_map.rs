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

use std::collections::HashMap;
use std::mem;
use crate::info_file_parser::parser::ast::*;

pub type PropMap = HashMap<String, String>;

pub struct InfoFileToFlatPropertyMapVisitor<'a> {
    ast: &'a Root,
    map: PropMap,
    group_name_stack: Vec<&'a str>
}

impl<'a> InfoFileToFlatPropertyMapVisitor<'a> {
    pub fn new(ast: &'a Root) -> Self {
        Self {
            ast,
            map: HashMap::new(),
            group_name_stack: Vec::new()
        }
    }

    pub fn visit(&mut self) -> PropMap {
        for group in &self.ast.children {
            self.visit_group(group);
        }

        mem::take(&mut self.map)
    }

    fn visit_group(&mut self, group: &'a Group) {
        self.group_name_stack.push(&group.group_name);

        for member in &group.children {
            match member {
                GroupMember::KvPair(kv) => self.visit_kv_pair(kv),
                GroupMember::Group(g) => self.visit_group(g)
            }
        }

        self.group_name_stack.pop();
    }

    fn visit_kv_pair(&mut self, kv_pair: &'a KvPair) {
        let property_name = if self.group_name_stack.is_empty() {
            kv_pair.key.clone()
        } else {
            format!("{}::{}", self.group_name_stack.join("::"), kv_pair.key)
        };

        let property_value = kv_pair.value.to_string();

        self.map.insert(property_name, property_value);
    }
}
