/// RAII Guard for tracking user actions
/// this tracks UI actions such as
/// Selecting, deselecting, deleting nodes
/// creating nodes, connecting edges, disconnecting edges
/// moving nodes, loading files.

#[derive(Debug, Clone)]
pub struct ActionQueue {}

pub struct ActionGuard {}

impl ActionGuard {
    fn submit(&mut self) {}
}

impl Drop for ActionGuard {
    fn Drop(mut self) {
        self.submit();
    }
}

impl ActionQueue {
    pub fn new() -> Self {
        Self {}
    }

    pub fn start_tx(&mut self) -> ActionGuard {
        ActionGuard {}
    }

    pub fn redo(&mut self) {}

    pub fn undo(&mut self) {}
}
