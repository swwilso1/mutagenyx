//! The `pretty_printing` module provides services for pretty-printing the input source or AST
//! to the tool's pretty-printing format.  Use these services to change the input file into a form
//! that you can easily compare with the generated mutants using a diff tool.

use crate::PrettyPrintCLArgs;
use metamorph_lib::error::MetamorphError;
use metamorph_lib::language_interface::*;
use metamorph_lib::preferences::{PreferenceValue, Preferences};
use metamorph_lib::pretty_printer::PrettyPrinter;
use metamorph_lib::recognizer::Recognizer;
use metamorph_lib::super_ast::language_for_ast;
use metamorph_lib::SuperAST;
use std::{path::PathBuf, str::FromStr};

/// Iterate through the files in the args.file_names vector and pretty-print each file.
///
/// # Arguments
///
/// * `args` - The [`PrettyPrintCLArgs`] object.
pub fn pretty_print_files(args: PrettyPrintCLArgs) {
    let mut preferences = Preferences::new();
    preferences.set_value_for_key(
        "solidity_compiler",
        PreferenceValue::String(args.solidity_compiler),
    );
    preferences.set_value_for_key(
        "vyper_compiler",
        PreferenceValue::String(args.vyper_compiler),
    );

    for file_name in args.file_names {
        let original_file = PathBuf::from_str(&file_name).unwrap();
        let original_file_str = original_file.file_name().unwrap();
        match pretty_print_file(&file_name, &args.output_directory, &preferences) {
            Ok(_buf) => log::info!(
                "Pretty-printing original file {:?} to {}",
                original_file_str,
                &args.output_directory
            ),
            Err(e) => println!("Unable to pretty-print {:?}: {}", original_file_str, e),
        }
    }
}

/// Pretty-print an individual file.
///
/// # Arguments
///
/// * `file_name` - The path to the file to pretty-print in the file system.
/// * `output_directory` - The path to the location to save the pretty-printed file.
/// * `preferences` - [`Preferences`] object.
pub fn pretty_print_file(
    file_name: &String,
    output_directory: &String,
    preferences: &Preferences,
) -> Result<PathBuf, MetamorphError> {
    // Convert the output_directory to a PathBuf
    let out_dir = PathBuf::from_str(output_directory).unwrap();

    let recognizer = Recognizer::new(&preferences);

    // Recognize the language.
    let recognize_result = recognizer.recognize_file(file_name)?;

    // Get the language interface object for the language.
    let mut language_object =
        LanguageInterface::get_language_object_for_language(&recognize_result.language)?;

    // TODO: We need to have the module that loads either source or AST.
    // Load the ast.
    let ast =
        language_object.load_ast_from_file(file_name, &recognize_result.file_type, &preferences)?;

    // Calculate the name of the output file.
    let input_file_path = PathBuf::from(file_name);
    let base_file_name = input_file_path.file_name().unwrap();
    let file_extension = String::from(".") + language_object.get_extension_for_output_file();
    let sfile_name = String::from(file_name);
    let extension = if sfile_name.ends_with(file_extension.as_str()) {
        ""
    } else {
        file_extension.as_str()
    };
    let outfile_name = out_dir.join(String::from(base_file_name.to_str().unwrap()) + extension);

    let outfile = String::from(outfile_name.to_str().unwrap());

    // Try to create the output directory:
    if let Err(e) = std::fs::create_dir_all(outfile_name.parent().unwrap()) {
        return Err(MetamorphError::from(e));
    }

    // Create a pretty printer for printing this AST.
    let mut pretty_printer = PrettyPrinter::new(4, 150);
    let _write_result =
        language_object.pretty_print_ast_to_file(&ast, &outfile, &mut pretty_printer)?;

    Ok(outfile_name)
}

/// Pretty print an AST to the file named `file_name` in `output_dir`.
///
/// If `file_name` is a complete path, the function will take the basename of the path and use
/// the basename for filename.
///
/// # Arguments
///
/// * `ast` - Any reference to a [`SuperAST`] abstract syntax tree.
/// * `file_name` - A reference to the text of the file name. May be a complete path.
/// * `output_dir` - The directory in which to place the pretty-printed ast.
pub fn pretty_print_ast(
    ast: &SuperAST,
    file_name: &str,
    output_dir: &PathBuf,
) -> Result<PathBuf, MetamorphError> {
    let language = language_for_ast(ast);

    let mut language_object = LanguageInterface::get_language_object_for_language(&language)?;

    let input_file_name = PathBuf::from(file_name);
    let base_file_name = input_file_name.file_name().unwrap();
    let file_extension = String::from(".") + language_object.get_extension_for_output_file();
    let sfile_name = String::from(file_name);
    let extension = if sfile_name.ends_with(file_extension.as_str()) {
        ""
    } else {
        file_extension.as_str()
    };

    let outfile_name = output_dir.join(String::from(base_file_name.to_str().unwrap()) + extension);

    let outfile = String::from(outfile_name.to_str().unwrap());

    // Try to create the output directory:
    if let Err(e) = std::fs::create_dir_all(outfile_name.parent().unwrap()) {
        return Err(MetamorphError::from(e));
    }

    // Create a pretty printer for printing this AST.
    let mut pretty_printer = PrettyPrinter::new(4, 150);
    let _write_result =
        language_object.pretty_print_ast_to_file(&ast, &outfile, &mut pretty_printer)?;

    Ok(outfile_name)
}
