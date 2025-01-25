// PECOS/crates/pecos-engines/src/channels/shared_memory.rs
use super::{CommandChannel, MeasurementChannel};
use crate::errors::QueueError;
use crate::types::{CommandBatch, MeasurementResult};
use memmap2::MmapMut;
use std::fs::OpenOptions;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Clone)]
pub struct SharedMemoryChannel {
    mmap: Arc<parking_lot::Mutex<MmapMut>>,
    write_pos: Arc<AtomicU64>,
    read_pos: Arc<AtomicU64>,
}

impl SharedMemoryChannel {
    pub fn new(path: &str, size: usize) -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        file.set_len(size as u64)?;
        let mmap = unsafe { MmapMut::map_mut(&file)? };

        Ok(Self {
            mmap: Arc::new(parking_lot::Mutex::new(mmap)),
            write_pos: Arc::new(AtomicU64::new(0)),
            read_pos: Arc::new(AtomicU64::new(0)),
        })
    }

    pub fn create_for_shot() -> std::io::Result<Self> {
        use uuid::Uuid;
        Self::new(&format!("/tmp/qsim_{}", Uuid::new_v4()), 1024 * 1024)
    }
}

impl CommandChannel for SharedMemoryChannel {
    fn send_commands(&mut self, cmds: CommandBatch) -> Result<(), QueueError> {
        let mut mmap = self.mmap.lock();
        let mut write_pos = self.write_pos.load(Ordering::SeqCst) as usize;

        // Write commands to shared memory
        for cmd in cmds {
            let cmd_str = serde_json::to_string(&cmd)?;
            let cmd_bytes = cmd_str.as_bytes();
            mmap[write_pos..write_pos + cmd_bytes.len()].copy_from_slice(cmd_bytes);
            write_pos += cmd_bytes.len();
        }

        self.write_pos.store(write_pos as u64, Ordering::SeqCst);
        Ok(())
    }

    fn flush(&mut self) -> Result<(), QueueError> {
        self.mmap.lock().flush()?;
        Ok(())
    }
}

impl MeasurementChannel for SharedMemoryChannel {
    fn receive_measurement(&mut self) -> Result<MeasurementResult, QueueError> {
        let mmap = self.mmap.lock();
        let read_pos = self.read_pos.load(Ordering::SeqCst) as usize;

        // Read measurement from shared memory (u32 = 4 bytes)
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&mmap[read_pos..read_pos + 4]);
        let measurement = u32::from_le_bytes(buf);

        self.read_pos.fetch_add(4, Ordering::SeqCst);
        Ok(measurement)
    }
}
