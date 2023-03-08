//! The `solidity::language_interface` module provides the implementation for the [`JSONLanguageDelegate`]
//! trait and the function `get_solidity_sub_language_interface`.

use crate::compiler_details::*;
use crate::error::MutagenyxError;
use crate::id::Id;
use crate::json::*;
use crate::json_ast_id_maker::JSONIDMaker;
use crate::json_ast_permitter::JSONPermitter;
use crate::json_comment_inserter::JSONCommentInserter;
use crate::json_language_delegate::JSONLanguageDelegate;
use crate::json_namer::JSONNamer;
use crate::mutation_visitor::NodePath;
use crate::mutator::*;
use crate::namer::Namer;
use crate::node_printer::NodePrinterFactory;
use crate::permissions::Permissions;
use crate::permit::Permit;
use crate::preferences::*;
use crate::pretty_print_visitor::PrettyPrintVisitor;
use crate::pretty_printer::PrettyPrinter;
use crate::solidity::ast::SolidityAST;
use crate::solidity::commenter::SolidityCommenterFactory;
use crate::solidity::compiler_details::{
    ALLOW_PATHS_KEY, BASE_PATH_KEY, INCLUDE_PATHS_KEY, REMAPPINGS_KEY,
};
use crate::solidity::mutators::SolidityMutatorFactory;
use crate::solidity::node_finder::SolidityNodeFinderFactory;
use crate::solidity::pretty_printer::SolidityNodePrinterFactory;
use crate::super_ast::SuperAST;
use crate::utility::shell_execute;
use crate::visitor::Visitor;
use crate::Language;
use serde_json::Value;
use std::env;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

/// The type that implements [`JSONLanguageDelegate`].
pub struct SolidityLanguageSubDelegate {
    node_printer_factory: Box<dyn NodePrinterFactory<SolidityAST>>,
}

impl SolidityLanguageSubDelegate {
    // Create a new Solidity language delegate.
    pub fn new() -> SolidityLanguageSubDelegate {
        SolidityLanguageSubDelegate {
            node_printer_factory: Box::new(SolidityNodePrinterFactory::default()),
        }
    }
}

impl JSONLanguageDelegate for SolidityLanguageSubDelegate {
    fn recover_ast<'a>(&self, super_ast: &'a SuperAST) -> Result<&'a Value, MutagenyxError> {
        let solidity_ast = match super_ast {
            SuperAST::Solidity(sast) => sast,
            _ => return Err(MutagenyxError::ASTTypeNotSupported),
        };
        Ok(solidity_ast)
    }

    fn get_value_as_super_ast(&self, value: Value) -> Result<SuperAST, MutagenyxError> {
        if <SolidityLanguageSubDelegate as JSONLanguageDelegate>::json_is_language_ast_json(
            self, &value,
        ) {
            return Ok(SuperAST::Solidity(value));
        }
        Err(MutagenyxError::LanguageNotRecognized)
    }

    fn get_mutator_factory(&self) -> Box<dyn MutatorFactory<Value>> {
        Box::new(SolidityMutatorFactory {})
    }

    fn get_pretty_print_visitor<'a>(
        &'a self,
        w: &'a mut dyn Write,
        printer: &'a mut PrettyPrinter,
    ) -> Box<dyn Visitor<Value> + 'a> {
        return Box::new(PrettyPrintVisitor::new(
            w,
            printer,
            self.node_printer_factory.as_ref(),
        ));
    }

    fn json_is_language_ast_json(&self, value: &Value) -> bool {
        if let Some(node_type) = value.get_str_for_key("nodeType") {
            if node_type == "SourceUnit" {
                return true;
            }
        }
        false
    }

    fn convert_source_file_to_ast(
        &self,
        file_name: &str,
        prefs: &Preferences,
    ) -> Result<SuperAST, MutagenyxError> {
        if let Ok(s) = file_is_source_file(file_name, prefs) {
            let value = load_json_from_file_with_name(&s)?;
            return <SolidityLanguageSubDelegate as JSONLanguageDelegate>::get_value_as_super_ast(
                self, value,
            );
        }

        Err(MutagenyxError::SourceDoesNotCompile(String::from(
            file_name,
        )))
    }

    fn file_is_language_source_file(&self, file_name: &str, prefs: &Preferences) -> bool {
        file_is_source_file(file_name, prefs).is_ok()
    }

    fn implements(&self) -> Language {
        Language::Solidity
    }

    fn get_file_extension(&self) -> &str {
        "sol"
    }

    fn default_compiler_settings(&self) -> Preferences {
        let mut preferences = Preferences::new();
        preferences.set_string_for_key(PATH_KEY, "solc");
        preferences
    }

    fn get_node_permitter<'a>(
        &'a self,
        permissions: &'a Permissions,
    ) -> Box<dyn Permit<Value> + '_> {
        Box::new(JSONPermitter::new(permissions))
    }

    fn mutant_compiles(&self, file_name: &str, prefs: &Preferences) -> bool {
        file_compiles(file_name, prefs)
    }

    fn get_node_id_maker(&self) -> Box<dyn Id<Value>> {
        Box::new(JSONIDMaker::new(|n| {
            n.get_int_for_key("id").map(|id| id as u64)
        }))
    }

    fn insert_comment_by_path(&self, ast: &mut Value, comment_node: Value, node_path: &NodePath) {
        let id_maker = self.get_node_id_maker();
        let finder_factory = Box::new(SolidityNodeFinderFactory {});
        let commenter_factory = Box::new(SolidityCommenterFactory {});
        let json_comment_inserter =
            JSONCommentInserter::new(finder_factory, commenter_factory, id_maker);
        json_comment_inserter.insert_comment_by_path(ast, comment_node, node_path)
    }

    fn get_namer(&self) -> Box<dyn Namer<Value>> {
        Box::new(JSONNamer::new(|v| {
            v.get_str_for_key("name").map(String::from)
        }))
    }
}

/// Retrieve Solidity compiler flags from a [`Preferences`] object.
///
/// # Arguments
///
/// * `prefs` - The [`Preferences`] object.
fn get_solidity_compiler_flags_from_preferences(prefs: &Preferences) -> Vec<String> {
    let mut args: Vec<String> = Vec::new();

    let language_key = format!["{}", Language::Solidity];
    if let Some(language_prefs) = prefs.get_preferences_for_key(&language_key) {
        if let Some(compiler_prefs) = language_prefs.get_preferences_for_key(COMPILER_KEY) {
            if let Some(base_path) = compiler_prefs.get_string_for_key(BASE_PATH_KEY) {
                args.push(String::from("--base-path"));
                args.push(base_path);
            }
            if let Some(include_path_array) = compiler_prefs.get_array_for_key(INCLUDE_PATHS_KEY) {
                for path in &include_path_array {
                    if let PreferenceValue::String(s) = path {
                        args.push(String::from("--include-path"));
                        args.push(s.clone());
                    }
                }
            }
            if let Some(allow_paths_array) = compiler_prefs.get_array_for_key(ALLOW_PATHS_KEY) {
                if !allow_paths_array.is_empty() {
                    args.push(String::from("--allow-paths"));
                    let mut i: usize = 0;
                    while i < allow_paths_array.len() {
                        let value = &allow_paths_array[i];
                        if let PreferenceValue::String(s) = value {
                            let mut path_value = s.clone();
                            if i < (allow_paths_array.len() - 1) {
                                path_value += ",";
                            }
                            args.push(path_value);
                        }
                        i += 1;
                    }
                }
            }
            if let Some(remappings_array) = compiler_prefs.get_array_for_key(REMAPPINGS_KEY) {
                for mapping in &remappings_array {
                    if let PreferenceValue::String(s) = mapping {
                        args.push(s.clone());
                    }
                }
            }
        }
    }

    args
}

/// Retrieve the Solidity compiler from a [`Preferences`] object.
///
/// # Arguments
///
/// * `prefs` - The [`Preferences`] containing the compiler path.
fn get_solidity_compiler_from_preferences(prefs: &Preferences) -> String {
    let mut solidity_compiler = String::from("solc");

    let language_key = format!["{}", Language::Solidity];
    if let Some(language_prefs) = prefs.get_preferences_for_key(&language_key) {
        if let Some(compiler_prefs) = language_prefs.get_preferences_for_key(COMPILER_KEY) {
            if let Some(path) = compiler_prefs.get_string_for_key(PATH_KEY) {
                solidity_compiler = path;
            }
        }
    }

    solidity_compiler
}

/// Try to execute the Solidity compiler on the command line.
///
/// # Arguments
///
/// * `file_name` - The path to the source file to compile.
/// * `prefs` - The [`Preferences`] object that may contain a value for `solidity_compiler`.
fn file_is_source_file(file_name: &str, prefs: &Preferences) -> Result<String, MutagenyxError> {
    let file_path = PathBuf::from_str(file_name).unwrap();
    let base_name = file_path.file_name().unwrap().to_str().unwrap();
    let tmp_dir = env::temp_dir();
    let separator = String::from(std::path::MAIN_SEPARATOR);
    let out_path =
        String::from(tmp_dir.to_str().unwrap()) + separator.as_str() + base_name + "_json.ast";
    let args = vec![
        "--ast-compact-json",
        "--overwrite",
        "-o",
        tmp_dir.to_str().unwrap(),
        file_name,
    ];

    let solidity_compiler = get_solidity_compiler_from_preferences(prefs);
    let mut full_compiler_args: Vec<String> = get_solidity_compiler_flags_from_preferences(prefs);

    for arg in &args {
        full_compiler_args.push(String::from(*arg));
    }

    log::debug!(
        "Invoking Solidity compiler {} with {:?}",
        solidity_compiler,
        full_compiler_args
    );

    match shell_execute(&solidity_compiler, full_compiler_args) {
        Ok(output) => {
            if output.status.success() {
                Ok(out_path)
            } else {
                Err(MutagenyxError::SourceDoesNotCompile(String::from(
                    file_name,
                )))
            }
        }
        Err(_e) => Err(MutagenyxError::SourceDoesNotCompile(String::from(
            file_name,
        ))),
    }
}

/// Checks to see if the contents of file located at `file_name` compiles.
///
/// # Arguments
///
/// * `file_name` - The name of the file in the file system to compile.
/// * `prefs` - The [`Preferences`] object containing compiler settings.
fn file_compiles(file_name: &str, prefs: &Preferences) -> bool {
    let tmp_dir = env::temp_dir();
    let args = vec!["--overwrite", "-o", tmp_dir.to_str().unwrap(), file_name];

    let solidity_compiler = get_solidity_compiler_from_preferences(prefs);
    let mut full_compiler_args: Vec<String> = get_solidity_compiler_flags_from_preferences(prefs);

    for arg in &args {
        full_compiler_args.push(String::from(*arg));
    }

    log::debug!(
        "Attempting to compile {} with Solidity compiler '{}' and args: {:?}",
        file_name,
        solidity_compiler,
        full_compiler_args
    );

    match shell_execute(&solidity_compiler, full_compiler_args) {
        Ok(output) => {
            if !output.status.success() {
                let stdout_contents = core::str::from_utf8(output.stdout.as_slice()).unwrap();
                let stderr_contents = core::str::from_utf8(output.stderr.as_slice()).unwrap();
                log::debug!(
                    "Compilation failed:\n\tstdout: {}\n\tstderr: {}",
                    stdout_contents,
                    stderr_contents
                );
            }
            output.status.success()
        }
        Err(_e) => false,
    }
}
