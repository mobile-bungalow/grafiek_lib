use std::collections::HashMap;

use crate::error::Error;
use crate::gpu_pool::{GPUResourcePool, create_gpu_texture_empty};
use crate::history::{Event, History, Message, Mutation};
use crate::node::{ConnectionProbe, Node, NodeId};
use crate::ops::{self, Input, Output};
use crate::registry::consts::{CHECK, CHECK_DATA, FLECK, SPECK, TRANSPARENT_SPECK};
use crate::traits::{Operation, OperationFactory, OperationFactoryEntry};
use crate::value::TextureHandle;
use crate::{SlotDef, Value, ValueMut};
use petgraph::prelude::*;
use petgraph::visit::Topo;
use wgpu::{Device, Queue, Texture};

#[derive(Debug)]
pub struct ExecutionContext {
    pub device: Device,
    pub queue: Queue,
    textures: GPUResourcePool,
}

impl ExecutionContext {
    pub fn texture(&self, handle: &TextureHandle) -> Option<&Texture> {
        self.textures.get_texture(handle.id?)
    }

    /// Ensure the texture exists with the correct dimensions, replacing in-place if needed.
    /// This is intended for render targets that are about to be overwritten anyways, it zeros them.
    pub fn ensure_texture(&mut self, handle: &mut TextureHandle) {
        match handle.id {
            None => {
                handle.id = Some(self.textures.alloc_texture(&self.device, handle));
            }
            Some(id) => {
                let needs_resize = self.textures.get_texture(id).map_or(false, |tex| {
                    let size = tex.size();
                    size.width != handle.width || size.height != handle.height
                });
                if needs_resize {
                    let texture = create_gpu_texture_empty(&self.device, handle);
                    self.textures.replace_texture(id, texture);
                }
            }
        }
    }
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
    //pub default_format: TextureFormat,
    pub on_message: Option<MessageHandler>,
}

/// The main entry point into the library
pub struct Engine {
    // The underlying graph model
    graph: StableDiGraph<Node, Edge>,
    // Searchable list of operator factories
    registry: OpRegistry,
    // Context passed to operators
    ctx: ExecutionContext,
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
        let mut textures = GPUResourcePool::new();

        log::info!("loading initial textures");
        textures.insert_texture(&desc.device, &desc.queue, SPECK, &[0, 0, 0, 255]);
        textures.insert_texture(&desc.device, &desc.queue, FLECK, &[255; 4]);
        textures.insert_texture(&desc.device, &desc.queue, TRANSPARENT_SPECK, &[0; 4]);
        textures.insert_texture(&desc.device, &desc.queue, CHECK, &CHECK_DATA);

        let mut out = Self {
            graph: StableDiGraph::default(),
            registry: OpRegistry::default(),
            history: History::default(),
            ctx: ExecutionContext {
                device: desc.device,
                queue: desc.queue,
                textures,
            },
            on_message: desc.on_message,
            last_id: NodeId(0),
        };

        log::info!("loading grafiek::core operators");
        out.register_op::<ops::Input>()?;
        out.register_op::<ops::Output>()?;
        out.register_op::<ops::Arithmetic>()?;
        out.register_op::<ops::Grayscale>()?;
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
        let node = Node::new(operation, id);

        let index = self.graph.add_node(node);

        self.graph[index].setup(&mut self.ctx)?;
        self.graph[index].configure(&self.ctx)?;
        self.sync_output_textures(index, &[]);

        let record = self.graph[index].record().clone();
        self.emit(Mutation::CreateNode { idx: index, record });

        Ok(index)
    }
    /// Delete a node
    ///
    /// emits [Mutation::DeleteNode]
    pub fn delete_node(&mut self, index: NodeIndex) -> Result<(), Error> {
        let edges = self.graph.edges(index);

        let edges: Vec<_> = edges
            .filter_map(|edge| {
                let (from, to) = self.graph.edge_endpoints(edge.id())?;
                let weight = edge.weight();
                Some((from, to, weight.sink_slot, weight.source_slot))
            })
            .collect();

        for (from, to, sink, source) in edges {
            self.disconnect(from, to, sink, source)?;
        }

        self.ctx.textures.release_node_textures(index);

        let node = self.graph.remove_node(index);

        if let Some(node) = node {
            self.emit(Mutation::DeleteNode {
                idx: index,
                record: node.record().clone(),
            });
        }

        Ok(())
    }

    /// Set a node's position.
    ///
    /// Emits: [`Mutation::MoveNode`]
    pub fn set_node_position(
        &mut self,
        index: NodeIndex,
        position: (f32, f32),
    ) -> Result<(), Error> {
        let node = self
            .graph
            .node_weight_mut(index)
            .ok_or_else(|| Error::NodeNotFound(format!("Node {:?}", index)))?;

        let old_position = node.record().position;
        node.record_mut().position = position;

        self.emit(Mutation::MoveNode {
            node: index,
            old_position,
            new_position: position,
        });

        Ok(())
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

        let connected_type = self.graph[from]
            .signature()
            .output(from_slot)
            .map(|s| s.value_type)
            .unwrap_or(crate::ValueType::Any);

        // Notify the target node about the connection
        let old_outputs = self.graph[to].snapshot_outputs();
        if let Err(e) = self.graph[to].on_edge_connected(to_slot, connected_type) {
            log::error!("on_edge_connected failed: {e}");
        }
        self.sync_output_textures(to, &old_outputs);

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
        let connected_type = self.graph[from]
            .signature()
            .output(from_slot)
            .map(|s| s.value_type)
            .unwrap_or(crate::ValueType::Any);

        let edge_id = self
            .graph
            .edges_connecting(from, to)
            .find(|e| e.weight().source_slot == from_slot && e.weight().sink_slot == to_slot)
            .map(|e| e.id())
            .ok_or(Error::EdgeNotFound { from_slot, to_slot })?;

        self.graph.remove_edge(edge_id);

        self.graph[to].clear_incoming(to_slot);

        let old_outputs = self.graph[to].snapshot_outputs();
        if let Err(e) = self.graph[to].on_edge_disconnected(to_slot, connected_type) {
            log::error!("on_edge_disconnected failed: {e}");
        }
        self.sync_output_textures(to, &old_outputs);

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

    pub fn inputs(&self) -> impl Iterator<Item = NodeIndex> + '_ {
        self.graph.node_indices().filter(|&idx| {
            self.graph
                .node_weight(idx)
                .map(|n| n.operation::<crate::ops::Input>().is_some())
                .unwrap_or_default()
        })
    }

    pub fn outputs(&self) -> impl Iterator<Item = NodeIndex> + '_ {
        self.graph.node_indices().filter(|&idx| {
            self.graph
                .node_weight(idx)
                .map(|n| n.operation::<crate::ops::Output>().is_some())
                .unwrap_or_default()
        })
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

        let t = node.edit_output(0, f)?;

        if node.is_dirty() {
            self.emit(Event::GraphDirtied);
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

        let res: Result<(), _> =
            (0..node.input_count()).try_for_each(|slot| node.edit_input(slot, &mut f));

        if node.is_dirty() {
            self.emit(Event::GraphDirtied)
        }

        res.map(|_| ())
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
            self.emit(Event::GraphDirtied);
            self.reconfigure_node(index)?;
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

        let res: Result<(), _> =
            (0..node.config_count()).try_for_each(|slot| node.edit_config(slot, &mut f));

        if node.is_dirty() {
            self.emit(Event::GraphDirtied);
            self.reconfigure_node(index)?;
        }

        res
    }

    /// Try to downcast a node's operation to a concrete type.
    /// It's a bad idea to modify the interior of your Operator outside
    /// of the node lifecycle!
    pub fn operation<T: 'static>(&self, index: NodeIndex) -> Option<&T> {
        self.graph.node_weight(index)?.operation()
    }

    /// Set a node's display label.
    pub fn set_label(&mut self, index: NodeIndex, label: &str) {
        if let Some(node) = self.graph.node_weight_mut(index) {
            let new_label = if label.is_empty() {
                None
            } else {
                Some(label.to_string())
            };
            let record = node.record_mut();
            let old_label = record.label.take();
            record.label = new_label.clone();

            self.emit(Mutation::SetLabel {
                node: index,
                old_label,
                new_label,
            });
        }
    }

    /// Get graph output value by index (from OutputOp nodes).
    /// Index corresponds to the order Output nodes were added to the graph.
    pub fn result(&self, index: usize) -> Option<&Value> {
        self.output_nodes()
            .nth(index)
            .and_then(|n| n.input(0).map(|(_, v)| v))
    }

    /// Iterate over all graph output values.
    /// Returns values from all Output nodes in the order they were added.
    pub fn results(&self) -> impl Iterator<Item = &Value> {
        self.output_nodes()
            .filter_map(|n| n.input(0).map(|(_, v)| v))
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
            if let Err(e) = self.graph[node].execute(&mut self.ctx) {
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
                let value = self.graph[node]
                    .output(edge.source_slot)
                    .map(|(_, v)| v.clone());
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

        let dirties_graph = match &message {
            Message::Mutation(m) => m.dirties_graph(),
            Message::Event(_) => false,
        };

        if let Message::Mutation(ref m) = message {
            self.history.push(m.clone());
        }

        if let Some(ref mut handler) = self.on_message {
            handler(message);
        }

        // Trailt messages with GraphDirtied if they mutated state
        if dirties_graph && let Some(ref mut handler) = self.on_message {
            handler(Message::Event(Event::GraphDirtied));
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

    // TODO: actually apply the mutation
    // We don't have any keybinds working yet
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

// Validation
impl Engine {
    /// Reconfigure a node and disconnect any edges invalidated by the new signature.
    fn reconfigure_node(&mut self, index: NodeIndex) -> Result<(), Error> {
        let old_outputs = self.graph[index].snapshot_outputs();
        self.graph[index].configure(&self.ctx)?;
        self.disconnect_invalid_edges(index);
        self.sync_output_textures(index, &old_outputs);
        Ok(())
    }

    /// Check all edges connected to a node and disconnect any that are no longer valid.
    fn disconnect_invalid_edges(&mut self, index: NodeIndex) {
        let edges: Vec<_> = self
            .graph
            .edges(index)
            .map(|e| (e.id(), e.source(), e.target(), e.weight().clone()))
            .collect();

        for (edge_id, from, to, weight) in edges {
            let is_valid = self.graph[from].probe_connect(
                &self.graph[to],
                weight.source_slot,
                weight.sink_slot,
            ) == ConnectionProbe::Ok;

            if !is_valid {
                self.graph.remove_edge(edge_id);
                self.graph[to].clear_incoming(weight.sink_slot);
                self.emit(Mutation::Disconnect {
                    from_node: from,
                    from_slot: weight.source_slot,
                    to_node: to,
                    to_slot: weight.sink_slot,
                });
            }
        }
    }
}

// Textures
impl Engine {
    /// Get the GPU texture for a handle.
    pub fn get_texture(&self, handle: &TextureHandle) -> Option<&Texture> {
        self.ctx.textures.get_texture(handle.id?)
    }

    /// Upload pixel data to a texture output slot. Updates handle dimensions and allocates GPU texture.
    pub fn upload_texture(
        &mut self,
        index: NodeIndex,
        slot: usize,
        width: u32,
        height: u32,
        data: &[u8],
    ) -> Result<(), Error> {
        let node = self
            .graph
            .node_weight_mut(index)
            .ok_or(Error::NodeNotFound(format!("Node not found: {index:?}")))?;

        let outputs = node.output_values_mut();
        let output = outputs.get_mut(slot).ok_or(Error::NoOutputSlot(slot))?;

        let Value::Texture(handle) = output else {
            return Err(Error::Script("Output is not a texture".into()));
        };

        if let Some(old_id) = handle.id {
            self.ctx.textures.release_texture(old_id);
        }

        handle.width = width;
        handle.height = height;

        let id = self.ctx.textures.alloc_texture_with_data(
            &self.ctx.device,
            &self.ctx.queue,
            index,
            handle,
            data,
        );

        handle.id = Some(id);

        self.emit(Event::GraphDirtied);
        Ok(())
    }

    /// Sync texture allocations after configure. Preserves IDs where possible.
    fn sync_output_textures(&mut self, index: NodeIndex, old_outputs: &[Value]) {
        let new_len = self.graph[index].output_values_mut().len();

        for (slot, output) in self.graph[index].output_values_mut().iter_mut().enumerate() {
            let Value::Texture(handle) = output else {
                continue;
            };
            // Transfer ID from old handle if it exists
            if handle.id.is_none() {
                if let Some(Value::Texture(old)) = old_outputs.get(slot) {
                    handle.id = old.id;
                }
            }
            self.ctx.ensure_texture(handle);
        }

        // Release orphaned textures from removed slots
        for old in old_outputs.iter().skip(new_len) {
            if let Value::Texture(h) = old
                && let Some(id) = h.id
            {
                self.ctx.textures.release_texture(id);
            }
        }
    }
}
