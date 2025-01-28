use clap::{Args, Parser, Subcommand};
use env_logger::Env;
use log::debug;
use pecos::prelude::*;
use std::error::Error;
use std::fs;
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
    /// Run quantum program (supports QIR and PHIR/JSON formats)
    Run(RunArgs),
}

#[derive(Args)]
struct CompileArgs {
    /// Path to the quantum program (LLVM IR)
    program: String,
}

#[derive(Args)]
struct RunArgs {
    /// Path to the quantum program (LLVM IR or JSON)
    program: String,

    /// Number of shots for parallel execution
    #[arg(short, long, default_value_t = 1)]
    shots: usize,

    /// Number of parallel workers
    #[arg(short, long, default_value_t = 1)]
    workers: usize,

    /// Depolarizing noise probability (between 0 and 1)
    #[arg(short = 'p', long = "noise", value_parser = parse_noise_probability)]
    noise_probability: Option<f64>,
}

fn detect_program_type(path: &Path) -> Result<ProgramType, Box<dyn Error>> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("json") => {
            // Read JSON and verify format
            let content = fs::read_to_string(path)?;
            let json: serde_json::Value = serde_json::from_str(&content)?;

            if let Some("PHIR/JSON") = json.get("format").and_then(|f| f.as_str()) {
                Ok(ProgramType::PHIR)
            } else {
                Err("Invalid JSON format - expected PHIR/JSON".into())
            }
        }
        Some("ll") => Ok(ProgramType::QIR),
        _ => Err("Unsupported file format. Expected .ll or .json".into()),
    }
}

enum ProgramType {
    QIR,
    PHIR,
}

fn setup_engine(program_path: &Path) -> Result<Box<dyn ClassicalEngine>, Box<dyn Error>> {
    debug!("Program path: {}", program_path.display());
    let build_dir = program_path.parent().unwrap().join("build");
    debug!("Build directory: {}", build_dir.display());
    std::fs::create_dir_all(&build_dir)?;

    match detect_program_type(program_path)? {
        ProgramType::QIR => Ok(Box::new(QirClassicalEngine::new(program_path, &build_dir))),
        ProgramType::PHIR => Ok(Box::new(PHIREngine::new(program_path)?)),
    }
}

fn get_program_path(program: &str) -> Result<PathBuf, Box<dyn Error>> {
    debug!("Resolving program path");

    // Get the current directory for relative path resolution
    let current_dir = std::env::current_dir()?;
    debug!("Current directory: {}", current_dir.display());

    // Resolve the path
    let path = if Path::new(program).is_absolute() {
        PathBuf::from(program)
    } else {
        current_dir.join(program)
    };

    // Check if file exists
    if !path.exists() {
        return Err(format!("Program file not found: {}", path.display()).into());
    }

    Ok(path.canonicalize()?)
}

fn parse_noise_probability(arg: &str) -> Result<f64, String> {
    let prob: f64 = arg
        .parse()
        .map_err(|_| "Must be a valid floating point number")?;
    if !(0.0..=1.0).contains(&prob) {
        return Err("Noise probability must be between 0 and 1".into());
    }
    Ok(prob)
}

fn run_program(args: &RunArgs) -> Result<(), Box<dyn Error>> {
    let program_path = get_program_path(&args.program)?;
    let classical_engine = setup_engine(&program_path)?;

    // For QIR, ensure it's compiled first
    if let ProgramType::QIR = detect_program_type(&program_path)? {
        classical_engine.compile()?;
    }

    // Create state vector simulator with appropriate number of qubits
    let simulator = StateVec::new(2); // TODO: Get number of qubits from program analysis

    // Create the quantum engine using the factory function
    let quantum_engine = new_quantum_engine_full(simulator); // Use full engine for StateVec

    let cmd_channel = StdioChannel::from_stdio()?;

    // Setup hybrid engine with simulator-equipped quantum engine
    let engine = HybridEngine::new(
        classical_engine,
        quantum_engine,
        cmd_channel.clone(),
        cmd_channel,
    );

    // Set up noise model if requested
    if let Some(prob) = args.noise_probability {
        let noise_model = DepolarizingNoise::new(prob);
        engine.set_noise_model(Some(Box::new(noise_model)));
    }

    // Run simulation - results are printed inside run_parallel
    engine.run_parallel(args.shots, args.workers)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logger with default "info" level if not specified
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Compile(args) => {
            let program_path = get_program_path(&args.program)?;
            match detect_program_type(&program_path)? {
                ProgramType::QIR => {
                    let engine = setup_engine(&program_path)?;
                    engine.compile()?;
                }
                ProgramType::PHIR => {
                    println!("PHIR/JSON programs don't require compilation");
                }
            }
        }
        Commands::Run(args) => run_program(args)?,
    }

    Ok(())
}
