use gambit_lib::mutation::MutationType;
use rand_pcg::*;
use std::path::PathBuf;

#[derive(Debug)]
pub struct GeneratorParameters<'a> {
    pub file_name: String,
    pub number_of_mutants: usize,
    pub rng: &'a mut Pcg64,
    pub output_directory: PathBuf,
    pub mutations: Vec<MutationType>,
    pub verify_mutant_viability: bool,
}

impl<'a> GeneratorParameters<'a> {
    pub fn new_from_parameters(
        name: &str,
        mutants: usize,
        number_generator: &'a mut Pcg64,
        out_dir: PathBuf,
        mutations: &Vec<MutationType>,
        verify_viability: bool,
    ) -> GeneratorParameters<'a> {
        GeneratorParameters {
            file_name: String::from(name),
            number_of_mutants: mutants,
            rng: number_generator,
            output_directory: out_dir,
            mutations: mutations.clone(),
            verify_mutant_viability: verify_viability,
        }
    }
}
