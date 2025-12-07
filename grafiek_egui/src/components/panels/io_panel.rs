use egui::{RichText, ScrollArea};
use grafiek_engine::Engine;

pub fn show_io_panel(
    ctx: &egui::Context,
    engine: &mut Engine,
    visible: &mut bool,
    top_panel_height: f32,
) {
    if !*visible {
        return;
    }

    let panel_frame = egui::Frame::default()
        .fill(ctx.style().visuals.window_fill.linear_multiply(0.5))
        .stroke(ctx.style().visuals.window_stroke)
        .inner_margin(8.0)
        .outer_margin(0.0);

    egui::Area::new(egui::Id::new("io_panel"))
        .fixed_pos(egui::pos2(0.0, top_panel_height))
        .show(ctx, |ui| {
            let screen_height = ui.ctx().viewport_rect().height() - top_panel_height;
            panel_frame.show(ui, |ui| {
                ui.set_width(250.0);
                ui.set_min_height(screen_height.max(0.));

                ScrollArea::vertical().show(ui, |ui| {
                    ui.vertical_centered_justified(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("I/O Panel").size(15.0));
                            if ui.button("<<  [Esc]").clicked() {
                                *visible = false;
                            }
                        });

                        ui.separator();
                    });

                    ui.heading("Inputs");
                    ui.separator();

                    let input_nodes: Vec<_> = engine
                        .inputs()
                        .map(|idx| {
                            let node = engine.get_node(idx).unwrap();
                            let label = node.label().to_string();
                            (idx, label)
                        })
                        .collect();

                    for (input_idx, label) in input_nodes {
                        ui.horizontal(|ui| {
                            ui.label(&label);
                            let _ = engine.edit_graph_input(input_idx, |slot_def, value| {
                                crate::components::value::value_editor(ui, slot_def, value);
                            });
                        });
                    }

                    ui.add_space(20.0);
                    ui.heading("Outputs");
                    ui.separator();

                    for output_idx in engine.outputs() {
                        if let Some(node) = engine.get_node(output_idx) {
                            ui.horizontal(|ui| {
                                ui.label(node.label());
                                if let Some(value) = node.input_value(0) {
                                    ui.label(format!("{}", value));
                                }
                            });
                        }
                    }
                });
            });
        });
}
