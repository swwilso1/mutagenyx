//! The `solidity::language_interface` module provides the implementation for the [`JSONLanguageDelegate<W>`]
//! trait and the function `get_solidity_sub_language_interface`.

use crate::compiler_details::*;
use crate::error::MetamorphError;
use crate::json::*;
use crate::json_language_delegate::JSONLanguageDelegate;
use crate::mutator::*;
use crate::node_printer::NodePrinterFactory;
use crate::preferences::*;
use crate::pretty_print_visitor::PrettyPrintVisitor;
use crate::pretty_printer::PrettyPrinter;
use crate::solidity::ast::SolidityAST;
use crate::solidity::compiler_details::{BASE_PATH_KEY, INCLUDE_PATHS_KEY, REMAPPINGS_KEY};
use crate::solidity::mutators::SolidityMutatorFactory;
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

/// Return the object that conforms to [`JSONLanguageDelegate<W>`].
pub fn get_solidity_delegate<W: Write + 'static>() -> Box<dyn JSONLanguageDelegate<W>> {
    return Box::new(SolidityLanguageSubDelegate::new());
}

/// The type that implements [`JSONLanguageDelegate<W>`].
pub struct SolidityLanguageSubDelegate<W: Write> {
    node_printer_factory: Box<dyn NodePrinterFactory<W, SolidityAST>>,
}

impl<W: Write> SolidityLanguageSubDelegate<W> {
    // Create a new Solidity language delegate.
    fn new() -> SolidityLanguageSubDelegate<W> {
        SolidityLanguageSubDelegate {
            node_printer_factory: Box::new(SolidityNodePrinterFactory::default()),
        }
    }
}

impl<W: Write> JSONLanguageDelegate<W> for SolidityLanguageSubDelegate<W> {
    fn recover_ast<'a>(&self, super_ast: &'a SuperAST) -> Result<&'a Value, MetamorphError> {
        let solidity_ast = match super_ast {
            SuperAST::Solidity(sast) => sast,
            _ => return Err(MetamorphError::ASTTypeNotSupported),
        };
        Ok(solidity_ast)
    }

    fn get_value_as_super_ast(&self, value: Value) -> Result<SuperAST, MetamorphError> {
        if <SolidityLanguageSubDelegate<W> as JSONLanguageDelegate<W>>::json_is_language_ast_json(
            self, &value,
        ) {
            return Ok(SuperAST::Solidity(value));
        }
        Err(MetamorphError::LanguageNotRecognized)
    }

    fn get_mutator_factory(&self) -> Box<dyn MutatorFactory<Value>> {
        return Box::new(SolidityMutatorFactory {});
    }

    fn get_pretty_print_visitor<'a>(
        &'a self,
        w: &'a mut W,
        printer: &'a mut PrettyPrinter,
    ) -> Box<dyn Visitor<Value> + 'a> {
        return Box::new(PrettyPrintVisitor::new(
            w,
            printer,
            &self.node_printer_factory,
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
    ) -> Result<SuperAST, MetamorphError> {
        if let Ok(s) = file_is_source_file(file_name, prefs) {
            let value = load_json_from_file_with_name(&s)?;
            return <SolidityLanguageSubDelegate<W> as JSONLanguageDelegate<W>>::get_value_as_super_ast(self, value);
        }

        Err(MetamorphError::SourceDoesNotCompile(String::from(
            file_name,
        )))
    }

    fn file_is_language_source_file(&self, file_name: &str, prefs: &Preferences) -> bool {
        if let Ok(_) = file_is_source_file(file_name, prefs) {
            return true;
        }
        false
    }

    fn implements(&self) -> Language {
        Language::Solidity
    }

    fn get_file_extension(&self) -> &str {
        return "sol";
    }

    fn default_compiler_settings(&self) -> Preferences {
        let mut preferences = Preferences::new();
        preferences.set_string_for_key(PATH_KEY, "solc");
        preferences
    }
}

/// Try to execute the Solidity compiler on the command line.
///
/// # Arguments
///
/// * `file_name` - The path to the source file to compile.
/// * `prefs` - The [`Preferences`] object that may contain a value for `solidity_compiler`.
fn file_is_source_file(file_name: &str, prefs: &Preferences) -> Result<String, MetamorphError> {
    let file_path = PathBuf::from_str(file_name).unwrap();
    let base_name = file_path.file_name().unwrap().to_str().unwrap();
    let tmp_dir = env::temp_dir();
    let out_path = String::from(tmp_dir.to_str().unwrap()) + base_name + "_json.ast";
    let args = vec![
        "--ast-compact-json",
        "--overwrite",
        "-o",
        tmp_dir.to_str().unwrap(),
        file_name,
    ];

    let mut full_compiler_args: Vec<String> = Vec::new();

    let mut solidity_compiler = String::from("solc");

    let language_key = format!["{}", Language::Solidity];
    if let Some(language_prefs) = prefs.get_preferences_for_key(&language_key) {
        if let Some(compiler_prefs) = language_prefs.get_preferences_for_key(COMPILER_KEY) {
            if let Some(path) = compiler_prefs.get_string_for_key(PATH_KEY) {
                solidity_compiler = path;
            }
            if let Some(base_path) = compiler_prefs.get_string_for_key(BASE_PATH_KEY) {
                full_compiler_args.push(String::from("--base-path"));
                full_compiler_args.push(base_path.clone());
            }
            if let Some(include_path_array) = compiler_prefs.get_array_for_key(INCLUDE_PATHS_KEY) {
                for path in &include_path_array {
                    match path {
                        PreferenceValue::String(s) => {
                            full_compiler_args.push(String::from("--include-path"));
                            full_compiler_args.push(s.clone());
                        }
                        _ => {}
                    }
                }
            }
            if let Some(remappings_array) = compiler_prefs.get_array_for_key(REMAPPINGS_KEY) {
                for mapping in &remappings_array {
                    match mapping {
                        PreferenceValue::String(s) => {
                            full_compiler_args.push(s.clone());
                        }
                        _ => {}
                    }
                }
            }
        }
    } else if let Some(compiler) = prefs.get_value_for_key("solidity_compiler") {
        match compiler {
            PreferenceValue::String(s) => solidity_compiler = s,
            _ => {}
        }
    }

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
                Err(MetamorphError::SourceDoesNotCompile(String::from(
                    file_name,
                )))
            }
        }
        Err(_e) => Err(MetamorphError::SourceDoesNotCompile(String::from(
            file_name,
        ))),
    }
}
