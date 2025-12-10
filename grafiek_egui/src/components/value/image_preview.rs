use std::collections::HashMap;

use egui::{TextureId as EguiTextureId, Vec2};
use grafiek_engine::TextureId;

struct CachedTexture {
    egui_id: EguiTextureId,
    size: (u32, u32),
}

#[derive(Default)]
pub struct TextureCache {
    cache: HashMap<TextureId, CachedTexture>,
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

pub fn texture_preview(
    ui: &mut egui::Ui,
    texture_id: EguiTextureId,
    width: u32,
    height: u32,
    max_size: f32,
) -> egui::Response {
    let aspect = width as f32 / height as f32;

    let size = if width > height {
        Vec2::new(max_size, max_size / aspect)
    } else {
        Vec2::new(max_size * aspect, max_size)
    };

    ui.image(egui::ImageSource::Texture(egui::load::SizedTexture {
        id: texture_id,
        size,
    }))
}
