use egui::{RichText, ScrollArea, TextEdit};
use grafiek_engine::{Engine, NodeIndex};

pub fn show_inspector_panel(
    ctx: &egui::Context,
    engine: &mut Engine,
    inspect_node: &mut Option<NodeIndex>,
    top_panel_height: f32,
) {
    let Some(engine_idx) = *inspect_node else {
        return;
    };

    let Some(node) = engine.get_node(engine_idx) else {
        *inspect_node = None;
        return;
    };

    let old_label = node.label().to_string();
    let mut label = old_label.to_string();
    let op_path = format!(
        "{}/{}",
        node.record().op_path.library,
        node.record().op_path.operator
    );
    let config_count = node.config_count();
    let input_count = node.input_count();
    let output_count = node.output_count();

    let panel_width = 380.0;
    let margin = 8.0;
    let mut open = true;

    egui::Window::new("Inspector")
        .id(egui::Id::new("inspector_panel"))
        .open(&mut open)
        .resizable(false)
        .default_width(panel_width)
        .anchor(
            egui::Align2::RIGHT_TOP,
            [-margin, margin + top_panel_height / 2.],
        )
        .show(ctx, |ui| {
            ui.set_min_width(panel_width);

            ui.horizontal(|ui| {
                ui.label(RichText::new("Name").strong());
                ui.add(TextEdit::singleline(&mut label).desired_width(f32::INFINITY));
            });

            if old_label != label {
                engine.set_label(engine_idx, &label);
            }

            ui.add_space(4.0);

            ui.collapsing("Details", |ui| {
                ui.label(RichText::new(&op_path));
                ui.label(RichText::new(format!("idx: {:?}", engine_idx)));
                ui.label(RichText::new(format!(
                    "inputs: {} | outputs: {} | configs: {}",
                    input_count, output_count, config_count
                )));
            });

            let mut first = false;
            ScrollArea::vertical().show(ui, |ui| {
                let _ = engine.edit_all_node_configs(engine_idx, |slot_def, value| {
                    if slot_def.common.on_node_body {
                        return;
                    }

                    if first {
                        ui.separator();
                        first = !first;
                    }

                    ui.add_space(4.0);
                    ui.label(RichText::new(slot_def.name.as_ref()).strong());
                    crate::components::value::value_editor(ui, slot_def, value);
                });
            });
        });

    if !open {
        *inspect_node = None;
    }
}
