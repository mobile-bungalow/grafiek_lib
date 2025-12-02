use std::collections::HashMap;

use crate::error::Error;
use crate::history::{Event, History, Message, Mutation};
use crate::node::{ConnectionProbe, Node, NodeId};
use crate::ops::{self, Input, Output};
use crate::traits::{Operation, OperationFactory, OperationFactoryEntry};
use crate::{SlotDef, Value, ValueMut};
use petgraph::prelude::*;
use petgraph::visit::Topo;
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
        node.configure()?;

        let record = node.record().clone();
        let index = self.graph.add_node(node);

        self.emit(Mutation::CreateNode { idx: index, record });

        Ok(index)
    }

    /// Connect an output slot of one node to an input slot of another.
    ///
    /// If the target input already has a connection, it will be replaced
    /// and a `Disconnect` mutation will be emitted before the `Connect`.
    ///
    /// Emits: [`Mutation::Disconnect`] (if replacing), [`Mutation::Connect`]
    pub fn connect(
        &mut self,
        from: NodeIndex,
        to: NodeIndex,
        from_slot: usize,
        to_slot: usize,
    ) -> Result<(), Error> {
        // Validate nodes exist and check type compatibility
        let from_node = self
            .graph
            .node_weight(from)
            .ok_or_else(|| Error::NodeNotFound(format!("Source node {:?}", from)))?;

        let to_node = self
            .graph
            .node_weight(to)
            .ok_or_else(|| Error::NodeNotFound(format!("Target node {:?}", to)))?;

        match from_node.probe_connect(to_node, from_slot, to_slot) {
            ConnectionProbe::Ok => {}
            ConnectionProbe::NoSourceSlot => {
                return Err(Error::NoOutputSlot(from_slot));
            }
            ConnectionProbe::NoSinkSlot => {
                return Err(Error::NoInputSlot(to_slot));
            }
            ConnectionProbe::Incompatible => {
                return Err(Error::IncompatibleTypes { from_slot, to_slot });
            }
            ConnectionProbe::CreatesLoop => {
                return Err(Error::CreatesLoop);
            }
        }

        // Check if connection would create a cycle (is there a path from `to` to `from`?)
        if petgraph::algo::has_path_connecting(&self.graph, to, from, None) {
            return Err(Error::CreatesLoop);
        }

        // Check for existing edge on target input slot and remove if present
        let existing_edge = self
            .graph
            .edges_directed(to, Direction::Incoming)
            .find(|e| e.weight().sink_slot == to_slot);

        if let Some(edge) = existing_edge {
            let old_from = edge.source();
            let old_from_slot = edge.weight().source_slot;
            let edge_id = edge.id();

            self.graph.remove_edge(edge_id);

            self.emit(Mutation::Disconnect {
                from_node: old_from,
                from_slot: old_from_slot,
                to_node: to,
                to_slot,
            });
        }

        // Add the new edge
        self.graph.add_edge(
            from,
            to,
            Edge {
                source_slot: from_slot,
                sink_slot: to_slot,
            },
        );

        self.emit(Mutation::Connect {
            from_node: from,
            from_slot,
            to_node: to,
            to_slot,
        });

        Ok(())
    }

    /// Disconnect an edge between two nodes.
    ///
    /// Emits: [`Mutation::Disconnect`]
    pub fn disconnect(
        &mut self,
        from: NodeIndex,
        to: NodeIndex,
        from_slot: usize,
        to_slot: usize,
    ) -> Result<(), Error> {
        // Find and remove the edge
        let edge_id = self
            .graph
            .edges_connecting(from, to)
            .find(|e| e.weight().source_slot == from_slot && e.weight().sink_slot == to_slot)
            .map(|e| e.id())
            .ok_or_else(|| Error::EdgeNotFound { from_slot, to_slot })?;

        self.graph.remove_edge(edge_id);

        // Clear the incoming value on the target node
        self.graph[to].clear_incoming(to_slot);

        self.emit(Mutation::Disconnect {
            from_node: from,
            from_slot,
            to_node: to,
            to_slot,
        });

        Ok(())
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

    /// Edit all inputs on a node.
    pub fn edit_all_node_inputs<F>(&mut self, index: NodeIndex, mut f: F) -> Result<(), Error>
    where
        F: FnMut(&SlotDef, ValueMut),
    {
        let node = self
            .graph
            .node_weight_mut(index)
            .ok_or(Error::NodeNotFound(format!("Node not found: {index:?}")))?;

        let res: Result<(), _> = (0..node.input_count())
            .map(|slot| node.edit_input(slot, &mut f))
            .collect();

        if node.is_dirty() {
            self.emit(Event::GraphDirtied)
        }

        res.and_then(|_| Ok(()))
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

        let t = node.edit_input(slot, f)?;

        if node.is_dirty() {
            self.emit(Event::GraphDirtied)
        }

        Ok(t)
    }

    /// Edit a node's config slot directly.
    pub fn edit_node_config<F, T>(
        &mut self,
        index: NodeIndex,
        slot: usize,
        f: F,
    ) -> Result<T, Error>
    where
        F: FnOnce(&SlotDef, ValueMut) -> T,
    {
        let node = self
            .graph
            .node_weight_mut(index)
            .ok_or(Error::NodeNotFound(format!("Node not found: {index:?}")))?;

        let t = node.edit_config(slot, f)?;

        if node.is_dirty() {
            self.emit(Event::GraphDirtied)
        }

        Ok(t)
    }

    /// Edit all config slots on a node.
    pub fn edit_all_node_configs<F>(&mut self, index: NodeIndex, mut f: F) -> Result<(), Error>
    where
        F: FnMut(&SlotDef, ValueMut),
    {
        let node = self
            .graph
            .node_weight_mut(index)
            .ok_or(Error::NodeNotFound(format!("Node not found: {index:?}")))?;

        let res: Result<(), _> = (0..node.config_count())
            .map(|slot| node.edit_config(slot, &mut f))
            .collect();

        if node.is_dirty() {
            self.emit(Event::GraphDirtied)
        }

        res.and_then(|_| Ok(()))
    }

    /// Try to downcast a node's operation to a concrete type.
    /// Returns None if the node doesn't exist or the operation is not of type T.
    /// This is read-only and will not dirty the node.
    pub fn operation<T: 'static>(&self, index: NodeIndex) -> Option<&T> {
        self.graph.node_weight(index)?.operation()
    }

    /// Get graph output value by index (from OutputOp nodes).
    /// Index corresponds to the order Output nodes were added to the graph.
    pub fn result(&self, index: usize) -> Option<&Value> {
        self.output_nodes()
            .nth(index)
            .and_then(|n| n.input_value(0))
    }

    /// Iterate over all graph output values.
    /// Returns values from all Output nodes in the order they were added.
    pub fn results(&self) -> impl Iterator<Item = &Value> {
        self.output_nodes().filter_map(|n| n.input_value(0))
    }

    /// Iterate over all Output nodes in the graph.
    fn output_nodes(&self) -> impl Iterator<Item = &Node> {
        self.graph.node_weights().filter(|node| {
            let rec = node.record();
            rec.op_path.library == Output::LIBRARY && rec.op_path.operator == Output::OPERATOR
        })
    }

    /// Execute the graph in topological order.
    /// Each node's outputs are pushed to downstream nodes before they execute.
    pub fn execute(&mut self) {
        self.emit(Event::ExecutionStarted);

        let mut topo = Topo::new(&self.graph);
        while let Some(node) = topo.next(&self.graph) {
            if let Err(e) = self.graph[node].execute(&mut self.exe_ctx) {
                // TODO: emit error state here
                log::error!("Node execution failed: {e}");
            }

            self.emit(Event::NodeExecuted { node });

            let mut dependants = self
                .graph
                .neighbors_directed(node, Direction::Outgoing)
                .detach();

            while let Some((edge, dep)) = dependants.next(&self.graph) {
                let edge = self.graph[edge].clone();
                let value = self.graph[node].output_value(edge.source_slot).cloned();
                if let Some(value) = value {
                    self.graph[dep].push_incoming(edge.sink_slot, value);
                }
            }
        }

        self.emit(Event::ExecutionCompleted);
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
