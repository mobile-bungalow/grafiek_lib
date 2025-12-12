use std::collections::HashMap;
use std::sync::Arc;

use egui::{TextureId as EguiTextureId, Vec2};
use grafiek_engine::{Engine, TextureHandle, TextureId};

struct CachedTexture {
    egui_id: EguiTextureId,
    size: (u32, u32),
}

#[derive(Default)]
pub struct TextureCache {
    cache: HashMap<TextureId, CachedTexture>,
}

pub fn show_texture_preview(
    ui: &mut egui::Ui,
    engine: &Engine,
    texture_cache: &mut TextureCache,
    render_state: &Arc<eframe::egui_wgpu::RenderState>,
    handle: &TextureHandle,
    max_width: f32,
) -> bool {
    let Some(tex_id) = handle.id() else {
        return false;
    };
    let Some(wgpu_tex) = engine.get_texture(handle) else {
        return false;
    };

    let egui_tex = texture_cache.get_or_register(ui.ctx(), render_state, tex_id, wgpu_tex);
    let aspect = handle.width() as f32 / handle.height() as f32;
    let size = Vec2::new(max_width, max_width / aspect);
    ui.image(egui::load::SizedTexture::new(egui_tex, size));
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
        let tex_size = wgpu_texture.size();
        let current_size = (tex_size.width, tex_size.height);

        // Check if we have a cached entry
        if let Some(cached) = self.cache.get_mut(&engine_id) {
            if cached.size == current_size {
                return cached.egui_id;
            }
            // Size changed - need to update
            let view = wgpu_texture.create_view(&wgpu::TextureViewDescriptor::default());
            let mut renderer = render_state.renderer.write();
            renderer.update_egui_texture_from_wgpu_texture(
                &render_state.device,
                &view,
                wgpu::FilterMode::Linear,
                cached.egui_id,
            );
            cached.size = current_size;
            return cached.egui_id;
        }

        // No cache entry - register new
        let view = wgpu_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut renderer = render_state.renderer.write();
        let egui_id =
            renderer.register_native_texture(&render_state.device, &view, wgpu::FilterMode::Linear);
        self.cache.insert(
            engine_id,
            CachedTexture {
                egui_id,
                size: current_size,
            },
        );
        egui_id
    }

    /// Invalidate a texture (call when it's been resized/replaced).
    pub fn invalidate(&mut self, ctx: &egui::Context, engine_id: TextureId) {
        if let Some(cached) = self.cache.remove(&engine_id) {
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
