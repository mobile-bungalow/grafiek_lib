use serde::{Deserialize, Serialize};

use super::slot::{SlotBuilder, SlotDef, TypedSlotMut};
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

    pub fn add_input<T: AsValueType>(
        &mut self,
        name: impl Into<std::borrow::Cow<'static, str>>,
    ) -> SlotBuilder<'_, T> {
        SlotBuilder::new(&mut self.inputs, name)
    }

    pub fn add_output<T: AsValueType>(
        &mut self,
        name: impl Into<std::borrow::Cow<'static, str>>,
    ) -> SlotBuilder<'_, T> {
        SlotBuilder::new(&mut self.outputs, name)
    }

    pub fn add_config<T: AsValueType>(
        &mut self,
        name: impl Into<std::borrow::Cow<'static, str>>,
    ) -> SlotBuilder<'_, T> {
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

    pub fn output(&self, index: usize) -> Option<&SlotDef> {
        self.outputs.get(index)
    }

    pub fn config(&self, index: usize) -> Option<&SlotDef> {
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

    pub fn clear_inputs(&mut self) {
        self.inputs.clear();
    }

    pub fn clear_outputs(&mut self) {
        self.outputs.clear();
    }

    pub fn clear_config(&mut self) {
        self.config.clear();
    }

    pub fn clear(&mut self) {
        self.inputs.clear();
        self.outputs.clear();
        self.config.clear();
    }

    pub fn input_by_name<T: AsValueType>(&mut self, name: &str) -> Option<TypedSlotMut<'_, T>> {
        let slot = self.inputs.iter_mut().find(|s| s.name() == name)?;
        if slot.value_type() != T::value_type() {
            return None;
        }
        Some(TypedSlotMut::new(slot))
    }

    pub fn output_by_name<T: AsValueType>(&mut self, name: &str) -> Option<TypedSlotMut<'_, T>> {
        let slot = self.outputs.iter_mut().find(|s| s.name() == name)?;
        if slot.value_type() != T::value_type() {
            return None;
        }
        Some(TypedSlotMut::new(slot))
    }

    pub fn config_by_name<T: AsValueType>(&mut self, name: &str) -> Option<TypedSlotMut<'_, T>> {
        let slot = self.config.iter_mut().find(|s| s.name() == name)?;
        if slot.value_type() != T::value_type() {
            return None;
        }
        Some(TypedSlotMut::new(slot))
    }

    pub(crate) fn validate_unique_names(&self) -> Result<(), crate::error::Error> {
        fn find_duplicate(slots: &[SlotDef]) -> Option<&str> {
            for (i, slot) in slots.iter().enumerate() {
                if slots[..i].iter().any(|s| s.name() == slot.name()) {
                    return Some(slot.name());
                }
            }
            None
        }

        if let Some(name) = find_duplicate(&self.inputs) {
            return Err(crate::error::Error::DuplicateSlotName(
                name.into(),
                "inputs".into(),
            ));
        }

        if let Some(name) = find_duplicate(&self.outputs) {
            return Err(crate::error::Error::DuplicateSlotName(
                name.into(),
                "outputs".into(),
            ));
        }

        if let Some(name) = find_duplicate(&self.config) {
            return Err(crate::error::Error::DuplicateSlotName(
                name.into(),
                "config".into(),
            ));
        }

        Ok(())
    }
}
