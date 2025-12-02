mod common;

use grafiek_engine::ops::{Add, Input, Output};
use grafiek_engine::{Engine, EngineDescriptor, Value, ValueMut};

#[test]
fn init() {
    let (device, queue) = common::setup_wgpu();
    Engine::init(EngineDescriptor {
        device,
        queue,
        on_message: None,
    })
    .unwrap();
}

#[test]
fn spawn_from_box() {
    let (device, queue) = common::setup_wgpu();
    let mut engine = Engine::init(EngineDescriptor {
        device,
        queue,
        on_message: None,
    })
    .unwrap();

    let _input_a = engine.add_node(Box::new(Input)).unwrap();
}

#[test]
fn spawn_from_path() {
    let (device, queue) = common::setup_wgpu();
    let mut engine = Engine::init(EngineDescriptor {
        device,
        queue,
        on_message: None,
    })
    .unwrap();

    let _input_a = engine.instance_node("core", "input").unwrap();
}

#[test]
fn test_add_operation_with_graph_inputs() {
    let (device, queue) = common::setup_wgpu();
    let mut engine = Engine::init(EngineDescriptor {
        device,
        queue,
        on_message: None,
    })
    .unwrap();

    let input_a = engine.add_node(Box::new(Input)).unwrap();
    let input_b = engine.add_node(Box::new(Input)).unwrap();
    let add = engine.add_node(Box::new(Add)).unwrap();
    let output = engine.add_node(Box::new(Output)).unwrap();

    engine.connect(input_a, add, 0, 0).unwrap();
    engine.connect(input_b, add, 0, 1).unwrap();
    engine.connect(add, output, 0, 0).unwrap();

    assert_eq!(engine.node_count(), 4);
    assert_eq!(engine.edge_count(), 3);

    engine
        .edit_graph_input(input_a, |_meta, value| {
            if let ValueMut::F32(v) = value {
                *v = 3.0;
            }
        })
        .unwrap();

    engine
        .edit_graph_input(input_b, |_meta, value| {
            if let ValueMut::F32(v) = value {
                *v = 4.0;
            }
        })
        .unwrap();

    engine.execute();

    let add_node = engine.get_node(add).unwrap();
    assert_eq!(add_node.input_count(), 2);
    assert_eq!(add_node.output_count(), 1);

    if let Some(Value::F32(v)) = engine.result(0) {
        assert_eq!(*v, 7.0);
    } else {
        panic!("Expected F32 value");
    }
}

#[test]
fn test_add_operation_with_node_inputs() {
    let (device, queue) = common::setup_wgpu();
    let mut engine = Engine::init(EngineDescriptor {
        device,
        queue,
        on_message: None,
    })
    .unwrap();

    let add = engine.add_node(Box::new(Add)).unwrap();
    let output = engine.add_node(Box::new(Output)).unwrap();

    engine.connect(add, output, 0, 0).unwrap();

    engine
        .edit_node_input(add, 0, |_meta, value| {
            if let ValueMut::F32(v) = value {
                *v = 3.0;
            }
        })
        .unwrap();

    engine
        .edit_node_input(add, 1, |_meta, value| {
            if let ValueMut::F32(v) = value {
                *v = 4.0;
            }
        })
        .unwrap();

    engine.execute();

    let output_node = engine.get_node(output).unwrap();
    let value = output_node.input_value(0).unwrap();
    if let Value::F32(v) = value {
        assert_eq!(*v, 7.0);
    } else {
        panic!("Expected F32 value");
    }
}
