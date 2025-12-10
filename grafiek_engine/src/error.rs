use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unknown operation type: {0}")]
    UnknownOperationType(String),

    #[error("Node was configured with two slots named {0} on its {1}.")]
    DuplicateSlotName(String, String),

    #[error("Duplicate operation type: {0}/{1}")]
    DuplicateOperationType(&'static str, &'static str),

    #[error("Node not found: {0}")]
    NodeNotFound(String),

    #[error("No port on node: {0}")]
    NoPort(usize),

    #[error("No output slot at index {0}")]
    NoOutputSlot(usize),

    #[error("No input slot at index {0}")]
    NoInputSlot(usize),

    #[error("Incompatible types: output slot {from_slot} cannot connect to input slot {to_slot}")]
    IncompatibleTypes { from_slot: usize, to_slot: usize },

    #[error("Connection would create a cycle in the graph")]
    CreatesLoop,

    #[error("Edge not found: from slot {from_slot} to slot {to_slot}")]
    EdgeNotFound { from_slot: usize, to_slot: usize },

    #[error("Node accessed while modifying graph input was not an instance of core/input.")]
    NotInputNode,

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

    #[error("Script error: {0}")]
    Script(String),
}
