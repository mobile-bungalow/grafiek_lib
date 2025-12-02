use derive_more::From;
use serde::{Deserialize, Serialize};

use crate::ValueType;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommonMetadata {
    // Descriptive helpful piece of text shown on hover.
    pub tooltip: String,
    // True if it would be fine to update this value every frame
    pub interactive: bool,
    // True if this should not be editable from the UI
    pub enabled: bool,
    // True if this piece of state should be hidden
    pub visible: bool,
}

pub trait InputMetadataFor<T> {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloatRange {
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub default: f32,
}

impl InputMetadataFor<f32> for FloatRange {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Angle {
    pub min: f32,
    pub max: f32,
    pub default: f32,
    pub unit: AngleUnit,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum AngleUnit {
    #[default]
    Radians,
    Degrees,
}

impl InputMetadataFor<f32> for Angle {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntRange {
    pub min: i32,
    pub max: i32,
    pub step: i32,
    pub default: i32,
}

impl InputMetadataFor<i32> for IntRange {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntEnum {
    pub options: Vec<(String, i32)>,
    pub default: i32,
}

impl InputMetadataFor<i32> for IntEnum {}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Boolean {
    pub default: bool,
}

impl InputMetadataFor<i32> for Boolean {} // bools stored as i32

#[derive(Debug, Clone, From, Serialize, Deserialize)]
pub enum InputExtendedMetadata {
    None,
    FloatRange(FloatRange),
    Angle(Angle),
    IntRange(IntRange),
    IntEnum(IntEnum),
    Boolean(Boolean),
    #[from]
    Custom(Vec<u8>),
}

impl Default for InputExtendedMetadata {
    fn default() -> Self {
        Self::None
    }
}

pub trait OutputMetadataFor<T> {}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NumericOutput {}

impl<T> OutputMetadataFor<T> for Vec<u8> {}
impl<T> InputMetadataFor<T> for Vec<u8> {}

#[derive(Debug, Clone, From, Serialize, Deserialize)]
pub enum OutputExtendedMetadata {
    None,
    #[from]
    Custom(Vec<u8>),
}

impl Default for OutputExtendedMetadata {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotMetadata<E> {
    pub name: String,
    pub extended: E,
    pub common: CommonMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotDef<E> {
    pub value_type: ValueType,
    pub metadata: SlotMetadata<E>,
}

impl<E: Default> SlotDef<E> {
    pub fn new(value_type: ValueType, name: impl Into<String>, extended: E) -> Self {
        Self {
            value_type,
            metadata: SlotMetadata {
                name: name.into(),
                extended,
                common: CommonMetadata::default(),
            },
        }
    }
}

pub type InputSlotDef = SlotDef<InputExtendedMetadata>;
pub type OutputSlotDef = SlotDef<OutputExtendedMetadata>;

pub struct InputSlotBuilder<'a, T> {
    registry: &'a mut Vec<InputSlotDef>,
    name: String,
    extended: InputExtendedMetadata,
    common: CommonMetadata,
    _marker: std::marker::PhantomData<T>,
}

impl<'a, T: crate::AsValueType> InputSlotBuilder<'a, T> {
    pub fn new(registry: &'a mut Vec<InputSlotDef>, name: impl Into<String>) -> Self {
        Self {
            registry,
            name: name.into(),
            extended: InputExtendedMetadata::None,
            common: CommonMetadata::default(),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn meta<M: InputMetadataFor<T> + Into<InputExtendedMetadata>>(
        mut self,
        metadata: M,
    ) -> Self {
        self.extended = metadata.into();
        self
    }

    pub fn tooltip(mut self, text: impl Into<String>) -> Self {
        self.common.tooltip = text.into();
        self
    }

    pub fn visible(mut self, is_visible: bool) -> Self {
        self.common.visible = is_visible;
        self
    }

    pub fn interactive(mut self, is_interactive: bool) -> Self {
        self.common.interactive = is_interactive;
        self
    }

    pub fn build(self) {
        self.registry.push(InputSlotDef {
            value_type: T::value_type(),
            metadata: SlotMetadata {
                name: self.name,
                extended: self.extended,
                common: self.common,
            },
        });
    }
}

pub struct OutputSlotBuilder<'a, T> {
    registry: &'a mut Vec<OutputSlotDef>,
    name: String,
    extended: OutputExtendedMetadata,
    common: CommonMetadata,
    _marker: std::marker::PhantomData<T>,
}

impl<'a, T: crate::AsValueType> OutputSlotBuilder<'a, T> {
    pub fn new(registry: &'a mut Vec<OutputSlotDef>, name: impl Into<String>) -> Self {
        Self {
            registry,
            name: name.into(),
            extended: OutputExtendedMetadata::None,
            common: CommonMetadata::default(),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn meta<M: OutputMetadataFor<T> + Into<OutputExtendedMetadata>>(
        mut self,
        metadata: M,
    ) -> Self {
        self.extended = metadata.into();
        self
    }

    pub fn tooltip(mut self, text: impl Into<String>) -> Self {
        self.common.tooltip = text.into();
        self
    }

    pub fn build(self) {
        self.registry.push(OutputSlotDef {
            value_type: T::value_type(),
            metadata: SlotMetadata {
                name: self.name,
                extended: self.extended,
                common: self.common,
            },
        });
    }
}
