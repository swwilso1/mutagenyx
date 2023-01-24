//! The `pretty_printing` module provides services for pretty-printing the input source or AST
//! to the tool's pretty-printing format.  Use these services to change the input file into a form
//! that you can easily compare with the generated mutants using a diff tool.

use crate::PrettyPrintCLArgs;
use gambit_lib::error::GambitError;
use gambit_lib::language_interface::*;
use gambit_lib::pretty_printer::PrettyPrinter;
use gambit_lib::recognizer::Recognizer;
use std::{path::PathBuf, str::FromStr};

/// Iterate through the files in the args.file_names vector and pretty-print each file.
///
/// # Arguments
///
/// * `args` - The [`PrettyPrintCLArgs`] object.
pub fn pretty_print_files(args: PrettyPrintCLArgs) {
    for file_name in args.file_names {
        if let Err(e) = pretty_print_file(&file_name, &args.output_directory) {
            println!("Unable to pretty-print {}: {}", file_name, e);
        }
    }
}

/// Pretty-print an individual file.
///
/// # Arguments
///
/// * `file_name` - The path to the file to pretty-print in the file system.
/// * `output_directory` - The path to the location to save the pretty-printed file.
fn pretty_print_file(file_name: &String, output_directory: &String) -> Result<(), GambitError> {
    // Convert the output_directory to a PathBuf
    let out_dir = PathBuf::from_str(output_directory).unwrap();

    // Recognize the language.
    let language = Recognizer::recognize_file(file_name)?;

    // Get the language interface object for the language.
    let mut language_object = LanguageInterface::get_language_object_for_language(&language)?;

    // TODO: We need to have the module that loads either source or AST.
    // Load the ast.
    let ast = language_object.load_ast_from_file(file_name)?;

    // Calculate the name of the output file.
    let input_file_path = PathBuf::from(file_name);
    let base_file_name = input_file_path.file_name().unwrap();
    let outfile_name = out_dir.join(
        String::from(base_file_name.to_str().unwrap())
            + "."
            + language_object.get_extension_for_output_file(),
    );

    let outfile = String::from(outfile_name.to_str().unwrap());

    // Try to create the output directory:
    if let Err(e) = std::fs::create_dir_all(outfile_name.parent().unwrap()) {
        return Err(GambitError::from(e));
    }

    // Create a pretty printer for printing this AST.
    let mut pretty_printer = PrettyPrinter::new(4, 150, "\n");
    let _write_result =
        language_object.pretty_print_ast_to_file(&ast, &outfile, &mut pretty_printer)?;

    Ok(())
}