// PECOS/crates/pecos-engines/src/channels/shared_memory.rs
use super::{CommandChannel, Message, MessageChannel};
use crate::errors::QueueError;
use memmap2::MmapMut;
use pecos_core::types::CommandBatch;
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
    /// Creates a new shared memory channel with a memory-mapped file.
    ///
    /// This method initializes a file at the specified `path` with the given `size`,
    /// creates a memory mapping for it, and prepares the shared memory channel for
    /// further operations.
    ///
    /// # Parameters
    /// - `path`: The file path to create/open the memory-mapped file.
    /// - `size`: The size of the memory-mapped file in bytes.
    ///
    /// # Returns
    /// On success, returns an instance of `SharedMemoryChannel` initialized
    /// with the specified file and size.
    ///
    /// # Errors
    /// This function returns a `std::io::Error` if:
    /// - The file cannot be created or opened at the specified `path`.
    /// - The file size cannot be set to `size`.
    /// - The memory mapping for the file fails due to safety or I/O issues.
    pub fn new(path: &str, size: usize) -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;

        file.set_len(size as u64)?;
        let mmap = unsafe { MmapMut::map_mut(&file)? };

        Ok(Self {
            mmap: Arc::new(parking_lot::Mutex::new(mmap)),
            write_pos: Arc::new(AtomicU64::new(0)),
            read_pos: Arc::new(AtomicU64::new(0)),
        })
    }

    /// Creates a `SharedMemoryChannel` instance with a unique file-based shared memory region.
    ///
    /// This method generates a unique file name in the `/tmp` directory using a UUID and
    /// initializes a memory-mapped file of the specified size (1 MB).
    ///
    /// # Returns
    /// - On success, returns a `SharedMemoryChannel` instance backed by a unique memory-mapped file.
    /// - On failure, returns a `std::io::Error`.
    ///
    /// # Errors
    /// This function returns a `std::io::Error` if:
    /// - The temporary file cannot be created in the `/tmp` directory.
    /// - The memory mapping for the file fails.
    /// - The system encounters an I/O error during any part of the creation process.
    pub fn create_for_shot() -> std::io::Result<Self> {
        use uuid::Uuid;
        Self::new(&format!("/tmp/qsim_{}", Uuid::new_v4()), 1024 * 1024)
    }
}

impl CommandChannel for SharedMemoryChannel {
    fn send_commands(&mut self, cmds: CommandBatch) -> Result<(), QueueError> {
        let mut mmap = self.mmap.lock();
        let mut write_pos = usize::try_from(self.write_pos.load(Ordering::SeqCst))
            .expect("Failed to convert write position to usize");

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

impl MessageChannel for SharedMemoryChannel {
    /// Receives a message (measurement) from the shared memory channel.
    ///
    /// This method reads 4 bytes from the current read position in the shared memory,
    /// interprets them as a `u32` value, updates the read position, and returns the measurement.
    ///
    /// # Errors
    /// This function returns a `QueueError` if:
    /// - The lock on the shared memory mapping fails.
    /// - The read operation attempts to access memory outside the initialized range.
    fn receive_message(&mut self) -> Result<Message, QueueError> {
        let mmap = self.mmap.lock();
        let read_pos = usize::try_from(self.read_pos.load(Ordering::SeqCst))
            .expect("Failed to convert read position to usize");

        // Read measurement from shared memory (u32 = 4 bytes)
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&mmap[read_pos..read_pos + 4]);
        let measurement = u32::from_le_bytes(buf);

        self.read_pos.fetch_add(4, Ordering::SeqCst);
        Ok(measurement)
    }
}
