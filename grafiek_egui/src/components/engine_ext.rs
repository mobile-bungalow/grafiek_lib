use std::sync::Arc;

use grafiek_engine::{
    Engine, ExtendedMetadata, NodeIndex, TextureHandle, TextureMeta, Value, ValueType,
};

use super::value::image_preview::{self, TextureCache};

/// Helper functions on the engine for UI display
pub trait EngineExt {
    /// Returns all texture outputs marked with `preview: true` for a node.
    fn preview_textures(&self, node: NodeIndex) -> Vec<&TextureHandle>;

    /// Shows image previews for a node in the UI.
    ///
    /// Returns true if any previews were shown.
    fn show_image_previews(
        &self,
        ui: &mut egui::Ui,
        node: NodeIndex,
        texture_cache: &mut TextureCache,
        render_state: &Arc<eframe::egui_wgpu::RenderState>,
    ) -> bool;
}

impl EngineExt for Engine {
    fn preview_textures(&self, node: NodeIndex) -> Vec<&TextureHandle> {
        let Some(engine_node) = self.get_node(node) else {
            return Vec::new();
        };

        engine_node
            .outputs()
            .filter_map(|(slot_def, value)| {
                let is_preview = matches!(
                    (slot_def.value_type(), slot_def.extended()),
                    (
                        ValueType::Texture,
                        ExtendedMetadata::Texture(TextureMeta { preview: true, .. })
                    )
                );

                if is_preview {
                    if let Value::Texture(handle) = value {
                        return Some(handle);
                    }
                }
                None
            })
            .collect()
    }

    fn show_image_previews(
        &self,
        ui: &mut egui::Ui,
        node: NodeIndex,
        texture_cache: &mut TextureCache,
        render_state: &Arc<eframe::egui_wgpu::RenderState>,
    ) -> bool {
        let handles = self.preview_textures(node);
        if handles.is_empty() {
            return false;
        }

        let mut shown = false;
        for handle in handles {
            if image_preview::show_texture_preview(ui, self, texture_cache, render_state, handle) {
                shown = true;
            }
        }
        shown
    }
}
