//! The Vyper `node_finder` module contains Vyper specific implementations of [`NodeFinder<AST>`]
//! trait objects capable of finding the correct nodes to insert a comment for Vyper ASTs.

use crate::id::Id;
use crate::json::JSONMutate;
use crate::json_finder_helpers::found_node_id_in_array;
use crate::node_finder::*;
use crate::vyper::ast::VyperAST;

struct DummyFinder {}
impl NodeFinder<VyperAST> for DummyFinder {}

struct GenericBlockFinder {}

impl NodeFinder<VyperAST> for GenericBlockFinder {
    fn node_is_a_direct_member(
        &self,
        node: &mut VyperAST,
        node_id: u64,
        id_maker: &dyn Id<VyperAST>,
    ) -> bool {
        if let Some(body_array) = node.get_array_for_key_mut("body") {
            return found_node_id_in_array(body_array, node_id, id_maker);
        }
        false
    }
}

struct IfFinder {}

impl NodeFinder<VyperAST> for IfFinder {
    fn node_is_a_direct_member(
        &self,
        node: &mut VyperAST,
        node_id: u64,
        id_maker: &dyn Id<VyperAST>,
    ) -> bool {
        if let Some(body_array) = node.get_array_for_key_mut("body") {
            if found_node_id_in_array(body_array, node_id, id_maker) {
                return true;
            }
        }

        if let Some(orelse_array) = node.get_array_for_key_mut("orelse") {
            return found_node_id_in_array(orelse_array, node_id, id_maker);
        }
        false
    }
}

/// Factory object that can generate [`NodeFinder<AST>`] trait objects for [`VyperAST`] nodes.
pub struct VyperNodeFinderFactory {}

impl NodeFinderFactory<VyperAST> for VyperNodeFinderFactory {
    fn node_finder_for(&self, node: &VyperAST) -> Box<dyn NodeFinder<VyperAST>> {
        if let Some(ast_type) = node.get_str_for_key("ast_type") {
            match ast_type {
                "For" => Box::new(GenericBlockFinder {}),
                "FunctionDef" => Box::new(GenericBlockFinder {}),
                "If" => Box::new(IfFinder {}),
                "Module" => Box::new(GenericBlockFinder {}),
                _ => Box::new(DummyFinder {}),
            }
        } else {
            Box::new(DummyFinder {})
        }
    }
}
