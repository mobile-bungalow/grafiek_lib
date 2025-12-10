use std::borrow::Cow;
use std::marker::PhantomData;

use derive_more::From;
use serde::{Deserialize, Serialize};

use crate::{AsValueType, TextureHandle, ValueType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonMetadata {
    /// Descriptive helpful piece of text shown on hover.
    pub tooltip: Option<String>,
    /// True if it would be fine to update this value every frame,
    /// false if you just want to update it on commit
    pub interactive: bool,
    /// True if this should not be editable from the UI
    pub enabled: bool,
    /// True if this piece of state should be hidden
    pub visible: bool,
    /// A ui hint indicating this should be shown in some kind of inspector
    /// or info panel as opposed to on the node body itself. This is primarily
    /// meant for config inputs, but if an input does not require a reconfigure
    /// of the node but should not be slottable in the UI, you can set this to true as well.
    pub on_node_body: bool,
}

impl Default for CommonMetadata {
    fn default() -> Self {
        Self {
            tooltip: None,
            interactive: true,
            enabled: true,
            visible: true,
            on_node_body: false,
        }
    }
}

// A marker trait which prevents SlotDefs from
// being defined for a [ValueType] for which the metadata
// does not describe.
pub trait MetadataFor<T> {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloatRange {
    pub min: f32,
    pub max: f32,
    pub step: f32,
}

impl Default for FloatRange {
    fn default() -> Self {
        Self {
            min: f32::MIN,
            max: f32::MAX,
            step: 1.0,
        }
    }
}

impl MetadataFor<f32> for FloatRange {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Angle {
    pub min: f32,
    pub max: f32,
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
}

impl Default for IntRange {
    fn default() -> Self {
        Self {
            min: i32::MIN,
            max: i32::MAX,
            step: 1,
        }
    }
}

impl MetadataFor<i32> for IntRange {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntEnum {
    pub options: Vec<(String, i32)>,
}

impl MetadataFor<i32> for IntEnum {}

impl<T> MetadataFor<T> for Vec<u8> {}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum StringKind {
    #[default]
    Plain,
    Glsl,
    Rune,
    Json,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StringMeta {
    pub kind: StringKind,
    pub multi_line: bool,
}
impl MetadataFor<String> for StringMeta {}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TextureMeta {
    /// Show this image on the node body, or somewhere else in the application.
    pub preview: bool,
    /// Allow a file picker to be used in assigning this data.
    pub allow_file: bool,
}
impl MetadataFor<TextureHandle> for TextureMeta {}

#[derive(Debug, Clone, From, Serialize, Deserialize, Default)]
pub enum ExtendedMetadata {
    #[default]
    None,
    FloatRange(FloatRange),
    Angle(Angle),
    IntRange(IntRange),
    IntEnum(IntEnum),
    Texture(TextureMeta),
    String(StringMeta),
    Custom(Vec<u8>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotDef {
    pub(crate) value_type: ValueType,
    pub(crate) name: Cow<'static, str>,
    #[serde(default)]
    pub(crate) extended: ExtendedMetadata,
    #[serde(default)]
    pub(crate) common: CommonMetadata,
    #[serde(default)]
    pub(crate) default_override: Option<crate::Value>,
}

impl Default for SlotDef {
    fn default() -> Self {
        Self {
            value_type: ValueType::Any,
            name: Cow::Borrowed(""),
            extended: ExtendedMetadata::None,
            common: CommonMetadata::default(),
            default_override: None,
        }
    }
}

impl SlotDef {
    /// Returns the name of this slot.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the value type of this slot.
    pub fn value_type(&self) -> ValueType {
        self.value_type
    }

    /// Returns the extended metadata for this slot.
    pub fn extended(&self) -> &ExtendedMetadata {
        &self.extended
    }

    /// Whether or not to render this element
    pub fn is_visible(&self) -> bool {
        self.common.visible
    }

    /// Returns whether this slot should be shown on the node body.
    pub fn on_node_body(&self) -> bool {
        self.common.on_node_body
    }

    /// Returns the default value for this slot, using the override if set,
    /// otherwise falling back to the type's default.
    pub fn default_value(&self) -> crate::Value {
        self.default_override
            .clone()
            .unwrap_or_else(|| self.value_type.default_value())
    }

    pub fn set_visible(&mut self, visible: bool) -> &mut Self {
        self.common.visible = visible;
        self
    }

    pub fn set_enabled(&mut self, enabled: bool) -> &mut Self {
        self.common.enabled = enabled;
        self
    }

    pub fn set_on_node_body(&mut self, on_node_body: bool) -> &mut Self {
        self.common.on_node_body = on_node_body;
        self
    }

    pub fn set_tooltip(&mut self, tooltip: impl Into<String>) -> &mut Self {
        self.common.tooltip = Some(tooltip.into());
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
    default: Option<T>,
    name: Cow<'static, str>,
    extended: ExtendedMetadata,
    common: CommonMetadata,
    _marker: std::marker::PhantomData<T>,
}

impl<'a> SlotBuilder<'a, TextureHandle> {
    /// Set the dimensions for the texture output.
    pub fn dimensions(mut self, width: u32, height: u32) -> Self {
        let mut tex = self.default.take().unwrap_or_default();
        tex.width = width.max(1);
        tex.height = height.max(1);
        self.default = Some(tex);
        self
    }
}

impl<'a, T: crate::AsValueType> SlotBuilder<'a, T> {
    pub fn new(registry: &'a mut Vec<SlotDef>, name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            registry,
            default: None,
            name: name.into(),
            extended: T::default_metadata().unwrap_or(ExtendedMetadata::None),
            common: CommonMetadata::default(),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn meta<M: MetadataFor<T> + Into<ExtendedMetadata>>(mut self, metadata: M) -> Self {
        self.extended = metadata.into();
        self
    }

    pub fn default(mut self, val: T) -> Self {
        self.default = Some(val);
        self
    }

    pub fn tooltip(mut self, text: impl Into<String>) -> Self {
        self.common.tooltip = Some(text.into());
        self
    }

    pub fn show_on_node_body(mut self, show_on_body: bool) -> Self {
        self.common.on_node_body = show_on_body;
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

    pub fn build(self)
    where
        T: Into<crate::Value>,
    {
        self.registry.push(SlotDef {
            value_type: T::value_type(),
            name: self.name,
            extended: self.extended,
            common: self.common,
            default_override: self.default.map(Into::into),
        });
    }
}

/// A type-safe mutable reference to a slot definition.
/// Ensures that default values and metadata match the slot's type.
pub struct TypedSlotMut<'a, T> {
    slot: &'a mut SlotDef,
    _marker: PhantomData<T>,
}

impl<'a, T: AsValueType> TypedSlotMut<'a, T> {
    pub(crate) fn new(slot: &'a mut SlotDef) -> Self {
        debug_assert_eq!(slot.value_type, T::value_type());
        Self {
            slot,
            _marker: PhantomData,
        }
    }

    /// Set the default value for this slot.
    pub fn set_default(&mut self, val: T) -> &mut Self
    where
        T: Into<crate::Value>,
    {
        self.slot.default_override = Some(val.into());
        self
    }

    /// Set extended metadata for this slot.
    pub fn set_meta<M: MetadataFor<T> + Into<ExtendedMetadata>>(&mut self, meta: M) -> &mut Self {
        self.slot.extended = meta.into();
        self
    }

    /// Set whether this slot should be shown on the node body.
    pub fn set_on_node_body(&mut self, on_node_body: bool) -> &mut Self {
        self.slot.common.on_node_body = on_node_body;
        self
    }

    /// Set whether this slot is interactive.
    pub fn set_interactive(&mut self, interactive: bool) -> &mut Self {
        self.slot.common.interactive = interactive;
        self
    }

    /// Set the tooltip for this slot.
    pub fn set_tooltip(&mut self, tooltip: impl Into<String>) -> &mut Self {
        self.slot.common.tooltip = Some(tooltip.into());
        self
    }

    /// Set visibility of this slot.
    pub fn set_visible(&mut self, visible: bool) -> &mut Self {
        self.slot.common.visible = visible;
        self
    }
}
