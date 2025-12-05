use petgraph::prelude::NodeIndex;

use crate::Value;
use crate::node::NodeRecord;

pub type SlotIndex = usize;

/// Events emitted by the engine. Mutations are undoable, Events are informational.
#[derive(Debug, Clone)]
pub enum Message {
    /// A mutation that changes the graph state (undoable)
    Mutation(Mutation),
    /// An informational event (not undoable)
    Event(Event),
}

impl From<Mutation> for Message {
    fn from(m: Mutation) -> Self {
        Message::Mutation(m)
    }
}

impl From<Event> for Message {
    fn from(e: Event) -> Self {
        Message::Event(e)
    }
}

/// Informational events that don't affect undo/redo
#[derive(Debug, Clone)]
pub enum Event {
    /// Graph validation produced errors
    ErrorsChanged { errors: Vec<GraphError> },
    /// Errors were cleared
    ErrorsCleared,
    /// Execution started
    ExecutionStarted,
    /// Execution completed
    ExecutionCompleted,
    /// A node was executed
    NodeExecuted { node: NodeIndex },
    /// Graph was marked dirty (needs re-execution)
    GraphDirtied,
}

/// A graph validation or execution error
#[derive(Debug, Clone)]
pub struct GraphError {
    pub node: Option<NodeIndex>,
    pub message: String,
}

/// A mutation that can be applied to the graph, stored for undo/redo
#[derive(Debug, Clone)]
pub enum Mutation {
    /// Node was created
    CreateNode { idx: NodeIndex, record: NodeRecord },
    /// Node was deleted
    DeleteNode { idx: NodeIndex, record: NodeRecord },
    /// Edge was connected
    Connect {
        from_node: NodeIndex,
        from_slot: SlotIndex,
        to_node: NodeIndex,
        to_slot: SlotIndex,
    },
    /// Edge was disconnected
    Disconnect {
        from_node: NodeIndex,
        from_slot: SlotIndex,
        to_node: NodeIndex,
        to_slot: SlotIndex,
    },
    /// Config value changed
    SetConfig {
        node: NodeIndex,
        slot: SlotIndex,
        old_value: Value,
        new_value: Value,
    },
    /// Input value changed (for unconnected inputs)
    SetInput {
        node: NodeIndex,
        slot: SlotIndex,
        old_value: Value,
        new_value: Value,
    },
    /// Node position changed
    MoveNode {
        node: NodeIndex,
        old_position: (f32, f32),
        new_position: (f32, f32),
    },
    /// Node label changed
    SetLabel {
        node: NodeIndex,
        old_label: Option<String>,
        new_label: Option<String>,
    },
}

/// Target for coalescing - identifies what a mutation operates on
#[derive(Debug, Clone, PartialEq, Eq)]
enum CoalesceTarget {
    NodeSlot {
        node: NodeIndex,
        slot: SlotIndex,
        is_config: bool,
    },
    NodePosition {
        node: NodeIndex,
    },
    None,
}

impl Mutation {
    /// Returns the coalesce target for this mutation, if coalescable
    fn coalesce_target(&self) -> CoalesceTarget {
        match self {
            Mutation::SetInput { node, slot, .. } => CoalesceTarget::NodeSlot {
                node: *node,
                slot: *slot,
                is_config: false,
            },
            Mutation::SetConfig { node, slot, .. } => CoalesceTarget::NodeSlot {
                node: *node,
                slot: *slot,
                is_config: true,
            },
            Mutation::MoveNode { node, .. } => CoalesceTarget::NodePosition { node: *node },
            _ => CoalesceTarget::None,
        }
    }

    /// Returns true if this mutation requires graph re-execution
    pub fn dirties_graph(&self) -> bool {
        match self {
            Mutation::Connect { .. }
            | Mutation::Disconnect { .. }
            | Mutation::DeleteNode { .. }
            | Mutation::SetConfig { .. }
            | Mutation::SetInput { .. } => true,

            Mutation::CreateNode { .. } | Mutation::MoveNode { .. } | Mutation::SetLabel { .. } => {
                false
            }
        }
    }

    /// Returns the inverse mutation for undo
    pub fn inverse(&self) -> Mutation {
        match self.clone() {
            Mutation::CreateNode { idx, record } => Mutation::DeleteNode { idx, record },
            Mutation::DeleteNode { idx, record } => Mutation::CreateNode { idx, record },
            Mutation::Connect {
                from_node,
                from_slot,
                to_node,
                to_slot,
            } => Mutation::Disconnect {
                from_node,
                from_slot,
                to_node,
                to_slot,
            },
            Mutation::Disconnect {
                from_node,
                from_slot,
                to_node,
                to_slot,
            } => Mutation::Connect {
                from_node,
                from_slot,
                to_node,
                to_slot,
            },
            Mutation::SetConfig {
                node,
                slot,
                old_value,
                new_value,
            } => Mutation::SetConfig {
                node,
                slot,
                old_value: new_value,
                new_value: old_value,
            },
            Mutation::SetInput {
                node,
                slot,
                old_value,
                new_value,
            } => Mutation::SetInput {
                node,
                slot,
                old_value: new_value,
                new_value: old_value,
            },
            Mutation::MoveNode {
                node,
                old_position,
                new_position,
            } => Mutation::MoveNode {
                node,
                old_position: new_position,
                new_position: old_position,
            },
            Mutation::SetLabel {
                node,
                old_label,
                new_label,
            } => Mutation::SetLabel {
                node,
                old_label: new_label,
                new_label: old_label,
            },
        }
    }
}

/// Simple undo/redo history with mutation coalescing
#[derive(Debug)]
pub struct History {
    undo_stack: Vec<Mutation>,
    redo_stack: Vec<Mutation>,
    max_size: usize,
}

impl Default for History {
    fn default() -> Self {
        Self::new(100)
    }
}

impl History {
    pub fn new(max_size: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_size,
        }
    }

    /// Record a mutation
    pub fn push(&mut self, mutation: Mutation) {
        // Coalesce continuous value changes on same slot
        if self.try_coalesce(&mutation) {
            return;
        }

        self.undo_stack.push(mutation);
        self.redo_stack.clear();
        self.trim();
    }

    /// Try to coalesce with the last mutation (for continuous value drags).
    /// Returns true if coalesced, false otherwise.
    fn try_coalesce(&mut self, mutation: &Mutation) -> bool {
        let Some(last) = self.undo_stack.last_mut() else {
            return false;
        };

        // Only coalesce coalescable mutation types
        if last.coalesce_target() == CoalesceTarget::None {
            return false;
        }

        // Check if targets match
        if last.coalesce_target() != mutation.coalesce_target() {
            return false;
        }

        match (last, mutation) {
            (
                Mutation::SetInput {
                    new_value: last_new,
                    ..
                },
                Mutation::SetInput { new_value, .. },
            ) => {
                *last_new = new_value.clone();
                true
            }
            (
                Mutation::SetConfig {
                    new_value: last_new,
                    ..
                },
                Mutation::SetConfig { new_value, .. },
            ) => {
                *last_new = new_value.clone();
                true
            }
            (
                Mutation::MoveNode {
                    new_position: last_new,
                    ..
                },
                Mutation::MoveNode { new_position, .. },
            ) => {
                *last_new = *new_position;
                true
            }
            _ => false,
        }
    }

    fn trim(&mut self) {
        while self.undo_stack.len() > self.max_size {
            self.undo_stack.remove(0);
        }
    }

    /// Undo the last mutation, returns the inverse mutation to apply
    pub fn undo(&mut self) -> Option<Mutation> {
        let mutation = self.undo_stack.pop()?;
        let inverse = mutation.inverse();
        self.redo_stack.push(mutation);
        Some(inverse)
    }

    /// Redo the last undone mutation
    pub fn redo(&mut self) -> Option<Mutation> {
        let mutation = self.redo_stack.pop()?;
        let result = mutation.clone();
        self.undo_stack.push(mutation);
        Some(result)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}
