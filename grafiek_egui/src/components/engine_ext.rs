use std::sync::Arc;

use grafiek_engine::{
    Engine, ExtendedMetadata, NodeIndex, StringKind, StringMeta, TextureHandle, TextureMeta, Value,
    ValueType,
    ops::{Input, Output},
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

    /// Returns true if the node has any script configs attached (Glsl or Rune).
    fn has_script(&self, node: NodeIndex) -> bool;

    /// Returns true if this is an Input node.
    fn is_input_node(&self, node: NodeIndex) -> bool;

    /// Returns true if this is an Output node.
    fn is_output_node(&self, node: NodeIndex) -> bool;

    /// Shows UI for an Input node's value (image picker or value editor).
    fn show_input_node_body(
        &mut self,
        ui: &mut egui::Ui,
        node: NodeIndex,
        texture_cache: &mut TextureCache,
        render_state: &Arc<eframe::egui_wgpu::RenderState>,
    );

    /// Shows UI for an Output node's value preview.
    fn show_output_node_body(
        &self,
        ui: &mut egui::Ui,
        node: NodeIndex,
        texture_cache: &mut TextureCache,
        render_state: &Arc<eframe::egui_wgpu::RenderState>,
    );
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

    fn has_script(&self, node: NodeIndex) -> bool {
        let Some(engine_node) = self.get_node(node) else {
            return false;
        };

        engine_node.configs().any(|(slot_def, _)| {
            matches!(
                slot_def.extended(),
                ExtendedMetadata::String(StringMeta {
                    kind: StringKind::Glsl | StringKind::Rune,
                    ..
                })
            )
        })
    }

    fn is_input_node(&self, node: NodeIndex) -> bool {
        self.get_node(node)
            .and_then(|n| n.operation::<Input>())
            .is_some()
    }

    fn is_output_node(&self, node: NodeIndex) -> bool {
        self.get_node(node)
            .and_then(|n| n.operation::<Output>())
            .is_some()
    }

    fn show_input_node_body(
        &mut self,
        ui: &mut egui::Ui,
        node: NodeIndex,
        texture_cache: &mut TextureCache,
        render_state: &Arc<eframe::egui_wgpu::RenderState>,
    ) {
        let Some(engine_node) = self.get_node(node) else {
            return;
        };

        // Check if this input has a texture output with allow_file
        let sig = engine_node.signature();
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

        if let Some(slot) = texture_slot {
            if let Some((_, Value::Texture(handle))) = engine_node.output(slot) {
                image_preview::show_texture_preview(ui, self, texture_cache, render_state, handle);
            }
            if ui.button("Load Image...").clicked() {
                crate::components::image_picker::pick_and_load_image(self, node, slot);
            }
        } else {
            let _ = self.edit_graph_input(node, |slot_def, value| {
                crate::components::value::value_editor(ui, slot_def, value);
            });
        }
    }

    fn show_output_node_body(
        &self,
        ui: &mut egui::Ui,
        node: NodeIndex,
        texture_cache: &mut TextureCache,
        render_state: &Arc<eframe::egui_wgpu::RenderState>,
    ) {
        let Some(engine_node) = self.get_node(node) else {
            return;
        };

        match engine_node.input(0) {
            Some((_, Value::Texture(handle))) => {
                image_preview::show_texture_preview(ui, self, texture_cache, render_state, handle);
            }
            Some((_, value)) => {
                ui.label(format!("{}", value));
            }
            None => {}
        }
    }
}
