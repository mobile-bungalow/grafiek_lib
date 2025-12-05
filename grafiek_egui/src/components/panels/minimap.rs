use egui::{Color32, Rect, Stroke, Vec2};
use egui_snarl::Snarl;
use grafiek_engine::Engine;

use crate::components::snarl::NodeData;

/// Get header color for a node based on its library
pub fn node_color(lib: &str) -> Color32 {
    match lib {
        "core" => Color32::from_rgb(50, 88, 80),
        "math" => Color32::from_rgb(60, 82, 130),
        _ => Color32::from_rgb(60, 80, 100),
    }
}

pub fn show_minimap(ctx: &egui::Context, engine: &Engine, snarl: &Snarl<NodeData>) {
    let viewport_rect = ctx.input(|i| i.viewport_rect());

    egui::Window::new("Minimap")
        .fixed_pos(egui::pos2(
            ctx.viewport_rect().width() - 160.0,
            ctx.viewport_rect().height() - 160.0,
        ))
        .fixed_size(egui::vec2(150.0, 150.0))
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .show(ctx, |ui| {
            let (response, painter) =
                ui.allocate_painter(egui::vec2(200.0, 150.0), egui::Sense::hover());

            let minimap_rect = response.rect;

            let mut min = viewport_rect.min;
            let mut max = viewport_rect.max;

            for (id, _) in snarl.node_ids() {
                if let Some(info) = snarl.get_node_info(id) {
                    let size = egui::vec2(150.0, 50.0);
                    min = min.min(info.pos);
                    max = max.max(info.pos + size);
                }
            }

            let padding = 50.0;
            min -= Vec2::splat(padding);
            max += Vec2::splat(padding);

            let world_size = max - min;

            let scale =
                (minimap_rect.width() / world_size.x).min(minimap_rect.height() / world_size.y);

            painter.rect_filled(minimap_rect, 2.0, Color32::from_gray(20));

            for (id, node_data) in snarl.node_ids() {
                if let Some(info) = snarl.get_node_info(id) {
                    let color = engine
                        .get_node(node_data.engine_node)
                        .map(|n| node_color(&n.record().op_path.library))
                        .unwrap_or(Color32::from_rgb(80, 80, 100));

                    let local_pos = (info.pos - min) * scale;
                    let local_size = egui::vec2(150.0, 50.0) * scale;

                    let node_rect = Rect::from_min_size(minimap_rect.min + local_pos, local_size);
                    painter.rect_filled(node_rect, 1.0, color);
                }
            }

            // Draw viewport indicator
            let viewport_min = (viewport_rect.min - min) * scale;
            let viewport_size = viewport_rect.size() * scale;

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
