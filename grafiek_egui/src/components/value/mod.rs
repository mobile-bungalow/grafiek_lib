mod code_editor;
pub mod image_preview;

use egui::{Color32, Id, Response, Ui};
use grafiek_engine::{ExtendedMetadata, SlotDef, ValueMut, ValueType};

use crate::components::snarl::{PinInfo, PinShape};
use crate::consts::pins;

/// Get the appropriate pin shape for a value type
pub fn pin_shape_for_type(value_type: ValueType) -> PinShape {
    match value_type {
        ValueType::Texture | ValueType::Buffer => PinShape::Diamond,
        ValueType::Any => PinShape::RoundedSquare,
        _ => PinShape::Circle,
    }
}

/// Get the appropriate pin color for a value type
pub fn pin_color_for_type(value_type: ValueType) -> Color32 {
    match value_type {
        ValueType::I32 => pins::I32,
        ValueType::F32 => pins::F32,
        ValueType::Bool => pins::BOOL,
        ValueType::Texture => pins::TEXTURE,
        ValueType::Buffer => pins::BUFFER,
        ValueType::String => pins::STRING,
        ValueType::Any => pins::ANY,
    }
}

/// Value editor that also updates pin shape based on type
pub fn value_editor_with_pin(
    ui: &mut Ui,
    slot: &SlotDef,
    value: ValueMut,
    pin: &mut PinInfo,
) -> Response {
    // Set pin shape based on value type
    *pin = pin
        .clone()
        .with_shape(pin_shape_for_type(slot.value_type()));
    value_editor(ui, slot, value)
}

pub fn value_editor(ui: &mut Ui, slot: &SlotDef, value: ValueMut) -> Response {
    // Create a stable ID for this slot
    let slot_id = Id::new(("value_editor", slot.name()));

    match (value, slot.extended()) {
        (ValueMut::F32(val), ExtendedMetadata::FloatRange(range)) => ui.add(
            egui::DragValue::new(val)
                .range(range.min..=range.max)
                .speed(range.step),
        ),
        (ValueMut::F32(val), _) => ui.add(egui::DragValue::new(val).speed(0.1)),

        (ValueMut::I32(val), ExtendedMetadata::IntEnum(int_enum)) => {
            enum_selector(ui, val, &int_enum.options)
        }
        (ValueMut::I32(val), ExtendedMetadata::IntRange(range)) => ui.add(
            egui::DragValue::new(val)
                .range(range.min..=range.max)
                .speed(range.step),
        ),
        (ValueMut::I32(val), _) => ui.add(egui::DragValue::new(val)),

        (ValueMut::Texture(_), _) => ui.label(""),

        (ValueMut::Buffer(_), _) => ui.label(""),

        (ValueMut::String(val), ExtendedMetadata::String(meta)) => {
            code_editor::code_editor_field(ui, slot_id, val, &meta.kind)
        }
        (ValueMut::String(val), _) => {
            code_editor::code_editor_field(ui, slot_id, val, &grafiek_engine::StringKind::Plain)
        }

        (ValueMut::Bool(val), _) => ui.checkbox(val, ""),

        (ValueMut::Null(_), _) => ui.label("null"),
    }
}

fn enum_selector(ui: &mut Ui, value: &mut i32, options: &[(String, i32)]) -> Response {
    let current = *value;
    let selected_idx = options.iter().position(|(_, v)| *v == current).unwrap_or(0);

    let selected_label = options
        .get(selected_idx)
        .map(|(label, _)| label.as_str())
        .unwrap_or("");

    let response = egui::ComboBox::from_id_salt(ui.next_auto_id())
        .selected_text(selected_label)
        .show_ui(ui, |ui| {
            for (label, val) in options {
                if ui.selectable_label(*value == *val, label).clicked() {
                    *value = *val;
                }
            }
        });

    response.response
}
