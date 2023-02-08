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
}

impl<'a> GeneratorParameters<'a> {
    /// Create a new parameters object.
    ///
    /// # Arguments
    ///
    /// * `name` - The location of the input file in the file system.
    /// * `mutants` - The number of mutants to generate for each input file.
    /// * `seed` - The seed of `number_generator`.
    /// * `number_generator` - The reference to the random number generator.
    /// *` out_dir` - The location in the file system to place the generated mutants.
    /// * `mutations` - The list of mutation algorithms to use when generating the mutants.
    /// * `verify_viability` - True if the generator should check that a mutant compiles correctly.
    /// * `print_original` - True if the generator should pretty-print a copy of the original AST.
    /// * `save_config_file` - True if the generator should create a config file for each file
    /// it attempts to mutate.
    /// * `prefs` - Other preference values.
    pub fn new_from_parameters(
        name: &str,
        mutants: usize,
        seed: u64,
        number_generator: Pcg64,
        out_dir: PathBuf,
        mutations: Vec<MutationType>,
        verify_viability: bool,
        print_original: bool,
        save_config_file: bool,
        prefs: &'a mut Preferences,
    ) -> GeneratorParameters<'a> {
        GeneratorParameters {
            file_name: String::from(name),
            number_of_mutants: mutants,
            rng_seed: seed,
            rng: number_generator,
            output_directory: out_dir,
            mutations,
            verify_mutant_viability: verify_viability,
            print_original,
            save_configuration_file: save_config_file,
            preferences: prefs,
        }
    }
}
