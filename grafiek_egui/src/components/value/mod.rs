use egui::{Response, Ui};
use grafiek_engine::{ExtendedMetadata, SlotDef, ValueMut};

/// Display a widget for editing a Value based on its type and metadata
pub fn value_editor(ui: &mut Ui, slot: &SlotDef, value: ValueMut) -> Response {
    match (value, &slot.extended) {
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
        (ValueMut::I32(val), ExtendedMetadata::Boolean(_)) => {
            let mut checked = *val != 0;
            let response = ui.checkbox(&mut checked, "");
            *val = if checked { 1 } else { 0 };
            response
        }
        (ValueMut::I32(val), _) => ui.add(egui::DragValue::new(val)),

        (ValueMut::Texture(_), _) => ui.label("Texture"),

        (ValueMut::Buffer(_), _) => todo!("Buffer!"),

        (ValueMut::String(val), _) => ui.label(val.as_str()), //string_editor(ui, val),

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
