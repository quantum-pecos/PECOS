// PECOS/crates/pecos-engines/src/qir/compiler.rs
use crate::errors::QueueError;
use log::{debug, info, trace};
use rand::Rng;
use rayon::prelude::*;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

pub struct QirCompiler {
    llc_path: String,
    clang_path: String,
    pub build_dir: String,
}

impl QirCompiler {
    #[must_use]
    pub fn new(llc_path: &str, clang_path: &str, build_dir: &str) -> Self {
        Self {
            llc_path: llc_path.to_string(),
            clang_path: clang_path.to_string(),
            build_dir: build_dir.to_string(),
        }
    }

    pub fn compile_ir(&self, program: &str) -> Result<(), Box<dyn Error>> {
        fs::create_dir_all(&self.build_dir)?;

        let bc_path = format!("{}/{}.bc", self.build_dir, program);
        let obj_path = format!("{}/{}.o", self.build_dir, program);
        let exe_path = format!("{}/{}", self.build_dir, program);

        // Convert LLVM IR to bitcode
        info!("Converting LLVM IR to bitcode...");
        let status = Command::new("llvm-as-13")
            .arg(format!("{program}.ll"))
            .arg("-o")
            .arg(&bc_path)
            .status()?;

        if !status.success() {
            return Err("Failed to convert LLVM IR to bitcode".into());
        }

        // Compile bitcode to object file
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

        // Link with our runtime
        let status = Command::new(&self.clang_path)
            .arg("-o")
            .arg(&exe_path)
            .arg(&obj_path)
            .arg("-L./target/debug/")
            .arg("-lpecos_engines")
            .status()?;

        if !status.success() {
            return Err("Linking failed".into());
        }

        println!("Compilation successful for {program}");
        Ok(())
    }

    fn run_single_shot(
        &self,
        program_path: &str,
        shot_idx: usize,
    ) -> Result<HashMap<String, u32>, QueueError> {
        debug!("[SHOT-{}] Starting execution", shot_idx);

        let mut child = Command::new(program_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| QueueError::ExecutionError(e.to_string()))?;

        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| QueueError::ExecutionError("Could not get stdin".into()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| QueueError::ExecutionError("Could not get stdout".into()))?;

        let mut results = HashMap::new();
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();

        trace!("[SHOT-{}] Starting to read output", shot_idx);
        while let Ok(len) = reader.read_line(&mut line) {
            if len == 0 {
                trace!("[SHOT-{}] Reached end of output", shot_idx);
                break;
            }

            match line.trim() {
                "FLUSH_BEGIN" => {
                    trace!("[SHOT-{}] Processing commands block", shot_idx);
                    loop {
                        line.clear();
                        let len = reader.read_line(&mut line)?;
                        if len == 0 {
                            break;
                        }

                        let trimmed = line.trim();
                        if trimmed == "FLUSH_END" {
                            trace!("[SHOT-{}] End of commands block", shot_idx);
                            break;
                        }

                        if let Some(cmd) = trimmed.strip_prefix("CMD ") {
                            debug!("[SHOT-{}] Processing command: {}", shot_idx, cmd);

                            if let Some(measurement) = Self::process_quantum_command(cmd)? {
                                writeln!(stdin, "{measurement}")
                                    .map_err(|e| QueueError::ExecutionError(e.to_string()))?;
                                stdin
                                    .flush()
                                    .map_err(|e| QueueError::ExecutionError(e.to_string()))?;
                            }
                        }
                    }
                }
                line if line.starts_with("RESULT") => {
                    debug!("[SHOT-{}] Got result line: {}", shot_idx, line);
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if let ["RESULT", key, value] = parts.as_slice() {
                        if let Ok(val) = value.parse::<u32>() {
                            results.insert((*key).to_string(), val);
                        }
                    }
                }
                _ => trace!("[SHOT-{}] {}", shot_idx, line.trim()),
            }

            line.clear();
        }

        debug!("[SHOT-{}] Retrieved results: {:?}", shot_idx, results);
        Ok(results)
    }

    pub fn run_parallel(
        &self,
        program_path: &str,
        total_shots: usize,
        workers: usize,
    ) -> Result<Vec<HashMap<String, u32>>, QueueError> {
        info!(
            "Starting parallel execution with {} shots and {} workers",
            total_shots, workers
        );

        let all_results = Arc::new(Mutex::new(Vec::with_capacity(total_shots)));

        rayon::ThreadPoolBuilder::new()
            .num_threads(workers)
            .build_global()
            .map_err(|e| QueueError::ExecutionError(e.to_string()))?;

        (0..total_shots).into_par_iter().try_for_each(|shot_idx| {
            let shot_result = self.run_single_shot(program_path, shot_idx)?;

            let mut results = all_results
                .lock()
                .map_err(|e| QueueError::LockError(e.to_string()))?;
            results.push(shot_result);

            Ok::<(), QueueError>(())
        })?;

        let final_results = Arc::try_unwrap(all_results)
            .map_err(|_| QueueError::LockError("Could not unwrap results".into()))?
            .into_inner()
            .map_err(|e| QueueError::LockError(e.to_string()))?;

        info!("Completed {} shots", final_results.len());

        // Print per-shot results at debug level
        for (idx, result) in final_results.iter().enumerate() {
            debug!("Shot {}: {:?}", idx, result);
        }

        // Always print measurement statistics
        self.print_measurement_statistics(&final_results);

        Ok(final_results)
    }

    fn print_measurement_statistics(&self, results: &[HashMap<String, u32>]) {
        let mut all_keys: Vec<String> = results
            .iter()
            .flat_map(|map| map.keys().cloned())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        all_keys.sort();

        println!("\nMeasurement Statistics:");
        for key in all_keys {
            let values: Vec<u32> = results
                .iter()
                .filter_map(|map| map.get(&key))
                .copied()
                .collect();

            let n = values.len();
            if n == 0 {
                continue;
            }

            let zeros = values.iter().filter(|&&x| x == 0).count();
            let ones = values.iter().filter(|&&x| x == 1).count();

            println!("\n{key}:");
            println!("  Total measurements: {n}");
            println!("  |0⟩: {} ({:.1}%)", zeros, 100.0 * zeros as f64 / n as f64);
            println!("  |1⟩: {} ({:.1}%)", ones, 100.0 * ones as f64 / n as f64);
        }
    }

    fn process_quantum_command(cmd: &str) -> Result<Option<u32>, QueueError> {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let mut rng = rand::thread_rng();

        match parts.first() {
            Some(&"RZ") => {
                debug!("Processing RZ gate");
                Ok(None)
            }
            Some(&"R1XY") => {
                debug!("Processing R1XY gate");
                Ok(None)
            }
            Some(&"SZZ") => {
                debug!("Processing SZZ gate");
                Ok(None)
            }
            Some(&"H") => {
                debug!("Processing H gate");
                Ok(None)
            }
            Some(&"CX") => {
                debug!("Processing CX gate");
                Ok(None)
            }
            Some(&"M") => {
                if parts.len() != 3 {
                    return Err(QueueError::OperationError(
                        "Invalid M command format".into(),
                    ));
                }
                let measurement = rng.gen_range(0..=1);
                debug!(
                    "Generated measurement {} for qubit {}",
                    measurement, parts[1]
                );
                Ok(Some(measurement))
            }
            _ => Err(QueueError::OperationError(format!(
                "Unknown command: {cmd}"
            ))),
        }
    }
}
