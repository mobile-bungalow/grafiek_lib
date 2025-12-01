use std::collections::HashMap;

use crate::error::Error;
use crate::history::{Event, History, Message, Mutation};
use crate::node::{Node, NodeId};
use crate::ops::{self, Input};
use crate::traits::{Operation, OperationFactory, OperationFactoryEntry};
use crate::{SlotDef, Value, ValueMut};
use petgraph::prelude::*;
use wgpu::{Device, Queue};

#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub device: Device,
    pub queue: Queue,
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

/// The main entry point into the library
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

// Initialization
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
        out.register_op::<ops::Output>()?;
        out.register_op::<ops::Add>()?;
        Ok(out)
    }

    pub fn register_op<T: OperationFactory>(&mut self) -> Result<(), Error> {
        let lib = self.registry.entry(T::LIBRARY).or_default();
        if lib.contains_key(T::OPERATOR) {
            return Err(Error::DuplicateOperationType(T::LIBRARY, T::OPERATOR));
        }
        lib.insert(T::OPERATOR, OperationFactoryEntry::new::<T>());
        Ok(())
    }

    fn next_id(&mut self) -> NodeId {
        self.last_id.0 += 1;
        self.last_id.clone()
    }
}

// Graph construction
impl Engine {
    /// Create a new node that was registered with [Engine::register_op], this includes all
    /// of the system operators that you have enabled.
    ///
    /// emits [Mutation::CreateNode]
    pub fn instance_node(&mut self, library: &str, name: &str) -> Result<NodeIndex, Error> {
        let factory = self
            .registry
            .get(library)
            .and_then(|m| m.get(name))
            .ok_or(Error::UnknownOperationType(format!("{library}/{name}")))?;

        let op = (factory.build)()?;

        self.add_node(op)
    }

    /// Create a new node directly from a trait object.
    ///
    /// emits [Mutation::CreateNode]
    pub fn add_node(&mut self, operation: Box<dyn Operation>) -> Result<NodeIndex, Error> {
        let id = self.next_id();
        let mut node = Node::new(operation, id);

        node.setup(&mut self.exe_ctx);
        node.configure(&mut self.exe_ctx)?;

        let record = node.record().clone();
        let index = self.graph.add_node(node);

        self.emit(Mutation::CreateNode { idx: index, record });

        Ok(index)
    }

    pub fn connect(
        &mut self,
        from: NodeIndex,
        to: NodeIndex,
        from_slot: usize,
        to_slot: usize,
    ) -> Result<(), Error> {
        todo!("connect")
    }

    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    pub fn get_node(&self, index: NodeIndex) -> Option<&Node> {
        self.graph.node_weight(index)
    }
}

// Value editing
impl Engine {
    /// Edit a graph input (InputOp output value)
    pub fn edit_graph_input<F, T>(&mut self, index: NodeIndex, f: F) -> Result<T, Error>
    where
        F: FnOnce(&SlotDef, ValueMut) -> T,
    {
        let node = self
            .graph
            .node_weight_mut(index)
            .ok_or(Error::NodeNotFound(format!("Node not found: {index:?}")))?;

        let rec = node.record();

        if !(rec.op_path.library == Input::LIBRARY && rec.op_path.operator == Input::OPERATOR) {
            return Err(Error::NotInputNode);
        }

        let t = node.edit_input(0, f)?;

        if node.is_dirty() {
            self.emit(Event::GraphDirtied)
        }

        Ok(t)
    }

    /// Edit a node's input slot directly
    pub fn edit_node_input<F, T>(&mut self, index: NodeIndex, slot: usize, f: F) -> Result<T, Error>
    where
        F: FnOnce(&SlotDef, ValueMut) -> T,
    {
        let node = self
            .graph
            .node_weight_mut(index)
            .ok_or(Error::NodeNotFound(format!("Node not found: {index:?}")))?;

        let rec = node.record();

        let t = node.edit_input(slot, f)?;

        if node.is_dirty() {
            self.emit(Event::GraphDirtied)
        }

        Ok(t)
    }

    /// Get graph output value (from OutputOp nodes)
    pub fn get_graph_output(&self, index: usize) -> Option<&Value> {
        todo!("get_graph_output")
    }
}

// Execution
impl Engine {
    pub fn execute(&mut self) {
        todo!("execute")
    }
}

// History
impl Engine {
    fn emit<T: Into<Message>>(&mut self, message: T) {
        let message = message.into();
        if let Message::Mutation(ref m) = message {
            self.history.push(m.clone());
        }
        if let Some(ref mut handler) = self.on_message {
            handler(message);
        }
    }

    pub fn undo(&mut self) -> Result<(), Error> {
        if let Some(mutation) = self.history.undo() {
            self.apply_mutation(mutation)?;
        }
        Ok(())
    }

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

    fn apply_mutation(&mut self, mutation: Mutation) -> Result<(), Error> {
        match mutation {
            Mutation::CreateNode { .. } => todo!(),
            Mutation::DeleteNode { .. } => todo!(),
            Mutation::Connect { .. } => todo!(),
            Mutation::Disconnect { .. } => todo!(),
            Mutation::SetConfig { .. } => todo!(),
            Mutation::SetInput { .. } => todo!(),
            Mutation::MoveNode { .. } => todo!(),
            Mutation::SetLabel { .. } => todo!(),
        }
    }
}

// Discovery
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
