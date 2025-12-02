use serde::{Deserialize, Serialize};

use super::slot::{InputSlotBuilder, InputSlotDef, OutputSlotBuilder, OutputSlotDef};
use crate::AsValueType;
use crate::traits::{ConfigSchema, InputSchema, OutputSchema};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SignatureRegistery {
    pub inputs: Vec<InputSlotDef>,
    pub outputs: Vec<OutputSlotDef>,
    pub config: Vec<InputSlotDef>,
}

impl SignatureRegistery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_input<T: AsValueType>(
        &mut self,
        name: impl Into<String>,
    ) -> InputSlotBuilder<'_, T> {
        InputSlotBuilder::new(&mut self.inputs, name)
    }

    pub fn add_output<T: AsValueType>(
        &mut self,
        name: impl Into<String>,
    ) -> OutputSlotBuilder<'_, T> {
        OutputSlotBuilder::new(&mut self.outputs, name)
    }

    pub fn add_config<T: AsValueType>(
        &mut self,
        name: impl Into<String>,
    ) -> InputSlotBuilder<'_, T> {
        InputSlotBuilder::new(&mut self.config, name)
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

    pub fn input(&self, index: usize) -> Option<&InputSlotDef> {
        self.inputs.get(index)
    }

    pub fn output(&self, index: usize) -> Option<&OutputSlotDef> {
        self.outputs.get(index)
    }

    pub fn config(&self, index: usize) -> Option<&InputSlotDef> {
        self.config.get(index)
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
