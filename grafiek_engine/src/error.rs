use thiserror::Error;

use crate::traits::QualifiedName;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unknown operation type: {0}")]
    UnknownOperationType(String),

    #[error("Duplicate operation type: {0}")]
    DuplicateOperationType(QualifiedName),

    #[error("Node not found: {0}")]
    NodeNotFound(String),

    #[error("Invalid edge: from node {from} to node {to}")]
    InvalidEdge { from: usize, to: usize },

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Value error: {0}")]
    Value(#[from] crate::value::ValueError),

    #[error("Input node has incoming connection and cannot be edited")]
    InputHasConnection,
}
