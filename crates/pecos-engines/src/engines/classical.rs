use super::ClassicalEngine;
use crate::errors::QueueError;
use crate::types::{CommandBatch, MeasurementResult, QuantumCommand, ShotResult};
use log::{debug, info};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

pub struct QirClassicalEngine {
    program_path: PathBuf, // Full path to .ll file
    build_dir: PathBuf,    // Build directory
    program_name: String,  // Base name without extension
    llc_path: String,
    clang_path: String,
    current_results: ShotResult,
    child_process: Option<Child>, // Track the running process
}

impl QirClassicalEngine {
    #[must_use]
    pub fn new(program_path: &Path, build_dir: &Path) -> Self {
        let program_name = program_path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        Self {
            program_path: program_path.to_path_buf(),
            build_dir: build_dir.to_path_buf(),
            program_name,
            llc_path: "llc-13".to_string(),
            clang_path: "clang-13".to_string(),
            current_results: ShotResult::default(),
            child_process: None,
        }
    }

    fn find_library_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let exe_path = std::env::current_exe()?;
        let exe_dir = exe_path.parent().ok_or("Cannot get executable directory")?;

        // Development case
        if let Some(target_dir) = exe_dir.parent() {
            let lib = target_dir.join("debug/libpecos_engines.so");
            if lib.exists() {
                return Ok(target_dir.join("debug"));
            }
        }

        // Installation case
        if exe_dir.ends_with("bin") {
            let lib_dir = exe_dir.parent().unwrap().join("lib");
            return Ok(lib_dir);
        }

        Err("Could not find libpecos_engines.so".into())
    }

    pub fn compile(&self) -> Result<(), Box<dyn std::error::Error>> {
        fs::create_dir_all(&self.build_dir)?;

        let bc_path = self.build_dir.join(format!("{}.bc", self.program_name));
        let obj_path = self.build_dir.join(format!("{}.o", self.program_name));
        let exe_path = self.build_dir.join(&self.program_name);

        debug!("Input file: {}", self.program_path.display());
        debug!("Object file: {}", obj_path.display());
        debug!("Executable: {}", exe_path.display());

        let lib_dir = Self::find_library_dir()?;
        debug!("Library directory: {}", lib_dir.display());

        info!("Converting LLVM IR to bitcode...");
        let status = Command::new("llvm-as-13")
            .arg(&self.program_path)
            .arg("-o")
            .arg(&bc_path)
            .status()?;

        if !status.success() {
            return Err("Failed to convert LLVM IR to bitcode".into());
        }

        info!("Compiling to native code...");
        let status = Command::new(&self.llc_path)
            .arg(&bc_path)
            .arg("-filetype=obj")
            .arg("-o")
            .arg(&obj_path)
            .status()?;

        if !status.success() {
            return Err("LLC compilation failed".into());
        }

        info!("Linking with runtime...");
        let output = Command::new(&self.clang_path)
            .arg("-v")
            .arg("-o")
            .arg(&exe_path)
            .arg(&obj_path)
            .arg(format!("-L{}", lib_dir.display()))
            .arg("-Wl,-rpath")
            .arg(lib_dir)
            .arg("-lpecos_engines")
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "Linking failed:\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }

        info!("Compilation successful: {}", exe_path.display());
        Ok(())
    }
}

impl ClassicalEngine for QirClassicalEngine {
    fn process_program(&mut self) -> Result<CommandBatch, QueueError> {
        // Clear previous results at start of each shot
        self.current_results = ShotResult::default();

        let exe_path = self.build_dir.join(&self.program_name);
        debug!("Running QIR program: {}", exe_path.display());

        // Clean up any existing process first
        if let Some(mut child) = self.child_process.take() {
            debug!("Cleaning up previous process");
            let _ = child.kill();
            let _ = child.wait();
        }

        // Start new process
        let mut child = Command::new(&exe_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| QueueError::ExecutionError(format!("Failed to spawn process: {e}")))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| QueueError::ExecutionError("Could not get stdout".into()))?;

        let mut reader = BufReader::new(stdout);
        let mut commands = Vec::new();
        let mut line = String::new();

        // Read until we hit a FLUSH_BEGIN
        while let Ok(len) = reader.read_line(&mut line) {
            if len == 0 {
                break;
            }

            if line.trim() == "FLUSH_BEGIN" {
                debug!("Processing commands block");
                loop {
                    line.clear();
                    let len = reader.read_line(&mut line)?;
                    if len == 0 {
                        break;
                    }

                    let trimmed = line.trim();
                    if trimmed == "FLUSH_END" {
                        debug!("End of commands block");
                        break;
                    }

                    if let Some(cmd) = trimmed.strip_prefix("CMD ") {
                        debug!("Received command: {}", cmd);
                        if let Ok(quantum_cmd) = QuantumCommand::parse_from_str(cmd) {
                            commands.push(quantum_cmd);
                        }
                    }
                }
                break;
            } else {
                debug!("Skipping line: {}", line.trim());
            }
            line.clear();
        }

        // Store process handles for later measurement handling
        self.child_process = Some(child);

        Ok(commands)
    }

    fn handle_measurement(&mut self, measurement: MeasurementResult) -> Result<(), QueueError> {
        debug!("Handling measurement: {}", measurement);

        // Get current qubit index from measurements size
        let qubit_idx = self.current_results.measurements.len();

        // Store measurement with result_id that matches the QIR program
        self.current_results
            .measurements
            .insert(format!("measurement_{qubit_idx}"), measurement);

        // Try to send back to process if still alive
        if let Some(child) = &mut self.child_process {
            if let Some(mut stdin) = child.stdin.take() {
                match writeln!(stdin, "{measurement}") {
                    Ok(()) => {
                        debug!(
                            "Successfully sent measurement {} to classical process",
                            measurement
                        );
                        child.stdin = Some(stdin);
                    }
                    Err(e) => {
                        debug!("Failed to send measurement to classical process: {}", e);
                    }
                }
            }
        }

        Ok(())
    }

    fn get_results(&self) -> Result<ShotResult, QueueError> {
        Ok(self.current_results.clone())
    }

    fn compile(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.compile()
    }
}

impl Drop for QirClassicalEngine {
    fn drop(&mut self) {
        if let Some(mut child) = self.child_process.take() {
            debug!("Cleaning up child process");
            let _ = child.kill();
            let _ = child.wait(); // Wait for the process to finish
        }
    }
}
