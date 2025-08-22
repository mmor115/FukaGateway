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

pub struct Group {
    pub group_name: String,
    pub children: Vec<GroupMember>
}

pub enum GroupMember {
    KvPair(KvPair),
    Group(Group)
}