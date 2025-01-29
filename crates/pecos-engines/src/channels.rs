// PECOS/crates/pecos-engines/src/channels.rs
use crate::errors::QueueError;
use pecos_core::types::CommandBatch;

pub trait CommandChannel: Send + Sync {
    /// Sends a batch of quantum commands to the channel for processing.
    ///
    /// # Parameters
    /// - `cmds`: A batch of quantum commands to be sent.
    ///
    /// # Errors
    /// This function returns a `QueueError` if:
    /// - There is an error locking the queue.
    /// - The operation fails for any reason.
    /// - An error occurs during serialization of the commands.
    fn send_commands(&mut self, cmds: CommandBatch) -> Result<(), QueueError>;
    /// Flushes any remaining commands in the channel, ensuring they are processed.
    ///
    /// # Errors
    /// This function returns a `QueueError` if:
    /// - There is an error locking the queue.
    /// - The flush operation fails for any reason.
    fn flush(&mut self) -> Result<(), QueueError>;
}

pub type Message = u32;

pub trait MessageChannel: Send + Sync {
    /// Receives a message from the channel.
    ///
    /// # Errors
    /// This function returns a `QueueError` if:
    /// - There is an error locking the queue.
    /// - The operation fails for any reason.
    /// - An error occurs during deserialization of the message.
    fn receive_message(&mut self) -> Result<Message, QueueError>;
}

pub mod stdio;

#[cfg(unix)]
pub mod shared_memory;
