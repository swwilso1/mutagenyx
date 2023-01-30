//! The `vyper::language_interface` module provides the implementation for the [`JSONLanguageDelegate<W>`]
//! trait and the function `get_vyper_sub_language_interface`.

use crate::error::MetamorphError;
use crate::json::*;
use crate::json_language_delegate::JSONLanguageDelegate;
use crate::language::Language;
use crate::mutator::*;
use crate::preferences::Preferences;
use crate::pretty_print_visitor::PrettyPrintVisitor;
use crate::pretty_printer::PrettyPrinter;
use crate::super_ast::SuperAST;
use crate::utility::shell_execute;
use crate::visitor::Visitor;
use crate::vyper::mutators::VyperMutatorFactory;
use crate::vyper::pretty_printer::VyperNodePrinterFactory;
use serde_json::Value;
use std::env;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

/// Return the object conforming to [`JSONLanguageDelegate<W>`]
pub fn get_vyper_sub_language_interface<W: Write>() -> Box<dyn JSONLanguageDelegate<W>> {
    return Box::new(VyperLanguageSubInterface {});
}

/// The type that implements [`JSONLanguageDelegate<W>`]
pub struct VyperLanguageSubInterface {}

impl<W: Write> JSONLanguageDelegate<W> for VyperLanguageSubInterface {
    fn recover_ast<'a>(&self, super_ast: &'a SuperAST) -> Result<&'a Value, MetamorphError> {
        let vyper_ast = match super_ast {
            SuperAST::Vyper(sast) => sast,
            _ => return Err(MetamorphError::ASTTypeNotSupported),
        };
        Ok(vyper_ast)
    }

    fn get_value_as_super_ast(&self, value: Value) -> Result<SuperAST, MetamorphError> {
        if <VyperLanguageSubInterface as JSONLanguageDelegate<W>>::json_is_language_ast_json(
            self, &value,
        ) {
            return Ok(SuperAST::Vyper(value));
        }
        Err(MetamorphError::LanguageNotRecognized)
    }

    fn get_mutator_factory(&self) -> Box<dyn MutatorFactory<Value>> {
        return Box::new(VyperMutatorFactory {});
    }

    fn get_pretty_print_visitor<'a>(
        &self,
        w: &'a mut W,
        printer: &'a mut PrettyPrinter,
    ) -> Box<dyn Visitor<Value> + 'a> {
        return Box::new(PrettyPrintVisitor::new(
            w,
            printer,
            Box::new(VyperNodePrinterFactory {}),
        ));
    }

    fn json_is_language_ast_json(&self, value: &Value) -> bool {
        if value.has_value_for_key("contract_name") && value.has_value_for_key("ast") {
            return true;
        }
        false
    }

    fn convert_source_file_to_ast(
        &self,
        file_name: &str,
        _prefs: &Preferences,
    ) -> Result<SuperAST, MetamorphError> {
        if let Ok(s) = file_is_source_file_with_vyper_from_pip(file_name) {
            let value = load_json_from_file_with_name(&s)?;
            return <VyperLanguageSubInterface as JSONLanguageDelegate<W>>::get_value_as_super_ast(
                self, value,
            );
        }

        if let Ok(s) = file_is_source_file_with_docker(file_name) {
            let value = load_json_from_file_with_name(&s)?;
            return <VyperLanguageSubInterface as JSONLanguageDelegate<W>>::get_value_as_super_ast(
                self, value,
            );
        }

        Err(MetamorphError::SourceDoesNotCompile(String::from(
            file_name,
        )))
    }

    fn file_is_language_source_file(&self, file_name: &str, _prefs: &Preferences) -> bool {
        if let Ok(_) = file_is_source_file_with_vyper_from_pip(file_name) {
            return true;
        }

        if let Ok(_) = file_is_source_file_with_docker(file_name) {
            return true;
        }

        false
    }

    fn implements(&self) -> Language {
        Language::Vyper
    }

    fn get_file_extension(&self) -> &str {
        return "vy";
    }
}

/// Try to execute the vyper compiler on the command line assuming that the user
/// has installed the pip version of the vyper compiler.
///
/// # Arguments
///
/// * `file_name` - The path to the source file to compile.
fn file_is_source_file_with_vyper_from_pip(file_name: &str) -> Result<String, MetamorphError> {
    let file_path = PathBuf::from_str(file_name).unwrap();
    let base_name = file_path.file_name().unwrap().to_str().unwrap();
    let tmp_dir = env::temp_dir();
    let out_path = String::from(tmp_dir.to_str().unwrap()) + base_name + ".json";
    let args = vec!["-f", "ast", "-o", out_path.as_str(), file_name];

    match shell_execute("vyper", args) {
        Ok(output) => {
            if output.status.success() {
                Ok(out_path)
            } else {
                println!(
                    "{}",
                    core::str::from_utf8(output.stderr.as_slice()).unwrap()
                );
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

/// Try to execute the vyper compiler in a docker container.
///
/// # Arguments
///
/// * `file_name` - The path to the source file to compile.
fn file_is_source_file_with_docker(file_name: &str) -> Result<String, MetamorphError> {
    // The docker command to invoke the Vyper compiler requires the path where the source file
    // is located to map into the container /code directory.
    let full_file_path = PathBuf::from_str(file_name).unwrap();
    let base_name = full_file_path.file_name().unwrap().to_str().unwrap();

    let tmp_dir = env::temp_dir();

    let out_path = String::from(tmp_dir.to_str().unwrap()) + base_name + ".json";

    let mut directory_name = full_file_path.clone();

    // Now get just the directory:
    directory_name.pop();

    let dir_name = directory_name.to_str().unwrap();

    let mut args: Vec<&str> = vec![];
    args.push("run");
    args.push("-v");

    let path_arg = format!("{dir_name}:/code");

    args.push(path_arg.as_str());
    args.push("vyperlang/vyper");
    args.push("-f");
    args.push("ast");
    args.push(file_name);
    args.push("-o");
    args.push(out_path.as_str());

    match shell_execute("docker", args) {
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
