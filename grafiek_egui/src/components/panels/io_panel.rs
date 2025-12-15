use std::sync::Arc;

use egui::{Frame, RichText, ScrollArea, panel::Side};
use grafiek_engine::{Engine, ExtendedMetadata, TextureMeta, Value, ValueType};

use crate::components::value::image_preview::{self, TextureCache};

pub fn show_io_panel(
    ctx: &egui::Context,
    engine: &mut Engine,
    texture_cache: &mut TextureCache,
    render_state: &Arc<eframe::egui_wgpu::RenderState>,
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

                        ui.label(&label);

                        if let Some(slot) = texture_slot {
                            if let Some((_, Value::Texture(handle))) = node.output(slot) {
                                image_preview::show_texture_preview(
                                    ui,
                                    engine,
                                    texture_cache,
                                    render_state,
                                    handle,
                                );
                            }

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

                        ui.add_space(8.0);
                    }

                    ui.add_space(20.0);
                    ui.heading("Outputs");
                    ui.separator();

                    for output_idx in engine.outputs() {
                        let Some(node) = engine.get_node(output_idx) else {
                            continue;
                        };

                        ui.label(node.label());

                        match node.input(0) {
                            Some((_, Value::Texture(handle))) => {
                                image_preview::show_texture_preview(
                                    ui,
                                    engine,
                                    texture_cache,
                                    render_state,
                                    handle,
                                );
                            }
                            Some((_, value)) => {
                                ui.label(format!("{}", value));
                            }
                            None => {}
                        }

                        ui.add_space(8.0);
                    }
                });
            });
        });
}

pub fn show_io_panel_next(
    ctx: &egui::Context,
    engine: &mut Engine,
    texture_cache: &mut TextureCache,
    render_state: &Arc<eframe::egui_wgpu::RenderState>,
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

    egui::SidePanel::new(Side::Left, "io_panel_two")
        .frame(Frame::NONE)
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

                        ui.label(&label);

                        if let Some(slot) = texture_slot {
                            if let Some((_, Value::Texture(handle))) = node.output(slot) {
                                image_preview::show_texture_preview(
                                    ui,
                                    engine,
                                    texture_cache,
                                    render_state,
                                    handle,
                                );
                            }

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

                        ui.add_space(8.0);
                    }

                    ui.add_space(20.0);
                    ui.heading("Outputs");
                    ui.separator();

                    for output_idx in engine.outputs() {
                        let Some(node) = engine.get_node(output_idx) else {
                            continue;
                        };

                        ui.label(node.label());

                        match node.input(0) {
                            Some((_, Value::Texture(handle))) => {
                                image_preview::show_texture_preview(
                                    ui,
                                    engine,
                                    texture_cache,
                                    render_state,
                                    handle,
                                );
                            }
                            Some((_, value)) => {
                                ui.label(format!("{}", value));
                            }
                            None => {}
                        }

                        ui.add_space(8.0);
                    }
                });
            });
        });
}
