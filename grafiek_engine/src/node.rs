use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::traits::{OpPath, Operation};
use crate::value::{Config, Inputs, Outputs};
use crate::{ExecutionContext, SignatureRegistery, SlotDef, Value, ValueMut};

/// Engine provided unique ID
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct NodeId(pub u64);

/// Serializable record of a node's state - can be saved to disk or undo queue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeRecord {
    /// Unique ID assigned by the engine
    pub id: NodeId,
    /// Path to the Operator in the registry
    pub op_path: OpPath,
    pub label: Option<String>,
    /// Position in graph space - 0,0 if invalid, client dependant
    /// WARNING: The client will have to set this on save.
    pub position: (f32, f32),
    /// Input values for any unconnected inputs - these must be restored
    /// If the node is disconnected or loaded from disk.
    pub input_values: Vec<Value>,
    /// Config values for any settings related to node operation
    pub config_values: Vec<Value>,
}

impl NodeRecord {
    pub fn new(id: NodeId, op_path: OpPath) -> Self {
        Self {
            id,
            op_path,
            label: None,
            position: (0.0, 0.0),
            input_values: vec![],
            config_values: vec![],
        }
    }
}

/// Thread-safe dirty flag that can be shared with background tasks
#[derive(Clone, Default)]
pub struct DirtyFlag(Arc<AtomicBool>);

impl DirtyFlag {
    pub fn new() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }

    pub fn set(&self) {
        self.0.store(true, Ordering::Release);
    }

    pub fn get(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }

    pub fn clear(&self) {
        self.0.store(false, Ordering::Release);
    }
}

/// Runtime node state including operation instance
pub struct Node {
    record: NodeRecord,
    signature: SignatureRegistery,
    output_values: Vec<Value>,
    /// these are None if there is no connected edge(s) to the corresponding slot
    incoming_input_values: Vec<Option<Value>>,
    operation: Box<dyn Operation>,
    dirty: DirtyFlag,
}

/// Result of probing whether a connection is valid.
/// Node-level concerns (slots and types) are checked by Node::probe_connect.
/// Graph-level concerns (loops, existing edges) are checked by Engine::connect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionProbe {
    /// Connection is valid
    Ok,
    /// Source output slot doesn't exist
    NoSourceSlot,
    /// Sink input slot doesn't exist
    NoSinkSlot,
    /// Types are incompatible (cannot cast source to sink)
    Incompatible,
    /// Connection would create a cycle in the graph
    CreatesLoop,
}

impl Node {
    pub fn new(operation: Box<dyn Operation>, id: NodeId) -> Self {
        Self {
            record: NodeRecord::new(id, operation.op_path()),
            signature: SignatureRegistery::default(),
            output_values: vec![],
            incoming_input_values: vec![],
            operation,
            dirty: DirtyFlag::new(),
        }
    }

    pub(crate) fn record(&self) -> &NodeRecord {
        &self.record
    }

    pub(crate) fn record_mut(&mut self) -> &mut NodeRecord {
        &mut self.record
    }

    pub fn label(&self) -> &str {
        self.record
            .label
            .as_deref()
            .unwrap_or(&self.record.op_path.operator)
    }

    pub fn position(&self) -> (f32, f32) {
        self.record.position
    }

    pub fn op_path(&self) -> &OpPath {
        &self.record.op_path
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty.get()
    }

    fn clear_dirty(&self) {
        self.dirty.clear();
    }

    fn mark_dirty(&self) {
        self.dirty.set();
    }

    /// Get a clone of the dirty flag for use in background tasks
    pub fn dirty_flag(&self) -> DirtyFlag {
        self.dirty.clone()
    }

    /// Check if this node's output can connect to another node's input.
    /// Only validates slot existence and type compatibility.
    pub fn probe_connect(&self, other: &Node, from_port: usize, to_port: usize) -> ConnectionProbe {
        let Some(output_def) = self.signature.output(from_port) else {
            return ConnectionProbe::NoSourceSlot;
        };

        let Some(input_def) = other.signature.input(to_port) else {
            return ConnectionProbe::NoSinkSlot;
        };

        if !output_def.value_type.can_cast_to(&input_def.value_type) {
            return ConnectionProbe::Incompatible;
        }

        ConnectionProbe::Ok
    }

    /// Get the signature for read access
    pub fn signature(&self) -> &SignatureRegistery {
        &self.signature
    }
}

// Slot access - unified API for inputs, outputs, and configs
impl Node {
    /// Number of input slots
    pub fn input_count(&self) -> usize {
        self.signature.input_count()
    }

    /// Number of output slots
    pub fn output_count(&self) -> usize {
        self.signature.output_count()
    }

    /// Number of config slots
    pub fn config_count(&self) -> usize {
        self.signature.config_count()
    }

    /// Get input slot metadata and value by index.
    pub fn input(&self, index: usize) -> Option<(&SlotDef, &Value)> {
        let def = self.signature.input(index)?;
        let value = self
            .incoming_input_values
            .get(index)
            .and_then(|v| v.as_ref())
            .or_else(|| self.record.input_values.get(index))?;
        Some((def, value))
    }

    /// Get output slot metadata and value by index.
    pub fn output(&self, index: usize) -> Option<(&SlotDef, &Value)> {
        let def = self.signature.output(index)?;
        let value = self.output_values.get(index)?;
        Some((def, value))
    }

    /// Get config slot metadata and value by index.
    pub fn config(&self, index: usize) -> Option<(&SlotDef, &Value)> {
        let def = self.signature.config(index)?;
        let value = self.record.config_values.get(index)?;
        Some((def, value))
    }

    /// Iterate over all inputs with their metadata and values.
    pub fn inputs(&self) -> impl Iterator<Item = (&SlotDef, &Value)> {
        (0..self.input_count()).filter_map(|i| self.input(i))
    }

    /// Iterate over all outputs with their metadata and values.
    pub fn outputs(&self) -> impl Iterator<Item = (&SlotDef, &Value)> {
        (0..self.output_count()).filter_map(|i| self.output(i))
    }

    /// Iterate over all configs with their metadata and values.
    pub fn configs(&self) -> impl Iterator<Item = (&SlotDef, &Value)> {
        (0..self.config_count()).filter_map(|i| self.config(i))
    }

    /// Check if any config slot is marked to show on node body.
    pub fn has_body_config(&self) -> bool {
        self.configs().any(|(def, _)| def.on_node_body())
    }

    /// Downcast the operation to a concrete type.
    /// Returns None if the operation is not of type T.
    pub fn operation<T: std::any::Any + 'static>(&self) -> Option<&T> {
        let op: &dyn std::any::Any = self.operation.as_ref();
        op.downcast_ref::<T>()
    }
}

// Lifecycle
impl Node {
    pub fn setup(&mut self, ctx: &mut ExecutionContext) -> Result<(), crate::error::Error> {
        self.operation.setup(ctx, &mut self.signature);

        self.signature.validate_unique_names()?;

        self.record.input_values = self
            .signature
            .inputs
            .iter()
            .map(|s| s.default_value())
            .collect();

        self.record.config_values = self
            .signature
            .config
            .iter()
            .map(|s| s.default_value())
            .collect();

        self.output_values = self
            .signature
            .outputs
            .iter()
            .map(|s| s.default_value())
            .collect();

        self.incoming_input_values = vec![None; self.input_count()];

        Ok(())
    }

    /// Directly edit a stored constant value on this node
    /// This edits records, if you have a connection into this node
    /// that superscedes changes here
    pub fn edit_input<F, T>(&mut self, idx: usize, f: F) -> Result<T, Error>
    where
        F: FnOnce(&SlotDef, ValueMut) -> T,
    {
        let slot = self
            .record
            .input_values
            .get_mut(idx)
            .ok_or(Error::NoPort(idx))?;

        let checkpoint = slot.checkpoint();
        let slot_mut = slot.as_mut();
        let slot_def = self.signature.input(idx).ok_or(Error::NoPort(idx))?;

        let t = f(slot_def, slot_mut);

        if self.record.input_values[idx].changed_since(&checkpoint) {
            self.mark_dirty();
        }

        Ok(t)
    }

    /// Directly edit a stored output value on this node
    /// only used on input system nodes
    pub(crate) fn edit_output<F, T>(&mut self, idx: usize, f: F) -> Result<T, Error>
    where
        F: FnOnce(&SlotDef, ValueMut) -> T,
    {
        let slot = self.output_values.get_mut(idx).ok_or(Error::NoPort(idx))?;

        let checkpoint = slot.checkpoint();
        let slot_mut = slot.as_mut();
        let slot_def = self.signature.output(idx).ok_or(Error::NoPort(idx))?;

        let t = f(slot_def, slot_mut);

        if self.output_values[idx].changed_since(&checkpoint) {
            self.mark_dirty();
        }

        Ok(t)
    }

    pub(crate) fn edit_config<F, T>(&mut self, idx: usize, f: F) -> Result<T, Error>
    where
        F: FnOnce(&SlotDef, ValueMut) -> T,
    {
        let slot = self
            .record
            .config_values
            .get_mut(idx)
            .ok_or(Error::NoPort(idx))?;
        let checkpoint = slot.checkpoint();
        let slot_mut = slot.as_mut();
        let slot_def = self.signature.config(idx).ok_or(Error::NoPort(idx))?;

        let t = f(slot_def, slot_mut);

        if self.record.config_values[idx].changed_since(&checkpoint) {
            self.mark_dirty();
        }

        Ok(t)
    }

    pub(crate) fn configure(&mut self, ctx: &ExecutionContext) -> crate::error::Result<()> {
        let config: Config = self
            .record
            .config_values
            .iter()
            .map(Value::as_ref)
            .collect();

        self.operation.configure(ctx, config, &mut self.signature)?;

        self.signature.validate_unique_names()?;

        self.output_values = self
            .signature
            .outputs
            .iter()
            .map(|s| s.default_value())
            .collect();

        Ok(())
    }

    pub fn teardown(&mut self, ctx: &mut ExecutionContext) {
        self.operation.teardown(ctx);
    }
}

// Execution
impl Node {
    /// Push an incoming value from an upstream node into this node's input slot.
    pub(crate) fn push_incoming(&mut self, slot: usize, value: Value) {
        if let Some(incoming) = self.incoming_input_values.get_mut(slot) {
            *incoming = Some(value);
        }
    }

    /// Clear an incoming value (when edge is disconnected).
    pub(crate) fn clear_incoming(&mut self, slot: usize) {
        if let Some(incoming) = self.incoming_input_values.get_mut(slot) {
            *incoming = None;
        }
    }

    /// Snapshot output values for diffing after reconfigure.
    pub(crate) fn snapshot_outputs(&self) -> Vec<Value> {
        self.output_values.clone()
    }

    /// Mutable access to output values for texture allocation.
    pub(crate) fn output_values_mut(&mut self) -> &mut Vec<Value> {
        &mut self.output_values
    }

    /// Execute this node's operation.
    /// Builds inputs from incoming values (or falls back to record values),
    /// then calls the operation's execute method.
    pub fn execute(&mut self, ctx: &mut ExecutionContext) -> crate::error::Result<()> {
        // select the default from the record inputs if the incoming edges do not exist.
        let inputs: Inputs = self
            .incoming_input_values
            .iter()
            .zip(self.record.input_values.iter())
            .map(|(incoming, record)| incoming.as_ref().unwrap_or(record).as_ref())
            .collect();

        let outputs: Outputs = self.output_values.iter_mut().map(Value::as_mut).collect();

        self.operation.execute(ctx, inputs, outputs)?;

        // Clear dirty flag after successful execution
        self.clear_dirty();

        Ok(())
    }

    pub(crate) fn on_edge_connected(
        &mut self,
        slot: usize,
        ty: crate::ValueType,
    ) -> crate::error::Result<()> {
        self.operation
            .on_edge_connected(slot, ty, &mut self.signature)
    }

    pub(crate) fn on_edge_disconnected(
        &mut self,
        slot: usize,
        ty: crate::ValueType,
    ) -> crate::error::Result<()> {
        self.operation
            .on_edge_disconnected(slot, ty, &mut self.signature)
    }
}
