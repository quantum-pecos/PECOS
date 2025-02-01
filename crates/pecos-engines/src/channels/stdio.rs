// PECOS/crates/pecos-engines/src/channels/stdio.rs
use super::{CommandChannel, Message, MessageChannel};
use crate::errors::QueueError;
use log::trace;
use pecos_core::types::{CommandBatch, QuantumCommand};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct StdioChannel {
    reader: Arc<Mutex<Box<dyn BufRead + Send + Sync>>>,
    writer: Arc<Mutex<Box<dyn Write + Send + Sync>>>,
}

impl StdioChannel {
    #[must_use]
    pub fn new(
        reader: Box<dyn BufRead + Send + Sync>,
        writer: Box<dyn Write + Send + Sync>,
    ) -> Self {
        Self {
            reader: Arc::new(Mutex::new(reader)),
            writer: Arc::new(Mutex::new(writer)),
        }
    }

    /// Creates a new `StdioChannel` that uses the standard input (`stdin`) and output (`stdout`) as its reader and writer.
    ///
    /// This method constructs the channel by wrapping `stdin` in a `BufReader` and `stdout` in a `BufWriter`,
    /// ensuring efficient reading and writing.
    ///
    /// # Returns
    /// - On success, returns an instance of `StdioChannel`.
    /// - On failure, returns a `std::io::Error`, which might occur while initializing the I/O streams.
    ///
    /// # Errors
    /// This function returns a `std::io::Error` if:
    /// - An error occurs while accessing the standard input or output streams.
    /// - System-level I/O errors prevent the construction of the channel.
    ///
    /// # Examples
    /// ```
    /// use pecos_engines::channels::stdio::StdioChannel;
    /// let channel = StdioChannel::from_stdio().expect("Failed to create StdioChannel from stdio");
    /// ```
    pub fn from_stdio() -> io::Result<Self> {
        Ok(Self {
            reader: Arc::new(Mutex::new(Box::new(BufReader::new(io::stdin())))),
            writer: Arc::new(Mutex::new(Box::new(BufWriter::new(io::stdout())))),
        })
    }

    /// Creates a `StdioChannel` instance with an anonymous pipe for testing or short-lived communication.
    ///
    /// This method sets up a pair of connected reader and writer pipes using `os_pipe`,
    /// wrapping the reader in a `BufReader` and the writer in a `BufWriter` for buffered I/O operations.
    ///
    /// # Returns
    /// - On success, returns a fully initialized `StdioChannel`.
    /// - On failure, returns an `std::io::Error` if the pipe creation fails.
    ///
    /// # Errors
    /// This function returns a `std::io::Error` if:
    /// - The operating system fails to create the anonymous pipe.
    /// - There is an error during initialization of the reader or writer.
    ///
    /// # Examples
    /// ```
    /// use pecos_engines::channels::stdio::StdioChannel;
    /// let channel = StdioChannel::create_for_shot().expect("Failed to create channel for shot");
    /// ```
    pub fn create_for_shot() -> io::Result<Self> {
        use os_pipe::pipe;
        let (reader, writer) = pipe()?;

        Ok(Self {
            reader: Arc::new(Mutex::new(Box::new(BufReader::new(reader)))),
            writer: Arc::new(Mutex::new(Box::new(BufWriter::new(writer)))),
        })
    }
}

impl CommandChannel for StdioChannel {
    /// Sends a batch of commands through the channel.
    ///
    /// This function writes the commands to the writer, formatting them into a
    /// specific protocol. The procedure includes:
    /// - Writing "`FLUSH_BEGIN`" before the commands.
    /// - Writing each command in the form "CMD <`formatted_command`>".
    /// - Writing "`FLUSH_END`" after all commands.
    ///
    /// The function ensures that the data is flushed to the writer before returning.
    ///
    /// # Parameters
    /// - `cmds`: A batch of quantum commands to be sent.
    ///
    /// # Errors
    /// This function returns a `QueueError` if:
    /// - The writer cannot be locked.
    /// - There is an I/O error while writing the commands or flushing the writer.
    fn send_commands(&mut self, cmds: CommandBatch) -> Result<(), QueueError> {
        let mut writer = self
            .writer
            .lock()
            .map_err(|e| QueueError::LockError(format!("Failed to lock writer: {e}")))?;

        writeln!(*writer, "FLUSH_BEGIN")?;

        for cmd in cmds {
            trace!("Sending command: {:?}", cmd);
            let cmd_str = format_command(&cmd);
            writeln!(*writer, "CMD {cmd_str}")?;
        }

        writeln!(*writer, "FLUSH_END")?;
        writer.flush()?;
        Ok(())
    }

    /// Flushes any remaining data in the writer, ensuring it is written out.
    ///
    /// # Errors
    /// This function returns a `QueueError` if:
    /// - There is an error locking the writer.
    /// - The flush operation fails for any reason.
    fn flush(&mut self) -> Result<(), QueueError> {
        let mut writer = self
            .writer
            .lock()
            .map_err(|e| QueueError::LockError(format!("Failed to lock writer: {e}")))?;
        writer.flush()?;
        Ok(())
    }
}

impl MessageChannel for StdioChannel {
    /// Receives a message (measurement) from the channel.
    ///
    /// This method tries to read a line of input, parses it into a `Message` (u32),
    /// and returns the result.
    ///
    /// # Errors
    /// This function returns a `QueueError` if:
    /// - There is an error locking the reader.
    /// - The operation fails to read a line from the reader.
    /// - The parsed measurement is invalid (not a valid `u32`).
    fn receive_message(&mut self) -> Result<Message, QueueError> {
        let mut reader = self
            .reader
            .lock()
            .map_err(|e| QueueError::LockError(format!("Failed to lock reader: {e}")))?;

        let mut line = String::new();
        reader.read_line(&mut line)?;

        let measurement = line
            .trim()
            .parse()
            .map_err(|e| QueueError::OperationError(format!("Invalid measurement: {e}")))?;

        trace!("Received measurement: {}", measurement);
        Ok(measurement)
    }
}

pub(crate) fn format_command(cmd: &QuantumCommand) -> String {
    use pecos_core::types::GateType::{Measure, CX, H, R1XY, RZ, SZZ};

    match &cmd.gate {
        RZ { theta } => format!("RZ {} {}", theta, cmd.qubits[0]),
        R1XY { phi, theta } => format!("R1XY {} {} {}", phi, theta, cmd.qubits[0]),
        SZZ => format!("SZZ {} {}", cmd.qubits[0], cmd.qubits[1]),
        H => format!("H {}", cmd.qubits[0]),
        CX => format!("CX {} {}", cmd.qubits[0], cmd.qubits[1]),
        Measure { result_id } => format!("M {} {}", cmd.qubits[0], result_id),
    }
}
