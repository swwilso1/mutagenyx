//! The `language_interface` module provides abstractions to interact with a programming language
//! module for the purpose of generating mutations. The programming language abstraction provides
//! common APIs that a programming language module can implement and thereby plug-in to the library.

use crate::error::MetamorphError;
use crate::json_ast_language_interface::JSONLanguageInterface;
use crate::language::Language;
use crate::mutation::MutationType;
use crate::pretty_printer::PrettyPrinter;
use crate::solidity::language_interface::get_solidity_sub_language_interface;
use crate::super_ast::SuperAST;
use crate::vyper::language_interface::get_vyper_sub_language_interface;
use rand_pcg::*;
use std::collections::HashMap;

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
    fn load_ast_from_file(&mut self, file_name: &str) -> Result<SuperAST, MetamorphError>;

    /// Prepare language specific mutator objects that will mutate the AST for the requested
    /// mutation algorithms. The function may return [`MetamorphError::MutationAlgorithmNotSupported`].
    ///
    /// # Arguments
    ///
    /// * `mutation_types` - A vector of MutationType enumerations. Each MutationType value
    /// corresponds to a requested mutation algorithm.
    fn select_mutators_for_mutation_types(
        &mut self,
        mutation_types: &Vec<MutationType>,
    ) -> Result<(), MetamorphError>;

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
    fn count_mutable_nodes(
        &mut self,
        ast: &SuperAST,
    ) -> Result<HashMap<MutationType, usize>, MetamorphError>;

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
    fn mutate_ast(
        &mut self,
        ast: &SuperAST,
        mutation_type: &MutationType,
        index: usize,
        rng: &mut Pcg64,
    ) -> Result<SuperAST, MetamorphError>;

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
    ) -> Result<(), MetamorphError>;

    /// Return the file extension that a caller should append to a file name that the caller will
    /// later pass to `pretty_print_mutated_ast_to_file`.
    fn get_extension_for_output_file(&self) -> &str;

    /// Check that the contents of the file named by `file_name` contain a program written in
    /// the language implementing the trait.
    ///
    /// # Arguments
    ///
    /// * `file_name` - The string slice referencing the text containing the file name.
    fn file_is_language_source_file(&self, file_name: &str) -> bool;

    /// Check that the contents of the file named by `file_name` contain an AST representation
    /// of a program for the language implementing the trait.
    ///
    /// # Arguments
    ///
    /// * `file_name` - The string slice referencing the text containing the file name.
    fn file_is_language_ast_file(&self, file_name: &str) -> bool;

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
    ) -> Result<Box<dyn MutableLanguage>, MetamorphError> {
        match language {
            Language::Solidity => Ok(Box::new(JSONLanguageInterface::new(
                get_solidity_sub_language_interface(),
            ))),
            Language::Vyper => Ok(Box::new(JSONLanguageInterface::new(
                get_vyper_sub_language_interface(),
            ))),
        }
    }

    /// Return a list of dynamic objects that conform to [`MutableLanguage`].  The list will contain
    /// one object for each language supported by the library.
    pub fn get_list_of_all_language_objects() -> Result<Vec<Box<dyn MutableLanguage>>, MetamorphError>
    {
        let mut language_list: Vec<Box<dyn MutableLanguage>> = vec![];
        language_list.push(Box::new(JSONLanguageInterface::new(
            get_solidity_sub_language_interface(),
        )));
        language_list.push(Box::new(JSONLanguageInterface::new(
            get_vyper_sub_language_interface(),
        )));
        Ok(language_list)
    }
}
