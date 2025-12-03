use std::borrow::Cow;

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

pub trait MetadataFor<T> {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloatRange {
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub default: f32,
}

impl Default for FloatRange {
    fn default() -> Self {
        Self {
            min: f32::MIN,
            max: f32::MAX,
            step: 1.0,
            default: 0.0,
        }
    }
}

impl MetadataFor<f32> for FloatRange {}

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

impl MetadataFor<f32> for Angle {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntRange {
    pub min: i32,
    pub max: i32,
    pub step: i32,
    pub default: i32,
}

impl Default for IntRange {
    fn default() -> Self {
        Self {
            min: i32::MIN,
            max: i32::MAX,
            step: 1,
            default: 0,
        }
    }
}

impl MetadataFor<i32> for IntRange {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntEnum {
    pub options: Vec<(String, i32)>,
    pub default: i32,
}

impl MetadataFor<i32> for IntEnum {}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Boolean {
    pub default: bool,
}

impl MetadataFor<i32> for Boolean {}
impl<T> MetadataFor<T> for Vec<u8> {}

#[derive(Debug, Clone, From, Serialize, Deserialize)]
pub enum ExtendedMetadata {
    None,
    #[from]
    FloatRange(FloatRange),
    #[from]
    Angle(Angle),
    #[from]
    IntRange(IntRange),
    #[from]
    IntEnum(IntEnum),
    #[from]
    Boolean(Boolean),
    #[from]
    Custom(Vec<u8>),
}

impl Default for ExtendedMetadata {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotDef {
    pub value_type: ValueType,
    pub name: Cow<'static, str>,
    #[serde(default)]
    pub extended: ExtendedMetadata,
    #[serde(default)]
    pub common: CommonMetadata,
}

impl Default for SlotDef {
    fn default() -> Self {
        Self {
            value_type: ValueType::Any,
            name: Cow::Borrowed(""),
            extended: ExtendedMetadata::None,
            common: CommonMetadata::default(),
        }
    }
}

impl SlotDef {
    pub const fn new(value_type: ValueType, name: &'static str) -> Self {
        Self {
            value_type,
            name: Cow::Borrowed(name),
            extended: ExtendedMetadata::None,
            common: CommonMetadata {
                tooltip: String::new(),
                interactive: false,
                enabled: false,
                visible: false,
            },
        }
    }

    pub fn with_metadata(
        value_type: ValueType,
        name: &'static str,
        extended: ExtendedMetadata,
    ) -> Self {
        Self {
            value_type,
            name: Cow::Borrowed(name),
            extended,
            common: CommonMetadata::default(),
        }
    }

    pub fn set_visible(&mut self, visible: bool) -> &mut Self {
        self.common.visible = visible;
        self
    }

    pub fn set_enabled(&mut self, enabled: bool) -> &mut Self {
        self.common.enabled = enabled;
        self
    }

    pub fn set_tooltip(&mut self, tooltip: impl Into<String>) -> &mut Self {
        self.common.tooltip = tooltip.into();
        self
    }

    pub fn set_label(&mut self, label: impl Into<Cow<'static, str>>) -> &mut Self {
        self.name = label.into();
        self
    }

    pub fn set_interactive(&mut self, interactive: bool) -> &mut Self {
        self.common.interactive = interactive;
        self
    }

    pub fn set_extended(&mut self, meta: impl Into<ExtendedMetadata>) -> &mut Self {
        self.extended = meta.into();
        self
    }
}

pub struct SlotBuilder<'a, T> {
    registry: &'a mut Vec<SlotDef>,
    name: Cow<'static, str>,
    extended: ExtendedMetadata,
    common: CommonMetadata,
    _marker: std::marker::PhantomData<T>,
}

impl<'a, T: crate::AsValueType> SlotBuilder<'a, T> {
    pub fn new(registry: &'a mut Vec<SlotDef>, name: &'static str) -> Self {
        Self {
            registry,
            name: Cow::Borrowed(name),
            extended: ExtendedMetadata::None,
            common: CommonMetadata::default(),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn meta<M: MetadataFor<T> + Into<ExtendedMetadata>>(mut self, metadata: M) -> Self {
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
        self.registry.push(SlotDef {
            value_type: T::value_type(),
            name: self.name,
            extended: self.extended,
            common: self.common,
        });
    }
}
