//! The `json_ast_language_interface` module abstracts the language interface code that is
//! generic for languages that have a JSON encoded AST. The interface code relies on
//! a trait object for [`JSONLanguageDelegate`] to provide the behavior that will differ from
//! language to language.

use crate::ast::ASTTraverser;
use crate::error::MutagenyxError;
use crate::json::*;
use crate::json_language_delegate::JSONLanguageDelegate;
use crate::language::Language;
use crate::language_interface::{MutableLanguage, MutateASTResult};
use crate::mutation::MutationType;
use crate::mutation_visitor::*;
use crate::mutator::*;
use crate::permissions::Permissions;
use crate::preferences::Preferences;
use crate::pretty_printer::PrettyPrinter;
use crate::recognizer::FileType;
use crate::super_ast::SuperAST;
use rand_pcg::Pcg64;
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::io::Write;

/// The interface object for the programming languages with JSON encoded ASTs.
pub struct JSONLanguageInterface {
    /// The map of mutators that the interface object can use when
    /// mutating a JSON-based AST.
    mutators: HashMap<MutationType, Box<dyn Mutator<Value>>>,

    /// The language specific delegate to use for invoking language-specific behavior.
    delegate: Box<dyn JSONLanguageDelegate>,
}

impl JSONLanguageInterface {
    /// Create a new language interface object
    ///
    /// # Arguments
    ///
    /// * `sub_interface` - The language specific delegate to use for language-specific behavior.
    pub fn new(delegate: Box<dyn JSONLanguageDelegate>) -> JSONLanguageInterface {
        JSONLanguageInterface {
            mutators: HashMap::new(),
            delegate,
        }
    }

    /// Given an instance of a SuperAST enum, return the concrete AST object if the
    /// SuperAST value is a JSON based AST.
    ///
    /// # Arguments
    ///
    /// * `ast` - A reference to the SuperAST object that might contain a JSON base AST.
    fn recover_json_ast<'b>(&self, ast: &'b SuperAST) -> Result<&'b Value, MutagenyxError> {
        // Defer the recovery of the AST to the language-specific delegate.
        self.delegate.recover_ast(ast)
    }
}

impl MutableLanguage for JSONLanguageInterface {
    fn load_ast_from_file(
        &mut self,
        file_name: &str,
        file_type: &FileType,
        prefs: &Preferences,
    ) -> Result<SuperAST, MutagenyxError> {
        match file_type {
            FileType::Source => self.delegate.convert_source_file_to_ast(file_name, prefs),
            FileType::AST => {
                let ast = load_json_from_file_with_name(file_name)?;

                // Defer the conversion of the JSON to the AST to the delegate.
                self.delegate.get_value_as_super_ast(ast)
            }
            FileType::Config => Err(MutagenyxError::ConfigFileNotSupported(String::from(
                file_name,
            ))),
        }
    }

    fn select_mutators_for_mutation_types(
        &mut self,
        mutation_types: &[MutationType],
    ) -> Result<(), MutagenyxError> {
        // Get the mutator factory
        let mutator_factory = self.delegate.get_mutator_factory();

        // Walk through the list of mutation types and convert the list into
        // a list of mutators that implement the mutation type.
        let mutators: Vec<Box<dyn Mutator<Value>>> = mutation_types
            .iter()
            // Try to convert the mutation type to a mutator
            .filter_map(|t| mutator_factory.mutator_for(t))
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
        rng: &mut Pcg64,
        permissions: &Permissions,
    ) -> Result<HashMap<MutationType, usize>, MutagenyxError> {
        let permitter = self.delegate.get_node_permitter(permissions);
        let namer = self.delegate.get_namer();
        let actual_ast = self.recover_json_ast(ast)?;

        let mut counter_visitor: MutableNodesCounter<Value> =
            MutableNodesCounter::new(&mut self.mutators, permitter, rng, namer);

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
        permissions: &Permissions,
        path_map: &NodePathMap,
    ) -> Result<MutateASTResult, MutagenyxError> {
        let permitter = self.delegate.get_node_permitter(permissions);
        let namer = self.delegate.get_namer();
        let id_maker = self.delegate.get_node_id_maker();

        let actual_ast = self.recover_json_ast(ast)?;

        let mut mutated_ast = actual_ast.clone();

        let mut mutation_maker: MutationMaker<Value> = MutationMaker::new(
            self.mutators.get_mut(mutation_type).unwrap().as_mut(),
            rng,
            index,
            permitter,
            id_maker,
            namer,
        );

        // Traverse the cloned AST, only mutating the index(th) node in the tree that the mutation
        // maker can mutate for `mutation_type`.
        ASTTraverser::traverse_mut(&mut mutated_ast, &mut mutation_maker);

        if let Some(e) = mutation_maker.error {
            return Err(e);
        }

        // We now have the path map and the index of the node we mutated in mutation_maker.mutated_node_id.
        // We can now walk down the ast to insert a comment.
        if let Some(node) = mutation_maker.mutator_comment {
            if let Some(node_path) = path_map.get(&mutation_maker.mutated_node_id) {
                self.delegate
                    .insert_comment_by_path(&mut mutated_ast, node, node_path);
            }
        }

        let ast = self.delegate.get_value_as_super_ast(mutated_ast)?;

        let mut mutate_ast_result = MutateASTResult::new();
        mutate_ast_result.mutator_result = mutation_maker.mutation_results;
        mutate_ast_result.ast = Some(ast);
        mutate_ast_result.mutator_result.index = Some(index);

        Ok(mutate_ast_result)
    }

    fn pretty_print_ast_to_file(
        &mut self,
        ast: &SuperAST,
        file_name: &str,
        pretty_printer: &mut PrettyPrinter,
    ) -> Result<(), MutagenyxError> {
        let actual_ast = self.recover_json_ast(ast)?;
        let mut f = std::fs::File::create(file_name)?;

        let mut pretty_print_visitor = self
            .delegate
            .get_pretty_print_visitor(&mut f, pretty_printer);

        // Traverse each node of the tree, process the node, and recover the original program.
        ASTTraverser::traverse(actual_ast, &mut *pretty_print_visitor);

        Ok(())
    }

    fn pretty_print_ast_to_stream(
        &mut self,
        ast: &SuperAST,
        stream: &mut dyn Write,
        pretty_printer: &mut PrettyPrinter,
    ) -> Result<(), MutagenyxError> {
        let actual_ast = self.recover_json_ast(ast)?;

        let mut pretty_print_visitor = self
            .delegate
            .get_pretty_print_visitor(stream, pretty_printer);

        // Traverse each node of the tree, process the node, and recover the original program.
        ASTTraverser::traverse(actual_ast, &mut *pretty_print_visitor);

        Ok(())
    }

    fn get_extension_for_output_file(&self) -> &str {
        self.delegate.get_file_extension()
    }

    fn file_is_language_source_file(&self, file_name: &str, prefs: &Preferences) -> bool {
        self.delegate.file_is_language_source_file(file_name, prefs)
    }

    fn convert_source_file_to_ast(
        &self,
        file_name: &str,
        prefs: &Preferences,
    ) -> Result<SuperAST, MutagenyxError> {
        self.delegate.convert_source_file_to_ast(file_name, prefs)
    }

    fn file_is_language_ast_file(&self, file_name: &str) -> bool {
        if let Ok(ast_candidate) = load_json_from_file_with_name(file_name) {
            return self.delegate.json_is_language_ast_json(&ast_candidate);
        }
        false
    }

    fn default_compiler_settings(&self) -> Preferences {
        self.delegate.default_compiler_settings()
    }

    fn mutant_compiles(&mut self, ast: &SuperAST, prefs: &Preferences) -> bool {
        // We will pretty print the AST to a file in the temp directory.
        let mut source_file = env::temp_dir();
        let file_name = String::from("mutant") + "." + self.get_extension_for_output_file();
        source_file.push(file_name);

        let mut pretty_printer = PrettyPrinter::new(4, 150);

        if self
            .pretty_print_ast_to_file(ast, source_file.to_str().unwrap(), &mut pretty_printer)
            .is_err()
        {
            return false;
        }

        let compile_result = self
            .delegate
            .mutant_compiles(source_file.to_str().unwrap(), prefs);

        if std::fs::remove_file(source_file.clone()).is_err() {
            log::debug!("Failed to remove temporary source file: {:?}", source_file);
        }

        compile_result
    }

    fn calculate_node_paths(
        &mut self,
        ast: &SuperAST,
        permissions: &Permissions,
    ) -> Result<NodePathMap, MutagenyxError> {
        let actual_ast = self.recover_json_ast(ast)?;
        let permitter = self.delegate.get_node_permitter(permissions);
        let namer = self.delegate.get_namer();
        let id_maker = self.delegate.get_node_id_maker();
        let mut path_visitor = PathVisitor::new(permitter, id_maker, namer);

        // Walk the AST and calculate the path to each node in the AST.
        ASTTraverser::traverse(actual_ast, &mut path_visitor);

        Ok(path_visitor.path_map)
    }

    fn implements(&self) -> Language {
        self.delegate.implements()
    }
}
