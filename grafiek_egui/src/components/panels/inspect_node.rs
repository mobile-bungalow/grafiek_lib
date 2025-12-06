use egui::{RichText, ScrollArea};
use grafiek_engine::{Engine, NodeIndex};

pub fn show_inspector_panel(
    ctx: &egui::Context,
    engine: &mut Engine,
    inspect_node: &mut Option<NodeIndex>,
) {
    let Some(engine_idx) = *inspect_node else {
        return;
    };

    // Check if this node still exists
    let Some(node) = engine.get_node(engine_idx) else {
        *inspect_node = None;
        return;
    };

    let title = node.label().to_string();
    let op_path = format!(
        "{}/{}",
        node.record().op_path.library,
        node.record().op_path.operator
    );
    let config_count = node.config_count();
    let input_count = node.input_count();
    let output_count = node.output_count();

    let mut open = true;

    egui::Window::new(&title)
        .id(egui::Id::new("inspector_panel"))
        .open(&mut open)
        .default_width(300.0)
        .show(ctx, |ui| {
            // Debug info
            ui.label(RichText::new(&op_path).small().weak());
            ui.label(
                RichText::new(format!("idx: {:?}", engine_idx))
                    .small()
                    .weak(),
            );
            ui.label(
                RichText::new(format!(
                    "inputs: {} | outputs: {} | configs: {}",
                    input_count, output_count, config_count
                ))
                .small()
                .weak(),
            );
            ui.separator();

            ScrollArea::vertical().show(ui, |ui| {
                // Show all configs
                for slot_idx in 0..config_count {
                    let _ = engine.edit_node_config(engine_idx, slot_idx, |slot_def, value| {
                        ui.add_space(4.0);
                        ui.label(RichText::new(slot_def.name.as_ref()).strong());

                        crate::components::value::value_editor(ui, slot_def, value);
                    });
                }
            });
        });

    if !open {
        *inspect_node = None;
    }
}
