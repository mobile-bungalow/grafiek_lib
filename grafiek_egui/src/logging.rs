#[cfg(not(target_arch = "wasm32"))]
use anyhow::Result;
use log::Log;

pub struct CombineLogger<L1, L2>(pub L1, pub L2);

impl<L1: Log, L2: Log> Log for CombineLogger<L1, L2> {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        self.0.enabled(metadata) || self.1.enabled(metadata)
    }

    fn log(&self, record: &log::Record<'_>) {
        self.0.log(record);
        self.1.log(record);
    }

    fn flush(&self) {
        self.0.flush();
        self.1.flush();
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn init(level: log::LevelFilter) -> Result<()> {
    let env_log = env_logger::Builder::from_default_env()
        .filter_level(level)
        .filter_module("naga", log::LevelFilter::Warn)
        .filter_module("wgpu", log::LevelFilter::Warn)
        .filter_module("wgpu_hal", log::LevelFilter::Warn)
        .format(|buf, record| {
            use std::io::Write;
            writeln!(
                buf,
                "[{} {}:{}] {}",
                record.level(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            )
        })
        .build();

    let egui = egui_logger::Builder::default().max_level(level).build();

    log::set_max_level(level);
    log::set_boxed_logger(Box::new(CombineLogger(env_log, egui)))?;

    Ok(())
}
