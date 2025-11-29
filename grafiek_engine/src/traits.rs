use crate::{
    Metadata, OpCategory, SignatureRegister, Value,
    error::{Error, Result},
};

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
trait Operation {}

trait Schema: Default {
    fn try_extract(values: &[()]) -> Result<Self>;
    fn try_write(&self, output: &mut [Value]) -> Result<()>;
    fn register(register: &mut SignatureRegister);
    fn metadata(field: &str) -> Metadata;
    fn fields() -> &'static [&'static str];
    fn len() -> usize;
}

trait Output: Schema {}
trait Input: Schema {}
trait Config: Schema {}

/// Trait for operations that can be registered and constructed from documents
pub trait OperationFactory: 'static {
    type Config: serde::Serialize + serde::de::DeserializeOwned + Default;

    fn operation_name() -> &'static str;
    fn library() -> &'static str;
    fn category() -> OpCategory;
    fn version() -> u32;

    fn create_boxed(config: Option<Self::Config>) -> Result<Box<dyn Operation>>;

    fn create_default() -> Result<Box<dyn Operation>> {
        Self::create_boxed(None)
    }
}
