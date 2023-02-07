//! The `mutation_generator` module provides the `generate_mutations` function that implements
//! the mutation generation algorithm.

use crate::generator_parameters::GeneratorParameters;
use crate::pretty_printing::pretty_print_ast;
use crate::MutateCLArgs;
use metamorph_lib::error::MetamorphError;
use metamorph_lib::language_interface::*;
use metamorph_lib::mutation::{get_all_mutation_algorithms, MutationType};
use metamorph_lib::preferences::{PreferenceValue, Preferences};
use metamorph_lib::recognizer::Recognizer;
use metamorph_lib::super_ast::SuperAST;
use rand::seq::SliceRandom;
use rand::RngCore;
use rand::SeedableRng;
use rand_pcg::*;
use std::collections::VecDeque;
use std::{path::PathBuf, str::FromStr};

/// Run the mutation generator algorithm.
///
/// # Arguments
///
/// * `args` - The command line arguments that control the mutation algorithm.
pub fn generate_mutants(args: MutateCLArgs) {
    // Select the mutation algorithms to use while generating mutations.  Args.all_mutations takes
    // precedence over individual algorithms selected in args.mutations.
    let mutations: Vec<MutationType>;
    if args.all_mutations {
        mutations = get_all_mutation_algorithms();
    } else {
        // Change the algorithm strings from the command line into actual MutationType values.
        mutations = args
            .mutations
            .iter()
            .filter(|m| match MutationType::from_str(m) {
                Ok(_) => true,
                _ => {
                    println!("Mutation algorithm {} not supported", m);
                    false
                }
            })
            .map(|m| MutationType::from_str(m).unwrap())
            .collect();
    }

    // Output some log information about the selected algorithms.
    let mut mutation_strings: Vec<String> = Vec::new();
    for mutation in &mutations {
        mutation_strings.push(mutation.to_string());
    }
    log::info!(
        "Generating mutations using algorithms: {:?}",
        mutation_strings
    );

    let mut rng = Pcg64::seed_from_u64(args.rng_seed);

    let mut preferences = Preferences::new();
    preferences.set_value_for_key(
        "solidity_compiler",
        PreferenceValue::String(args.solidity_compiler),
    );
    preferences.set_value_for_key(
        "vyper_compiler",
        PreferenceValue::String(args.vyper_compiler),
    );

    // Now, for each input file, generate the requested number and type of mutations.
    for file_name in args.file_names {
        let mut generator_params = GeneratorParameters::new_from_parameters(
            &file_name,
            args.num_mutants,
            &mut rng,
            PathBuf::from_str(&args.output_directory).unwrap(),
            &mutations,
            false,
            args.print_original,
            &preferences,
        );

        if let Err(e) = generate_mutations(&mut generator_params) {
            println!("Unable to generate mutations: {}", e);
        }
    }
}

/// An upper bound on the number times to try to generate a particular mutant for an input file.
static ATTEMPTS_TO_GENERATE_A_MUTANT: usize = 50;

/// Generate mutations according the parameters.
///
/// # Arguments
///
/// * `params` - The parameters that control the mutation generation algorithm.
fn generate_mutations(params: &mut GeneratorParameters) -> Result<(), MetamorphError> {
    // Try to recognize the language of the source file.  The file might be a source code file
    // or perhaps an AST file.
    let recognizer = Recognizer::new(params.preferences);
    let recognize_result = recognizer.recognize_file(&params.file_name)?;

    let mut language_object =
        LanguageInterface::get_language_object_for_language(&recognize_result.language)?;

    let ast = language_object.load_ast_from_file(
        &params.file_name,
        &recognize_result.file_type,
        params.preferences,
    )?;

    language_object.select_mutators_for_mutation_types(&params.mutations)?;

    let mutable_nodes_table = language_object.count_mutable_nodes(&ast)?;

    if mutable_nodes_table.is_empty() {
        return Err(MetamorphError::NoMutableNode);
    }

    // Only pretty-print the original file after verifying that we can load the AST, and that
    // we have valid mutators for the AST.
    if params.print_original {
        let original_file = PathBuf::from_str(&params.file_name).unwrap();
        pretty_print_ast(&ast, &params.file_name, &params.output_directory)?;
        log::info!(
            "Pretty-printing original file {:?} to {}",
            original_file.file_name().unwrap(),
            &params.output_directory.to_str().unwrap()
        );
    }

    // This list now holds the mutation types for which the AST has nodes to mutate.
    let mutation_type_list: Vec<MutationType> = mutable_nodes_table
        .iter()
        .filter(|(_, v)| **v > 0)
        .map(|(k, _)| *k)
        .collect();

    // Now we generate a list of mutation types of length self.parameters.number_of_mutants
    // with mutation types chosen randomly from list of usable mutation types.
    let mut mutation_kinds_todo: VecDeque<MutationType> = VecDeque::new();
    let mut requested_mutants_remaining = params.number_of_mutants;
    while requested_mutants_remaining > 0 {
        let mutation_type = match mutation_type_list.choose(params.rng) {
            Some(t) => t,
            None => continue,
        };
        mutation_kinds_todo.push_back(*mutation_type);
        requested_mutants_remaining -= 1;
    }

    let mut files_written: usize = 0;
    let mut attempts: usize = 0;
    let mut observed_asts: Vec<SuperAST> = vec![];

    'mutation_loops: while !mutation_kinds_todo.is_empty() {
        while attempts < ATTEMPTS_TO_GENERATE_A_MUTANT {
            // Get the next mutation type.
            let mutation_type = mutation_kinds_todo.get(0).unwrap();

            // Randomly try to pick the node to mutate.
            let number_of_mutable_nodes = mutable_nodes_table.get(mutation_type).unwrap();
            let index = if *number_of_mutable_nodes > 1 {
                (params.rng.next_u64() % *number_of_mutable_nodes as u64) as usize
            } else {
                0
            };

            // Generate the mutated AST.
            let mutated_ast = language_object.mutate_ast(&ast, mutation_type, index, params.rng)?;

            // See if we have already generated this AST before.  We only want to output unique
            // mutations.
            if observed_asts.contains(&mutated_ast) {
                attempts += 1;
                continue;
            }

            // Calculate the name of the output file.
            let input_file_path = PathBuf::from(&params.file_name);
            let base_file_name = input_file_path.file_name().unwrap();
            let outfile_name = params.output_directory.join(
                String::from(base_file_name.to_str().unwrap()) + "_" + &files_written.to_string(),
            );

            let outfile = String::from(outfile_name.to_str().unwrap());

            let final_file = pretty_print_ast(&mutated_ast, &outfile, &params.output_directory)?;

            log::info!(
                "{} used to create mutant written to {}",
                mutation_type,
                final_file.to_str().unwrap()
            );

            // Remove the item from the top of the VecDeque.
            mutation_kinds_todo.remove(0);

            // Add the AST to the list of ASTs that we have observed.
            observed_asts.push(mutated_ast);

            files_written += 1;

            // Reset attempts for the next mutation type.
            attempts = 0;
            continue 'mutation_loops;
        }

        // We reached the limits of attempts on the last mutation, move on to the next.
        attempts = 0;
        mutation_kinds_todo.remove(0);
    }

    Ok(())
}
