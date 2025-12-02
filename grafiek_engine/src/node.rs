use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::traits::{OpPath, Operation};
use crate::{ExecutionContext, SignatureRegistery, SlotDef, Value, ValueMut};

/// Engine provided unique ID
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    operation: Box<dyn Operation>,
    dirty: DirtyFlag,
}

/// Result of probing whether a connection is type-compatible.
/// This only validates node-level concerns (slots and types).
/// Graph-level concerns (existing edges) are handled by the Engine.
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
}

impl Node {
    pub fn new(operation: Box<dyn Operation>, id: NodeId) -> Self {
        Self {
            record: NodeRecord::new(id, operation.op_path()),
            signature: SignatureRegistery::default(),
            output_values: vec![],
            operation,
            dirty: DirtyFlag::new(),
        }
    }

    pub fn record(&self) -> &NodeRecord {
        &self.record
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

// Value access
impl Node {
    /// Get read access to an input value
    pub fn input_value(&self, index: usize) -> Option<&Value> {
        self.record.input_values.get(index)
    }

    /// Get mutable access to an input value
    pub fn input_mut(&mut self, index: usize) -> Option<ValueMut<'_>> {
        self.record.input_values.get_mut(index).map(Value::as_mut)
    }

    /// Get mutable access to a config value
    pub fn config_mut(&mut self, index: usize) -> Option<ValueMut<'_>> {
        self.record.config_values.get_mut(index).map(Value::as_mut)
    }

    /// Number of input slots
    pub fn input_count(&self) -> usize {
        self.signature.input_count()
    }

    /// Number of output slots
    pub fn output_count(&self) -> usize {
        self.signature.output_count()
    }
}

// Lifecycle
impl Node {
    pub fn setup(&mut self, ctx: &mut ExecutionContext) {
        self.operation.setup(ctx, &mut self.signature);

        // Populate value vectors with defaults based on signature
        self.record.input_values = self
            .signature
            .inputs
            .iter()
            .map(|s| s.value_type.default_value())
            .collect();
        self.record.config_values = self
            .signature
            .config
            .iter()
            .map(|s| s.value_type.default_value())
            .collect();
        self.output_values = self
            .signature
            .outputs
            .iter()
            .map(|s| s.value_type.default_value())
            .collect();
    }

    /// Directly edit a stored constant value on this node
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

    pub fn edit_config<F, T>(&mut self, idx: usize, f: F) -> Result<T, Error>
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

    pub fn configure(&mut self, ctx: &mut ExecutionContext) -> crate::error::Result<()> {
        self.operation
            .configure(&self.record.config_values, &mut self.signature)
    }

    pub fn teardown(&mut self, ctx: &mut ExecutionContext) {
        self.operation.teardown(ctx);
    }
}
