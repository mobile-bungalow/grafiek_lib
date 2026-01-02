use std::sync::Arc;

use crate::app::GrafiekApp;

pub mod app;
pub mod components;
pub mod consts;
pub mod keybinds;
pub mod logging;

fn wgpu_device_descriptor(_: &wgpu::Adapter) -> wgpu::DeviceDescriptor<'static> {
    #[cfg(not(target_arch = "wasm32"))]
    let (features, limits) = (
        wgpu::Features::PUSH_CONSTANTS,
        wgpu::Limits {
            max_push_constant_size: 128,
            ..Default::default()
        },
    );

    #[cfg(target_arch = "wasm32")]
    let (features, limits) = (wgpu::Features::empty(), wgpu::Limits::default());

    wgpu::DeviceDescriptor {
        label: Some("grafiek device"),
        required_features: features,
        required_limits: limits,
        ..Default::default()
    }
}

fn wgpu_configuration() -> eframe::egui_wgpu::WgpuConfiguration {
    #[cfg(not(target_arch = "wasm32"))]
    let instance_descriptor = wgpu::InstanceDescriptor::default();

    #[cfg(target_arch = "wasm32")]
    let instance_descriptor = wgpu::InstanceDescriptor {
        backends: wgpu::Backends::BROWSER_WEBGPU,
        ..Default::default()
    };

    let setup = eframe::egui_wgpu::WgpuSetupCreateNew {
        instance_descriptor,
        device_descriptor: Arc::new(wgpu_device_descriptor),
        ..Default::default()
    };
    eframe::egui_wgpu::WgpuConfiguration {
        wgpu_setup: setup.into(),
        on_surface_error: Arc::new(|e| {
            log::error!("wgpu surface error: {e:?}");
            eframe::egui_wgpu::SurfaceErrorAction::SkipFrame
        }),
        ..Default::default()
    }
}

fn create_app(
    cc: &eframe::CreationContext<'_>,
) -> Result<Box<dyn eframe::App>, Box<dyn std::error::Error + Send + Sync>> {
    let mut fonts = egui::FontDefinitions::default();
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
    cc.egui_ctx.set_fonts(fonts);

    let render_state = cc.wgpu_render_state.clone().ok_or("WGPU uninitialized")?;

    render_state
        .device
        .on_uncaptured_error(Arc::new(|e| match e {
            wgpu::Error::Internal {
                source,
                description,
            } => {
                log::error!("wgpu internal error: {description}: {source}");
            }
            wgpu::Error::OutOfMemory { source } => {
                log::error!("wgpu out of memory: {source}");
            }
            wgpu::Error::Validation {
                source,
                description,
            } => {
                log::error!("wgpu validation error: {description}: {source}");
            }
        }));

    let app = GrafiekApp::init(Arc::new(render_state)).map_err(|e| format!("{e:?}"))?;
    Ok(Box::new(app))
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> anyhow::Result<()> {
    let _ = logging::init(log::LevelFilter::Debug).inspect_err(|e| eprintln!("{e:?}"));
    log::info!("Starting Grafiek");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 800.0]),
        renderer: eframe::Renderer::Wgpu,
        wgpu_options: wgpu_configuration(),
        ..Default::default()
    };

    eframe::run_native("Grafiek", options, Box::new(|cc| create_app(cc)))
        .map_err(|e| anyhow::anyhow!("{e:?}"))
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    console_error_panic_hook::set_once();
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();
    log::info!("Starting Grafiek (web)");

    wasm_bindgen_futures::spawn_local(async {
        let web_options = eframe::WebOptions {
            wgpu_options: wgpu_configuration(),
            ..Default::default()
        };

        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("grafiek_canvas")
            .expect("No canvas element with id 'grafiek_canvas'")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("Element is not a canvas");

        eframe::WebRunner::new()
            .start(canvas, web_options, Box::new(|cc| create_app(cc)))
            .await
            .expect("Failed to start eframe");
    });
}
