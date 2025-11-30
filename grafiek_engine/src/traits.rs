use crate::{Metadata, OpCategory, SignatureRegister, Value, error::Result};

/// pass a type that implements this to Grafiek at start time.
/// The engine will make all the proper callbacks into this object to ensure
/// that your UI is synced with the engine graph model.
pub trait GrafiekObserver {
    /// Called when the graph has new errors to display
    fn show_errors(&mut self, errors: Vec<()>);
    /// Called when the error state goes from Some([..]) to None
    fn errors_cleared(&mut self);
    /// Called when a node must be added to the graph
    fn create_node(&mut self, node: (), id: ());
    /// Called when a node must be removed from the view
    fn remove_node(&mut self, node: ());
    /// Called when a node has been mutated
    fn node_updated(&mut self, id: (), node: ());
    /// An edge has been successfully added between two nodes
    fn add_edge(&mut self, node: ());
    /// An edge has been removed between two nodes
    fn remove_edge(&mut self, node: ());
    /// Execution has occurred
    fn execution(&mut self, node: ());
    /// The graph has been updated - the engine has determined that
    /// the resultant output of the graph would be different and has requested reevaluation
    fn graph_dirtied(&mut self, node: ());
}

/// Lifecycle and execution logic for
pub trait Operation {}

/// Convenience trait for describing reflective schemas for nodes defined in rust.
/// Allows for easy extraction of input and config and easy writing to output.
pub trait Schema: Default {
    fn try_extract(values: &[Value]) -> Result<Self>;
    fn try_write(&self, output: &mut [Value]) -> Result<()>;
    fn register(register: &mut SignatureRegister);
    fn metadata(field: &str) -> Metadata;
    fn fields() -> &'static [&'static str];
    /// Number of fields
    fn len() -> usize;
}

pub trait OutputSchema: Schema {}

pub trait InputSchema: Schema {}

pub trait ConfigSchema: Schema {}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct QualifiedName {
    operator_name: &'static str,
    library_name: &'static str,
}

impl std::fmt::Display for QualifiedName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.library_name, self.operator_name)
    }
}

/// Trait for operations that can be registered and constructed from documents
pub trait OperationFactory: 'static {
    const OPERATION_NAME: &'static str;
    const LIBRARY_NAME: &'static str;
    const CATEGORY: OpCategory;

    fn qualified_name() -> QualifiedName {
        QualifiedName {
            operator_name: Self::OPERATION_NAME,
            library_name: Self::LIBRARY_NAME,
        }
    }

    // TODO: we need to deal with migration logic at one point
    // Old version of nodes saved to disk will desync if the config
    // or inputs desync.
    //
    // This means at one point the setup workflow will work like
    // 1.) build vanilla with default config
    // 2.) migrate disk config
    // 3.) reconfigure node
    // ... connect and validate
    //fn migrate(config: &Record) -> Record;

    fn build() -> Result<Box<dyn Operation>>;
}

#[derive(Debug, Clone)]
pub(crate) struct OperationFactoryTable {
    pub build: fn() -> Result<Box<dyn Operation>>,
    pub category: OpCategory,
}

impl OperationFactoryTable {
    pub fn new<T: OperationFactory>() -> Self {
        Self {
            build: || T::build(),
            category: T::CATEGORY,
        }
    }
}
