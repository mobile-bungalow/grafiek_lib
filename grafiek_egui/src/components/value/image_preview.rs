use std::collections::HashMap;
use std::sync::Arc;

use egui::{Color32, TextureId as EguiTextureId, Vec2};
use grafiek_engine::{Engine, TextureHandle, TextureId};

use crate::consts::preview::BOX_SIZE;

struct CachedTexture {
    egui_id: EguiTextureId,
    generation: u64,
}

#[derive(Default)]
pub struct TextureCache {
    cache: HashMap<u64, CachedTexture>,
}

pub fn show_texture_preview(
    ui: &mut egui::Ui,
    engine: &Engine,
    texture_cache: &mut TextureCache,
    render_state: &Arc<eframe::egui_wgpu::RenderState>,
    handle: &TextureHandle,
) -> bool {
    let Some(tex_id) = handle.id() else {
        return false;
    };
    let Some(wgpu_tex) = engine.get_texture(handle) else {
        return false;
    };

    let egui_tex = texture_cache.get_or_register(ui.ctx(), render_state, tex_id, wgpu_tex);

    // Calculate image size to fit within letterbox while preserving aspect ratio
    let img_w = handle.width() as f32;
    let img_h = handle.height() as f32;
    let scale = (BOX_SIZE / img_w).min(BOX_SIZE / img_h);
    let size = Vec2::new(img_w * scale, img_h * scale);

    // Draw letterbox background and centered image
    ui.vertical_centered(|ui| {
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(BOX_SIZE), egui::Sense::hover());
        ui.painter().rect_filled(rect, 0.0, Color32::BLACK);

        let image_rect = egui::Rect::from_center_size(rect.center(), size);
        ui.painter().image(
            egui_tex,
            image_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            Color32::WHITE,
        );
    });

    true
}

impl TextureCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_or_register(
        &mut self,
        _ctx: &egui::Context,
        render_state: &eframe::egui_wgpu::RenderState,
        engine_id: TextureId,
        wgpu_texture: &wgpu::Texture,
    ) -> EguiTextureId {
        self.get_or_register_without_ctx(render_state, engine_id, wgpu_texture)
    }

    pub fn get_or_register_without_ctx(
        &mut self,
        render_state: &eframe::egui_wgpu::RenderState,
        engine_id: TextureId,
        wgpu_texture: &wgpu::Texture,
    ) -> EguiTextureId {
        // Check if we have a cached entry
        if let Some(cached) = self.cache.get_mut(&engine_id.stable_id) {
            if cached.generation == engine_id.generation {
                return cached.egui_id;
            }

            let view = wgpu_texture.create_view(&wgpu::TextureViewDescriptor::default());

            let mut renderer = render_state.renderer.write();
            renderer.update_egui_texture_from_wgpu_texture(
                &render_state.device,
                &view,
                wgpu::FilterMode::Linear,
                cached.egui_id,
            );
            return cached.egui_id;
        } else {
            let view = wgpu_texture.create_view(&wgpu::TextureViewDescriptor::default());
            let mut renderer = render_state.renderer.write();

            let egui_id = renderer.register_native_texture(
                &render_state.device,
                &view,
                wgpu::FilterMode::Linear,
            );

            self.cache.insert(
                engine_id.stable_id,
                CachedTexture {
                    generation: engine_id.generation,
                    egui_id,
                },
            );
            egui_id
        }
    }

    /// Invalidate a texture (call when it's been resized/replaced).
    pub fn invalidate(&mut self, ctx: &egui::Context, engine_id: TextureId) {
        if let Some(cached) = self.cache.remove(&engine_id.stable_id) {
            ctx.tex_manager().write().free(cached.egui_id);
        }
    }

    pub fn clear(&mut self, ctx: &egui::Context) {
        for cached in self.cache.values() {
            ctx.tex_manager().write().free(cached.egui_id);
        }
        self.cache.clear();
    }
}
