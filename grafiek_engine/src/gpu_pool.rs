use std::collections::HashMap;

use petgraph::graph::NodeIndex;
use serde::{Deserialize, Serialize};
use wgpu::{Device, Queue, Texture, TextureDescriptor, TextureUsages};

use crate::registry::consts::SYSTEM_TEXTURE_COUNT;
use crate::value::{TextureFormat, TextureHandle};

/// Stable texture identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct TextureId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureOwner {
    Engine,
    Node(NodeIndex),
}

#[derive(Debug)]
struct TextureEntry {
    texture: Texture,
    owner: TextureOwner,
}

/// Manages GPU textures and their ownership.
#[derive(Debug, Default)]
pub struct GPUResourcePool {
    textures: HashMap<TextureId, TextureEntry>,
    next_id: u64,
}

impl GPUResourcePool {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            next_id: SYSTEM_TEXTURE_COUNT,
        }
    }

    fn next_id(&mut self) -> TextureId {
        let id = TextureId(self.next_id);
        self.next_id += 1;
        id
    }

    /// Insert a system texture at its predefined ID.
    pub(crate) fn insert_texture(
        &mut self,
        device: &Device,
        queue: &Queue,
        handle: TextureHandle,
        data: &[u8],
    ) {
        let id = handle.id.expect("system texture must have predefined ID");
        let texture = create_gpu_texture(device, queue, &handle, data);
        self.textures.insert(
            id,
            TextureEntry {
                texture,
                owner: TextureOwner::Engine,
            },
        );
    }

    pub(crate) fn alloc_texture(&mut self, device: &Device, handle: &TextureHandle) -> TextureId {
        let id = self.next_id();
        let texture = create_gpu_texture_empty(device, handle);
        self.textures.insert(
            id,
            TextureEntry {
                texture,
                owner: TextureOwner::Engine,
            },
        );
        id
    }

    pub(crate) fn alloc_texture_with_data(
        &mut self,
        device: &Device,
        queue: &Queue,
        owner: NodeIndex,
        handle: &TextureHandle,
        data: &[u8],
    ) -> TextureId {
        let id = self.next_id();
        let texture = create_gpu_texture(device, queue, handle, data);
        self.textures.insert(
            id,
            TextureEntry {
                texture,
                owner: TextureOwner::Node(owner),
            },
        );
        id
    }

    pub fn get_texture(&self, id: TextureId) -> Option<&Texture> {
        self.textures.get(&id).map(|e| &e.texture)
    }

    pub fn replace_texture(&mut self, id: TextureId, texture: Texture) {
        if let Some(entry) = self.textures.get_mut(&id) {
            entry.texture = texture;
        }
    }

    pub fn release_texture(&mut self, id: TextureId) {
        self.textures.remove(&id);
    }

    pub fn release_node_textures(&mut self, node: NodeIndex) {
        self.textures
            .retain(|_, e| e.owner != TextureOwner::Node(node));
    }
}

fn texture_format_to_wgpu(fmt: TextureFormat) -> wgpu::TextureFormat {
    match fmt {
        TextureFormat::RGBAu8 => wgpu::TextureFormat::Rgba8Unorm,
        TextureFormat::RGBAu16 => wgpu::TextureFormat::Rgba16Unorm,
        TextureFormat::RGBAF32 => wgpu::TextureFormat::Rgba32Float,
        TextureFormat::BGRA8 => wgpu::TextureFormat::Bgra8Unorm,
    }
}

fn create_gpu_texture(
    device: &Device,
    queue: &Queue,
    handle: &TextureHandle,
    data: &[u8],
) -> Texture {
    let size = wgpu::Extent3d {
        width: handle.width,
        height: handle.height,
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&TextureDescriptor {
        label: None,
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: texture_format_to_wgpu(handle.fmt),
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        view_formats: &[],
    });

    let bytes_per_pixel = match handle.fmt {
        TextureFormat::RGBAu8 | TextureFormat::BGRA8 => 4,
        TextureFormat::RGBAu16 => 8,
        TextureFormat::RGBAF32 => 16,
    };

    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(handle.width * bytes_per_pixel),
            rows_per_image: Some(handle.height),
        },
        size,
    );

    texture
}

pub(crate) fn create_gpu_texture_empty(device: &Device, handle: &TextureHandle) -> Texture {
    let size = wgpu::Extent3d {
        width: handle.width,
        height: handle.height,
        depth_or_array_layers: 1,
    };

    device.create_texture(&TextureDescriptor {
        label: None,
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: texture_format_to_wgpu(handle.fmt),
        usage: TextureUsages::TEXTURE_BINDING
            | TextureUsages::COPY_DST
            | TextureUsages::STORAGE_BINDING
            | TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    })
}
