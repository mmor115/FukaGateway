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