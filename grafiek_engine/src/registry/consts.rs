use crate::value::{TextureFormat, TextureHandle};

pub const ENGINE_OWNER: usize = 0;

pub const SPECK_ID: u32 = 0;
pub const FLECK_ID: u32 = 1;
pub const TRANSPECK_ID: u32 = 2;
pub const CHECK_ID: u32 = 3;

/// A 1x1 black texture (tiny little speck).
pub const SPECK: TextureHandle = TextureHandle {
    id: Some(SPECK_ID),
    width: 1,
    height: 1,
    fmt: TextureFormat::RGBAu8,
};

/// A 1x1 white texture (tiny little fleck).
pub const FLECK: TextureHandle = TextureHandle {
    id: Some(FLECK_ID),
    width: 1,
    height: 1,
    fmt: TextureFormat::RGBAu8,
};

/// A 1x1 transparent texture.
pub const TRANSPARENT_SPECK: TextureHandle = TextureHandle {
    id: Some(TRANSPECK_ID),
    width: 1,
    height: 1,
    fmt: TextureFormat::RGBAu8,
};

// A 2x2 black and magenta check texture
pub const CHECK: TextureHandle = TextureHandle {
    id: Some(CHECK_ID),
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
