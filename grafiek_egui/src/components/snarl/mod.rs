mod background;
mod pin;

use std::sync::Arc;

use egui::{Pos2, Stroke};
use egui_phosphor::regular::WARNING;
use egui_snarl::{InPin, OutPin, Snarl, ui::SnarlViewer};
use grafiek_engine::{Engine, NodeIndex};

use crate::components::engine_ext::EngineExt;

pub use pin::{PinInfo, PinShape, PinSide};

pub mod style;
pub use style::style;

use crate::app::ViewState;
use crate::components::value::image_preview::TextureCache;
use crate::consts::colors::INSPECTED;

pub struct SnarlView<'a> {
    pub view: &'a mut ViewState,
    pub engine: &'a mut Engine,
    pub texture_cache: &'a mut TextureCache,
    pub render_state: &'a Arc<eframe::egui_wgpu::RenderState>,
}

#[derive(Clone)]
pub struct NodeData {
    pub op_type: String,
    pub engine_node: NodeIndex,
}

pub struct SnarlState {
    pub engine_to_snarl: std::collections::HashMap<NodeIndex, egui_snarl::NodeId>,
    pub viewport: egui::Rect,
    /// The egui Id used by the snarl widget, needed for querying selection
    pub snarl_id: Option<egui::Id>,
}

impl Default for SnarlState {
    fn default() -> Self {
        Self {
            engine_to_snarl: Default::default(),
            viewport: egui::Rect {
                min: Pos2::new(0.0, 0.0),
                max: Pos2::new(1200.0, 900.0),
            },
            snarl_id: None,
        }
    }
}

impl SnarlState {
    pub fn cleanup_node(&mut self, _node: egui_snarl::NodeId, engine_idx: NodeIndex) {
        self.engine_to_snarl.remove(&engine_idx);
    }
}

impl<'a> SnarlViewer<NodeData> for SnarlView<'a> {
    fn draw_background(
        &mut self,
        _background: Option<&egui_snarl::ui::BackgroundPattern>,
        viewport: &egui::Rect,
        _snarl_style: &egui_snarl::ui::SnarlStyle,
        style: &egui::Style,
        painter: &egui::Painter,
        _snarl: &Snarl<NodeData>,
    ) {
        self.view.snarl_ui.viewport = *viewport;
        background::draw_grid(viewport, style, painter);
    }

    fn title(&mut self, node: &NodeData) -> String {
        let idx = node.engine_node;
        self.engine
            .get_node(idx)
            .map(|n| n.label().to_string())
            .unwrap_or_else(|| node.op_type.clone())
    }

    fn show_header(
        &mut self,
        node: egui_snarl::NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut egui::Ui,
        snarl: &mut Snarl<NodeData>,
    ) {
        let Some(data) = snarl.get_node(node) else {
            return;
        };

        let title = self.title(data);

        ui.horizontal(|ui| {
            ui.label(title);

            let Some(errors) = self.engine.node_errors(data.engine_node) else {
                return;
            };

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let badge = egui::RichText::new(format!("{} {}", WARNING, errors.len()))
                    .color(egui::Color32::from_rgb(255, 100, 100));

                let response = ui.label(badge);
                response.on_hover_ui(|ui| {
                    for error in errors {
                        if let Some(script_err) = error.as_script_error() {
                            for loc_err in &script_err.errors {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "{}:{}",
                                            loc_err.line, loc_err.column
                                        ))
                                        .color(egui::Color32::LIGHT_GRAY)
                                        .monospace(),
                                    );
                                    ui.label(&loc_err.message);
                                });
                            }
                        } else {
                            ui.label(error.to_string());
                        }
                    }
                });
            });
        });
    }

    fn inputs(&mut self, node: &NodeData) -> usize {
        self.engine
            .get_node(node.engine_node)
            .map_or(0, |n| n.input_count())
    }

    fn outputs(&mut self, node: &NodeData) -> usize {
        self.engine
            .get_node(node.engine_node)
            .map_or(0, |n| n.output_count())
    }

    fn header_frame(
        &mut self,
        default: egui::Frame,
        node: egui_snarl::NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        snarl: &Snarl<NodeData>,
    ) -> egui::Frame {
        if let Some(snarl_node) = snarl.get_node(node)
            && let Some(node) = self.engine.get_node(snarl_node.engine_node)
        {
            let lib = node.op_path().library.as_str();
            let header_color = crate::components::panels::minimap::node_color(lib);

            return default.fill(header_color);
        }

        default
    }

    fn has_body(&mut self, node: &NodeData) -> bool {
        let Some(n) = self.engine.get_node(node.engine_node) else {
            return false;
        };
        n.has_body_config() || !self.engine.preview_textures(node.engine_node).is_empty()
    }

    fn show_body(
        &mut self,
        node: egui_snarl::NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut egui::Ui,
        snarl: &mut Snarl<NodeData>,
    ) {
        let idx = snarl[node].engine_node;

        egui::Frame::NONE
            .inner_margin(egui::Margin::symmetric(0, 10))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 10.0;

                    let _ = self.engine.edit_all_node_configs(idx, |slot_def, value| {
                        if !slot_def.on_node_body() {
                            return;
                        }

                        ui.horizontal(|ui| {
                            ui.label(slot_def.name());
                            crate::components::value::value_editor(ui, slot_def, value);
                        });
                    });

                    self.engine
                        .show_image_previews(ui, idx, self.texture_cache, self.render_state);
                });
            });
    }

    fn show_input(
        &mut self,
        pin: &InPin,
        ui: &mut egui::Ui,
        snarl: &mut Snarl<NodeData>,
    ) -> impl egui_snarl::ui::SnarlPin + 'static {
        let node = &snarl[pin.id.node];
        let idx = node.engine_node;
        let slot_idx = pin.id.input;
        let connected = !pin.remotes.is_empty();

        let mut pin_info = PinInfo::default().with_side(PinSide::Left);

        let _ = self
            .engine
            .edit_node_input(idx, slot_idx, |slot_def, value| {
                let value_type = slot_def.value_type();
                pin_info = pin_info
                    .clone()
                    .with_shape(crate::components::value::pin_shape_for_type(value_type))
                    .with_fill(crate::components::value::pin_color_for_type(value_type));

                ui.horizontal(|ui| {
                    ui.label(slot_def.name());
                    if !connected {
                        crate::components::value::value_editor_with_pin(
                            ui,
                            slot_def,
                            value,
                            &mut pin_info,
                        );
                    }
                });
            });

        pin_info
    }

    fn show_output(
        &mut self,
        pin: &OutPin,
        ui: &mut egui::Ui,
        snarl: &mut Snarl<NodeData>,
    ) -> impl egui_snarl::ui::SnarlPin + 'static {
        let node = &snarl[pin.id.node];
        let idx = node.engine_node;

        let Some(engine_node) = self.engine.get_node(idx) else {
            ui.label("out");
            return PinInfo::default();
        };

        let Some((slot_def, _)) = engine_node.outputs().nth(pin.id.output) else {
            ui.label("out");
            return PinInfo::default();
        };

        let value_type = slot_def.value_type();
        let pin_shape = crate::components::value::pin_shape_for_type(value_type);
        let pin_color = crate::components::value::pin_color_for_type(value_type);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(slot_def.name());
        });

        PinInfo::default()
            .with_side(PinSide::Right)
            .with_shape(pin_shape)
            .with_fill(pin_color)
    }

    fn has_graph_menu(&mut self, _pos: egui::Pos2, _snarl: &mut Snarl<NodeData>) -> bool {
        true
    }

    fn has_node_menu(&mut self, _node: &NodeData) -> bool {
        true
    }

    fn node_frame(
        &mut self,
        default: egui::Frame,
        node: egui_snarl::NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        snarl: &Snarl<NodeData>,
    ) -> egui::Frame {
        let Some(node) = snarl.get_node(node) else {
            return default;
        };

        if self.view.show_inspect_node == Some(node.engine_node) {
            default.stroke(Stroke {
                width: 2.0,
                color: INSPECTED,
            })
        } else {
            default
        }
    }

    fn show_node_menu(
        &mut self,
        node: egui_snarl::NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut egui::Ui,
        snarl: &mut Snarl<NodeData>,
    ) {
        let Some(data) = snarl.get_node(node) else {
            return;
        };

        if ui.button("Inspect").clicked() {
            self.view.show_inspect_node = Some(data.engine_node);
            ui.close();
        }

        ui.separator();

        if ui.button("Delete").clicked() {
            let _ = self.engine.delete_node(data.engine_node);
        }

        ui.button("Copy").clicked();

        ui.button("Cut").clicked();
    }

    fn show_graph_menu(
        &mut self,
        pos: egui::Pos2,
        ui: &mut egui::Ui,
        _snarl: &mut Snarl<NodeData>,
    ) {
        ui.label("Add Node");
        ui.separator();

        let categories = self.engine.node_categories();
        let mut picked = None;

        for category in categories {
            let operators = self.engine.iter_category(category);
            ui.menu_button(category, |ui| {
                for operator in operators {
                    if ui.button(operator).clicked() {
                        ui.close();
                        picked = Some((pos, category, operator));
                    }
                }
            });
        }

        if let Some((pos, library, name)) = picked {
            match self.engine.instance_node(library, name) {
                Ok(idx) => {
                    let _ = self.engine.set_node_position(idx, (pos.x, pos.y));
                }
                Err(e) => {
                    let msg = format!("Failed to create node {}/{}: {}", library, name, e);
                    log::error!("{}", msg);
                    self.view.notifications.error(msg);
                }
            }
        }
    }

    fn connect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<NodeData>) {
        let from_node = snarl[from.id.node].engine_node;
        let to_node = snarl[to.id.node].engine_node;

        if let Err(e) = self
            .engine
            .connect(from_node, to_node, from.id.output, to.id.input)
        {
            let msg = format!("Failed to connect: {}", e);
            log::error!("{}", msg);
            self.view.notifications.error(msg);
        }
    }

    fn disconnect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<NodeData>) {
        let from_node = snarl[from.id.node].engine_node;
        let to_node = snarl[to.id.node].engine_node;

        if let Err(e) = self
            .engine
            .disconnect(from_node, to_node, from.id.output, to.id.input)
        {
            let msg = format!("Failed to disconnect: {}", e);
            log::error!("{}", msg);
            self.view.notifications.error(msg);
        }
    }
}
