//! The Solidity `node_finder` module contains Solidity specific implementations of [`NodeFinder<AST>`]
//! trait objects capable of finding the correct nodes to insert a comment for Solidity ASTs.

use crate::id::Id;
use crate::json::JSONMutate;
use crate::json_finder_helpers::found_node_id_in_array;
use crate::node_finder::*;
use crate::solidity::ast::SolidityAST;

/// Return true if `node` contains a child node with id of `node_id`.
///
/// The function checks to see if `node` has a node for `key`. If it does
/// then the function checks to see if the node is a Block or a single
/// statement that contains a node with `node_id`.
///
/// # Arguments
///
/// * `node` - The node to search for `node_id`.
/// * `key` - The specific key to look at for `node_id`.
/// * `node_id` - The id of the mutated node.
/// * `id_maker` - A [`Id<AST>`] trait object that can convert nodes to node ids.
fn found_node_id_in_node_with_key(
    node: &mut SolidityAST,
    key: &str,
    node_id: u64,
    id_maker: &dyn Id<SolidityAST>,
) -> bool {
    if let Some(body_node) = node.get_mut(key) {
        if let Some(node_type) = body_node.get_str_for_key("nodeType") {
            if node_type == "Block" {
                // Check to see if the block itself has the node_id.
                if let Some(id) = body_node.get_int_for_key("id") {
                    if node_id == id as u64 {
                        return true;
                    }
                }
                if let Some(statements_array) = body_node.get_array_for_key_mut("statements") {
                    if found_node_id_in_array(statements_array, node_id, id_maker) {
                        return true;
                    }
                }
            } else {
                // body_node is just one statement.
                if let Some(_found_node) = body_node.get_node_with_id_mut(node_id, id_maker) {
                    return true;
                }
            }
        }
    }
    false
}

/// Return true if the function finds a node with `node_id` in the `statements` key/node of
/// `node`.
///
/// # Arguments
///
/// * `node` - The node to search for a `statements` array that might contain `node_id`.
/// * `node_id` - The id of the mutated node.
/// * `id_maker` - A [`Id<AST>`] trait object that can convert nodes to node ids.
fn found_node_in_statements_array(
    node: &mut SolidityAST,
    node_id: u64,
    id_maker: &dyn Id<SolidityAST>,
) -> bool {
    if let Some(statements_array) = node.get_array_for_key_mut("statements") {
        return found_node_id_in_array(statements_array, node_id, id_maker);
    }
    false
}

struct DummyFinder {}

impl NodeFinder<SolidityAST> for DummyFinder {}

struct ContractDefinitionFinder {}

impl NodeFinder<SolidityAST> for ContractDefinitionFinder {
    fn node_is_a_direct_member(
        &self,
        node: &mut SolidityAST,
        node_id: u64,
        id_maker: &dyn Id<SolidityAST>,
    ) -> bool {
        if let Some(statements_array) = node.get_array_for_key_mut("nodes") {
            if found_node_id_in_array(statements_array, node_id, id_maker) {
                return true;
            }
        }
        false
    }
}

struct DoWhileStatementFinder {}

impl NodeFinder<SolidityAST> for DoWhileStatementFinder {
    fn node_is_a_direct_member(
        &self,
        node: &mut SolidityAST,
        node_id: u64,
        id_maker: &dyn Id<SolidityAST>,
    ) -> bool {
        found_node_id_in_node_with_key(node, "body", node_id, id_maker)
    }
}

struct ForStatementFinder {}

impl NodeFinder<SolidityAST> for ForStatementFinder {
    fn node_is_a_direct_member(
        &self,
        node: &mut SolidityAST,
        node_id: u64,
        id_maker: &dyn Id<SolidityAST>,
    ) -> bool {
        found_node_id_in_node_with_key(node, "body", node_id, id_maker)
    }
}

struct FunctionDefinitionFinder {}

impl NodeFinder<SolidityAST> for FunctionDefinitionFinder {
    fn node_is_a_direct_member(
        &self,
        node: &mut SolidityAST,
        node_id: u64,
        id_maker: &dyn Id<SolidityAST>,
    ) -> bool {
        found_node_id_in_node_with_key(node, "body", node_id, id_maker)
    }
}

struct GenericBlockFinder {}

impl NodeFinder<SolidityAST> for GenericBlockFinder {
    fn node_is_a_direct_member(
        &self,
        node: &mut SolidityAST,
        node_id: u64,
        id_maker: &dyn Id<SolidityAST>,
    ) -> bool {
        found_node_in_statements_array(node, node_id, id_maker)
    }
}

struct IfStatementFinder {}

impl NodeFinder<SolidityAST> for IfStatementFinder {
    fn node_is_a_direct_member(
        &self,
        node: &mut SolidityAST,
        node_id: u64,
        id_maker: &dyn Id<SolidityAST>,
    ) -> bool {
        if found_node_id_in_node_with_key(node, "trueBody", node_id, id_maker) {
            return true;
        }

        if found_node_id_in_node_with_key(node, "falseBody", node_id, id_maker) {
            return true;
        }
        false
    }
}

struct ModifierDefinitionFinder {}

impl NodeFinder<SolidityAST> for ModifierDefinitionFinder {
    fn node_is_a_direct_member(
        &self,
        node: &mut SolidityAST,
        node_id: u64,
        id_maker: &dyn Id<SolidityAST>,
    ) -> bool {
        found_node_id_in_node_with_key(node, "body", node_id, id_maker)
    }
}

struct TryCatchClauseFinder {}

impl NodeFinder<SolidityAST> for TryCatchClauseFinder {
    fn node_is_a_direct_member(
        &self,
        node: &mut SolidityAST,
        node_id: u64,
        id_maker: &dyn Id<SolidityAST>,
    ) -> bool {
        found_node_id_in_node_with_key(node, "block", node_id, id_maker)
    }
}

struct WhileStatementFinder {}

impl NodeFinder<SolidityAST> for WhileStatementFinder {
    fn node_is_a_direct_member(
        &self,
        node: &mut SolidityAST,
        node_id: u64,
        id_maker: &dyn Id<SolidityAST>,
    ) -> bool {
        found_node_id_in_node_with_key(node, "body", node_id, id_maker)
    }
}

/// Factory object that can generate [`NodeFinder<AST>`] trait objects for [`SolidityAST`] nodes.
pub struct SolidityNodeFinderFactory {}

impl NodeFinderFactory<SolidityAST> for SolidityNodeFinderFactory {
    fn node_finder_for(&self, node: &SolidityAST) -> Box<dyn NodeFinder<SolidityAST>> {
        if let Some(node_type) = node.get_str_for_key("nodeType") {
            match node_type {
                "Block" => Box::new(GenericBlockFinder {}),
                "ContractDefinition" => Box::new(ContractDefinitionFinder {}),
                "DoWhileStatement" => Box::new(DoWhileStatementFinder {}),
                "ForStatement" => Box::new(ForStatementFinder {}),
                "FunctionDefinition" => Box::new(FunctionDefinitionFinder {}),
                "IfStatement" => Box::new(IfStatementFinder {}),
                "ModifierDefinition" => Box::new(ModifierDefinitionFinder {}),
                "TryCatchClause" => Box::new(TryCatchClauseFinder {}),
                "UncheckedBlock" => Box::new(GenericBlockFinder {}),
                "WhileStatement" => Box::new(WhileStatementFinder {}),
                _ => Box::new(DummyFinder {}),
            }
        } else {
            Box::new(DummyFinder {})
        }
    }
}
