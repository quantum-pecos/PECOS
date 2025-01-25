// PECOS/crates/pecos-engines/src/qir/command_queue.rs
use crate::types::QuantumCommand;
use std::collections::VecDeque;

pub struct CommandQueue {
    queue: VecDeque<QuantumCommand>,
}

impl Default for CommandQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandQueue {
    #[must_use]
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    pub fn push(&mut self, cmd: QuantumCommand) {
        self.queue.push_back(cmd);
    }

    pub fn pop(&mut self) -> Option<QuantumCommand> {
        self.queue.pop_front()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.queue.len()
    }
}
