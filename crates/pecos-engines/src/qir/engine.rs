// PECOS/crates/pecos-engines/src/qir/qir_engine
use crate::channels::Message;
use crate::engines::ClassicalEngine;
use crate::errors::QueueError;
use log::{debug, info};
use pecos_core::types::{CommandBatch, QuantumCommand, ShotResult};
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

    fn find_and_copy_library(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let lib_name = "libpecos_engines.so";
        let exe_path = std::env::current_exe()?;
        let exe_dir = exe_path.parent().ok_or("Cannot get executable directory")?;

        // 1. Development case - direct target directory
        if let Some(target_dir) = exe_dir.parent() {
            for profile in ["debug", "release"] {
                let lib = target_dir.join(profile).join("deps").join(lib_name);
                if lib.exists() {
                    let dest = self.build_dir.join(lib_name);
                    debug!(
                        "Copying library from {} to {}",
                        lib.display(),
                        dest.display()
                    );
                    fs::copy(&lib, &dest)?;
                    return Ok(self.build_dir.clone());
                }
            }
        }

        // 2. Development case - workspace target directory
        if let Some(workspace_dir) = exe_dir.parent().and_then(|d| d.parent()) {
            for profile in ["debug", "release"] {
                let lib = workspace_dir
                    .join("target")
                    .join(profile)
                    .join("deps")
                    .join(lib_name);
                if lib.exists() {
                    let dest = self.build_dir.join(lib_name);
                    debug!(
                        "Copying library from {} to {}",
                        lib.display(),
                        dest.display()
                    );
                    fs::copy(&lib, &dest)?;
                    return Ok(self.build_dir.clone());
                }
            }
        }

        // 3. Installation case
        if exe_dir.ends_with("bin") {
            let lib = exe_dir.parent().unwrap().join("lib").join(lib_name);
            if lib.exists() {
                let dest = self.build_dir.join(lib_name);
                debug!(
                    "Copying library from {} to {}",
                    lib.display(),
                    dest.display()
                );
                fs::copy(&lib, &dest)?;
                return Ok(self.build_dir.clone());
            }
        }

        // 4. Direct path in ~/.cargo/lib
        if let Ok(home) = std::env::var("HOME") {
            let lib = PathBuf::from(home).join(".cargo/lib").join(lib_name);
            if lib.exists() {
                let dest = self.build_dir.join(lib_name);
                debug!(
                    "Copying library from {} to {}",
                    lib.display(),
                    dest.display()
                );
                fs::copy(&lib, &dest)?;
                return Ok(self.build_dir.clone());
            }
        }

        Err("Could not find libpecos_engines.so in any standard location".into())
    }

    /// Compiles the QIR program into a native executable.
    ///
    /// Steps:
    /// 1. Converts the input LLVM IR file to bitcode.
    /// 2. Compiles the bitcode to a native object file using `llc`.
    /// 3. Links the object file with the required runtime library using `clang`.
    ///
    /// The compiled executable is stored in the build directory specified for this engine.
    ///
    /// # Errors
    /// - Returns an error if the build directory cannot be created.
    /// - Returns an error if any of the commands (`llvm-as-13`, `llc`, `clang`) fail.
    /// - Returns an error if the required runtime library cannot be found or copied.
    ///
    /// # Returns
    /// - `Ok(())` if the compilation succeeds.
    /// - `Err(Box<dyn std::error::Error>)` if any step in the process fails.
    pub fn compile(&self) -> Result<(), Box<dyn std::error::Error>> {
        fs::create_dir_all(&self.build_dir)?;

        let bc_path = self.build_dir.join(format!("{}.bc", self.program_name));
        let obj_path = self.build_dir.join(format!("{}.o", self.program_name));
        let exe_path = self.build_dir.join(&self.program_name);

        debug!("Input file: {}", self.program_path.display());
        debug!("Object file: {}", obj_path.display());
        debug!("Executable: {}", exe_path.display());

        // Copy library to build directory and get its location
        let lib_dir = self.find_and_copy_library()?;
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
            .arg("-L.") // Look in current directory
            .arg("-Wl,-rpath,$ORIGIN") // Use relative path for runtime
            .arg("-lpecos_engines")
            .current_dir(&lib_dir) // Run from directory containing library
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
            }

            debug!("Skipping line: {}", line.trim());

            line.clear();
        }

        // Store process handles for later measurement handling
        self.child_process = Some(child);

        Ok(commands)
    }

    fn handle_measurement(&mut self, measurement: Message) -> Result<(), QueueError> {
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
