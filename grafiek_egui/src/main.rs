use anyhow::{Context, Result};

use crate::app::GrafiekApp;

pub mod app;
pub mod components;
pub mod logging;

fn main() -> Result<()> {
    // TODO: wgpu is really noisy on debug. We should filter it conditionally
    let _ = logging::init(log::LevelFilter::Info).inspect_err(|e| eprintln!("{e:?}"));

    log::info!("Starting Grafiek Egui");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 800.0]),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };

    eframe::run_native(
        "Grafiek",
        options,
        Box::new(|cc| {
            let Some(render_state) = cc.wgpu_render_state.as_ref() else {
                return Err("WGPU unitialized".into());
            };

            let _device = render_state.device.clone();
            let _queue = render_state.queue.clone();

            let app = GrafiekApp::init().context("failed to initialize app")?;

            Ok(Box::new(app))
        }),
    )
    .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}
