use crate::app::GrafiekApp;

impl GrafiekApp {
    pub fn handle_keypress(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.view_state.show_io = !self.view_state.show_io;
        }
    }
}
