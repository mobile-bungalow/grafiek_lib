use crate::gpu_pool::TextureId;
use crate::value::{TextureFormat, TextureHandle};

/// 1x1 black texture.
pub const SPECK: TextureHandle = TextureHandle {
    id: Some(TextureId(0)),
    width: 1,
    height: 1,
    fmt: TextureFormat::RGBAu8,
};

/// 1x1 white texture.
pub const FLECK: TextureHandle = TextureHandle {
    id: Some(TextureId(1)),
    width: 1,
    height: 1,
    fmt: TextureFormat::RGBAu8,
};

/// 1x1 transparent texture.
pub const TRANSPARENT_SPECK: TextureHandle = TextureHandle {
    id: Some(TextureId(2)),
    width: 1,
    height: 1,
    fmt: TextureFormat::RGBAu8,
};

/// 2x2 black/magenta check pattern.
pub const CHECK: TextureHandle = TextureHandle {
    id: Some(TextureId(3)),
    width: 2,
    height: 2,
    fmt: TextureFormat::RGBAu8,
};

pub(crate) const CHECK_DATA: [u8; 16] = [
    0, 0, 0, 255, // black
    255, 0, 255, 255, // magenta
    255, 0, 255, 255, // magenta
    0, 0, 0, 255, // black
];

/// Number of reserved system texture IDs.
pub(crate) const SYSTEM_TEXTURE_COUNT: u64 = 4;
