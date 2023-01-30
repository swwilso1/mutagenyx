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

    /// A reference to a random number generator for use by the mutators that mutate
    /// the program ASTs.
    pub rng: &'a mut Pcg64,

    /// The directory in the file system to put the generated mutations.
    pub output_directory: PathBuf,

    /// The mutation algorithms to use to generate the mutants.
    pub mutations: Vec<MutationType>,

    /// True if the mutation generator should verify with the language compiler that the generated
    /// mutants compile correctly.
    pub verify_mutant_viability: bool,

    /// If true, then pretty-print a copy of the original AST in the output directory.
    pub print_original: bool,

    /// Preferences
    pub preferences: &'a Preferences,
}

impl<'a> GeneratorParameters<'a> {
    /// Create a new parameters object.
    ///
    /// # Arguments
    ///
    /// * `name` - The location of the input file in the file system.
    /// * `mutants` - The number of mutants to generate for each input file.
    /// * `number_generator` - The reference to the random number generator.
    /// *` out_dir` - The location in the file system to place the generated mutants.
    /// * `mutations` - The list of mutation algorithms to use when generating the mutants.
    /// * `verify_viability` - True if the generator should check that a mutant compiles correctly.
    /// * `print_original` - True if the generator should pretty-print a copy of the original AST.
    /// * `prefs` - Other preference values.
    pub fn new_from_parameters(
        name: &str,
        mutants: usize,
        number_generator: &'a mut Pcg64,
        out_dir: PathBuf,
        mutations: &Vec<MutationType>,
        verify_viability: bool,
        print_original: bool,
        prefs: &'a Preferences,
    ) -> GeneratorParameters<'a> {
        GeneratorParameters {
            file_name: String::from(name),
            number_of_mutants: mutants,
            rng: number_generator,
            output_directory: out_dir,
            mutations: mutations.clone(),
            verify_mutant_viability: verify_viability,
            print_original,
            preferences: prefs,
        }
    }
}
