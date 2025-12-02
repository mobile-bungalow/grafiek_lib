mod common;

#[test]
fn find_category() {
    let engine = common::engine();
    assert!(engine.node_categories().any(|c| c == "core"));
}

#[test]
fn find_operator() {
    let engine = common::engine();
    assert!(engine.iter_category("core").any(|o| o == "input"));
}
