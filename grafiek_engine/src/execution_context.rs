use wgpu::{Device, Queue, Texture};

use crate::{
    TextureHandle,
    gpu_pool::{GPUResourcePool, create_gpu_texture_empty},
};

/// Timing information for graph execution, set by the application.
#[derive(Debug, Clone, Copy, Default)]
pub struct TimeInfo {
    /// Current time in seconds
    pub time: f32,
    /// Time since last frame in seconds
    pub delta: f32,
    /// Current frame number
    pub frame: u64,
}

#[derive(Debug, Default)]
pub struct ExecutionState {
    pub timing: TimeInfo,
}

#[derive(Debug)]
pub struct ExecutionContext {
    pub device: Device,
    pub queue: Queue,
    pub(crate) textures: GPUResourcePool,
    pub(crate) state: ExecutionState,
}

impl ExecutionContext {
    pub fn texture(&self, handle: &TextureHandle) -> Option<&Texture> {
        self.textures.get_texture(handle.id?)
    }

    pub fn time(&self) -> f32 {
        self.state.timing.time
    }

    pub fn timing(&self) -> &TimeInfo {
        &self.state.timing
    }

    pub(crate) fn set_timing(&mut self, timing: TimeInfo) {
        self.state.timing = timing;
    }

    /// Ensure the texture exists with the correct dimensions, replacing in-place if needed.
    /// This is intended for render targets that are about to be overwritten anyways, it zeros them.
    pub fn ensure_texture(&mut self, handle: &mut TextureHandle) {
        match handle.id {
            None => {
                handle.id = self.textures.alloc_texture(&self.device, handle).into();
            }
            Some(id) => {
                let needs_resize = self.textures.get_texture(id).map_or(false, |tex| {
                    let size = tex.size();
                    size.width != handle.width || size.height != handle.height
                });
                if needs_resize {
                    let texture = create_gpu_texture_empty(&self.device, handle);
                    handle.id = self.textures.replace_texture(id, texture).into();
                }
            }
        }
    }
}
