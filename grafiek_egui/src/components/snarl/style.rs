use egui_snarl::ui::{NodeLayout, NodeLayoutKind, SnarlStyle};

pub const fn style() -> SnarlStyle {
    let mut style = egui_snarl::ui::SnarlStyle::new();

    style.node_frame = Some(egui::Frame {
        inner_margin: egui::Margin::same(8),
        outer_margin: egui::Margin::same(4),
        corner_radius: egui::CornerRadius::same(8),
        fill: egui::Color32::from_gray(30),
        stroke: egui::Stroke::NONE,
        shadow: egui::Shadow::NONE,
    });

    style.node_layout = Some(NodeLayout {
        kind: NodeLayoutKind::FlippedSandwich,
        ..NodeLayout::coil()
    });

    style
}
