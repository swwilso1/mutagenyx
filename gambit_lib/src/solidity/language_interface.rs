//! The `solidity::language_interface` module provides implementations for the various generic
//! interfaces required to interact with a specific language sub-library instance.

use crate::ast::ASTTraverser;
use crate::error::GambitError;
use crate::json::*;
use crate::language::Language;
use crate::language_interface::MutableLanguage;
use crate::mutation::MutationType;
use crate::mutator::*;
use crate::pretty_printer::PrettyPrinter;
use crate::solidity::ast::*;
use crate::solidity::mutators::SolidityMutatorFactory;
use crate::solidity::pretty_printer::SolidityPrettyPrintVisitor;
use crate::solidity::visitor::*;
use crate::super_ast::SuperAST;
use rand_pcg::Pcg64;
use std::collections::HashMap;

/// The interface object for the Solidity programming language.
pub struct SolidityLanguageInterface {
    /// The map of mutators that the interface object can use when
    /// mutating a Solidity AST.
    mutators: HashMap<MutationType, Box<dyn Mutator<SolidityAST>>>,
}

impl SolidityLanguageInterface {
    /// Create a new Solidity interface object
    pub fn new() -> SolidityLanguageInterface {
        SolidityLanguageInterface {
            mutators: HashMap::new(),
        }
    }

    /// Given an instance of a SuperAST enum, return the concrete SolidityAST object if the
    /// SuperAST value is a Solidity AST.
    ///
    /// # Arguments
    ///
    /// * `ast` - A reference to the SuperAST object that might contain a Solidity AST.
    fn recover_solidity_ast(ast: &SuperAST) -> Result<&SolidityAST, GambitError> {
        let solidity_ast = match ast {
            SuperAST::Solidity(sast) => sast,
            _ => return Err(GambitError::ASTTypeNotSupported),
        };
        Ok(solidity_ast)
    }
}

impl MutableLanguage for SolidityLanguageInterface {
    fn load_ast_from_file(&mut self, file_name: &str) -> Result<SuperAST, GambitError> {
        // TODO: Need to update this code with a recognizer that determines if the source is an AST
        // or a source file.
        let solidity_ast = load_json_from_file_with_name(file_name)?;
        Ok(SuperAST::Solidity(solidity_ast))
    }

    fn select_mutators_for_mutation_types(
        &mut self,
        mutation_types: &Vec<MutationType>,
    ) -> Result<(), GambitError> {
        // Walk through the list of mutation types and convert the list into
        // a list of mutators that implement the mutation type.
        let mutators: Vec<Box<dyn Mutator<SolidityAST>>> = mutation_types
            .iter()
            // Try to convert the mutation type to a mutator
            .map(|t| SolidityMutatorFactory::mutator_for(t))
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
    ) -> Result<HashMap<MutationType, usize>, GambitError> {
        let mut counter_visitor = SolidityMutationNodeCounter::new(&self.mutators);
        let solidity_ast = SolidityLanguageInterface::recover_solidity_ast(ast)?;

        // Traverse the AST and count the number of nodes that a mutator can mutate for each
        // mutation type supported in the mutator map.
        ASTTraverser::traverse(solidity_ast, &mut counter_visitor);

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
    ) -> Result<SuperAST, GambitError> {
        let solidity_ast = SolidityLanguageInterface::recover_solidity_ast(ast)?;

        let mut mutated_ast = solidity_ast.clone();

        let mut mutation_maker =
            SolidityMutationMaker::new(self.mutators.get(mutation_type).unwrap(), rng, index);

        // Traverse the cloned AST, only mutating the index (th) node in the tree that the mutation
        // maker can mutate for `mutation_type`.
        ASTTraverser::traverse_mut(&mut mutated_ast, &mut mutation_maker);

        Ok(SuperAST::Solidity(mutated_ast))
    }

    fn pretty_print_ast_to_file(
        &mut self,
        ast: &SuperAST,
        file_name: &str,
        pretty_printer: &mut PrettyPrinter,
    ) -> Result<(), GambitError> {
        let solidity_ast = SolidityLanguageInterface::recover_solidity_ast(ast)?;
        let mut f = match std::fs::File::create(file_name) {
            Ok(file) => file,
            Err(e) => return Err(GambitError::from(e)),
        };

        let mut solidity_pretty_printer_visitor =
            SolidityPrettyPrintVisitor::new(&mut f, pretty_printer);

        // Traverse each node of the tree, process the node, and recover the original program.
        ASTTraverser::traverse(solidity_ast, &mut solidity_pretty_printer_visitor);

        Ok(())
    }

    fn get_extension_for_output_file(&self) -> &str {
        return "sol";
    }

    fn file_is_language_source_file(&self, _file_name: &str) -> bool {
        // TODO: Implement check of source file to see that it is a solidity source file.
        // Probably need to run it through the compiler.
        false
    }

    fn file_is_language_ast_file(&self, file_name: &str) -> bool {
        if let Ok(ast_candidate) = load_json_from_file_with_name(file_name) {
            if let Some(node_type) = ast_candidate.get_str_for_key("nodeType") {
                if node_type == "SourceUnit" {
                    return true;
                }
            }
        }
        false
    }

    fn implements(&self) -> Language {
        Language::Solidity
    }
}
