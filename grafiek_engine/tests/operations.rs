mod common;

use grafiek_engine::Engine;

#[test]
fn init() {
    let (device, queue) = common::setup_wgpu();
    Engine::init(device, queue).unwrap();
}
