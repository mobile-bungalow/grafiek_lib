use serde::{Deserialize, Serialize};

use super::{SlotDef, SlotMetadata};
use crate::ValueType;

/// Serializable signature definition for a node.
/// Defines the inputs, outputs, and configuration slots.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SignatureRegistery {
    pub inputs: Vec<SlotDef>,
    pub outputs: Vec<SlotDef>,
    pub config: Vec<SlotDef>,
}

impl SignatureRegistery {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an input slot
    pub fn add_input(&mut self, value_type: ValueType, metadata: SlotMetadata) {
        self.inputs.push(SlotDef::new(value_type, metadata));
    }

    /// Add an output slot
    pub fn add_output(&mut self, value_type: ValueType, metadata: SlotMetadata) {
        self.outputs.push(SlotDef::new(value_type, metadata));
    }

    /// Add a config slot
    pub fn add_config(&mut self, value_type: ValueType, metadata: SlotMetadata) {
        self.config.push(SlotDef::new(value_type, metadata));
    }

    /// Get input slot definition by index
    pub fn input(&self, index: usize) -> Option<&SlotDef> {
        self.inputs.get(index)
    }

    /// Get output slot definition by index
    pub fn output(&self, index: usize) -> Option<&SlotDef> {
        self.outputs.get(index)
    }

    /// Get config slot definition by index
    pub fn config(&self, index: usize) -> Option<&SlotDef> {
        self.config.get(index)
    }

    /// Number of input slots
    pub fn input_count(&self) -> usize {
        self.inputs.len()
    }

    /// Number of output slots
    pub fn output_count(&self) -> usize {
        self.outputs.len()
    }

    /// Number of config slots
    pub fn config_count(&self) -> usize {
        self.config.len()
    }

    /// Clear all slots (used when reconfiguring)
    pub fn clear(&mut self) {
        self.inputs.clear();
        self.outputs.clear();
        self.config.clear();
    }
}
