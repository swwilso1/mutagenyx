mod generator_parameters;
mod mutation_generator;

use crate::generator_parameters::GeneratorParameters;
use crate::mutation_generator::generate_mutations;
use clap::Parser;
use gambit_lib::mutation::MutationType;
use rand::SeedableRng;
use rand_pcg::Pcg64;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, str::FromStr};

fn generate_mutants(args: MutateCLArgs) {
    let mutations = args
        .mutations
        .iter()
        .map(|m| MutationType::from_str(m).unwrap())
        .collect();

    let mut rng = Pcg64::seed_from_u64(args.rng_seed);

    for file_name in args.file_names {
        let mut generator_params = GeneratorParameters::new_from_parameters(
            &file_name,
            args.num_mutants,
            &mut rng,
            PathBuf::from_str(&args.output_directory).unwrap(),
            &mutations,
            false,
        );

        if let Err(e) = generate_mutations(&mut generator_params) {
            println!("Unable to generate mutations: {}", e);
        }
    }
}

/// Command line arguments
#[derive(Debug, Clone, Parser, Deserialize, Serialize)]
#[clap(rename_all = "kebab-case")]
pub struct MutateCLArgs {
    /// Directory to store mutants
    #[clap(long, default_value = "out")]
    pub output_directory: String,

    /// JSON file(s) to mutate
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
    #[clap(long, required = true, multiple = true)]
    pub mutations: Vec<String>,
}

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub enum GambitCommand {
    Mutate(MutateCLArgs),
}

fn main() {
    let _ = env_logger::builder().try_init();
    match GambitCommand::parse() {
        GambitCommand::Mutate(params) => {
            generate_mutants(params);
        }
    }
}
