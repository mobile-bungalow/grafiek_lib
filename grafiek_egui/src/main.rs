use anyhow::{Context, Result};
use std::sync::Arc;

use crate::app::GrafiekApp;

pub mod app;
pub mod components;
pub mod consts;
pub mod keybinds;
pub mod logging;

fn main() -> Result<()> {
    // TODO: wgpu is really noisy on debug. We should filter it conditionally
    let _ = logging::init(log::LevelFilter::Debug).inspect_err(|e| eprintln!("{e:?}"));

    log::info!("Starting Grafiek Egui");

    let wgpu_options = eframe::egui_wgpu::WgpuConfiguration {
        wgpu_setup: eframe::egui_wgpu::WgpuSetup::CreateNew(
            eframe::egui_wgpu::WgpuSetupCreateNew {
                device_descriptor: Arc::new(|_adapter| wgpu::DeviceDescriptor {
                    label: Some("grafiek device"),
                    required_features: wgpu::Features::PUSH_CONSTANTS,
                    required_limits: wgpu::Limits {
                        max_push_constant_size: 128,
                        ..Default::default()
                    },
                    ..Default::default()
                }),
                ..Default::default()
            },
        ),
        ..Default::default()
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 800.0]),
        renderer: eframe::Renderer::Wgpu,
        wgpu_options,
        ..Default::default()
    };

    eframe::run_native(
        "Grafiek",
        options,
        Box::new(|cc| {
            let Some(render_state) = cc.wgpu_render_state.as_ref() else {
                return Err("WGPU unitialized".into());
            };

            let device = render_state.device.clone();
            let queue = render_state.queue.clone();

            let app = GrafiekApp::init(device, queue).context("failed to initialize app")?;

            Ok(Box::new(app))
        }),
    )
    .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}
