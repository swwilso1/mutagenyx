//! The `mutation_generator` module provides the `generate_mutations` function that implements
//! the mutation generation algorithm.

use crate::compiler_settings::*;
use crate::generator_parameters::GeneratorParameters;
use crate::pretty_printing::{pretty_print_ast, pretty_print_ast_to_stream};
use crate::MutateCLArgs;
use mutagenyx_lib::compiler_details::*;
use mutagenyx_lib::config_file::*;
use mutagenyx_lib::error::MutagenyxError;
use mutagenyx_lib::language_interface::*;
use mutagenyx_lib::mutation::{get_all_mutation_algorithms, MutationType};
use mutagenyx_lib::mutator_result::MutatorResult;
use mutagenyx_lib::permissions::*;
use mutagenyx_lib::recognizer::{FileType, Recognizer};
use rand::seq::SliceRandom;
use rand::RngCore;
use rand::SeedableRng;
use rand_pcg::*;
use std::collections::{HashMap, VecDeque};
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

    if !names.is_empty() {
        for function_name in names {
            // Give permission to mutate the node with the function name.
            permissions.add_permission(PermissionAction::Mutate(PermissionScope::Name(
                PermissionObject::Value(function_name.clone(), Permission::Allow),
            )));

            // Also grant permission to mutate any children of this node.
            permissions.add_permission(PermissionAction::Mutate(PermissionScope::Children(
                PermissionObject::Value(function_name.clone(), Permission::Allow),
            )));
        }
    } else {
        permissions.add_permission(PermissionAction::Mutate(PermissionScope::Any(
            Permission::Allow,
        )));
    }

    permissions.add_permission(PermissionAction::Visit(PermissionScope::Any(
        Permission::Allow,
    )));

    permissions
}

/// Run the mutation generator algorithm.
///
/// # Arguments
///
/// * `args` - The command line arguments that control the mutation algorithm.
pub fn generate_mutants(args: MutateCLArgs) -> Result<(), MutagenyxError> {
    // Select the mutation algorithms to use while generating mutations.  Args.all_mutations takes
    // precedence over individual algorithms selected in args.mutations.
    let mutations: Vec<MutationType> = if args.all_mutations {
        get_all_mutation_algorithms()
    } else {
        // Change the algorithm strings from the command line into actual MutationType values.
        get_mutation_types_from_strings(&args.mutation)
    };

    let solidity_compiler_settings = SolidityCompilerSettings {
        solidity_compiler: args.solidity_compiler,
        solidity_base_path: args.solidity_base_path,
        solidity_include_path: args.solidity_include_path,
        solidity_allow_paths: args.solidity_allow_paths,
        solidity_remappings: args.solidity_remapping,
    };

    let vyper_compiler_settings = VyperCompilerSettings {
        vyper_compiler: args.vyper_compiler,
        vyper_root_path: args.vyper_root_path,
    };

    // We allow the user to pass in compilers for each supported language on the command line.
    let compiler_paths = CompilerSettings {
        solidity: solidity_compiler_settings,
        vyper: vyper_compiler_settings,
    };

    let mut preferences = compiler_paths.to_preferences();

    let mut generator_parameters: Vec<GeneratorParameters> = Vec::new();

    // Select a random number generator seed based on args.rng_seed. If args.rng_seed is less
    // than 0, then use a seed based off of time (we don't need cryptographic randomness).
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

    let pcg = Pcg64::seed_from_u64(seed);

    for file_name in args.file {
        let mut actual_preferences = preferences.clone();
        let actual_file_name = file_name.clone();
        let mut actual_mutations = mutations.clone();
        let mut actual_functions = args.function.clone();
        let mut actual_number_of_mutants = args.num_mutants;
        let mut actual_verify = args.validate_mutants;
        let mut actual_output_directory = PathBuf::from_str(&args.output_directory).unwrap();
        let mut actual_use_stdout = args.stdout;

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

            if let Some(output_directory) = configuration_details.output_directory {
                if output_directory.to_str() == Some("stdout") {
                    actual_use_stdout = true;
                } else {
                    actual_output_directory = output_directory.clone();
                }
            }

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
                    output_directory: actual_output_directory.clone(),
                    use_stdout: actual_use_stdout,
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
                rng: pcg.clone(),
                output_directory: actual_output_directory,
                use_stdout: actual_use_stdout,
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
fn generate_mutations(params: &mut GeneratorParameters) -> Result<(), MutagenyxError> {
    // Try to recognize the language of the source file.  The file might be a source code file
    // or perhaps an AST file.
    let recognizer = Recognizer::new(&mut params.preferences);
    let recognize_result = recognizer.recognize_file(&params.file_name)?;

    let mut language_object =
        LanguageInterface::get_language_object_for_language(&recognize_result.language)?;

    // create the mutation permissions
    let function_mutation_permissions = convert_function_names_to_permissions(&params.functions);

    let ast = language_object.load_ast_from_file(
        &params.file_name,
        &recognize_result.file_type,
        &params.preferences,
    )?;

    language_object.select_mutators_for_mutation_types(&params.mutations)?;

    let mutable_nodes_table = language_object.count_mutable_nodes(
        &ast,
        &mut params.rng,
        &function_mutation_permissions,
    )?;

    // Calculate the paths to each node in the AST by node id.  The mutation step may make
    // use of this information to insert comments in the AST.
    let node_path_map =
        language_object.calculate_node_paths(&ast, &function_mutation_permissions)?;

    if mutable_nodes_table.is_empty() {
        return Err(MutagenyxError::NoMutableNode);
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
            println!(
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
            output_directory: Some(params.output_directory.clone()),
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
            println!(
                "Writing configuration file {}",
                out_file_path.to_str().unwrap()
            );
            details.write_to_file_as_json(out_file_path.to_str().unwrap())?;
        }
    }

    // This list now holds the mutation types for which the AST has nodes to mutate.
    let mut mutation_type_list: Vec<MutationType> = mutable_nodes_table
        .iter()
        .filter(|(_, v)| **v > 0)
        .map(|(k, _)| *k)
        .collect();

    // Sort the list so we get more deterministic behavior when selecting mutation algorithms.
    mutation_type_list.sort();

    println!(
        "Of the requested mutation algorithms, the AST contains nodes for {:?}",
        get_mutation_strings_from_types(&mutation_type_list)
    );

    // Now we generate a list of mutation types of length self.parameters.number_of_mutants
    // with mutation types chosen randomly from list of usable mutation types.
    let mut mutation_kinds_todo: VecDeque<MutationType> = VecDeque::new();

    // We can have a case where the user requests multiple mutants from only one mutation algorithm
    // and the AST has only a small number of mutable nodes for that algorithm (where smaller means
    // some number less than the requested number of mutants).  If that case happens we do not want
    // the tool to spend time trying to generate unique mutants for the requested number of mutants
    // that exceeds the available number of mutable nodes.

    // The tuple values here represent:
    // 1 - The number of times the algorithm selector has selected the mutation type.
    // 2 - The number of nodes mutable by the mutation type in the AST.
    // 3 - True if the counting algorithm has displayed a message about reaching the limit
    // for the use of the mutation type and false if not.
    let mut selected_algorithm_map: HashMap<MutationType, (usize, usize, bool)> = HashMap::new();

    // fill the map.
    for mutation_type in &mutation_type_list {
        let number_of_mutable_nodes = mutable_nodes_table.get(mutation_type).unwrap();
        selected_algorithm_map.insert(*mutation_type, (0, *number_of_mutable_nodes, false));
    }

    // Now try to randomly select the mutation algorithms, but only allow the max number of
    // algorithm usages by the number of mutable nodes for each algorithm.
    let mut requested_mutants_remaining: usize = params.number_of_mutants;

    let mut viable_mutations_selected: usize = 0;
    let mut available_mutations: usize = 0;

    // Run through the selected_algorithm_map and sum the max possible mutations for each algorithm.
    for data_tuple in selected_algorithm_map.values() {
        available_mutations += data_tuple.1;
    }

    println!(
        "AST supports at most {} different mutations using the requested mutation algorithms",
        available_mutations
    );

    while requested_mutants_remaining > 0 {
        // Select a random algorithm
        let mutation_type = match mutation_type_list.choose(&mut params.rng) {
            Some(t) => t,
            None => continue,
        };

        // Get the tuple containing the total times the algorithm was used, the total mutable
        // nodes for the algorithm, and the boolean that captures whether this algorithm has
        // displayed a message about reaching the max allowed mutations for the mutation type.
        let data_tuple = selected_algorithm_map.get_mut(mutation_type).unwrap();

        if data_tuple.0 < data_tuple.1 {
            data_tuple.0 += 1;
            mutation_kinds_todo.push_back(*mutation_type);
            viable_mutations_selected += 1;
        } else {
            if viable_mutations_selected >= available_mutations {
                println!("Reached the limit of mutable nodes in the AST, lowering requested mutants by {} to {}", requested_mutants_remaining, mutation_kinds_todo.len());
                break;
            }

            // Only report on reaching the mutable node limit once.
            if !data_tuple.2 {
                log::debug!("Reached the maximum allowable usages of algorithm {}, trying another algorithm", mutation_type);
                data_tuple.2 = true;
            }
            continue;
        }

        requested_mutants_remaining -= 1;
    }

    let mut files_written: usize = 0;
    let mut attempts: usize = 0;
    let mut observed_mutator_results: Vec<MutatorResult> = Vec::new();

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
            let mutate_ast_result = language_object.mutate_ast(
                &ast,
                mutation_type,
                index,
                &mut params.rng,
                &function_mutation_permissions,
                &node_path_map,
            )?;

            // See if we have already seen this ast result before. This step prevents us from
            // making non-unique mutations.
            if observed_mutator_results.contains(&mutate_ast_result.mutator_result) {
                attempts += 1;
                continue;
            }

            let mutated_ast = mutate_ast_result.ast.unwrap();

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

                println!(
                    "{} used to create mutant written to {}",
                    mutation_type,
                    final_file.to_str().unwrap()
                );
            }

            // Remove the item from the top of the VecDeque.
            mutation_kinds_todo.remove(0);

            // Add the mutator result to the list of results that we have observed.
            observed_mutator_results.push(mutate_ast_result.mutator_result);

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
