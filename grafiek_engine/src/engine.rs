use std::collections::HashMap;

use crate::error::Error;
use crate::history::{History, Message, Mutation};
use crate::node::{Node, NodeId};
use crate::ops;
use crate::traits::{OperationFactory, OperationFactoryEntry};
use petgraph::prelude::*;
use wgpu::{Device, Queue};

#[derive(Debug, Clone)]
pub struct ExecutionContext {
    // at some point we should pass a command buffer as
    // well. The context should encourage users to execute asynchronously.>
    device: Device,
    queue: Queue,
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub source_slot: usize,
    pub sink_slot: usize,
}

type OpRegistry = HashMap<&'static str, HashMap<&'static str, OperationFactoryEntry>>;
type MessageHandler = Box<dyn FnMut(Message) + Send>;

/// Descriptor for initializing the engine
pub struct EngineDescriptor {
    pub device: Device,
    pub queue: Queue,
    pub on_message: Option<MessageHandler>,
}

/// The main entry point into the library - contains all of the book keeping
/// reflection and validation logic for maintaining a grafiek map including
/// helpers for implmenting a frontend.
pub struct Engine {
    // The underlying graph model
    graph: StableDiGraph<Node, Edge>,
    // Searchable list of operator factories
    registry: OpRegistry,
    // Passed to each operator on run
    exe_ctx: ExecutionContext,
    // Undo/redo history
    history: History,
    // Optional message handler for UI sync
    on_message: Option<MessageHandler>,
    // The last issued NodeId
    last_id: NodeId,
}

// initialization related functions
impl Engine {
    pub fn init(desc: EngineDescriptor) -> Result<Self, Error> {
        let mut out = Self {
            graph: StableDiGraph::default(),
            registry: OpRegistry::default(),
            history: History::default(),
            exe_ctx: ExecutionContext {
                device: desc.device,
                queue: desc.queue,
            },
            on_message: desc.on_message,
            last_id: NodeId(0),
        };

        log::info!("loading grafiek::core operators");
        out.register_op::<ops::Input>()?;
        Ok(out)
    }

    /// Emit a message to the handler
    fn emit<T: Into<Message>>(&mut self, message: T) {
        let message = message.into();
        // Record mutations to history
        if let Message::Mutation(ref m) = message {
            self.history.push(m.clone());
        }
        if let Some(ref mut handler) = self.on_message {
            handler(message);
        }
    }

    pub fn register_op<T: OperationFactory>(&mut self) -> Result<(), Error> {
        let lib = self.registry.entry(T::LIBRARY).or_default();
        if lib.contains_key(T::OPERATOR) {
            return Err(Error::DuplicateOperationType(T::LIBRARY, T::OPERATOR));
        }
        lib.insert(T::OPERATOR, OperationFactoryEntry::new::<T>());
        Ok(())
    }

    pub fn add_node(&mut self, node: Box<dyn Operation>) -> Result<NodeIndex, Error> {
        let data = Node::new(node, self.next_id());
        let index = self.graph.add_node(data);

        let node = self
            .graph
            .node_weight_mut(index)
            .expect("Insertion and immediate retrieval into graph failed.");

        node.setup();
        node.reconfigure();

        Ok(index)
    }

    pub fn instance_node(&mut self, library: &str, operator: &str) -> Result<NodeIndex, Error> {
        let entry = self
            .registry
            .get(library)
            .and_then(|lib| lib.get(operator))
            .ok_or_else(|| Error::UnknownOperationType(format!("{}/{}", library, operator)))?;

        let operation = (entry.build)()?;
        self.add_node(operation)
    }
}

// Runtime
impl Engine {
    /// Edit a graph input (InputOp output value) with automatic mutation tracking
    pub fn edit_graph_input<F>(&mut self, index: NodeIndex, f: F) -> Result<(), Error>
    where
        F: FnOnce(ValueMut),
    {
        let node = self
            .graph
            .node_weight_mut(index)
            .ok_or(|| Error::NodeNotFound(format!("Node not found at index - {index:?}")))?;

        if node.op_type() != ops::InputOp::TYPE_NAME {
            return Err(e);
        }

        node.override_output(index, 0, |_info, value| f(value));
        Ok(())
    }
}

// Private book keeping stuff
impl Engine {
    pub fn next_id(&mut self) -> NodeId {
        self.last_id.0 += 1;
        self.last_id.clone()
    }
}

// Node Discover related functions
impl Engine {
    pub fn node_categories(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.registry.keys().copied()
    }

    pub fn iter_category(&self, category: &str) -> impl Iterator<Item = &'static str> + '_ {
        self.registry
            .get(category)
            .into_iter()
            .flat_map(|m| m.keys().copied())
    }
}

// History / undo-redo related functions
impl Engine {
    /// Undo the last mutation
    pub fn undo(&mut self) -> Result<(), Error> {
        if let Some(mutation) = self.history.undo() {
            self.apply_mutation(mutation)?;
        }
        Ok(())
    }

    /// Redo the last undone mutation
    pub fn redo(&mut self) -> Result<(), Error> {
        if let Some(mutation) = self.history.redo() {
            self.apply_mutation(mutation)?;
        }
        Ok(())
    }

    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    /// Apply a mutation to the graph (used by undo/redo)
    fn apply_mutation(&mut self, mutation: Mutation) -> Result<(), Error> {
        match mutation {
            Mutation::CreateNode { .. } => {
                todo!("recreate node from record")
            }
            Mutation::DeleteNode { .. } => {
                todo!("delete node")
            }
            Mutation::Connect { .. } => {
                todo!("connect nodes")
            }
            Mutation::Disconnect { .. } => {
                todo!("disconnect nodes")
            }
            Mutation::SetConfig { .. } => {
                todo!("set config value")
            }
            Mutation::SetInput { .. } => {
                todo!("set input value")
            }
            Mutation::MoveNode { .. } => {
                todo!("move node")
            }
            Mutation::SetLabel { .. } => {
                todo!("set label")
            }
        }
    }
}
