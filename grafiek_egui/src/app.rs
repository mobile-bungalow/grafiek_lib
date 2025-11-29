use anyhow::Result;
#[derive(Default)]
pub struct GrafiekApp {}

impl GrafiekApp {
    pub fn init() -> Result<Self> {
        Ok(Self {})
    }
}

impl eframe::App for GrafiekApp {
    fn update(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {}
}
