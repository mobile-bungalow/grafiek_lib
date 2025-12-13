use egui::Color32;

/// Preview image dimensions
pub mod preview {
    /// Size of the letterbox container for node body image previews
    pub const BOX_SIZE: f32 = 150.0;
}

/// UI color palette
pub mod colors {
    use super::Color32;

    pub const SELECTED: Color32 = Color32::from_rgb(66, 135, 245);
    pub const INSPECTED: Color32 = Color32::from_rgb(230, 175, 80);

    pub const CATEGORY_SYSTEM: Color32 = Color32::from_rgb(100, 100, 100);
    pub const CATEGORY_MATH: Color32 = Color32::from_rgb(80, 120, 200);
    pub const CATEGORY_GRAPHICS: Color32 = Color32::from_rgb(200, 80, 120);

    pub const DEFAULT_NODE: Color32 = Color32::from_rgb(60, 60, 60);
}

/// Pin colors by value type
pub mod pins {
    use super::Color32;

    pub const I32: Color32 = Color32::from_rgb(90, 160, 90);
    pub const F32: Color32 = Color32::from_rgb(120, 180, 120);
    pub const BOOL: Color32 = Color32::from_rgb(180, 100, 100);
    pub const TEXTURE: Color32 = Color32::from_rgb(100, 150, 200);
    pub const BUFFER: Color32 = Color32::from_rgb(180, 130, 200);
    pub const STRING: Color32 = Color32::from_rgb(200, 180, 100);
    pub const ANY: Color32 = Color32::from_rgb(200, 200, 200);
}

// TODO: define these syntaxes for the code editor unless
// we go a different editor route
pub mod syntax {
    pub const RUNE: () = ();
    pub const GLSL: () = ();
    pub const JSON: () = ();
}
