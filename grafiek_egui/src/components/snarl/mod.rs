mod background;
mod pin;

use egui::Pos2;
use egui_snarl::{InPin, InPinId, OutPin, OutPinId, Snarl, ui::SnarlViewer};
use grafiek_engine::{EdgeIndex, Engine, NodeIndex};
use std::collections::HashMap;

use pin::{PinInfo, PinSide};

pub mod style;
pub use style::style;

use crate::app::ViewState;

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
    pub wire_to_edge: HashMap<(OutPinId, InPinId), EdgeIndex>,
    pub engine_to_snarl: HashMap<NodeIndex, egui_snarl::NodeId>,
    pub viewport: egui::Rect,
}

impl Default for SnarlState {
    fn default() -> Self {
        Self {
            wire_to_edge: Default::default(),
            engine_to_snarl: Default::default(),
            viewport: egui::Rect {
                min: Pos2::new(0.0, 0.0),
                max: Pos2::new(1200.0, 900.0),
            },
        }
    }
}

impl SnarlState {
    pub fn cleanup_node(&mut self, node: egui_snarl::NodeId, engine_idx: NodeIndex) {
        self.wire_to_edge
            .retain(|(out_pin, in_pin), _| out_pin.node != node && in_pin.node != node);
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

    fn show_input(
        &mut self,
        pin: &InPin,
        ui: &mut egui::Ui,
        snarl: &mut Snarl<NodeData>,
    ) -> impl egui_snarl::ui::SnarlPin + 'static {
        let node = &snarl[pin.id.node];
        let idx = node.engine_node;

        let Some(engine_node) = self.engine.get_node(idx) else {
            ui.label("in");
            return PinInfo::default();
        };

        let Some(slot_def) = engine_node.signature().input(pin.id.input) else {
            ui.label("in");
            return PinInfo::default();
        };

        ui.label(slot_def.name.as_ref());
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
}
