use egui::Color32;

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

// TODO: define these syntaxes for the code editor unless
// we go a different editor route
pub mod syntax {
    pub const RUNE: () = ();
    pub const GLSL: () = ();
    pub const JSON: () = ();
}
