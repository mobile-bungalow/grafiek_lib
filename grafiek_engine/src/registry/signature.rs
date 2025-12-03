use serde::{Deserialize, Serialize};

use super::slot::{SlotBuilder, SlotDef};
use crate::AsValueType;
use crate::traits::{ConfigSchema, InputSchema, OutputSchema};

/// Collects slot definitions for an operation's inputs, outputs, and config.
/// Slots are registered via push-based builders during [Operation::setup].
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

    pub fn push_input_raw(&mut self, def: SlotDef) {
        self.inputs.push(def);
    }

    pub fn push_config_raw(&mut self, def: SlotDef) {
        self.config.push(def);
    }

    pub fn push_output_raw(&mut self, def: SlotDef) {
        self.outputs.push(def);
    }

    pub fn add_input<T: AsValueType>(&mut self, name: &'static str) -> SlotBuilder<'_, T> {
        SlotBuilder::new(&mut self.inputs, name)
    }

    pub fn add_output<T: AsValueType>(&mut self, name: &'static str) -> SlotBuilder<'_, T> {
        SlotBuilder::new(&mut self.outputs, name)
    }

    pub fn add_config<T: AsValueType>(&mut self, name: &'static str) -> SlotBuilder<'_, T> {
        SlotBuilder::new(&mut self.config, name)
    }

    pub fn register_inputs<S: InputSchema>(&mut self) {
        S::register(self);
    }

    pub fn register_outputs<S: OutputSchema>(&mut self) {
        S::register(self);
    }

    pub fn register_config<S: ConfigSchema>(&mut self) {
        S::register(self);
    }

    pub fn input(&self, index: usize) -> Option<&SlotDef> {
        self.inputs.get(index)
    }

    pub fn input_mut(&mut self, index: usize) -> Option<&mut SlotDef> {
        self.inputs.get_mut(index)
    }

    pub fn output(&self, index: usize) -> Option<&SlotDef> {
        self.outputs.get(index)
    }

    pub fn output_mut(&mut self, index: usize) -> Option<&mut SlotDef> {
        self.outputs.get_mut(index)
    }

    pub fn config(&self, index: usize) -> Option<&SlotDef> {
        self.config.get(index)
    }

    pub fn config_mut(&mut self, index: usize) -> Option<&mut SlotDef> {
        self.config.get_mut(index)
    }

    pub fn input_count(&self) -> usize {
        self.inputs.len()
    }

    pub fn output_count(&self) -> usize {
        self.outputs.len()
    }

    pub fn config_count(&self) -> usize {
        self.config.len()
    }

    pub fn clear(&mut self) {
        self.inputs.clear();
        self.outputs.clear();
        self.config.clear();
    }
}
