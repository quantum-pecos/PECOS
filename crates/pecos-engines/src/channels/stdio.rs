// PECOS/crates/pecos-engines/src/channels/stdio.rs
use super::{CommandChannel, MeasurementChannel};
use crate::errors::QueueError;
use crate::types::{CommandBatch, MeasurementResult, QuantumCommand};
use log::trace;
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

    pub fn from_stdio() -> io::Result<Self> {
        Ok(Self {
            reader: Arc::new(Mutex::new(Box::new(BufReader::new(io::stdin())))),
            writer: Arc::new(Mutex::new(Box::new(BufWriter::new(io::stdout())))),
        })
    }

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

    fn flush(&mut self) -> Result<(), QueueError> {
        let mut writer = self
            .writer
            .lock()
            .map_err(|e| QueueError::LockError(format!("Failed to lock writer: {e}")))?;
        writer.flush()?;
        Ok(())
    }
}

impl MeasurementChannel for StdioChannel {
    fn receive_measurement(&mut self) -> Result<MeasurementResult, QueueError> {
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
    use crate::types::GateType::{Measure, RXY, RZ, ZZ};

    match &cmd.gate {
        RZ { theta } => format!("RZ {} {}", theta, cmd.qubits[0]),
        RXY { phi, theta } => format!("RXY {} {} {}", phi, theta, cmd.qubits[0]),
        ZZ => format!("ZZ {} {}", cmd.qubits[0], cmd.qubits[1]),
        Measure { result_id } => format!("M {} {}", cmd.qubits[0], result_id),
    }
}
