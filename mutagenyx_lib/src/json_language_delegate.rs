//! The `json_language_delegate` provides an interface to language-specific behavior for each
//! language the has a JSON encoded AST.

use crate::error::MutagenyxError;
use crate::id::Id;
use crate::language::Language;
use crate::mutation_visitor::NodePath;
use crate::mutator::MutatorFactory;
use crate::namer::Namer;
use crate::permissions::Permissions;
use crate::permit::Permit;
use crate::preferences::Preferences;
use crate::pretty_printer::PrettyPrinter;
use crate::super_ast::SuperAST;
use crate::visitor::Visitor;
use serde_json::Value;
use std::io::Write;

/// Trait that defines a delegate interface for engaging language specific behavior.
pub trait JSONLanguageDelegate {
    /// Given a SuperAST enumeration, return the JSON Value object carried by the SuperAST object.
    ///
    /// # Arguments
    ///
    /// * `super_ast` - The SuperAST object that internally holds the JSON AST.
    fn recover_ast<'a>(&self, super_ast: &'a SuperAST) -> Result<&'a Value, MutagenyxError>;

    /// Checks a JSON AST for conformance to the language AST and if the JSON conforms,
    /// return the SuperAST wrapped JSON AST.
    ///
    /// # Arguments
    ///
    /// * `value` - The JSON AST.
    fn get_value_as_super_ast(&self, value: Value) -> Result<SuperAST, MutagenyxError>;

    /// Get a language-specific object that conforms to the [`MutatorFactory<T>`] trait for getting
    /// access to language specific [`crate::mutator::Mutator<T>`] objects.
    fn get_mutator_factory(&self) -> Box<dyn MutatorFactory<Value>>;

    /// Get a language-specific object that conforms to the [`Visitor<T>`] trait for traversing
    /// the AST for the purpose of pretty-printing the AST back to source code form.
    ///
    /// # Arguments
    ///
    /// * `w` - The [`Write`] trait object that will receive the formatted output.
    /// * `printer` - The [`PrettyPrinter`] object that will produce the formatted output.
    fn get_pretty_print_visitor<'a>(
        &'a self,
        w: &'a mut dyn Write,
        printer: &'a mut PrettyPrinter,
    ) -> Box<dyn Visitor<Value> + 'a>;

    /// Return true if the JSON AST in `value` conforms to the language AST specification.
    ///
    /// # Arguments
    ///
    /// * `value` - The JSON AST.
    fn json_is_language_ast_json(&self, value: &Value) -> bool;

    /// Convert the source code file in `file_name` to a SuperAST.
    ///
    /// # Arguments
    ///
    /// * `file_name` - The path to the source file in the file system
    fn convert_source_file_to_ast(
        &self,
        file_name: &str,
        prefs: &Preferences,
    ) -> Result<SuperAST, MutagenyxError>;

    /// Return true if the source file contains a program in the language.
    ///
    /// # Arguments
    ///
    /// * `file_name` - The location of the file on disk
    fn file_is_language_source_file(&self, file_name: &str, prefs: &Preferences) -> bool;

    /// Return the Language value implemented by the trait object.
    fn implements(&self) -> Language;

    /// Return the file extension to use with the generated source file name.
    fn get_file_extension(&self) -> &str;

    /// Return a [`Preferences`] object containing compiler default settings.
    fn default_compiler_settings(&self) -> Preferences;

    /// Return a [`Permit`] object that will contain permission settings for
    /// mutating an AST.
    fn get_node_permitter<'a>(
        &'a self,
        permissions: &'a Permissions,
    ) -> Box<dyn Permit<Value> + '_>;

    /// Return true if the program in `file_name` successfully compiles.
    ///
    /// # Arguments
    ///
    /// * `file_name` - The string slice referencing the text comprising the file name.
    /// * `prefs` - The [`Preferences`] object that contains compiler settings.
    fn mutant_compiles(&self, file_name: &str, prefs: &Preferences) -> bool;

    /// Return a trait object for [`Id<AST>`] that can uniquely identify a node in the AST.
    fn get_node_id_maker(&self) -> Box<dyn Id<Value>>;

    /// Find the correct node in `node_path` to insert `comment`.
    ///
    /// # Arguments
    ///
    /// * `ast` - The starting AST node.
    /// * `comment_node` - The node to insert into `ast`.
    /// * `node_path` - The path to the node that was mutated.
    fn insert_comment_by_path(
        &self,
        _ast: &mut Value,
        _comment_node: Value,
        _node_path: &NodePath,
    ) {
    }

    /// Return a trait object that can provide a name for JSON AST nodes.
    fn get_namer(&self) -> Box<dyn Namer<Value>>;
}
