//! The main module contains the code to process the command line for the gambit program and
//! run the mutation generator.

mod generator_parameters;
mod mutation_generator;
mod pretty_printing;

use crate::mutation_generator::generate_mutants;
use crate::pretty_printing::pretty_print_files;
use clap::Parser;
use serde::{Deserialize, Serialize};

/// Mutate sub-command command line arguments.
#[derive(Debug, Clone, Parser, Deserialize, Serialize)]
#[clap(rename_all = "kebab-case")]
pub struct MutateCLArgs {
    /// Directory to store mutants
    #[clap(long, default_value = "out")]
    pub output_directory: String,

    /// Input file(s) to mutate
    // TODO: Build input file recognition system.
    #[clap(short, long, required = true, multiple = true)]
    pub file_names: Vec<String>,

    /// Random number generator seed.
    #[clap(long, default_value = "0")]
    pub rng_seed: u64,

    /// Number of mutants
    #[clap(long, default_value = "5")]
    pub num_mutants: usize,

    /// Mutation algorithms to enable
    #[clap(long, multiple = true)]
    pub mutations: Vec<String>,

    /// Enable all mutation algorithms, takes precedence over --mutations
    #[clap(short, long)]
    pub all_mutations: bool,

    /// Pretty-print the original source or AST in the output directory.
    #[clap(long)]
    pub print_original: bool,
}

/// Pretty-print sub-command command line arguments.
#[derive(Debug, Clone, Parser, Deserialize, Serialize)]
#[clap(rename_all = "kebab-case")]
pub struct PrettyPrintCLArgs {
    /// Directory to store pretty-printed copy of source
    #[clap(short, long, default_value = "out")]
    pub output_directory: String,

    /// Input file(s) to pretty-print
    #[clap(short, long, required = true, multiple = true)]
    pub file_names: Vec<String>,
}

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub enum GambitCommand {
    Mutate(MutateCLArgs),
    PrettyPrint(PrettyPrintCLArgs),
}

fn main() {
    let _ = env_logger::builder().try_init();
    match GambitCommand::parse() {
        GambitCommand::Mutate(params) => {
            generate_mutants(params);
        }
        GambitCommand::PrettyPrint(params) => {
            pretty_print_files(params);
        }
    }
}
