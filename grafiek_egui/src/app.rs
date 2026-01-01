use std::sync::{
    Arc,
    mpsc::{self, Receiver, Sender},
};

use anyhow::Result;
use egui_notify::Toasts;
use egui_snarl::Snarl;
use egui_snarl::ui::get_selected_nodes;
use grafiek_engine::history::{Event, Message, Mutation};
use grafiek_engine::{Engine, EngineDescriptor, NodeIndex};

use crate::components::{
    close_prompt::ClosePrompt,
    menu_bar::MenuBar,
    panels::{BottomPanel, show_minimap},
    snarl::{self, NodeData, SnarlState, SnarlView},
    value::image_preview::TextureCache,
};

#[derive(Default)]
pub struct ViewState {
    pub show_logs: bool,
    pub show_bottom: bool,
    pub show_settings: bool,
    pub show_debug: bool,
    pub show_minimap: bool,
    pub show_inspect_node: Option<NodeIndex>,
    pub close_prompt: ClosePrompt,
    pub snarl_ui: SnarlState,
    pub notifications: Toasts,
    pub playing: bool,
}

pub struct GrafiekApp {
    pub engine: Engine,
    /// What should be on screen
    pub view_state: ViewState,
    /// snarl state - disjoint borrows in graph display
    pub snarl: Snarl<snarl::NodeData>,
    /// Message receiver from engine
    message_rx: Receiver<Message>,
    /// Cache for displaying engine textures in egui
    pub texture_cache: TextureCache,
    /// Reference to wgpu render state for texture registration
    render_state: Arc<eframe::egui_wgpu::RenderState>,
}

impl GrafiekApp {
    pub fn init(render_state: Arc<eframe::egui_wgpu::RenderState>) -> Result<Self> {
        let (tx, rx): (Sender<Message>, Receiver<Message>) = mpsc::channel();

        let device = render_state.device.clone();
        let queue = render_state.queue.clone();

        let engine = Engine::init(EngineDescriptor {
            device,
            queue,
            on_message: Some(Box::new(move |msg| {
                let _ = tx.send(msg);
            })),
        })?;

        let app = Self {
            engine,
            view_state: Default::default(),
            snarl: Default::default(),
            message_rx: rx,
            texture_cache: TextureCache::new(),
            render_state,
        };

        Ok(app)
    }

    pub fn needs_save(&self) -> bool {
        // TODO: check save state
        true
    }

    /// Get an egui texture ID for displaying an engine texture.
    pub fn get_egui_texture(
        &mut self,
        ctx: &egui::Context,
        handle: &grafiek_engine::TextureHandle,
    ) -> Option<egui::TextureId> {
        let engine_id = handle.id()?;
        let wgpu_texture = self.engine.get_texture(handle)?;
        Some(
            self.texture_cache
                .get_or_register(ctx, &self.render_state, engine_id, wgpu_texture),
        )
    }

    pub fn save_project(&mut self) {
        // TODO: implement save logic
    }

    /// Process engine messages to sync snarl state
    fn process_messages(&mut self) -> bool {
        let mut out = false;
        while let Ok(msg) = self.message_rx.try_recv() {
            match msg {
                Message::Mutation(mutation) => self.handle_mutation(mutation),
                Message::Event(event) => {
                    log::debug!("Engine event: {:?}", event);
                    if let Event::GraphDirtied = event {
                        out = true;
                    }
                }
            }
        }
        out
    }

    /// The engine emits state change messages and we update the UI state here.
    /// It's unfortunate but egui snarl isn't capable of slotting in an abstract
    /// node store.
    fn handle_mutation(&mut self, mutation: Mutation) {
        match mutation {
            Mutation::CreateNode { idx, record } => {
                let node_data = NodeData {
                    op_type: record.op_path.operator.to_string(),
                    engine_node: idx,
                };

                let position = egui::pos2(record.position.0, record.position.1);
                let snarl_id = self.snarl.insert_node(position, node_data);
                self.view_state
                    .snarl_ui
                    .engine_to_snarl
                    .insert(idx, snarl_id);
            }
            Mutation::MoveNode {
                node, new_position, ..
            } => {
                if let Some(&snarl_id) = self.view_state.snarl_ui.engine_to_snarl.get(&node)
                    && let Some(node_info) = self.snarl.get_node_info_mut(snarl_id)
                {
                    node_info.pos = egui::pos2(new_position.0, new_position.1);
                }
            }
            Mutation::DeleteNode { idx, .. } => {
                if let Some(snarl_id) = self.view_state.snarl_ui.engine_to_snarl.remove(&idx) {
                    self.snarl.remove_node(snarl_id);
                }
            }
            Mutation::Connect {
                from_node,
                from_slot,
                to_node,
                to_slot,
            } => {
                if let (Some(&from_snarl), Some(&to_snarl)) = (
                    self.view_state.snarl_ui.engine_to_snarl.get(&from_node),
                    self.view_state.snarl_ui.engine_to_snarl.get(&to_node),
                ) {
                    let out_pin = egui_snarl::OutPinId {
                        node: from_snarl,
                        output: from_slot,
                    };

                    let in_pin = egui_snarl::InPinId {
                        node: to_snarl,
                        input: to_slot,
                    };

                    self.snarl.connect(out_pin, in_pin);
                }
            }
            Mutation::Disconnect {
                from_node,
                from_slot,
                to_node,
                to_slot,
            } => {
                if let (Some(&from_snarl), Some(&to_snarl)) = (
                    self.view_state.snarl_ui.engine_to_snarl.get(&from_node),
                    self.view_state.snarl_ui.engine_to_snarl.get(&to_node),
                ) {
                    let out_pin = egui_snarl::OutPinId {
                        node: from_snarl,
                        output: from_slot,
                    };

                    let in_pin = egui_snarl::InPinId {
                        node: to_snarl,
                        input: to_slot,
                    };

                    self.snarl.disconnect(out_pin, in_pin);
                }
            }
            // These mutations don't require snarl sync
            Mutation::SetConfig { .. } | Mutation::SetInput { .. } | Mutation::SetLabel { .. } => {}
        }
    }
}

impl eframe::App for GrafiekApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Conditionally show close prompt on shutdown
        self.show_close_prompt(ctx);
        self.handle_keypress(ctx);

        let (menu_response, actions) = MenuBar::show(ctx, &mut self.view_state);

        if actions.execute {
            self.engine.execute();
        }

        egui::Window::new("Log")
            .open(&mut self.view_state.show_logs)
            .show(ctx, |ui| {
                egui_logger::logger_ui().show(ui);
            });

        BottomPanel::show(
            ctx,
            &mut self.engine,
            &mut self.view_state.show_inspect_node,
            &mut self.view_state.show_bottom,
        );

        egui::CentralPanel::default().show(ctx, |ui| {
            let snarl_id = ui.make_persistent_id("snarl");
            self.view_state.snarl_ui.snarl_id = Some(snarl_id);

            let view = &mut SnarlView {
                view: &mut self.view_state,
                engine: &mut self.engine,
                texture_cache: &mut self.texture_cache,
                render_state: &self.render_state,
            };

            self.snarl.show(view, &snarl::style(), "snarl", ui);

            // Promote single selection to inspected
            let selected = get_selected_nodes(snarl_id, ctx);
            if selected.len() == 1 {
                let selected_snarl_id = selected[0];
                if let Some((&engine_idx, _)) = self
                    .view_state
                    .snarl_ui
                    .engine_to_snarl
                    .iter()
                    .find(|&(_, &s)| s == selected_snarl_id)
                {
                    self.view_state.show_inspect_node = Some(engine_idx);
                }
            }
        });

        if self.view_state.show_minimap {
            show_minimap(
                ctx,
                &self.engine,
                &self.snarl,
                &self.view_state.snarl_ui.viewport,
            );
        }

        self.view_state.notifications.show(ctx);

        let dirty = self.process_messages();

        if dirty {
            self.engine.execute();
        }
    }
}
