use grafiek_engine::{Engine, EngineDescriptor};
use wgpu::{self, ExperimentalFeatures};

pub fn engine() -> Engine {
    let (device, queue) = setup_wgpu();
    Engine::init(EngineDescriptor {
        device,
        queue,
        on_message: None,
    })
    .unwrap()
}

pub fn setup_wgpu() -> (wgpu::Device, wgpu::Queue) {
    let instance = if cfg!(windows) {
        wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::DX12,
            ..Default::default()
        })
    } else {
        wgpu::Instance::default()
    };

    let adapter = pollster::block_on(async {
        instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await
            .expect("Failed to find an appropriate adapter")
    });

    let mut required_limits = wgpu::Limits::default().using_resolution(adapter.limits());
    required_limits.max_push_constant_size = 128;

    let (device, queue) = pollster::block_on(async {
        adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::PUSH_CONSTANTS
                    | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
                    | wgpu::Features::CLEAR_TEXTURE,
                required_limits,
                memory_hints: wgpu::MemoryHints::Performance,
                experimental_features: ExperimentalFeatures::disabled(),
                trace: wgpu::Trace::Off,
            })
            .await
            .expect("Failed to create device")
    });

    device.on_uncaptured_error(std::sync::Arc::new(|e| match e {
        wgpu::Error::Internal {
            source,
            description,
        } => {
            panic!("wgpu internal error: {source}, {description}");
        }
        wgpu::Error::OutOfMemory { .. } => {
            panic!("Out of GPU memory");
        }
        wgpu::Error::Validation {
            source,
            description,
        } => {
            panic!("wgpu validation error: {description}: {source}");
        }
    }));

    (device, queue)
}
