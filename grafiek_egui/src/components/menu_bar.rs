use egui;

use crate::app::ViewState;

#[derive(Default)]
pub struct MenuBarActions {
    pub save: bool,
    pub load: bool,
    pub execute: bool,
}

pub struct MenuBar;

impl MenuBar {
    pub fn show(
        ctx: &egui::Context,
        view_state: &mut ViewState,
    ) -> (egui::InnerResponse<()>, MenuBarActions) {
        let mut actions = MenuBarActions::default();

        let response = egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("Grafiek", |ui| {
                    if ui.button("About").clicked() {
                        ui.close();
                    }
                    if ui.button("Settings").clicked() {
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("File", |ui| {
                    if ui.button("Save").clicked() {
                        actions.save = true;
                        ui.close();
                    }

                    if ui.button("Load").clicked() {
                        actions.load = true;
                        ui.close();
                    }
                });

                ui.menu_button("Graph", |ui| {
                    if ui.button("Execute").clicked() {
                        actions.execute = true;
                        ui.close();
                    }
                });

                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut view_state.show_io, "I/O Panel");
                    ui.checkbox(&mut view_state.show_debug, "Debug Info");
                    ui.checkbox(&mut view_state.show_logs, "Logs");
                    ui.checkbox(&mut view_state.show_minimap, "Minimap");
                });
            });
        });

        (response, actions)
    }
}
