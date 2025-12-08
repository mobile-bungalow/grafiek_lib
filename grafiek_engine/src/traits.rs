use std::any::Any;

use crate::error::Result;
use crate::registry::SignatureRegistery;
use crate::value::{Config, Inputs, Outputs};
use crate::{AsValueType, ExecutionContext, ValueType};

// Node lifecycle
// 1.) Config and Inputs deserialized.
// 3.)  - Configure Called
// 3.a) - Configure called on every change to the config object
// 4.) On Edge Connected called on every connection
// 4.a) - validate edges called on connection
// 5.) On edge disconnected called on every disconnection
// 6.) Execute called
// 7.) Serialized to disk.
// This trait should assist with most of these stages
/// Dynamic operation trait that each node type implements
pub trait Operation: Any {
    /// If your operation maintains state between calls it's important to note so! some applications
    /// might forbid anything except idempotent operations.
    fn is_stateful(&self) -> bool;

    /// Called when node is added to the graph - this is when the
    /// node defines it's default state by registering it's input signature, output signature
    /// and default config to the engine. immediately after this is invoked
    /// [Operation::configure] is called with the default config - or if we are reloading
    /// from disk, a saved config.
    ///
    /// TODO: The saved config in the last step should be migrated when we actually have users
    /// who are trying to load graphs from previous versions of the engine.
    fn setup(&mut self, ctx: &mut ExecutionContext, registry: &mut SignatureRegistery);

    /// Configure the operation based on config values
    /// Called after config values are updated to allow operation to reconfigure itself as well as directly
    /// after setup.
    fn configure(
        &mut self,
        _ctx: &ExecutionContext,
        _config: Config,
        _registry: &mut SignatureRegistery,
    ) -> Result<()> {
        Ok(())
    }

    /// Execute this operation with the given inputs and outputs
    fn execute(
        &mut self,
        _ctx: &mut ExecutionContext,
        _inputs: Inputs,
        _outputs: Outputs,
    ) -> Result<()> {
        Ok(())
    }

    /// Get the type name for this operation (used for serialization)
    fn op_path(&self) -> OpPath;

    /// Called when node is removed from graph - make sure to clean up
    /// any resources you left in the execution context
    fn teardown(&mut self, _ctx: &mut ExecutionContext) {}

    /// Optional debug print for dumping internal state
    /// Default implementation does nothing
    fn debug_print(&self, _writer: &mut dyn std::io::Write) -> Result<()> {
        Ok(())
    }

    /// Called when an edge is connected to this operation
    /// Default implementation does nothing
    fn on_edge_connected(
        &mut self,
        _slot: usize,
        _connected_type: ValueType,
        _registry: &mut SignatureRegistery,
    ) -> Result<()> {
        Ok(())
    }

    /// Called when an edge is disconnected from this operation
    fn on_edge_disconnected(
        &mut self,
        _slot: usize,
        _connected_type: ValueType,
        _registry: &mut SignatureRegistery,
    ) -> Result<()> {
        Ok(())
    }
}

pub trait Schema: Default {
    fn register(registry: &mut SignatureRegistery);
    fn try_extract(values: Config) -> Result<Self>;
}

pub trait OutputSchema: Schema {
    fn try_write(&self, output: Outputs) -> Result<()>;
}

pub trait InputSchema: Schema {}
pub trait ConfigSchema: Schema {}

/// You probably won't have to implement this by hand. Instead use the
/// derive macro. Enums with const integer representations can be
/// used as fields in structs which derive schema.
pub trait SchemaEnum: Default {
    const VARIANTS: &'static [(&str, i32)];
}

impl<T> AsValueType for T
where
    T: SchemaEnum,
{
    const VALUE_TYPE: ValueType = ValueType::I32;

    fn default_metadata() -> Option<crate::ExtendedMetadata> {
        Some(crate::ExtendedMetadata::IntEnum(crate::IntEnum {
            options: T::VARIANTS
                .iter()
                .map(|(name, val)| (name.to_string(), *val))
                .collect(),
        }))
    }
}

/// Unique identifier for an operation type.
#[derive(Clone, Debug, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct OpPath {
    pub library: String,
    pub operator: String,
}

/// Trait for operations that can be registered and constructed from documents
pub trait OperationFactory: 'static {
    const LIBRARY: &'static str;
    const OPERATOR: &'static str;

    fn op_path() -> OpPath {
        OpPath {
            library: Self::LIBRARY.to_owned(),
            operator: Self::OPERATOR.to_owned(),
        }
    }

    /// Human-readable label for the operation (can be duplicated across operations)
    const LABEL: &'static str;

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

/// Hand build vtable for constructing Operators
#[derive(Debug, Clone)]
pub(crate) struct OperationFactoryEntry {
    pub build: fn() -> Result<Box<dyn Operation>>,
}

impl OperationFactoryEntry {
    pub fn new<T: OperationFactory>() -> Self {
        Self {
            build: || T::build(),
        }
    }
}
