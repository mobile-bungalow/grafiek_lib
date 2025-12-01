mod common;

use grafiek_engine::Engine;

#[test]
fn find_category() {
    let (device, queue) = common::setup_wgpu();
    let engine = Engine::init(device, queue).unwrap();
    assert!(engine.node_categories().any(|c| c == "core"));
}

#[test]
fn find_operator() {
    let (device, queue) = common::setup_wgpu();
    let engine = Engine::init(device, queue).unwrap();
    assert!(engine.iter_category("core").any(|o| o == "input"));
}
