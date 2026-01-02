use egui::{Context, RichText, ScrollArea, TextEdit};
use grafiek_engine::{Engine, NodeIndex};

use crate::components::engine_ext::EngineExt;

const MIN_HEIGHT: f32 = 150.0;
const DEFAULT_HEIGHT: f32 = 200.0;
const COLLAPSED_HEIGHT: f32 = 20.0;
const MAX_HEIGHT: f32 = 500.0;

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
            return;
        }

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

                let has_scripts = inspect_node
                    .map(|idx| engine.has_script(idx))
                    .unwrap_or(false);

                let available_width = ui.available_width();
                let available_height = ui.available_height();

                ui.horizontal(|ui| {
                    if has_scripts {
                        let script_width = available_width * 0.6;
                        let inspector_width = available_width * 0.4;

                        ui.allocate_ui(egui::vec2(script_width, available_height), |ui| {
                            Self::show_scripts(ui, engine, inspect_node);
                        });
                        ui.separator();
                        ui.allocate_ui(egui::vec2(inspector_width, available_height), |ui| {
                            Self::show_inspector(ui, engine, inspect_node);
                        });
                    } else {
                        ui.allocate_ui(egui::vec2(available_width, available_height), |ui| {
                            Self::show_inspector(ui, engine, inspect_node);
                        });
                    }
                });
            });
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

    fn show_scripts(ui: &mut egui::Ui, engine: &mut Engine, inspect_node: &mut Option<NodeIndex>) {
        use grafiek_engine::{ExtendedMetadata, StringKind, StringMeta};

        let Some(idx) = *inspect_node else { return };
        let Some(node) = engine.get_node(idx) else {
            return;
        };

        let script_slot = node.configs().enumerate().find_map(|(i, (slot_def, _))| {
            let ExtendedMetadata::String(StringMeta { kind, .. }) = slot_def.extended() else {
                return None;
            };
            matches!(kind, StringKind::Glsl | StringKind::Rune)
                .then(|| (i, slot_def.name().to_string()))
        });

        let Some((slot_idx, name)) = script_slot else {
            return;
        };

        let popup_id = egui::Id::new(("script_popup", idx, slot_idx));
        let hot_reload_id = egui::Id::new(("hot_reload", idx, slot_idx));
        let pending_source_id = egui::Id::new(("pending_source", idx, slot_idx));

        let popup_open = ui.data(|d| d.get_temp::<bool>(popup_id).unwrap_or(false));
        let mut hot_reload = ui.data(|d| d.get_temp::<bool>(hot_reload_id).unwrap_or(true));

        let current_source = engine
            .get_node(idx)
            .and_then(|n| n.configs().nth(slot_idx))
            .and_then(|(_, v)| match v {
                grafiek_engine::Value::String(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_default();

        let mut pending_source: String = ui
            .data(|d| d.get_temp::<String>(pending_source_id))
            .unwrap_or_else(|| current_source.clone());

        let has_pending_changes = pending_source != current_source;

        let script_errors = Self::collect_script_errors(engine, idx);
        let lints: Vec<_> = script_errors
            .iter()
            .map(|(line, _, msg)| egui_code_editor::lint::Lint::error(*line as usize, msg.clone()))
            .collect();

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(&name).strong());
                Self::show_error_count(ui, &script_errors);
                ui.separator();
                Self::show_compile_controls(
                    ui,
                    engine,
                    idx,
                    slot_idx,
                    &mut hot_reload,
                    hot_reload_id,
                    has_pending_changes,
                    &pending_source,
                );
                ui.separator();
                if ui.small_button("Detach").clicked() {
                    ui.data_mut(|d| d.insert_temp(popup_id, true));
                }
                if ui.small_button("Open Externally").clicked() {
                    todo!("Open in external editor");
                }
            });
            ui.separator();

            if popup_open {
                ui.label(RichText::new("Editing in detached window").weak().italics());
            } else {
                let editor_height = ui.available_height();

                ScrollArea::both()
                    .id_salt("script_editor")
                    .min_scrolled_height(editor_height)
                    .max_height(editor_height)
                    .show(ui, |ui| {
                        Self::show_code_editor(
                            ui,
                            engine,
                            idx,
                            slot_idx,
                            hot_reload,
                            &mut pending_source,
                            "inline",
                            &lints,
                        );
                    });

                ui.data_mut(|d| d.insert_temp(pending_source_id, pending_source.clone()));
            }
        });

        if popup_open {
            let mut open = true;
            egui::Window::new(&name)
                .id(popup_id)
                .open(&mut open)
                .default_size([600.0, 400.0])
                .resizable(true)
                .show(ui.ctx(), |ui| {
                    ui.horizontal(|ui| {
                        Self::show_compile_controls(
                            ui,
                            engine,
                            idx,
                            slot_idx,
                            &mut hot_reload,
                            hot_reload_id,
                            has_pending_changes,
                            &pending_source,
                        );
                    });

                    ScrollArea::both().show(ui, |ui| {
                        Self::show_code_editor(
                            ui,
                            engine,
                            idx,
                            slot_idx,
                            hot_reload,
                            &mut pending_source,
                            "popup",
                            &lints,
                        );
                    });

                    ui.data_mut(|d| d.insert_temp(pending_source_id, pending_source.clone()));
                });
            if !open {
                ui.data_mut(|d| d.insert_temp(popup_id, false));
            }
        }
    }

    fn collect_script_errors(engine: &Engine, idx: NodeIndex) -> Vec<(u32, u32, String)> {
        engine
            .node_errors(idx)
            .into_iter()
            .flatten()
            .filter_map(|e| e.as_script_error())
            .flat_map(|se| se.errors.iter())
            .map(|e| (e.line, e.column, e.message.clone()))
            .collect()
    }

    fn show_error_count(ui: &mut egui::Ui, errors: &[(u32, u32, String)]) {
        if !errors.is_empty() {
            ui.label(
                RichText::new(format!("{} error(s)", errors.len()))
                    .color(egui::Color32::from_rgb(255, 100, 100)),
            );
        }
    }

    fn show_compile_controls(
        ui: &mut egui::Ui,
        engine: &mut Engine,
        idx: NodeIndex,
        slot_idx: usize,
        hot_reload: &mut bool,
        hot_reload_id: egui::Id,
        has_pending_changes: bool,
        pending_source: &String,
    ) {
        if ui.checkbox(hot_reload, "Hot-reload").changed() {
            ui.data_mut(|d| d.insert_temp(hot_reload_id, *hot_reload));
            if *hot_reload && has_pending_changes {
                Self::apply_source(engine, idx, slot_idx, pending_source);
            }
        }
        let compile_text = if has_pending_changes {
            "Compile*"
        } else {
            "Compile"
        };
        if ui.small_button(compile_text).clicked() && has_pending_changes {
            Self::apply_source(engine, idx, slot_idx, pending_source);
        }
    }

    fn apply_source(engine: &mut Engine, idx: NodeIndex, slot_idx: usize, source: &String) {
        let _ = engine.edit_node_config(idx, slot_idx, |_, value| {
            if let grafiek_engine::ValueMut::String(s) = value {
                *s = source.clone();
            }
        });
    }

    fn show_code_editor(
        ui: &mut egui::Ui,
        engine: &mut Engine,
        idx: NodeIndex,
        slot_idx: usize,
        hot_reload: bool,
        pending_source: &mut String,
        id_suffix: &str,
        lints: &[egui_code_editor::lint::Lint],
    ) {
        use egui_code_editor::{CodeEditor, ColorTheme, Syntax};

        let id = format!("script_code_editor_{}", id_suffix);
        if hot_reload {
            let _ = engine.edit_node_config(idx, slot_idx, |_, value| {
                if let grafiek_engine::ValueMut::String(s) = value {
                    CodeEditor::default()
                        .id_source(&id)
                        .with_syntax(Syntax::rust())
                        .with_theme(ColorTheme::GRUVBOX_DARK)
                        .with_numlines(true)
                        .with_lints(lints.to_vec())
                        .show(ui, s);
                    *pending_source = s.clone();
                }
            });
        } else {
            CodeEditor::default()
                .id_source(&id)
                .with_syntax(Syntax::rust())
                .with_theme(ColorTheme::GRUVBOX_DARK)
                .with_numlines(true)
                .with_lints(lints.to_vec())
                .show(ui, pending_source);
        }
    }

    fn show_inspector(
        ui: &mut egui::Ui,
        engine: &mut Engine,
        inspect_node: &mut Option<NodeIndex>,
    ) {
        ui.vertical(|ui| {
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
                ui.label("Label");
                ui.add(TextEdit::singleline(&mut label).desired_width(100.0));
            });
            ui.label(RichText::new(format!("Type {}", &op_path)).weak());

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
        });
    }
}
