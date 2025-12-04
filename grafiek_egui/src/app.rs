use anyhow::Result;
use egui_notify::Toasts;
use egui_snarl::Snarl;
use grafiek_engine::{Engine, NodeIndex};

use crate::components::{
    close_prompt::ClosePrompt,
    menu_bar::MenuBar,
    snarl::{self, SnarlState, SnarlView, style},
};

#[derive(Default)]
pub struct ViewState {
    pub show_logs: bool,
    pub show_io: bool,
    pub show_settings: bool,
    pub show_debug: bool,
    pub show_minimap: bool,
    pub show_inspect_node: Option<NodeIndex>,
    pub close_prompt: ClosePrompt,
    pub snarl_ui: SnarlState,
    pub notifications: Toasts,
}

pub struct GrafiekApp {
    pub engine: Engine,
    /// What should be on screen
    pub view_state: ViewState,
    /// snarl state - disjoint borrows in graph display
    pub snarl: Snarl<snarl::NodeData>,
}

impl GrafiekApp {
    pub fn init(engine: Engine) -> Result<Self> {
        Ok(Self {
            engine,
            view_state: Default::default(),
            snarl: Default::default(),
        })
    }

    pub fn needs_save(&self) -> bool {
        true
    }

    pub fn save_project(&mut self) {
        // TODO: implement save logic
    }
}

impl eframe::App for GrafiekApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Conditionally show close prompt on shutdown
        self.show_close_prompt(ctx);

        MenuBar::show(ctx, &mut self.view_state);

        egui::CentralPanel::default().show(ctx, |ui| {
            let view = &mut SnarlView {
                view: &mut self.view_state,
                engine: &mut self.engine,
            };

            self.snarl.show(view, &snarl::style(), "snarl", ui);
        });
    }
}
