use crate::TextureHandle;

#[derive(Debug, Copy, Clone)]
pub enum OpCategory {
    Arithmetic,
    Scripting,
    Graphics,
    Vector,
    Engine,
    Logic,
    Color,
}

impl TextureHandle {
    /// 1x1 black pixel texture (default/fallback)
    pub const SPECK_TEX: TextureHandle = TextureHandle(0);
    /// 1x1 white pixel texture
    pub const WHITE_TEX: TextureHandle = TextureHandle(1);
    /// 1x1 transparent pixel texture
    pub const TRANSPARENT_TEX: TextureHandle = TextureHandle(2);
    /// 2x2 checkerboard pattern (for missing textures)
    pub const MISSING_TEX: TextureHandle = TextureHandle(3);
    /// First index available for user textures
    pub const USER_TEX_START: u32 = 16;
}
