// PECOS/crates/pecos-cli/src/main.rs
use clap::{Args, Parser, Subcommand};
use env_logger::Env;
use log::debug;
use pecos::prelude::*;
use std::error::Error;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(
    name = "pecos",
    version = env!("CARGO_PKG_VERSION"),
    about = "A quantum error correction simulator",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile QIR program to native code
    Compile(CompileArgs),
    /// Run QIR program with quantum simulation
    Run(RunArgs),
}

#[derive(Args)]
struct CompileArgs {
    /// Path to the quantum program (LLVM IR)
    program: String,
}

#[derive(Args)]
struct RunArgs {
    /// Path to the quantum program (LLVM IR)
    program: String,

    /// Number of shots for parallel execution
    #[arg(short, long, default_value_t = 1)]
    shots: usize,

    /// Number of parallel workers
    #[arg(short, long, default_value_t = 1)]
    workers: usize,
}

fn get_program_path(program: &str) -> Result<PathBuf, Box<dyn Error>> {
    debug!("Resolving program path");
    let program_path = if Path::new(program)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("ll"))
    {
        program.to_string()
    } else {
        format!("{program}.ll")
    };

    let current_dir = std::env::current_dir()?;
    debug!("Current directory: {}", current_dir.display());
    let path = if Path::new(&program_path).is_absolute() {
        PathBuf::from(&program_path)
    } else {
        current_dir.join(&program_path)
    };

    if !path.exists() {
        return Err(format!("Program file not found: {}", path.display()).into());
    }

    Ok(path.canonicalize()?)
}

fn setup_engine(program_path: &Path) -> Result<QirClassicalEngine, Box<dyn Error>> {
    debug!("Program path: {}", program_path.display());
    let build_dir = program_path.parent().unwrap().join("build");
    debug!("Build directory: {}", build_dir.display());
    std::fs::create_dir_all(&build_dir)?;
    Ok(QirClassicalEngine::new(program_path, &build_dir))
}

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logger with default "info" level if not specified
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Compile(args) => {
            let program_path = get_program_path(&args.program)?;
            let engine = setup_engine(&program_path)?;
            engine.compile()?;
        }
        Commands::Run(args) => {
            let program_path = get_program_path(&args.program)?;
            let classical_engine = setup_engine(&program_path)?;

            // First ensure it's compiled
            classical_engine.compile()?;

            // Setup hybrid engine
            let quantum_engine = QuantumSimulator::new();
            let cmd_channel = StdioChannel::from_stdio()?;
            let engine = HybridEngine::new(
                Box::new(classical_engine),
                Box::new(quantum_engine),
                cmd_channel.clone(),
                cmd_channel,
            );

            // Run simulation
            let results = engine.run_parallel(args.shots, args.workers)?;

            // Print statistics
            let stats = engine.compute_statistics(&results);
            engine.print_statistics(&stats);
        }
    }

    Ok(())
}
