//! The `mutation_generator` module provides the `generate_mutations` function that implements
//! the mutation generation algorithm.

use crate::generator_parameters::GeneratorParameters;
use crate::pretty_printing::{pretty_print_ast, pretty_print_ast_to_stream};
use crate::MutateCLArgs;
use metamorph_lib::compiler_details::*;
use metamorph_lib::config_file::*;
use metamorph_lib::error::MetamorphError;
use metamorph_lib::language::Language;
use metamorph_lib::language_interface::*;
use metamorph_lib::mutation::{get_all_mutation_algorithms, MutationType};
use metamorph_lib::permissions::Permissions;
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
///
/// # Arguments
///
/// * `array` - The array of mutation types to convert to strings.
fn get_mutation_strings_from_types(array: &[MutationType]) -> Vec<String> {
    array.iter().map(|t| t.to_string()).collect()
}

/// Convert a vector of [`String`] to a vector of [`MutationType`].
fn get_mutation_types_from_strings(array: &[String]) -> Vec<MutationType> {
    array
        .iter()
        .filter_map(|s| MutationType::from_str(s).ok())
        .collect()
}

/// Helper function to convert a vector of strings representing function names to a Permissions
/// object suitable for use in AST traversal.
///
/// # Arguments
///
/// * `names` - The vector of function names.
fn convert_function_names_to_permissions(names: &Vec<String>) -> Permissions {
    let mut permissions = Permissions::new();

    for function_name in names {
        let permission_value = String::from("mutate.") + function_name.as_str();
        permissions.set_permission(&permission_value, true);
    }

    permissions
}

/// Simple struct to collect references to compiler paths.
struct CompilerPaths<'a> {
    /// Path to solidity compiler.
    solidity: &'a String,

    /// Path to vyper compiler.
    vyper: &'a String,
}

/// Generate a basic language Preferences layout starting from compiler paths.
///
/// The Preferences object is not JSON, but in JSON, the return Preferences has the following
/// form.
///
/// {
///     "Solidity:" {
///         "compiler": {
///             "path": "..."
///         }
///     },
///     "Vyper": {
///         "compiler": {
///             "path": "..."
///         }
///     }
/// }
fn generate_preferences_compiler_paths(compiler_paths: &CompilerPaths) -> Preferences {
    let mut solidity_compiler_prefs = Preferences::new();
    solidity_compiler_prefs.set_string_for_key(PATH_KEY, compiler_paths.solidity);

    let mut solidity_prefs = Preferences::new();
    solidity_prefs.set_preferences_for_key(COMPILER_KEY, solidity_compiler_prefs);

    let mut vyper_compiler_prefs = Preferences::new();
    vyper_compiler_prefs.set_string_for_key(PATH_KEY, compiler_paths.vyper);

    let mut vyper_prefs = Preferences::new();
    vyper_prefs.set_preferences_for_key(COMPILER_KEY, vyper_compiler_prefs);

    let mut preferences = Preferences::new();
    let solidity_key = format!("{}", Language::Solidity);
    let vyper_key = format!("{}", Language::Vyper);
    preferences.set_preferences_for_key(&solidity_key, solidity_prefs);
    preferences.set_preferences_for_key(&vyper_key, vyper_prefs);

    preferences
}

/// Run the mutation generator algorithm.
///
/// # Arguments
///
/// * `args` - The command line arguments that control the mutation algorithm.
pub fn generate_mutants(args: MutateCLArgs) -> Result<(), MetamorphError> {
    // Select the mutation algorithms to use while generating mutations.  Args.all_mutations takes
    // precedence over individual algorithms selected in args.mutations.
    let mutations: Vec<MutationType> = if args.all_mutations {
        get_all_mutation_algorithms()
    } else {
        // Change the algorithm strings from the command line into actual MutationType values.
        get_mutation_types_from_strings(&args.mutations)
    };

    // We allow the user to pass in compilers for each supported language on the command line.
    let compiler_paths = CompilerPaths {
        solidity: &args.solidity_compiler,
        vyper: &args.vyper_compiler,
    };

    let mut preferences = generate_preferences_compiler_paths(&compiler_paths);

    let mut generator_parameters: Vec<GeneratorParameters> = Vec::new();

    for file_name in args.file_names {
        let mut actual_preferences = preferences.clone();
        let actual_file_name = file_name.clone();
        let mut actual_mutations = mutations.clone();
        let mut actual_functions = args.functions.clone();
        let mut actual_number_of_mutants = args.num_mutants;
        let mut actual_verify = args.validate_mutants;

        // Select a random number generator seed based on args.rng_seed. If args.rng_seed is less
        // than 0, then use a seed based off of time (we don't need cryptographic security).
        let mut seed: u64 = if args.rng_seed < 0 {
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

        // Try to recognize the language of the source file.  The file might be a source code file,
        // an AST file, or a configuration file.
        let recognizer = Recognizer::new(&mut preferences);
        let recognize_result = recognizer.recognize_file(&actual_file_name)?;

        if recognize_result.file_type == FileType::Config {
            // If we have a config file, then we need to extract the mutation parameters from
            // the configuration file.
            let configuration_details = ConfigurationFileDetails::new_from_file(&actual_file_name)?;

            if let Some(compiler_details) = &configuration_details.compiler_details {
                let language_key = format!["{}", recognize_result.language];
                if let Some(mut language_preferences) =
                    actual_preferences.get_preferences_for_key(&language_key)
                {
                    language_preferences
                        .set_preferences_for_key(COMPILER_KEY, compiler_details.clone());
                    actual_preferences.set_preferences_for_key(&language_key, language_preferences);
                }
            }

            actual_number_of_mutants = configuration_details.number_of_mutants as usize;

            // Here we update the random number generator from the configuration details seed if the
            // details have a new seed.
            if let Some(s) = configuration_details.seed {
                seed = s;
            };

            // Check to see if the configuration file requested a different set of mutation algorithms.
            if configuration_details.all_mutations {
                actual_mutations = get_all_mutation_algorithms();
            } else if !configuration_details.mutations.is_empty() {
                actual_mutations = configuration_details.mutations.clone();
            }

            if !configuration_details.functions.is_empty() {
                actual_functions = configuration_details.functions;
            }

            actual_verify = configuration_details.verify_mutants;

            // The configuration files can have multiple files to mutate using the same settings
            // for each file. Go through the filenames list and add a generator parameter object
            // for each file in the list.
            for path_buf in configuration_details.filenames {
                let file_to_mutate_name = String::from(path_buf.to_str().unwrap());

                let generator_params = GeneratorParameters {
                    file_name: file_to_mutate_name,
                    number_of_mutants: actual_number_of_mutants,
                    rng_seed: seed,
                    rng: Pcg64::seed_from_u64(seed),
                    output_directory: PathBuf::from_str(&args.output_directory).unwrap(),
                    use_stdout: args.stdout,
                    mutations: actual_mutations.clone(),
                    verify_mutant_viability: actual_verify,
                    print_original: args.print_original,
                    save_configuration_file: args.save_config_files,
                    preferences: actual_preferences.clone(),
                    functions: actual_functions.clone(),
                };

                generator_parameters.push(generator_params);
            }
        } else {
            let generator_params = GeneratorParameters {
                file_name: actual_file_name,
                number_of_mutants: actual_number_of_mutants,
                rng_seed: seed,
                rng: Pcg64::seed_from_u64(seed),
                output_directory: PathBuf::from_str(&args.output_directory).unwrap(),
                use_stdout: args.stdout,
                mutations: actual_mutations,
                verify_mutant_viability: actual_verify,
                print_original: args.print_original,
                save_configuration_file: args.save_config_files,
                preferences: actual_preferences,
                functions: actual_functions,
            };

            generator_parameters.push(generator_params);
        }
    }

    // Now, for each set of parameters, invoke the mutator.
    for params in &mut generator_parameters {
        generate_mutations(params)?;
    }

    Ok(())
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
    let recognizer = Recognizer::new(&mut params.preferences);
    let recognize_result = recognizer.recognize_file(&params.file_name)?;

    let mut language_object =
        LanguageInterface::get_language_object_for_language(&recognize_result.language)?;

    // create the mutation permissions
    let function_mutation_permissions = convert_function_names_to_permissions(&params.functions);

    if !params.mutations.is_empty() {
        log::info!(
            "Generating mutations using algorithms: {:?}",
            get_mutation_strings_from_types(&params.mutations)
        );
    }

    let ast = language_object.load_ast_from_file(
        &params.file_name,
        &recognize_result.file_type,
        &params.preferences,
    )?;

    language_object.select_mutators_for_mutation_types(&params.mutations)?;

    let mutable_nodes_table =
        language_object.count_mutable_nodes(&ast, &function_mutation_permissions)?;

    if mutable_nodes_table.is_empty() {
        return Err(MetamorphError::NoMutableNode);
    }

    // Only pretty-print the original file after verifying that we can load the AST, and that
    // we have valid mutators for the AST.
    if params.print_original {
        if params.use_stdout {
            let mut stdout = std::io::stdout();
            pretty_print_ast_to_stream(&ast, &mut stdout)?;
        } else {
            let original_file = PathBuf::from_str(&params.file_name).unwrap();
            pretty_print_ast(&ast, &params.file_name, &params.output_directory)?;
            log::info!(
                "Pretty-printing original file {:?} to {}",
                original_file.file_name().unwrap(),
                &params.output_directory.to_str().unwrap()
            );
        }
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

        let filenames: Vec<PathBuf> = vec![PathBuf::from(params.file_name.clone())];

        let details = ConfigurationFileDetails {
            language: Some(recognize_result.language),
            filenames,
            number_of_mutants: params.number_of_mutants as i64,
            seed: Some(params.rng_seed),
            mutations: params.mutations.clone(),
            all_mutations: false,
            compiler_details,
            functions: params.functions.clone(),
            verify_mutants: params.verify_mutant_viability,
        };

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

        if params.use_stdout {
            let mut stdout = std::io::stdout();
            details.write_to_stream_as_json(&mut stdout)?;
        } else {
            log::info!(
                "Writing configuration file {}",
                out_file_path.to_str().unwrap()
            );
            details.write_to_file_as_json(out_file_path.to_str().unwrap())?;
        }
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
            let mutated_ast = language_object.mutate_ast(
                &ast,
                mutation_type,
                index,
                &mut params.rng,
                &function_mutation_permissions,
            )?;

            // See if we have already generated this AST before.  We only want to output unique
            // mutations.
            if observed_asts.contains(&mutated_ast) {
                attempts += 1;
                continue;
            }

            if params.verify_mutant_viability
                && !language_object.mutant_compiles(&mutated_ast, &params.preferences)
            {
                attempts += 1;
                continue;
            }

            if params.use_stdout {
                let mut stdout = std::io::stdout();
                pretty_print_ast_to_stream(&mutated_ast, &mut stdout)?;
            } else {
                // Calculate the name of the output file.
                let input_file_path = PathBuf::from(&params.file_name);
                let base_file_name = input_file_path.file_name().unwrap();
                let outfile_name = params.output_directory.join(
                    String::from(base_file_name.to_str().unwrap())
                        + "_"
                        + &files_written.to_string(),
                );

                let outfile = String::from(outfile_name.to_str().unwrap());

                let final_file =
                    pretty_print_ast(&mutated_ast, &outfile, &params.output_directory)?;

                log::info!(
                    "{} used to create mutant written to {}",
                    mutation_type,
                    final_file.to_str().unwrap()
                );
            }

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
