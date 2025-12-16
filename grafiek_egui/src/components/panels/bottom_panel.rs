use egui::{Context, RichText, ScrollArea, TextEdit};
use grafiek_engine::{Engine, NodeIndex};

const MIN_HEIGHT: f32 = 150.0;
const DEFAULT_HEIGHT: f32 = 200.0;
const COLLAPSED_HEIGHT: f32 = 20.0;
const MAX_HEIGHT: f32 = 500.0;
const COLUMN_MIN_WIDTH: f32 = 200.0;

#[derive(Default)]
pub struct BottomPanel;

impl BottomPanel {
    pub fn show(
        ctx: &Context,
        engine: &mut Engine,
        inspect_node: &mut Option<NodeIndex>,
        collapsed: &mut bool,
    ) {
        if *collapsed {
            egui::TopBottomPanel::bottom("bottom_panel")
                .resizable(false)
                .exact_height(COLLAPSED_HEIGHT)
                .show_separator_line(false)
                .show(ctx, |ui| {
                    ui.take_available_height();
                    ui.horizontal(|ui| {
                        Self::show_collapse_button(ui, collapsed, true);
                    });
                });
        } else {
            egui::TopBottomPanel::bottom("bottom_panel")
                .resizable(true)
                .min_height(MIN_HEIGHT)
                .max_height(MAX_HEIGHT)
                .default_height(DEFAULT_HEIGHT)
                .show_separator_line(true)
                .show(ctx, |ui| {
                    ui.take_available_height();
                    ui.horizontal(|ui| {
                        Self::show_collapse_button(ui, collapsed, false);
                    });
                    ui.allocate_space(egui::vec2(ui.available_width(), 0.0));

                    ui.columns(3, |columns| {
                        Self::show_engine_info(&mut columns[0], engine);
                        Self::show_center_panel(&mut columns[1]);
                        Self::show_inspector(&mut columns[2], engine, inspect_node);
                    });
                });
        }
    }

    fn show_collapse_button(ui: &mut egui::Ui, collapsed: &mut bool, is_collapsed: bool) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let icon = if is_collapsed {
                egui_phosphor::regular::CARET_UP
            } else {
                egui_phosphor::regular::CARET_DOWN
            };
            if ui.button(icon).clicked() {
                *collapsed = !*collapsed;
            }
        });
    }

    fn show_engine_info(ui: &mut egui::Ui, _engine: &Engine) {
        ui.centered_and_justified(|ui| {
            ui.label(RichText::new("Engine").weak());
        });
    }

    fn show_center_panel(ui: &mut egui::Ui) {
        ui.centered_and_justified(|ui| {
            ui.label(RichText::new("Tools").weak());
        });
    }

    fn show_inspector(
        ui: &mut egui::Ui,
        engine: &mut Engine,
        inspect_node: &mut Option<NodeIndex>,
    ) {
        let Some(engine_idx) = *inspect_node else {
            ui.centered_and_justified(|ui| {
                ui.label("Select a node to inspect");
            });
            return;
        };

        let Some(node) = engine.get_node(engine_idx) else {
            *inspect_node = None;
            return;
        };

        let old_label = node.label().to_string();
        let mut label = old_label.clone();
        let op_path = format!("{}/{}", node.op_path().library, node.op_path().operator);

        ui.label(RichText::new("Inspector").strong());
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Name");
            ui.add(TextEdit::singleline(&mut label).desired_width(100.0));
        });
        ui.label(RichText::new(&op_path).weak());

        if old_label != label {
            engine.set_label(engine_idx, &label);
        }

        ui.separator();

        ScrollArea::vertical().show(ui, |ui| {
            let _ = engine.edit_all_node_configs(engine_idx, |slot_def, value| {
                if slot_def.on_node_body() || !slot_def.is_visible() {
                    return;
                }

                ui.horizontal(|ui| {
                    ui.label(RichText::new(slot_def.name()).strong());
                    crate::components::value::value_editor(ui, slot_def, value);
                });
            });
        });
    }
}
