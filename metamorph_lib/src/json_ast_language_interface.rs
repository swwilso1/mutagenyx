//! The `json_ast_language_interface` module abstracts the language interface code that is
//! generic for languages that have a JSON encoded AST. The interface code relies on
//! a trait object for [`JSONLanguageDelegate<W>`] to provide the behavior that will differ from
//! language to language.

use crate::ast::ASTTraverser;
use crate::error::MetamorphError;
use crate::json::*;
use crate::json_language_delegate::JSONLanguageDelegate;
use crate::language::Language;
use crate::language_interface::MutableLanguage;
use crate::mutation::MutationType;
use crate::mutation_visitor::*;
use crate::mutator::*;
use crate::preferences::Preferences;
use crate::pretty_printer::PrettyPrinter;
use crate::recognizer::FileType;
use crate::super_ast::SuperAST;
use rand_pcg::Pcg64;
use serde_json::Value;
use std::collections::HashMap;

/// The interface object for the programming languages with JSON encoded ASTs.
pub struct JSONLanguageInterface {
    /// The map of mutators that the interface object can use when
    /// mutating a JSON-based AST.
    mutators: HashMap<MutationType, Box<dyn Mutator<Value>>>,

    /// The language specific delegate to use for invoking language-specific behavior.
    sub_language_interface: Box<dyn JSONLanguageDelegate<std::fs::File>>,
}

impl JSONLanguageInterface {
    /// Create a new language interface object
    ///
    /// # Arguments
    ///
    /// * `sub_interface` - The language specific delegate to use for language-specific behavior.
    pub fn new(
        sub_interface: Box<dyn JSONLanguageDelegate<std::fs::File>>,
    ) -> JSONLanguageInterface {
        JSONLanguageInterface {
            mutators: HashMap::new(),
            sub_language_interface: sub_interface,
        }
    }

    /// Given an instance of a SuperAST enum, return the concrete AST object if the
    /// SuperAST value is a JSON based AST.
    ///
    /// # Arguments
    ///
    /// * `ast` - A reference to the SuperAST object that might contain a JSON base AST.
    fn recover_json_ast<'a>(&self, ast: &'a SuperAST) -> Result<&'a Value, MetamorphError> {
        // Defer the recovery of the AST to the language-specific delegate.
        self.sub_language_interface.recover_ast(ast)
    }
}

impl MutableLanguage for JSONLanguageInterface {
    fn load_ast_from_file(
        &mut self,
        file_name: &str,
        file_type: &FileType,
        prefs: &Preferences,
    ) -> Result<SuperAST, MetamorphError> {
        match file_type {
            FileType::Source => self
                .sub_language_interface
                .convert_source_file_to_ast(file_name, prefs),
            FileType::AST => {
                let ast = load_json_from_file_with_name(file_name)?;

                // Defer the conversion of the JSON to the AST to the delegate.
                self.sub_language_interface.get_value_as_super_ast(ast)
            }
            FileType::Config => Err(MetamorphError::ConfigFileNotSupported(String::from(
                file_name,
            ))),
        }
    }

    fn select_mutators_for_mutation_types(
        &mut self,
        mutation_types: &Vec<MutationType>,
    ) -> Result<(), MetamorphError> {
        // Get the mutator factory
        let mutator_factory = self.sub_language_interface.get_mutator_factory();

        // Walk through the list of mutation types and convert the list into
        // a list of mutators that implement the mutation type.
        let mutators: Vec<Box<dyn Mutator<Value>>> = mutation_types
            .iter()
            // Try to convert the mutation type to a mutator
            .map(|t| mutator_factory.mutator_for(t))
            // Only use the results that have an actual mutator
            .filter(|m| match m {
                Some(_) => true,
                None => false,
            })
            .map(|o| o.unwrap())
            .collect();

        // Now fill the mutator map
        for mutator in mutators {
            self.mutators.insert(mutator.implements(), mutator);
        }
        Ok(())
    }

    fn count_mutable_nodes(
        &mut self,
        ast: &SuperAST,
    ) -> Result<HashMap<MutationType, usize>, MetamorphError> {
        let mut counter_visitor = MutableNodesCounter::<Value>::new(&self.mutators);
        let actual_ast = self.recover_json_ast(ast)?;

        // Traverse the AST and count the number of nodes that a mutator can mutate for each
        // mutation type supported in the mutator map.
        ASTTraverser::traverse(actual_ast, &mut counter_visitor);

        // Construct and populate the map that maps the number of mutable nodes to the mutation type.
        let mut node_map: HashMap<MutationType, usize> = HashMap::new();

        for (key, value) in counter_visitor.counter_table {
            node_map.insert(key, value as usize);
        }

        Ok(node_map)
    }

    fn mutate_ast(
        &mut self,
        ast: &SuperAST,
        mutation_type: &MutationType,
        index: usize,
        rng: &mut Pcg64,
    ) -> Result<SuperAST, MetamorphError> {
        let actual_ast = self.recover_json_ast(ast)?;

        let mut mutated_ast = actual_ast.clone();

        let mut mutation_maker =
            MutationMaker::<Value>::new(self.mutators.get(mutation_type).unwrap(), rng, index);

        // Traverse the cloned AST, only mutating the index(th) node in the tree that the mutation
        // maker can mutate for `mutation_type`.
        ASTTraverser::traverse_mut(&mut mutated_ast, &mut mutation_maker);

        self.sub_language_interface
            .get_value_as_super_ast(mutated_ast)
    }

    fn pretty_print_ast_to_file(
        &mut self,
        ast: &SuperAST,
        file_name: &str,
        pretty_printer: &mut PrettyPrinter,
    ) -> Result<(), MetamorphError> {
        let actual_ast = self.recover_json_ast(ast)?;
        let mut f = std::fs::File::create(file_name)?;

        let mut pretty_print_visitor = self
            .sub_language_interface
            .get_pretty_print_visitor(&mut f, pretty_printer);

        // Traverse each node of the tree, process the node, and recover the original program.
        ASTTraverser::traverse(actual_ast, &mut *pretty_print_visitor);

        Ok(())
    }

    fn get_extension_for_output_file(&self) -> &str {
        self.sub_language_interface.get_file_extension()
    }

    fn file_is_language_source_file(&self, file_name: &str, prefs: &Preferences) -> bool {
        self.sub_language_interface
            .file_is_language_source_file(file_name, prefs)
    }

    fn convert_source_file_to_ast(
        &self,
        file_name: &str,
        prefs: &Preferences,
    ) -> Result<SuperAST, MetamorphError> {
        self.sub_language_interface
            .convert_source_file_to_ast(file_name, prefs)
    }

    fn file_is_language_ast_file(&self, file_name: &str) -> bool {
        if let Ok(ast_candidate) = load_json_from_file_with_name(file_name) {
            return self
                .sub_language_interface
                .json_is_language_ast_json(&ast_candidate);
        }
        false
    }

    fn default_compiler_settings(&self) -> Preferences {
        self.sub_language_interface.default_compiler_settings()
    }

    fn implements(&self) -> Language {
        self.sub_language_interface.implements()
    }
}
