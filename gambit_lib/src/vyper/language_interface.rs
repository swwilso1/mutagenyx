//! The `vyper::language_interface` module provides the implementation for the [`JSONLanguageDelegate<W>`]
//! trait and the function `get_vyper_sub_language_interface`.

use crate::error::GambitError;
use crate::json_language_delegate::JSONLanguageDelegate;
use crate::language::Language;
use crate::mutator::*;
use crate::pretty_printer::PrettyPrinter;
use crate::super_ast::SuperAST;
use crate::visitor::Visitor;
use crate::vyper::mutators::VyperMutatorFactory;
use crate::vyper::pretty_printer::VyperPrettyPrintVisitor;
use serde_json::Value;
use std::io::Write;

/// Return the object conforming to [`JSONLanguageDelegate<W>`]
pub fn get_vyper_sub_language_interface<W: Write>() -> Box<dyn JSONLanguageDelegate<W>> {
    return Box::new(VyperLanguageSubInterface {});
}

/// The type that implements [`JSONLanguageDelegate<W>`]
pub struct VyperLanguageSubInterface {}

impl<W: Write> JSONLanguageDelegate<W> for VyperLanguageSubInterface {
    fn recover_ast<'a>(&self, super_ast: &'a SuperAST) -> Result<&'a Value, GambitError> {
        let vyper_ast = match super_ast {
            SuperAST::Vyper(sast) => sast,
            _ => return Err(GambitError::ASTTypeNotSupported),
        };
        Ok(vyper_ast)
    }

    fn get_value_as_super_ast(&self, value: Value) -> Result<SuperAST, GambitError> {
        if <VyperLanguageSubInterface as JSONLanguageDelegate<W>>::json_is_language_ast_json(
            self, &value,
        ) {
            return Ok(SuperAST::Vyper(value));
        }
        Err(GambitError::LanguageNotRecognized)
    }

    fn get_mutator_factory(&self) -> Box<dyn MutatorFactory<Value>> {
        return Box::new(VyperMutatorFactory {});
    }

    fn get_pretty_print_visitor<'a>(
        &self,
        w: &'a mut W,
        printer: &'a mut PrettyPrinter,
    ) -> Box<dyn Visitor<Value> + 'a> {
        return Box::new(VyperPrettyPrintVisitor::new(w, printer));
    }

    fn json_is_language_ast_json(&self, _value: &Value) -> bool {
        false
    }

    fn implements(&self) -> Language {
        Language::Vyper
    }
}
