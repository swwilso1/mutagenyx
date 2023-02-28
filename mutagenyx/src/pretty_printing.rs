//! The `pretty_printing` module provides services for pretty-printing the input source or AST
//! using the tool's pretty-printing format.  Use these services to change the input file into a form
//! that you can easily compare with the generated mutants using a diff tool.

use crate::compiler_settings::*;
use crate::PrettyPrintCLArgs;
use mutagenyx_lib::error::MutagenyxError;
use mutagenyx_lib::language_interface::*;
use mutagenyx_lib::preferences::Preferences;
use mutagenyx_lib::pretty_printer::PrettyPrinter;
use mutagenyx_lib::recognizer::Recognizer;
use mutagenyx_lib::super_ast::language_for_ast;
use mutagenyx_lib::SuperAST;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// Iterate through the files in the args.file_names vector and pretty-print each file.
///
/// # Arguments
///
/// * `args` - The [`PrettyPrintCLArgs`] object.
pub fn pretty_print_files(args: PrettyPrintCLArgs) {
    let solidity_compiler_settings = SolidityCompilerSettings {
        solidity_compiler: args.solidity_compiler,
        solidity_base_path: args.solidity_base_path,
        solidity_include_path: args.solidity_include_path,
        solidity_allow_paths: args.solidity_allow_paths,
        solidity_remappings: args.solidity_remappings,
    };

    let vyper_compiler_settings = VyperCompilerSettings {
        vyper_compiler: args.vyper_compiler,
        vyper_root_path: args.vyper_root_path,
    };
    let compiler_paths = CompilerSettings {
        solidity: solidity_compiler_settings,
        vyper: vyper_compiler_settings,
    };

    let mut preferences = compiler_paths.to_preferences();

    for file_name in args.file_names {
        if args.stdout {
            let mut stdout = std::io::stdout();
            match pretty_print_file_to_stream(&file_name, &mut stdout, &mut preferences) {
                Ok(_) => return,
                Err(e) => println!("Unable to pretty-print {}: {}", file_name, e),
            }
        } else {
            let original_file = PathBuf::from_str(&file_name).unwrap();
            let original_file_str = original_file.file_name().unwrap();
            match pretty_print_file(&file_name, &args.output_directory, &mut preferences) {
                Ok(_buf) => log::info!(
                    "Pretty-printing original file {:?} to {}",
                    original_file_str,
                    &args.output_directory
                ),
                Err(e) => println!("Unable to pretty-print {:?}: {}", original_file_str, e),
            }
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
    output_directory: &str,
    preferences: &mut Preferences,
) -> Result<PathBuf, MutagenyxError> {
    // Convert the output_directory to a PathBuf
    let out_dir = PathBuf::from_str(output_directory).unwrap();

    let recognizer = Recognizer::new(preferences);

    // Recognize the language.
    let recognize_result = recognizer.recognize_file(file_name)?;

    // Get the language interface object for the language.
    let mut language_object =
        LanguageInterface::get_language_object_for_language(&recognize_result.language)?;

    // TODO: We need to have the module that loads either source or AST.
    // Load the ast.
    let ast =
        language_object.load_ast_from_file(file_name, &recognize_result.file_type, preferences)?;

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
    std::fs::create_dir_all(outfile_name.parent().unwrap())?;

    // Create a pretty printer for printing this AST.
    let mut pretty_printer = PrettyPrinter::new(4, 150);
    language_object.pretty_print_ast_to_file(&ast, &outfile, &mut pretty_printer)?;

    Ok(outfile_name)
}

/// Pretty-print an individual file and write the output to `stream`.
///
/// # Arguments
///
/// * `file_name` - The name of the file to pretty-print.
/// * `stream` - The [`Write`] trait object that will receive the pretty-printed output.
/// * `preferences` - The [`Preferences`] object containing compiler settings.
pub fn pretty_print_file_to_stream(
    file_name: &str,
    stream: &mut dyn Write,
    preferences: &mut Preferences,
) -> Result<(), MutagenyxError> {
    let recognizer = Recognizer::new(preferences);

    // Recognize the language.
    let recognize_result = recognizer.recognize_file(file_name)?;

    // Get the language interface object for the language.
    let mut language_object =
        LanguageInterface::get_language_object_for_language(&recognize_result.language)?;

    // TODO: We need to have the module that loads either source or AST.
    // Load the ast.
    let ast =
        language_object.load_ast_from_file(file_name, &recognize_result.file_type, preferences)?;

    pretty_print_ast_to_stream(&ast, stream)
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
    output_dir: &Path,
) -> Result<PathBuf, MutagenyxError> {
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
    std::fs::create_dir_all(outfile_name.parent().unwrap())?;

    // Create a pretty printer for printing this AST.
    let mut pretty_printer = PrettyPrinter::new(4, 150);
    language_object.pretty_print_ast_to_file(ast, &outfile, &mut pretty_printer)?;

    Ok(outfile_name)
}

/// Pretty-print the AST in `ast` to the [`Write`] object `stream`
///
/// # Arguments
///
/// * `ast` - Any reference to a [`SuperAST`] abstract syntax tree.
/// * `stream` - A [`Write`] trait object that can receive the pretty-printed output.
pub fn pretty_print_ast_to_stream(
    ast: &SuperAST,
    stream: &mut dyn Write,
) -> Result<(), MutagenyxError> {
    let language = language_for_ast(ast);
    let mut language_object = LanguageInterface::get_language_object_for_language(&language)?;
    let mut pretty_printer = PrettyPrinter::new(4, 150);
    language_object.pretty_print_ast_to_stream(ast, stream, &mut pretty_printer)?;
    Ok(())
}
