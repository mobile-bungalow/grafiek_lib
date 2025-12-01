use std::cell::Cell;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::traits::{OpPath, Operation};
use crate::{SignatureRegistery, SlotMetadata, Value, ValueMut, ValueType};

/// Engine provided unique ID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeId(pub u64);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeRecord {
    pub from_id: NodeId,
    pub from_port: usize,
    pub to_id: NodeId,
    pub to_port: usize,
}

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
    /// Hashmap of user provided key value pairs
    /// TODO: when we know what serialization format we are using use a value here.
    pub userdata: HashMap<String, ()>,
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
            userdata: HashMap::new(),
        }
    }
}

/// Runtime node state including operation instance
pub struct Node {
    record: NodeRecord,
    signature: SignatureRegistery,
    output_values: Vec<Value>,
    operation: Box<dyn Operation>,
    dirty: Cell<bool>,
}

impl Node {
    pub fn new(operation: Box<dyn Operation>, id: NodeId) -> Self {
        Self {
            record: NodeRecord::new(id, operation.op_path()),
            signature: SignatureRegistery::default(),
            output_values: vec![],
            operation,
            dirty: false.into(),
        }
    }
    pub fn record(&self) -> &NodeRecord {
        &self.record
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty.get()
    }

    pub fn clear_dirty(&self) {
        self.dirty.set(false);
    }

    pub fn config_slot_count(&self) -> usize {
        self.record.config_values.len()
    }

    pub fn input_slot_count(&self) -> usize {
        self.record.input_values.len()
    }

    pub fn output_slot_count(&self) -> usize {
        self.output_values.len()
    }
}

impl Node {
    /// Get mutable access to an input value
    pub fn input_mut(&mut self, index: usize) -> Option<ValueMut<'_>> {
        self.dirty.set(true);
        self.record.input_values.get_mut(index).map(Value::as_mut)
    }

    /// Get mutable access to a config value
    pub fn config_mut(&mut self, index: usize) -> Option<ValueMut<'_>> {
        self.dirty.set(true);
        self.record.config_values.get_mut(index).map(Value::as_mut)
    }
}
