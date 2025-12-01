mod common;

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
fn test_add_operation_with_graph_inputs() {
    let (device, queue) = common::setup_wgpu();
    let mut engine = Engine::init(EngineDescriptor {
        device,
        queue,
        on_message: None,
    })
    .unwrap();

    let input_a = engine.add_node(InputOp::new()).unwrap();
    let input_b = engine.add_node(InputOp::new()).unwrap();
    let arithmetic = engine.add_node(ArithmeticOp::new()).unwrap();
    let output = engine.add_node(OutputOp::new()).unwrap();

    engine.connect(input_a, arithmetic, 0, 0).unwrap();
    engine.connect(input_b, arithmetic, 0, 1).unwrap();
    engine.connect(arithmetic, output, 0, 0).unwrap();

    assert_eq!(engine.node_count(), 4);
    assert_eq!(engine.edge_count(), 3);

    engine
        .edit_graph_input(input_a, |value| {
            if let ValueMut::F32(v) = value {
                *v = 3.0;
            }
        })
        .expect("edit_input returned None");

    engine
        .edit_graph_input(input_b, |value| {
            if let ValueMut::F32(v) = value {
                *v = 4.0;
            }
        })
        .expect("edit_input returned None");

    // Changing inputs should set the exe flag high
    assert!(engine.needs_execution());
    engine.execute();

    let arithmetic_node = engine.get_node(arithmetic).unwrap();
    assert_eq!(arithmetic_node.input_count(), 2);
    assert_eq!(arithmetic_node.output_count(), 1);

    if let Value::F32(v) = engine.get_output(0) {
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

    let arithmetic = engine.add_node(ArithmeticOp::new()).unwrap();
    let output = engine.add_node(OutputOp::new()).unwrap();

    engine.connect(arithmetic, output, 0, 0).unwrap();

    let guard = engine.get_node_mut(arithmetic).unwrap();

    guard
        .edit_node_input(0, |v| {
            if let ValueMut::F32(v) = value {
                *v = 3.0;
            }
        })
        .unwrap();

    guard
        .edit_node_input(1, |v| {
            if let ValueMut::F32(v) = value {
                *v = 4.0;
            }
        })
        .unwrap();

    // Changing inputs should set the exe flag high
    assert!(engine.needs_execution());
    engine.execute();

    let output_node = engine.get_node(output).unwrap();
    let (_info, value) = output_node.input_slot(0).unwrap();
    if let Value::F32(v) = value {
        assert_eq!(*v, 7.0);
    } else {
        panic!("Expected F32 value");
    }
}
