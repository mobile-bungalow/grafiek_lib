use std::f32;

use egui::{Color32, Rect, Stroke, Vec2};
use egui_snarl::Snarl;
use grafiek_engine::Engine;

use crate::components::snarl::NodeData;

const MAP_PAD: f32 = 50.0;
const NODE_SIZE: Vec2 = egui::vec2(150.0, 50.0);
const MAP_SIZE: Vec2 = egui::vec2(160., 160.);
const TOP_PANEL_H: f32 = 20.;

//TODO: We should export the default operator library constants
pub fn node_color(lib: &str) -> Color32 {
    match lib {
        "core" => Color32::from_rgb(50, 88, 80),
        "math" => Color32::from_rgb(60, 82, 130),
        _ => Color32::from_rgb(60, 80, 100),
    }
}

pub fn show_minimap(
    ctx: &egui::Context,
    engine: &Engine,
    snarl: &Snarl<NodeData>,
    snarl_viewport: &Rect,
) {
    egui::Window::new("Minimap")
        .fixed_pos(egui::pos2(
            ctx.viewport_rect().width() - MAP_SIZE.x - 10.,
            TOP_PANEL_H,
        ))
        .fixed_size(MAP_SIZE)
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .show(ctx, |ui| {
            let (response, painter) =
                ui.allocate_painter(egui::vec2(200.0, 150.0), egui::Sense::hover());

            let minimap_rect = response.rect;

            let min = snarl_viewport.min;
            let max = snarl_viewport.max;

            let (mut min, mut max) = snarl.nodes_pos().fold((min, max), |(min, max), (pos, _)| {
                let min = min.min(pos);
                let max = max.max(pos + NODE_SIZE);
                (min, max)
            });

            min -= Vec2::splat(MAP_PAD);
            max += Vec2::splat(MAP_PAD);

            // assuming padding full wXh of the virtual space
            let world_size = max - min;

            // How much we have to scale down to avoid cutting anything off
            let scale =
                (minimap_rect.width() / world_size.x).min(minimap_rect.height() / world_size.y);

            painter.rect_filled(minimap_rect, 2.0, Color32::from_gray(20));

            for node_data in snarl.nodes_info() {
                let color = engine
                    .get_node(node_data.value.engine_node)
                    .map(|n| node_color(&n.op_path().library))
                    .unwrap_or(Color32::from_rgb(80, 80, 100));

                let local_pos = (node_data.pos - min) * scale;
                let local_size = NODE_SIZE * scale;

                let node_rect = Rect::from_min_size(minimap_rect.min + local_pos, local_size);
                painter.rect_filled(node_rect, 1.0, color);
            }

            let viewport_min = (snarl_viewport.min - min) * scale;
            let viewport_size = snarl_viewport.size() * scale;

            let viewport_minimap =
                Rect::from_min_size(minimap_rect.min + viewport_min, viewport_size);

            painter.rect_stroke(
                viewport_minimap,
                1.0,
                Stroke::new(2.0, Color32::from_rgb(100, 150, 200)),
                egui::StrokeKind::Outside,
            );
        });
}
