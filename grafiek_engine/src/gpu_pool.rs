use std::collections::HashMap;

use petgraph::graph::NodeIndex;
use wgpu::{Device, Queue, Texture, TextureDescriptor, TextureUsages};

use crate::value::{TextureFormat, TextureHandle};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureOwner {
    Engine,
    /// Textures owned by a specific node
    Node(NodeIndex),
}

#[derive(Debug)]
struct TextureEntry {
    owner: TextureOwner,
    texture: Texture,
}

/// Manages GPU textures and their ownership.
#[derive(Debug, Default)]
pub struct GPUResourcePool {
    /// Map from texture ID to entry
    textures: HashMap<u32, TextureEntry>,
    /// The next ID to assign (automatically tracks past system textures)
    next_id: u32,
}

impl GPUResourcePool {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            next_id: 0,
        }
    }

    /// Register a system texture with predetermined ID and data.
    pub fn register_system_texture(
        &mut self,
        device: &Device,
        queue: &Queue,
        handle: TextureHandle,
        data: &[u8],
    ) {
        let id = handle.id.expect("system texture must have id");
        self.next_id = self.next_id.max(id + 1);

        let texture = create_gpu_texture(device, queue, &handle, data);
        self.textures.insert(
            id,
            TextureEntry {
                owner: TextureOwner::Engine,
                texture,
            },
        );
    }

    /// Allocate a texture for the given handle, returning the assigned ID.
    pub fn allocate(&mut self, device: &Device, owner: NodeIndex, handle: &TextureHandle) -> u32 {
        let id = self.next_id;
        self.next_id += 1;

        let texture = create_gpu_texture_empty(device, handle);
        self.textures.insert(
            id,
            TextureEntry {
                owner: TextureOwner::Node(owner),
                texture,
            },
        );

        id
    }

    /// Get the GPU texture by ID.
    pub fn get(&self, id: u32) -> Option<&Texture> {
        self.textures.get(&id).map(|e| &e.texture)
    }

    /// Release a specific texture by ID.
    pub fn release(&mut self, id: u32) {
        self.textures.remove(&id);
    }

    /// Release all textures owned by a node.
    pub fn release_node_textures(&mut self, node: NodeIndex) {
        self.textures
            .retain(|_, entry| entry.owner != TextureOwner::Node(node));
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

fn create_gpu_texture_empty(device: &Device, handle: &TextureHandle) -> Texture {
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
