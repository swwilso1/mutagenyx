//! The main module contains the code to process the command line for the mutagenyx program and
//! run the mutation generator.

mod compiler_settings;
mod generator_parameters;
mod mutation_generator;
mod mutations_info;
mod pretty_printing;

use crate::mutation_generator::generate_mutants;
use crate::mutations_info::display_mutations_info;
use crate::pretty_printing::pretty_print_files;
use chrono::Local;
use clap::{ArgGroup, Args, Parser, Subcommand};
use env_logger::TimestampPrecision;
use std::io::Write;

#[derive(Parser)]
#[command(author, version, long_about = None)]
#[command(about = "Mutator for programs in various programming languages.")]
#[command(propagate_version = true)]
struct MutagenyxCommand {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    Algorithms(AlgorithmsCLArgs),
    Mutate(MutateCLArgs),
    PrettyPrint(PrettyPrintCLArgs),
}

/// Arguments for listing details about mutation algorithms
#[derive(Args, Debug, Clone)]
#[command(group(
ArgGroup::new("info")
.required(true)
.args(["list", "describe"]),
))]
pub struct AlgorithmsCLArgs {
    /// List the available mutation algorithms
    #[arg(short, long)]
    pub list: bool,

    /// Describe the available mutation algorithms
    #[arg(short, long)]
    pub describe: bool,
}

/// Arguments for generating mutations from source, AST, or configuration files.
#[derive(Args, Debug, Clone)]
pub struct MutateCLArgs {
    /// Directory to store mutants
    #[arg(short, long, default_value = "out")]
    pub output_directory: String,

    /// Input file(s) to mutate
    #[arg(short, long, required = true)]
    pub file_names: Vec<String>,

    /// Random number generator seed.
    #[arg(long, default_value_t = -1)]
    pub rng_seed: i64,

    /// Number of mutants
    #[arg(long, default_value_t = 1)]
    pub num_mutants: usize,

    /// Mutation algorithms to enable
    #[arg(long)]
    pub mutations: Vec<String>,

    /// Enable all mutation algorithms, takes precedence over --mutations
    #[arg(short, long)]
    pub all_mutations: bool,

    /// Pretty-print the original source or AST in the output directory.
    #[arg(long)]
    pub print_original: bool,

    /// Save the mutation information used to mutate each file in a config (.mgnx) file
    /// in the output directory
    #[arg(long)]
    pub save_config_files: bool,

    /// Write output to stdout instead of the directory given in `output_directory`.
    #[arg(long)]
    pub stdout: bool,

    /// Only generate mutations in the listed functions.
    #[arg(long)]
    pub functions: Vec<String>,

    /// Check each mutant for correctness. (Currently attempts to compile each mutant)
    #[arg(long)]
    pub validate_mutants: bool,

    /// Solidity compiler
    #[arg(long, default_value = "solc")]
    pub solidity_compiler: String,

    /// Solidity base path (equivalent to --base-path)
    #[arg(long)]
    pub solidity_base_path: Option<String>,

    /// Solidity include paths (equivalent to --include-path)
    #[arg(long, requires = "solidity_base_path")]
    pub solidity_include_path: Vec<String>,

    /// Solidity allow paths (equivalent to --allow-paths)
    #[arg(long)]
    pub solidity_allow_paths: Vec<String>,

    /// Solidity re-mapping (context:prefix=path)
    #[arg(long)]
    pub solidity_remappings: Vec<String>,

    /// Vyper compiler
    #[arg(long, default_value = "vyper")]
    pub vyper_compiler: String,

    /// Vyper root path (equivalent to -p)
    #[arg(long)]
    pub vyper_root_path: Option<String>,
}

/// Arguments for pretty-printing source or AST input.
#[derive(Args, Debug, Clone)]
pub struct PrettyPrintCLArgs {
    /// Directory to store pretty-printed copy of source
    #[arg(short, long, default_value = "out")]
    pub output_directory: String,

    /// Input file(s) to pretty-print
    #[arg(short, long, required = true)]
    pub file_names: Vec<String>,

    /// Write output to stdout instead of the directory given in `output_directory`.
    #[arg(long)]
    pub stdout: bool,

    /// Solidity compiler
    #[arg(long, default_value = "solc")]
    pub solidity_compiler: String,

    /// Solidity base path (equivalent to --base-path)
    #[arg(long)]
    pub solidity_base_path: Option<String>,

    /// Solidity include paths (equivalent to --include-path)
    #[arg(long, requires = "solidity_base_path")]
    pub solidity_include_path: Vec<String>,

    /// Solidity allow paths (equivalent to --allow-paths)
    #[arg(long)]
    pub solidity_allow_paths: Vec<String>,

    /// Solidity re-mapping (context:prefix=path)
    #[arg(long)]
    pub solidity_remappings: Vec<String>,

    /// Vyper compiler
    #[arg(long, default_value = "vyper")]
    pub vyper_compiler: String,

    /// Vyper root path (equivalent to -p)
    #[arg(long)]
    pub vyper_root_path: Option<String>,
}

fn main() {
    let _ = env_logger::builder()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .format_timestamp(Some(TimestampPrecision::Seconds))
        .try_init();

    let mutagenyx_command = MutagenyxCommand::parse();
    match &mutagenyx_command.command {
        Commands::Algorithms(alg_args) => {
            display_mutations_info(alg_args.clone());
        }
        Commands::Mutate(mutate_args) => {
            if let Err(e) = generate_mutants(mutate_args.clone()) {
                println!("Unable to generate mutations: {}", e);
            }
        }
        Commands::PrettyPrint(pretty_print_args) => {
            pretty_print_files(pretty_print_args.clone());
        }
    }
}
