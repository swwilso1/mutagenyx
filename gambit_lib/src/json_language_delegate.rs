//! The `json_language_delegate` provides an interface to language-specific behavior for each
//! language the has a JSON encoded AST.

use crate::error::GambitError;
use crate::language::Language;
use crate::mutator::MutatorFactory;
use crate::pretty_printer::PrettyPrinter;
use crate::super_ast::SuperAST;
use crate::visitor::Visitor;
use serde_json::Value;
use std::io::Write;

/// Trait that defines a delegate interface for engaging language specific behavior.
///
/// # Trait Argument
///
/// * `W` - a type bounded by the Write trait.
pub trait JSONLanguageDelegate<W: Write> {
    /// Given a SuperAST enumeration, return the JSON Value object carried by the SuperAST object.
    ///
    /// # Arguments
    ///
    /// * `super_ast` - The SuperAST object that internally holds the JSON AST.
    fn recover_ast<'a>(&self, super_ast: &'a SuperAST) -> Result<&'a Value, GambitError>;

    /// Checks a JSON AST for conformance to the language AST and if the JSON conforms,
    /// return the SuperAST wrapped JSON AST.
    ///
    /// # Arguments
    ///
    /// * `value` - The JSON AST.
    fn get_value_as_super_ast(&self, value: Value) -> Result<SuperAST, GambitError>;

    /// Get a language-specific object that conforms to the [`MutatorFactory<T>`] trait for getting
    /// access to language specific [`crate::mutator::Mutator<T>`] objects.
    fn get_mutator_factory(&self) -> Box<dyn MutatorFactory<Value>>;

    /// Get a language-specific object that conforms to the ['Visitor<T>`] trait for traversing
    /// the AST for the purpose of pretty-printing the AST back to source code form.
    ///
    /// # Arguments
    ///
    /// * `w` - The [`Write`] trait object that will receive the formatted output.
    /// * `printer` - The [`PrettyPrinter`] object that will produce the formatted output.
    fn get_pretty_print_visitor<'a>(
        &self,
        w: &'a mut W,
        printer: &'a mut PrettyPrinter,
    ) -> Box<dyn Visitor<Value> + 'a>;

    /// Return true if the JSON AST in `value` conforms to the language AST specification.
    ///
    /// # Arguments
    ///
    /// * `value` - The JSON AST.
    fn json_is_language_ast_json(&self, value: &Value) -> bool;

    /// Return the Language value implemented by the trait object.
    fn implements(&self) -> Language;

    /// Return the file extension to use with the generated source file name.
    fn get_file_extension(&self) -> &str;
}
