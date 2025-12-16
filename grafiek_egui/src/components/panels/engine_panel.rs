use egui::{Context, Ui};
use egui_phosphor::regular::{PAUSE, PLAY};
use grafiek_engine::Engine;

pub fn show_engine_panel(ui: &mut Ui, ctx: &Context, engine: &mut Engine, play: &mut bool) {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        let play_pause = if *play { PLAY } else { PAUSE };

        if ui.button(play_pause).clicked() {
            *play = !*play;
        }

        ui.label("Time")
    });
}
