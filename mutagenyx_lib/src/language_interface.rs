//! The `language_interface` module provides abstractions to interact with a programming language
//! module for the purpose of generating mutations. The programming language abstraction provides
//! common APIs that a programming language module can implement and thereby plug-in to the library.

use crate::error::MutagenyxError;
use crate::json_ast_language_interface::JSONLanguageInterface;
use crate::language::Language;
use crate::mutation::MutationType;
use crate::mutation_visitor::NodePathMap;
use crate::permissions::Permissions;
use crate::preferences::Preferences;
use crate::pretty_printer::PrettyPrinter;
use crate::recognizer::FileType;
use crate::solidity::delegate::get_solidity_delegate;
use crate::super_ast::SuperAST;
use crate::vyper::delegate::get_vyper_delegate;
use rand_pcg::*;
use std::collections::HashMap;
use std::io::Write;

/// Trait that defines the interface to a programming language module.
pub trait MutableLanguage {
    /// Load a programming language abstract syntax tree from `file_name`.  The function will return
    /// a [`SuperAST`] enumeration that encapsulates the concrete abstract syntax tree.
    ///
    /// The function does not require the file referenced by `file_name` to contain an AST.
    /// The file may contain source code as well.  The adopter of the trait must provide code
    /// to handle either a source file or an AST.
    ///
    /// # Arguments
    ///
    /// * `file_name` - The string slice referencing the text containing the file name.
    /// * `file_type` - The type of the file.
    /// * `prefs` - The [`Preferences`] object.
    fn load_ast_from_file(
        &mut self,
        file_name: &str,
        file_type: &FileType,
        prefs: &Preferences,
    ) -> Result<SuperAST, MutagenyxError>;

    /// Prepare language specific mutator objects that will mutate the AST for the requested
    /// mutation algorithms. The function may return [`MutagenyxError::MutationAlgorithmNotSupported`].
    ///
    /// # Arguments
    ///
    /// * `mutation_types` - A vector of MutationType enumerations. Each MutationType value
    /// corresponds to a requested mutation algorithm.
    fn select_mutators_for_mutation_types(
        &mut self,
        mutation_types: &[MutationType],
    ) -> Result<(), MutagenyxError>;

    /// Traverse the abstract syntax tree `ast` to count the number of nodes in the tree that
    /// each requested mutation algorithm can mutate.  Return a [`HashMap`] of counts by
    /// [`MutationType`].
    ///
    /// This caller should call this function after `select_mutators_for_mutation_types`
    /// otherwise the interface object will not have a record of which mutations to use when
    /// looking for mutable nodes in the AST.
    ///
    /// # Arguments
    ///
    /// * `ast` - The [`SuperAST`] object that encapsulates the concrete language AST.
    /// * `rng` - The random number generator to use for selecting parts of nodes that have
    /// divergent paths (such as the choice between an if or else block).
    /// * `permissions` - A [`Permissions`] object containing permission settings that control
    /// how to count the nodes.
    fn count_mutable_nodes(
        &mut self,
        ast: &SuperAST,
        rng: &mut Pcg64,
        permissions: &Permissions,
    ) -> Result<HashMap<MutationType, usize>, MutagenyxError>;

    /// Make a copy of `ast`, traverse the copy and mutate one node in the AST using the
    /// `mutation_type` algorithm.  Return the mutated AST encapsulated in a [`SuperAST`] object.
    ///
    /// # Arguments
    ///
    /// * `ast` - The syntax tree to mutate.
    /// * `mutation_type` - The mutation algorithm to use to mutate `ast`.
    /// * `index` - The index of the node to mutate. An AST may contain multiple nodes that the
    /// mutators can mutate for a given algorithm. The `index` value indicates which node to mutate.
    /// The index values index nodes in AST traversal order.
    /// * `rng` - The random number generator for the mutator objects to use. The mutator
    /// objects may randomly select variations withing a mutation algorithm when mutating a node.
    /// * `permissions` - A [`Permissions`] object that controls how to mutate the ast.
    /// * `path_map` - A precalculated path object containing the paths to every node in the
    /// AST.
    fn mutate_ast(
        &mut self,
        ast: &SuperAST,
        mutation_type: &MutationType,
        index: usize,
        rng: &mut Pcg64,
        permissions: &Permissions,
        path_map: &NodePathMap,
    ) -> Result<SuperAST, MutagenyxError>;

    /// Pretty-print the contents of `ast` to the file named in `file_name`.
    ///
    /// # Arguments
    ///
    /// * `ast` - A [`SuperAST`] object encapsulating the concrete language AST.
    /// * `file_name` - The string slice referencing the text containing the file name.
    /// * `pretty_printer` - the [`PrettyPrinter`] object used to help write structured
    /// output to the file.
    fn pretty_print_ast_to_file(
        &mut self,
        ast: &SuperAST,
        file_name: &str,
        pretty_printer: &mut PrettyPrinter,
    ) -> Result<(), MutagenyxError>;

    /// Pretty-print the contents of `ast` to `stream`.
    ///
    /// # Arguments
    ///
    /// * `ast` - A [`SuperAST`] object encapsulating the concrete language AST.
    /// * `stream` - The [`Write`] object that will receive the printed output.
    /// * `pretty_printer` - the [`PrettyPrinter`] object used to help write structured output to
    /// `stream`.
    fn pretty_print_ast_to_stream(
        &mut self,
        ast: &SuperAST,
        handle: &mut dyn Write,
        pretty_printer: &mut PrettyPrinter,
    ) -> Result<(), MutagenyxError>;

    /// Return the file extension that a caller should append to a file name that the caller will
    /// later pass to `pretty_print_mutated_ast_to_file`.
    fn get_extension_for_output_file(&self) -> &str;

    /// Check that the contents of the file named by `file_name` contain a program written in
    /// the language implementing the trait.
    ///
    /// # Arguments
    ///
    /// * `file_name` - The string slice referencing the text containing the file name.
    /// * `prefs` - The [`Preferences`] object.
    fn file_is_language_source_file(&self, file_name: &str, prefs: &Preferences) -> bool;

    /// Convert the contents of the source file named by `file_name` to an AST object that the
    /// mutators can use.
    ///
    /// # Arguments
    /// * `file_name` - The path of the source file in the file system.
    fn convert_source_file_to_ast(
        &self,
        file_name: &str,
        prefs: &Preferences,
    ) -> Result<SuperAST, MutagenyxError>;

    /// Check that the contents of the file named by `file_name` contain an AST representation
    /// of a program for the language implementing the trait.
    ///
    /// # Arguments
    ///
    /// * `file_name` - The string slice referencing the text containing the file name.
    fn file_is_language_ast_file(&self, file_name: &str) -> bool;

    /// Return a [`Preferences`] object containing default compiler settings for the language.
    fn default_compiler_settings(&self) -> Preferences;

    /// Return true if the program in `file_name` compiles.
    ///
    /// # Arguments
    ///
    /// * `file_name` - The string slice referencing the text containing the file name.
    /// * `prefs` - The preferences object that contains compiler settings.
    fn mutant_compiles(&mut self, ast: &SuperAST, prefs: &Preferences) -> bool;

    /// Calculate all the paths to all the nodes in `ast`.
    ///
    /// # Arguments
    ///
    /// * `ast` - The AST object.
    /// * `permissions` - [`Permissions`] that direct the traversal of `ast` to count the nodes.
    fn calculate_node_paths(
        &mut self,
        ast: &SuperAST,
        permissions: &Permissions,
    ) -> Result<NodePathMap, MutagenyxError>;

    /// Returns the [`Language`] type that the language sub-module implements.
    fn implements(&self) -> Language;
}

/// The LanguageInterface type provides functions to get an object that conforms to the [`MutableLanguage`]
/// trait for the requested [`Language`] and to enumerate all the supported languages in the library.
pub struct LanguageInterface {}

impl LanguageInterface {
    /// Return a dynamic object that conforms to the [`MutableLanguage`] trait.
    ///
    /// # Arguments
    ///
    /// * language - The language that the return object should support.
    pub fn get_language_object_for_language(
        language: &Language,
    ) -> Result<Box<dyn MutableLanguage>, MutagenyxError> {
        match language {
            Language::Solidity => Ok(Box::new(
                JSONLanguageInterface::new(get_solidity_delegate()),
            )),
            Language::Vyper => Ok(Box::new(JSONLanguageInterface::new(get_vyper_delegate()))),
        }
    }

    /// Return a list of dynamic objects that conform to [`MutableLanguage`].  The list will contain
    /// one object for each language supported by the library.
    pub fn get_list_of_all_language_objects(
    ) -> Result<Vec<Box<dyn MutableLanguage>>, MutagenyxError> {
        let language_list: Vec<Box<dyn MutableLanguage>> = vec![
            Box::new(JSONLanguageInterface::new(get_solidity_delegate())),
            Box::new(JSONLanguageInterface::new(get_vyper_delegate())),
        ];
        Ok(language_list)
    }
}
