//! The `generator_parameters` module provides the definition and functions for [`GeneratorParameters`].

use metamorph_lib::mutation::MutationType;
use metamorph_lib::preferences::Preferences;
use rand_pcg::*;
use std::path::PathBuf;

/// Object to capture the parameters needed to guide the mutation generator algorithm.
#[derive(Debug)]
pub struct GeneratorParameters<'a> {
    /// The location of the input file in the file system
    pub file_name: String,

    /// The number of mutants to generate per input file.
    pub number_of_mutants: usize,

    /// The seed of the random number generator.
    pub rng_seed: u64,

    /// A random number generator for use by the mutators that mutate
    /// the program ASTs.
    pub rng: Pcg64,

    /// The directory in the file system to put the generated mutations.
    pub output_directory: PathBuf,

    /// Instead of `output_directory` use stdout to write output.
    pub use_stdout: bool,

    /// The mutation algorithms to use to generate the mutants.
    pub mutations: Vec<MutationType>,

    /// True if the mutation generator should verify with the language compiler that the generated
    /// mutants compile correctly.
    pub verify_mutant_viability: bool,

    /// If true, then pretty-print a copy of the original AST in the output directory.
    pub print_original: bool,

    /// If true, save a config file for each file the tool attempts to mutate.
    pub save_configuration_file: bool,

    /// Preferences
    pub preferences: &'a mut Preferences,

    /// Function names to mutate.
    pub functions: Vec<String>,
}
