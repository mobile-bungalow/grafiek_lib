mod common;

use grafiek_engine::ops::{ArithOp, Arithmetic, Input, Output};
use grafiek_engine::{Value, ValueMut};

#[test]
fn init() {
    common::engine();
}

#[test]
fn spawn_from_box() {
    let mut engine = common::engine();
    engine.add_node(Box::new(Input)).unwrap();
}

#[test]
fn spawn_from_path() {
    let mut engine = common::engine();
    engine.instance_node("core", "input").unwrap();
}

#[test]
fn add_with_graph_inputs() {
    let mut engine = common::engine();

    let input_a = engine.add_node(Box::new(Input)).unwrap();
    let input_b = engine.add_node(Box::new(Input)).unwrap();
    let add = engine
        .add_node(Box::new(Arithmetic {
            operation: ArithOp::Add,
        }))
        .unwrap();
    let output = engine.add_node(Box::new(Output)).unwrap();

    engine.connect(input_a, add, 0, 0).unwrap();
    engine.connect(input_b, add, 0, 1).unwrap();
    engine.connect(add, output, 0, 0).unwrap();

    assert_eq!(engine.node_count(), 4);
    assert_eq!(engine.edge_count(), 3);

    engine
        .edit_graph_input(input_a, |_, value| {
            if let ValueMut::F32(v) = value {
                *v = 3.0;
            }
        })
        .unwrap();

    engine
        .edit_graph_input(input_b, |_, value| {
            if let ValueMut::F32(v) = value {
                *v = 4.0;
            }
        })
        .unwrap();

    engine.execute();

    let add_node = engine.get_node(add).unwrap();
    assert_eq!(add_node.input_count(), 2);
    assert_eq!(add_node.output_count(), 1);

    match engine.result(0) {
        Some(Value::F32(v)) => assert_eq!(*v, 7.0),
        _ => panic!("expected F32"),
    }
}

#[test]
fn add_with_node_inputs() {
    let mut engine = common::engine();

    let add = engine
        .add_node(Box::new(Arithmetic {
            operation: ArithOp::Add,
        }))
        .unwrap();
    let output = engine.add_node(Box::new(Output)).unwrap();

    engine.connect(add, output, 0, 0).unwrap();

    engine
        .edit_node_input(add, 0, |_, value| {
            if let ValueMut::F32(v) = value {
                *v = 3.0;
            }
        })
        .unwrap();

    engine
        .edit_node_input(add, 1, |_, value| {
            if let ValueMut::F32(v) = value {
                *v = 4.0;
            }
        })
        .unwrap();

    engine.execute();

    let output_node = engine.get_node(output).unwrap();
    match output_node.input_value(0) {
        Some(Value::F32(v)) => assert_eq!(*v, 7.0),
        _ => panic!("expected F32"),
    }
}
