use egui::{RichText, ScrollArea};
use grafiek_engine::{Engine, ExtendedMetadata, TextureMeta, ValueType};

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

                    let input_indices: Vec<_> = engine.inputs().collect();
                    for idx in input_indices {
                        let Some(node) = engine.get_node(idx) else {
                            continue;
                        };
                        let label = node.label().to_string();
                        let sig = node.signature();

                        // Find texture output slots with allow_file
                        let texture_slot = (0..sig.output_count()).find(|&i| {
                            sig.output(i).is_some_and(|slot| {
                                matches!(
                                    (slot.value_type(), slot.extended()),
                                    (
                                        ValueType::Texture,
                                        ExtendedMetadata::Texture(TextureMeta {
                                            allow_file: true,
                                            ..
                                        })
                                    )
                                )
                            })
                        });

                        ui.horizontal(|ui| {
                            ui.label(&label);
                            if let Some(slot) = texture_slot {
                                if ui.button("Load Image...").clicked() {
                                    crate::components::image_picker::pick_and_load_image(
                                        engine, idx, slot,
                                    );
                                }
                            } else {
                                let _ = engine.edit_graph_input(idx, |slot_def, value| {
                                    crate::components::value::value_editor(ui, slot_def, value);
                                });
                            }
                        });
                    }

                    ui.add_space(20.0);
                    ui.heading("Outputs");
                    ui.separator();

                    for output_idx in engine.outputs() {
                        if let Some(node) = engine.get_node(output_idx) {
                            ui.horizontal(|ui| {
                                ui.label(node.label());
                                if let Some((_, value)) = node.input(0) {
                                    ui.label(format!("{}", value));
                                }
                            });
                        }
                    }
                });
            });
        });
}
