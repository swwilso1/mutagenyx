//! The `solidity::language_interface` module provides the implementation for the [`JSONLanguageDelegate<W>`]
//! trait and the function `get_solidity_sub_language_interface`.

use crate::error::GambitError;
use crate::json::*;
use crate::json_language_delegate::JSONLanguageDelegate;
use crate::mutator::*;
use crate::pretty_print_visitor::PrettyPrintVisitor;
use crate::pretty_printer::PrettyPrinter;
use crate::solidity::mutators::SolidityMutatorFactory;
use crate::solidity::pretty_printer::SolidityNodePrinterFactory;
use crate::super_ast::SuperAST;
use crate::visitor::Visitor;
use crate::Language;
use serde_json::Value;
use std::io::Write;

/// Return the object that conforms to [`JSONLanguageDelegate<W>`].
pub fn get_solidity_sub_language_interface<W: Write>() -> Box<dyn JSONLanguageDelegate<W>> {
    return Box::new(SolidityLanguageSubInterface {});
}

/// The type that implements [`JSONLanguageDelegate<W>`].
pub struct SolidityLanguageSubInterface {}

impl<W: Write> JSONLanguageDelegate<W> for SolidityLanguageSubInterface {
    fn recover_ast<'a>(&self, super_ast: &'a SuperAST) -> Result<&'a Value, GambitError> {
        let solidity_ast = match super_ast {
            SuperAST::Solidity(sast) => sast,
            _ => return Err(GambitError::ASTTypeNotSupported),
        };
        Ok(solidity_ast)
    }

    fn get_value_as_super_ast(&self, value: Value) -> Result<SuperAST, GambitError> {
        if <SolidityLanguageSubInterface as JSONLanguageDelegate<W>>::json_is_language_ast_json(
            self, &value,
        ) {
            return Ok(SuperAST::Solidity(value));
        }
        Err(GambitError::LanguageNotRecognized)
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

    fn implements(&self) -> Language {
        Language::Solidity
    }

    fn get_file_extension(&self) -> &str {
        return "sol";
    }
}
