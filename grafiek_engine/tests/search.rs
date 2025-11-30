use grafiek_engine::Engine;

#[test]
fn find_category() {
    let engine = Engine::init().unwrap();
    assert!(engine.node_categories().any(|c| c == "core"));
}

#[test]
fn find_operator() {
    let engine = Engine::init().unwrap();
    assert!(engine.iter_category("core").any(|o| o == "input"));
}
