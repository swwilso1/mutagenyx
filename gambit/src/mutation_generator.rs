use crate::generator_parameters::GeneratorParameters;
use gambit_lib::error::GambitError;
use gambit_lib::language_interface::*;
use gambit_lib::mutation::MutationType;
use gambit_lib::pretty_printer::PrettyPrinter;
use gambit_lib::recognizer::Recognizer;
use gambit_lib::super_ast::SuperAST;
use rand::seq::SliceRandom;
use rand::RngCore;
use std::collections::VecDeque;
use std::path::PathBuf;

static ATTEMPTS_TO_GENERATE_A_MUTANT: usize = 50;

pub fn generate_mutations(params: &mut GeneratorParameters) -> Result<(), GambitError> {
    // Try to recognize the language of the source file.  The file might be a source code file
    // or perhaps an AST file.
    let mut recognized_language = Recognizer::recognize_source_file(&params.file_name);
    if recognized_language == None {
        recognized_language = Recognizer::recognize_ast_file(&params.file_name);
        if recognized_language == None {
            return Err(GambitError::LanguageNotRecognized);
        }
    }

    let language = recognized_language.unwrap();

    let mut language_object = LanguageInterface::get_language_object_for_language(&language)?;

    let ast = language_object.load_ast_from_file(&params.file_name)?;

    let _select_result = language_object.select_mutators_for_mutation_types(&params.mutations)?;

    let mutable_nodes_table = language_object.count_mutable_nodes(&ast)?;

    if mutable_nodes_table.is_empty() {
        return Err(GambitError::NoMutableNode);
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

            let mutated_ast = language_object.mutate_ast(&ast, mutation_type, index, params.rng)?;

            if observed_asts.contains(&mutated_ast) {
                attempts += 1;
                continue;
            }

            let input_file_path = PathBuf::from(&params.file_name);
            let base_file_name = input_file_path.file_name().unwrap();
            let outfile_name = params.output_directory.join(
                String::from(base_file_name.to_str().unwrap())
                    + "_"
                    + &files_written.to_string()
                    + "."
                    + language_object.get_extension_for_output_file(),
            );

            let outfile = String::from(outfile_name.to_str().unwrap());

            // Try to create the output directory:
            if let Err(e) = std::fs::create_dir_all(outfile_name.parent().unwrap()) {
                return Err(GambitError::from(e));
            }

            let mut pretty_printer = PrettyPrinter::new(4, 150, "\n");
            let _write_result = language_object.pretty_print_mutated_ast_to_file(
                &mutated_ast,
                &outfile,
                &mut pretty_printer,
            )?;

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
