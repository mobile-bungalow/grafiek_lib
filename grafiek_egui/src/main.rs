pub mod app;
pub mod components;
pub mod logging;
fn main() {
    if logging::init().is_err() {
        eprintln!("Logging failed to start.");
    }
}
