mod common;

use std::sync::mpsc::{self, Receiver, Sender};

use grafiek_engine::history::{Event, Message};
use grafiek_engine::ops::Input;
use grafiek_engine::{Engine, EngineDescriptor, ValueMut};

struct TestMessages {
    rx: Receiver<Message>,
}

impl TestMessages {
    fn new() -> (Self, Sender<Message>) {
        let (tx, rx) = mpsc::channel();
        (Self { rx }, tx)
    }

    fn drain(&self) -> Vec<Message> {
        self.rx.try_iter().collect()
    }

    fn clear(&self) {
        while self.rx.try_recv().is_ok() {}
    }
}

#[test]
fn input_node_emits_dirty_on_edit() {
    let (device, queue) = common::setup_wgpu();
    let (messages, tx) = TestMessages::new();

    let mut engine = Engine::init(EngineDescriptor {
        device,
        queue,
        on_message: Some(Box::new(move |msg| {
            tx.send(msg).unwrap();
        })),
    })
    .unwrap();

    let input = engine.add_node(Box::new(Input)).unwrap();
    messages.clear();

    engine
        .edit_graph_input(input, |_slot, value| {
            if let ValueMut::F32(v) = value {
                *v = 42.0;
            }
        })
        .unwrap();

    let msgs = messages.drain();
    assert_eq!(msgs.len(), 1);
    assert!(matches!(msgs[0], Message::Event(Event::GraphDirtied)));
}

#[test]
fn input_node_no_dirty_when_value_unchanged() {
    let (device, queue) = common::setup_wgpu();
    let (messages, tx) = TestMessages::new();

    let mut engine = Engine::init(EngineDescriptor {
        device,
        queue,
        on_message: Some(Box::new(move |msg| {
            tx.send(msg).unwrap();
        })),
    })
    .unwrap();

    let input = engine.add_node(Box::new(Input)).unwrap();
    messages.clear();

    // Edit but don't change the value (default is 0.0, set to 0.0)
    engine
        .edit_graph_input(input, |_slot, value| {
            if let ValueMut::F32(v) = value {
                *v = 0.0;
            }
        })
        .unwrap();

    let msgs = messages.drain();
    assert!(msgs.is_empty(), "Expected no messages, got {:?}", msgs);
}
