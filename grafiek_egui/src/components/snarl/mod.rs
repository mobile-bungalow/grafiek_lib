mod background;
mod pin;

use egui::Pos2;
use egui_snarl::{InPin, OutPin, Snarl, ui::SnarlViewer};
use grafiek_engine::{Engine, NodeIndex};

use pin::{PinInfo, PinSide};

pub mod style;
pub use style::style;

use crate::app::ViewState;

/// Pending node creation - position is tracked here since the engine
/// emits CreateNode after we call instance_node, but doesn't know the UI position
#[derive(Debug, Clone)]
pub struct PendingNodeCreate {
    pub position: Pos2,
}

pub struct SnarlView<'a> {
    pub view: &'a mut ViewState,
    pub engine: &'a mut Engine,
}

#[derive(Clone)]
pub struct NodeData {
    pub op_type: String,
    pub engine_node: NodeIndex,
}

pub struct SnarlState {
    pub engine_to_snarl: std::collections::HashMap<NodeIndex, egui_snarl::NodeId>,
    pub viewport: egui::Rect,
}

impl Default for SnarlState {
    fn default() -> Self {
        Self {
            engine_to_snarl: Default::default(),
            viewport: egui::Rect {
                min: Pos2::new(0.0, 0.0),
                max: Pos2::new(1200.0, 900.0),
            },
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
            .and_then(|n| n.record().label.clone())
            .unwrap_or_else(|| node.op_type.clone())
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
        if let Some(snarl_node) = snarl.get_node(node) {
            if let Some(node) = self.engine.get_node(snarl_node.engine_node) {
                let lib = node.record().op_path.library.as_str();
                let header_color = crate::components::panels::minimap::node_color(lib);

                return default.fill(header_color);
            }
        }

        default
    }

    fn has_body(&mut self, node: &NodeData) -> bool {
        self.engine
            .get_node(node.engine_node)
            .map(|n| n.config_count() > 0)
            .unwrap_or_default()
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

        let config_count = self
            .engine
            .get_node(idx)
            .map(|n| n.config_count())
            .unwrap_or(0);

        for slot_idx in 0..config_count {
            let _ = self
                .engine
                .edit_node_config(idx, slot_idx, |slot_def, value| {
                    ui.vertical(|ui| {
                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            ui.label(slot_def.name.as_ref());
                            crate::components::value::value_editor(ui, slot_def, value);
                        });
                        ui.add_space(10.0);
                    });
                });
        }
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

        let _ = self
            .engine
            .edit_node_input(idx, slot_idx, |slot_def, value| {
                ui.horizontal(|ui| {
                    ui.label(slot_def.name.as_ref());
                    if !connected {
                        crate::components::value::value_editor(ui, slot_def, value);
                    }
                });
            });

        PinInfo::default().with_side(PinSide::Left)
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

        let Some(slot_def) = engine_node.signature().output(pin.id.output) else {
            ui.label("out");
            return PinInfo::default();
        };

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(slot_def.name.as_ref());
        });

        PinInfo::default().with_side(PinSide::Right)
    }

    fn has_graph_menu(&mut self, _pos: egui::Pos2, _snarl: &mut Snarl<NodeData>) -> bool {
        true
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
                        // Store position for message handler to use
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
                    log::error!("Failed to create node {}/{}: {}", library, name, e);
                }
            }
        }
    }

    fn connect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<NodeData>) {
        let from_node = snarl[from.id.node].engine_node;
        let to_node = snarl[to.id.node].engine_node;

        // Call engine directly - it will emit Connect message
        if let Err(e) = self
            .engine
            .connect(from_node, to_node, from.id.output, to.id.input)
        {
            log::error!("Failed to connect: {}", e);
        }
    }

    fn disconnect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<NodeData>) {
        let from_node = snarl[from.id.node].engine_node;
        let to_node = snarl[to.id.node].engine_node;

        // Call engine directly - it will emit Disconnect message
        if let Err(e) = self
            .engine
            .disconnect(from_node, to_node, from.id.output, to.id.input)
        {
            log::error!("Failed to disconnect: {}", e);
        }
    }
}
