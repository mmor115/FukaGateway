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

use std::fmt::{Display, Formatter};

pub struct Root {
    pub children: Vec<Group>
}

pub struct KvPair {
    pub key: String,
    pub value: Value
}

pub enum Value {
    String(String),
    Number(f64),
    Bool(bool)
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(s) => f.write_str(s),
            Value::Number(n) => write!(f, "{}", *n),
            Value::Bool(b) => write!(f, "{}", *b)
        }
    }
}

pub struct Group {
    pub group_name: String,
    pub children: Vec<GroupMember>
}

pub enum GroupMember {
    KvPair(KvPair),
    Group(Group)
}