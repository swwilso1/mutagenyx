//! The Vyper `commenter` module contains Vyper specific implementations of [`Commenter<AST>`]
//! trait objects capable of inserting comment nodes into the correct places for Vyper ASTs.

use crate::commenter::*;
use crate::id::Id;
use crate::json::JSONMutate;
use crate::json_comment_helpers::inserted_comment_in_array;
use crate::vyper::ast::VyperAST;

struct DummyCommenter {}
impl Commenter<VyperAST> for DummyCommenter {}

struct GenericBlockCommenter {}

impl Commenter<VyperAST> for GenericBlockCommenter {
    fn insert_comment(
        &self,
        node: &mut VyperAST,
        node_id: u64,
        comment_node: VyperAST,
        id_maker: &dyn Id<VyperAST>,
    ) {
        if let Some(body_array) = node.get_array_for_key_mut("body") {
            inserted_comment_in_array(body_array, comment_node, node_id, id_maker);
        }
    }
}

struct IfCommenter {}

impl Commenter<VyperAST> for IfCommenter {
    fn insert_comment(
        &self,
        node: &mut VyperAST,
        node_id: u64,
        comment_node: VyperAST,
        id_maker: &dyn Id<VyperAST>,
    ) {
        if let Some(body_array) = node.get_array_for_key_mut("body") {
            if inserted_comment_in_array(body_array, comment_node.clone(), node_id, id_maker) {
                return;
            }
        }

        if let Some(orelse_array) = node.get_array_for_key_mut("orelse") {
            inserted_comment_in_array(orelse_array, comment_node, node_id, id_maker);
        }
    }
}

/// Factory object that can generate [`Commenter<AST>`] objects for [`VyperAST`] nodes.
pub struct VyperCommenterFactory {}

impl CommenterFactory<VyperAST> for VyperCommenterFactory {
    fn commenter_for(&self, node: &VyperAST) -> Box<dyn Commenter<VyperAST>> {
        if let Some(ast_type) = node.get_str_for_key("ast_type") {
            match ast_type {
                "For" => Box::new(GenericBlockCommenter {}),
                "FunctionDef" => Box::new(GenericBlockCommenter {}),
                "If" => Box::new(IfCommenter {}),
                "Module" => Box::new(GenericBlockCommenter {}),
                _ => Box::new(DummyCommenter {}),
            }
        } else {
            Box::new(DummyCommenter {})
        }
    }
}
