//! The `solidity::language_interface` module provides the implementation for the [`JSONLanguageDelegate<W>`]
//! trait and the function `get_solidity_sub_language_interface`.

use crate::error::MetamorphError;
use crate::json::*;
use crate::json_language_delegate::JSONLanguageDelegate;
use crate::mutator::*;
use crate::pretty_print_visitor::PrettyPrintVisitor;
use crate::pretty_printer::PrettyPrinter;
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
pub fn get_solidity_sub_language_interface<W: Write>() -> Box<dyn JSONLanguageDelegate<W>> {
    return Box::new(SolidityLanguageSubInterface {});
}

/// The type that implements [`JSONLanguageDelegate<W>`].
pub struct SolidityLanguageSubInterface {}

impl<W: Write> JSONLanguageDelegate<W> for SolidityLanguageSubInterface {
    fn recover_ast<'a>(&self, super_ast: &'a SuperAST) -> Result<&'a Value, MetamorphError> {
        let solidity_ast = match super_ast {
            SuperAST::Solidity(sast) => sast,
            _ => return Err(MetamorphError::ASTTypeNotSupported),
        };
        Ok(solidity_ast)
    }

    fn get_value_as_super_ast(&self, value: Value) -> Result<SuperAST, MetamorphError> {
        if <SolidityLanguageSubInterface as JSONLanguageDelegate<W>>::json_is_language_ast_json(
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
        &self,
        w: &'a mut W,
        printer: &'a mut PrettyPrinter,
    ) -> Box<dyn Visitor<Value> + 'a> {
        return Box::new(PrettyPrintVisitor::new(
            w,
            printer,
            Box::new(SolidityNodePrinterFactory {}),
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

    fn convert_source_file_to_ast(&self, file_name: &str) -> Result<SuperAST, MetamorphError> {
        if let Ok(s) = file_is_source_file(file_name) {
            let value = load_json_from_file_with_name(&s)?;
            return <SolidityLanguageSubInterface as JSONLanguageDelegate<W>>::get_value_as_super_ast(self, value);
        }

        Err(MetamorphError::SourceDoesNotCompile(String::from(
            file_name,
        )))
    }

    fn file_is_language_source_file(&self, file_name: &str) -> bool {
        if let Ok(_) = file_is_source_file(file_name) {
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
}

/// Try to execute the Solidity compiler on the command line.
///
/// # Arguments
///
/// * `file_name` - The path to the source file to compile.
fn file_is_source_file(file_name: &str) -> Result<String, MetamorphError> {
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
    ]; //file_path.to_str().unwrap()];

    match shell_execute("solc8.11", args) {
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
