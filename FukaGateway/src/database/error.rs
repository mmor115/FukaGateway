use thiserror::Error;

#[derive(Error, Debug)]
#[error("{0}")]
pub struct DatabaseError(#[from] polodb_core::Error);
