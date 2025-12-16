use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct LocatedError {
    pub message: String,
    pub line: u32,
    pub column: u32,
}

impl std::fmt::Display for LocatedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}: {}", self.line, self.column, self.message)
    }
}

#[derive(Debug, Clone)]
pub struct ScriptError {
    pub errors: Vec<LocatedError>,
}

impl ScriptError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            errors: vec![LocatedError {
                message: message.into(),
                line: 0,
                column: 0,
            }],
        }
    }
}

impl std::fmt::Display for ScriptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, err) in self.errors.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{}", err)?;
        }
        Ok(())
    }
}

impl ScriptError {
    /// Create from tweak_shader error with pragma line offset
    pub fn from_tweak_shader(err: tweak_shader::Error) -> Self {
        match err {
            tweak_shader::Error::ShaderCompilationFailed { errors, .. } => {
                let errors = errors
                    .into_iter()
                    .map(|e| LocatedError {
                        message: format!("{:?}", e.kind),
                        line: e.location.line,
                        column: e.location.column,
                    })
                    .collect();
                Self { errors }
            }
            other => Self::new(other.to_string()),
        }
    }
}

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

    #[error("{0}")]
    Script(ScriptError),
}

impl Error {
    /// Get structured script error info if this is a script error
    pub fn as_script_error(&self) -> Option<&ScriptError> {
        match self {
            Error::Script(e) => Some(e),
            _ => None,
        }
    }
}
