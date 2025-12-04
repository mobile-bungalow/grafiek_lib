use egui::{Context, ViewportBuilder, ViewportId};

use crate::app::GrafiekApp;

#[derive(Default)]
pub struct ClosePrompt {
    pub is_showing: bool,
    pub finalized: bool,
}

impl GrafiekApp {
    pub fn show_close_prompt(&mut self, ctx: &Context) {
        let close_requested = ctx.input(|i| i.viewport().close_requested());
        self.view_state.close_prompt.is_showing |= close_requested;

        if !self.view_state.close_prompt.is_showing || self.view_state.close_prompt.finalized {
            return;
        }

        if self.needs_save() {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }

        let viewport_id = ViewportId::from_hash_of("close_prompt_viewport");

        let vp = ViewportBuilder::default()
            .with_title("Unsaved Changes")
            .with_inner_size([320.0, 120.0])
            .with_resizable(false)
            .with_always_on_top();

        ctx.show_viewport_immediate(viewport_id, vp, |ctx, _class| {
            // If user closes this viewport, treat as cancel
            if ctx.input(|i| i.viewport().close_requested()) {
                self.view_state.close_prompt.is_showing = false;
            }

            egui::CentralPanel::default().show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    ui.label("You have unsaved changes.");
                    ui.label("What would you like to do?");
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.add_space(ui.available_width() / 5.0);
                        if ui.button("Save").clicked() {
                            self.view_state.close_prompt.finalized = true;
                            ctx.send_viewport_cmd_to(
                                ViewportId::ROOT,
                                egui::ViewportCommand::Close,
                            );
                        }
                        if ui.button("Don't Save").clicked() {
                            self.view_state.close_prompt.finalized = true;
                            ctx.send_viewport_cmd_to(
                                ViewportId::ROOT,
                                egui::ViewportCommand::Close,
                            );
                        }
                        if ui.button("Cancel").clicked() {
                            self.view_state.close_prompt.is_showing = false;
                        }
                        ui.add_space(ui.available_width() / 5.0);
                    });
                });
            });
        });
    }
}
