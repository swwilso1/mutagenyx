//! The main module contains the code to process the command line for the metamorph program and
//! run the mutation generator.

mod generator_parameters;
mod mutation_generator;
mod mutations_info;
mod pretty_printing;

use crate::mutation_generator::generate_mutants;
use crate::mutations_info::display_mutations_info;
use crate::pretty_printing::pretty_print_files;
use chrono::Local;
use clap::Parser;
use env_logger::TimestampPrecision;
use serde::{Deserialize, Serialize};
use std::io::Write;

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

    /// Solidity compiler
    #[clap(long, default_value = "solc")]
    pub solidity_compiler: String,

    /// Vyper compiler
    #[clap(long, default_value = "vyper")]
    pub vyper_compiler: String,
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

    /// Solidity compiler
    #[clap(long, default_value = "solc")]
    pub solidity_compiler: String,

    /// Vyper compiler
    #[clap(long, default_value = "vyper")]
    pub vyper_compiler: String,
}

/// Arguments for listing details about mutation algorithms
#[derive(Debug, Clone, Parser, Deserialize, Serialize)]
#[clap(rename_all = "kebab-case")]
pub struct AlgorithmsCLArgs {
    /// List the available mutation algorithms
    #[clap(short, long)]
    pub list: bool,

    /// Describe the available mutation algorithms
    #[clap(short, long)]
    pub describe: bool,
}

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub enum MetamorphCommand {
    Mutate(MutateCLArgs),
    PrettyPrint(PrettyPrintCLArgs),
    Algorithms(AlgorithmsCLArgs),
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
    match MetamorphCommand::parse() {
        MetamorphCommand::Mutate(params) => {
            generate_mutants(params);
        }
        MetamorphCommand::PrettyPrint(params) => {
            pretty_print_files(params);
        }
        MetamorphCommand::Algorithms(params) => {
            display_mutations_info(params);
        }
    }
}
