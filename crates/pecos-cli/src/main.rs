use clap::{Arg, Command};
use env_logger::Env;
use log::debug;
use pecos_engines::{
    channels::stdio::StdioChannel,
    engines::{classical::QirClassicalEngine, quantum::QuantumSimulator, HybridEngine},
};
use std::error::Error;
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let matches = Command::new("qir-runner")
        .version("1.0")
        .about("Compile and run quantum programs")
        .arg(
            Arg::new("mode")
                .help("Execution mode: compile or run")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("program")
                .help("Path to the quantum program (LLVM IR)")
                .required(true)
                .index(2),
        )
        .arg(
            Arg::new("shots")
                .short('s')
                .long("shots")
                .help("Number of shots for parallel execution")
                .value_name("SHOTS")
                .default_value("1")
                .value_parser(clap::value_parser!(usize)),
        )
        .arg(
            Arg::new("workers")
                .short('w')
                .long("workers")
                .help("Number of parallel workers")
                .value_name("WORKERS")
                .default_value("1")
                .value_parser(clap::value_parser!(usize)),
        )
        .get_matches();

    // Get path to the input program
    let program = matches.get_one::<String>("program").unwrap();
    let program_path = if program.ends_with(".ll") {
        program.to_string()
    } else {
        format!("{program}.ll")
    };

    let mode = matches.get_one::<String>("mode").unwrap();
    let total_shots = *matches.get_one::<usize>("shots").unwrap();
    let num_workers = *matches.get_one::<usize>("workers").unwrap();

    let current_dir = std::env::current_dir()?;
    debug!("Current directory: {}", current_dir.display());

    // Convert relative path to absolute
    let program_path = if Path::new(&program_path).is_absolute() {
        PathBuf::from(&program_path)
    } else {
        current_dir.join(&program_path)
    };

    // Make sure program path exists
    if !program_path.exists() {
        return Err(format!("Program file not found: {}", program_path.display()).into());
    }

    let program_path = program_path.canonicalize()?;
    debug!("Program path: {}", program_path.display());

    // Create build directory next to the input file
    let build_dir = program_path.parent().unwrap().join("build");
    debug!("Build directory: {}", build_dir.display());

    // Create build directory
    std::fs::create_dir_all(&build_dir)?;

    // Create classical engine with absolute paths
    let classical_engine = QirClassicalEngine::new(&program_path, &build_dir);

    match mode.as_str() {
        "compile" => {
            classical_engine.compile()?;
        }
        "run" => {
            // First ensure it's compiled
            classical_engine.compile()?;

            // Create quantum engine and channels
            let quantum_engine = QuantumSimulator::new();
            let cmd_channel = StdioChannel::from_stdio()?;
            let meas_channel = cmd_channel.clone();

            // Create hybrid engine
            let engine = HybridEngine::new(
                Box::new(classical_engine),
                Box::new(quantum_engine),
                cmd_channel,
                meas_channel,
            );

            // Run simulation
            let results = engine.run_parallel(total_shots, num_workers)?;

            // Print statistics
            let stats = engine.compute_statistics(&results);
            engine.print_statistics(&stats);
        }
        _ => {
            println!("Unknown mode: {mode}");
            println!("Usage: qir-runner <compile|run> <program> [-s <shots>] [-w <workers>]");
            std::process::exit(1);
        }
    }

    Ok(())
}
