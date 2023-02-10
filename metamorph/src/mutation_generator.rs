//! The `mutation_generator` module provides the `generate_mutations` function that implements
//! the mutation generation algorithm.

use crate::generator_parameters::GeneratorParameters;
use crate::pretty_printing::pretty_print_ast;
use crate::MutateCLArgs;
use metamorph_lib::compiler_details::*;
use metamorph_lib::config_file::*;
use metamorph_lib::error::MetamorphError;
use metamorph_lib::language::Language;
use metamorph_lib::language_interface::*;
use metamorph_lib::mutation::{get_all_mutation_algorithms, MutationType};
use metamorph_lib::preferences::Preferences;
use metamorph_lib::recognizer::{FileType, Recognizer};
use metamorph_lib::super_ast::SuperAST;
use rand::seq::SliceRandom;
use rand::RngCore;
use rand::SeedableRng;
use rand_pcg::*;
use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{path::PathBuf, str::FromStr};

/// Convert a vector of [`MutationType`] to a vector of [`String`].
fn get_mutation_strings_from_types(array: &[MutationType]) -> Vec<String> {
    array.iter().map(|t| t.to_string()).collect()
}

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

    let mut solidity_compiler_prefs = Preferences::new();
    solidity_compiler_prefs.set_string_for_key(PATH_KEY, &args.solidity_compiler);

    let mut solidity_prefs = Preferences::new();
    solidity_prefs.set_preferences_for_key(COMPILER_KEY, solidity_compiler_prefs);

    let mut vyper_compiler_prefs = Preferences::new();
    vyper_compiler_prefs.set_string_for_key(PATH_KEY, &args.vyper_compiler);

    let mut vyper_prefs = Preferences::new();
    vyper_prefs.set_preferences_for_key(COMPILER_KEY, vyper_compiler_prefs);

    let mut preferences = Preferences::new();
    let solidity_key = format!("{}", Language::Solidity);
    let vyper_key = format!("{}", Language::Vyper);
    preferences.set_preferences_for_key(&solidity_key, solidity_prefs);
    preferences.set_preferences_for_key(&vyper_key, vyper_prefs);

    // Now, for each input file, generate the requested number and type of mutations.
    for file_name in args.file_names {
        // We build a new random number generator for each file.  The file might be a config
        // file and contain a new random number generator seed.  The `generate_mutations` code
        // will handle loading a config file and will create a new rng from a seed from a config
        // file if the config file has a new seed.  Here, we just create a new seed from time,
        // or use the seed from the command line.
        let seed: u64 = if args.rng_seed < 0 {
            let start = SystemTime::now();
            let since_the_epoch = start.duration_since(UNIX_EPOCH);
            match since_the_epoch {
                // Todo: This step truncates a u128 to u64.  We probably need those extra bits.
                Ok(t) => t.as_millis() as u64,
                _ => 0,
            }
        } else {
            args.rng_seed as u64
        };

        let mut generator_params = GeneratorParameters {
            file_name,
            number_of_mutants: args.num_mutants.clone(),
            rng_seed: seed,
            rng: Pcg64::seed_from_u64(seed),
            output_directory: PathBuf::from_str(&args.output_directory).unwrap(),
            mutations: mutations.clone(),
            verify_mutant_viability: false,
            print_original: args.print_original.clone(),
            save_configuration_file: args.save_config_files,
            preferences: &mut preferences,
        };

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
    let mut recognize_result = recognizer.recognize_file(&params.file_name)?;

    let mut language_object =
        LanguageInterface::get_language_object_for_language(&recognize_result.language)?;

    if recognize_result.file_type == FileType::Config {
        // If we have a config file, then we need to override parameters with details from
        // the config file.
        let configuration_details = ConfigurationFileDetails::new_from_file(&params.file_name)?;

        if let Some(compiler_details) = &configuration_details.compiler_details {
            let language_key = format!["{}", recognize_result.language];
            if let Some(mut language_preferences) =
                params.preferences.get_preferences_for_key(&language_key)
            {
                language_preferences
                    .set_preferences_for_key(COMPILER_KEY, compiler_details.clone());
                params
                    .preferences
                    .set_preferences_for_key(&language_key, language_preferences);
            }
        }

        let mut prefs_copy = params.preferences.clone();
        let sub_recognizer = Recognizer::new(&mut prefs_copy);

        // The file in the config file may still be either an AST or a source code file. We need
        // to recognize that file.
        recognize_result =
            sub_recognizer.recognize_file(configuration_details.filename.to_str().unwrap())?;

        params.file_name = String::from(configuration_details.filename.to_str().unwrap());

        params.number_of_mutants = configuration_details.number_of_mutants as usize;

        // Here we update the random number generator from the configuration details seed if the
        // details have a new seed.
        if let Some(s) = configuration_details.seed {
            params.rng_seed = s;
            params.rng = Pcg64::seed_from_u64(s)
        };

        // Check to see if the configuration file requested a different set of mutation algorithms.
        if configuration_details.all_mutations {
            params.mutations = get_all_mutation_algorithms();
        } else if !configuration_details.mutations.is_empty() {
            params.mutations = configuration_details.mutations;
        }
    }

    if !params.mutations.is_empty() {
        log::info!(
            "Generating mutations using algorithms: {:?}",
            get_mutation_strings_from_types(&params.mutations)
        );
    }

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

    // Now, see if we need to create a configuration file with the details used to mutate
    // params.file_name.
    if params.save_configuration_file {
        let compiler_details = if let Some(details) = params
            .preferences
            .get_preferences_for_key(COMPILER_DETAILS_KEY)
        {
            Some(details)
        } else {
            let language_key = format!("{}", language_object.implements());
            if let Some(language_preferences) =
                params.preferences.get_preferences_for_key(&language_key)
            {
                language_preferences.get_preferences_for_key(COMPILER_KEY)
            } else {
                Some(language_object.default_compiler_settings())
            }
        };

        let details = ConfigurationFileDetails::new(
            Some(recognize_result.language),
            params.file_name.clone(),
            params.number_of_mutants as i64,
            Some(params.rng_seed),
            &params.mutations,
            false,
            compiler_details,
        );

        // Build the output file name.
        let input_file_name = PathBuf::from(&params.file_name);
        let mut base_file_name =
            String::from(input_file_name.file_name().unwrap().to_str().unwrap());
        let base_out_file_name: String = if let Some(index) = base_file_name.rfind('.') {
            base_file_name.drain(..index).collect()
        } else {
            base_file_name
        };
        let file_extension = String::from(".") + CONFIG_FILE_EXTENSION;
        let out_file_name = base_out_file_name + file_extension.as_str();
        let out_file_path = params.output_directory.join(out_file_name);

        log::info!(
            "Writing configuration file {}",
            out_file_path.to_str().unwrap()
        );
        details.write_to_file_as_json(out_file_path.to_str().unwrap())?;
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
        let mutation_type = match mutation_type_list.choose(&mut params.rng) {
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
            let mutated_ast =
                language_object.mutate_ast(&ast, mutation_type, index, &mut params.rng)?;

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
