//! The Solidity `commenter` module contains Solidity specific implementations of [`Commenter<AST>`]
//! trait objects capable of inserting comment nodes into the correct places for Solidity ASTs.

use crate::commenter::*;
use crate::id::Id;
use crate::json::JSONMutate;
use crate::json_comment_helpers::inserted_comment_in_array;
use crate::solidity::ast::SolidityAST;
use serde_json::{from_str, json, Value};

/// Return a new `Block` node that contains `array` in the Block's statements member.
///
/// # Arguments
///
/// * `array` - The array of nodes to put in the Block statements member.
fn new_block_node(array: Vec<SolidityAST>) -> SolidityAST {
    let block_str = "{\
        \"id\": 9999994,
        \"nodeType\": \"Block\"
    }";

    // Should we unwrap here?
    let mut json_value: Value = from_str(block_str).unwrap();
    json_value.set_node_for_key("statements", json![array]);
    json_value
}

/// Find a node for `key` in `node`, insert `comment_node` into the correct place if the node
/// has a `body` with a Block, otherwise create a new Block around a single statement and put
/// the comment as the first member of the Block's statement array.
///
/// # Arguments
///
/// * `node` - The node that will receive `comment_node`.
/// * `key` - The key value to look under to find a Block node or a single statement.
/// * `node_id` - The id of the node that was mutated.
/// * `comment_node` - The node containing a comment about the mutation to insert in `node`.
/// * `id_maker` - The [`Id<AST>`] trait object that can convert nodes to node ids.
fn insert_comment_into_node_with_key(
    node: &mut SolidityAST,
    key: &str,
    node_id: u64,
    comment_node: SolidityAST,
    id_maker: &dyn Id<SolidityAST>,
) -> bool {
    let mut inserted = false;
    if let Some(mut body_node) = node.take_value_for_key(key) {
        if let Some(node_type) = body_node.get_str_for_key("nodeType") {
            if node_type == "Block" {
                if let Some(id) = body_node.get_int_for_key("id") {
                    if node_id == id as u64 {
                        // The block id is the one that was mutated, just insert the comment at the
                        // top of the block.
                        if let Some(statements_array) =
                            body_node.get_array_for_key_mut("statements")
                        {
                            statements_array.insert(0, comment_node);
                        }
                        node.set_node_for_key("body", body_node);
                        return true;
                    }
                }
                if let Some(statements_array) = body_node.get_array_for_key_mut("statements") {
                    inserted = inserted_comment_in_array(
                        statements_array,
                        comment_node,
                        node_id,
                        id_maker,
                    );
                }
            } else {
                // The node only has one statement
                if body_node.get_node_with_id(node_id, id_maker).is_some() {
                    // This one node *does* have the node of interest.  We need to create a Block
                    // node and put that block into the place

                    let new_node_array: Vec<Value> = vec![comment_node, body_node];
                    body_node = new_block_node(new_node_array);
                    inserted = true;
                }
            }
        }
        node.set_node_for_key(key, body_node);
    }
    inserted
}

/// If `node` has a `statements` member, then insert `comment_node` in the array if the array
/// contains the mutated node (identified by `node_id`).
///
/// # Arguments
///
/// * `node` - The node that will receive the comment node.
/// * `node_id` - The id of the mutated node.
/// * `comment_node` - The comment node to insert into the array.
/// * `id_maker` - A [`Id<AST>`] trait object that can convert nodes to node ids.
fn insert_comment_into_statements_array(
    node: &mut SolidityAST,
    node_id: u64,
    comment_node: SolidityAST,
    id_maker: &dyn Id<SolidityAST>,
) {
    if let Some(statements_array) = node.get_array_for_key_mut("statements") {
        inserted_comment_in_array(statements_array, comment_node, node_id, id_maker);
    }
}

struct DummyCommenter {}

impl Commenter<SolidityAST> for DummyCommenter {}

struct ContractDefinitionCommenter {}

impl Commenter<SolidityAST> for ContractDefinitionCommenter {
    fn insert_comment(
        &self,
        node: &mut SolidityAST,
        node_id: u64,
        comment_node: SolidityAST,
        id_maker: &dyn Id<SolidityAST>,
    ) {
        if let Some(statements_array) = node.get_array_for_key_mut("nodes") {
            inserted_comment_in_array(statements_array, comment_node, node_id, id_maker);
        }
    }
}

struct DoWhileStatementCommenter {}

impl Commenter<SolidityAST> for DoWhileStatementCommenter {
    fn insert_comment(
        &self,
        node: &mut SolidityAST,
        node_id: u64,
        comment_node: SolidityAST,
        id_maker: &dyn Id<SolidityAST>,
    ) {
        insert_comment_into_node_with_key(node, "body", node_id, comment_node, id_maker);
    }
}

struct ForStatementCommenter {}

impl Commenter<SolidityAST> for ForStatementCommenter {
    fn insert_comment(
        &self,
        node: &mut SolidityAST,
        node_id: u64,
        comment_node: SolidityAST,
        id_maker: &dyn Id<SolidityAST>,
    ) {
        insert_comment_into_node_with_key(node, "body", node_id, comment_node, id_maker);
    }
}

struct FunctionDefinitionCommenter {}

impl Commenter<SolidityAST> for FunctionDefinitionCommenter {
    fn insert_comment(
        &self,
        node: &mut SolidityAST,
        node_id: u64,
        comment_node: SolidityAST,
        id_maker: &dyn Id<SolidityAST>,
    ) {
        insert_comment_into_node_with_key(node, "body", node_id, comment_node, id_maker);
    }
}

struct GenericBlockCommenter {}

impl Commenter<SolidityAST> for GenericBlockCommenter {
    fn insert_comment(
        &self,
        node: &mut SolidityAST,
        node_id: u64,
        comment_node: SolidityAST,
        id_maker: &dyn Id<SolidityAST>,
    ) {
        insert_comment_into_statements_array(node, node_id, comment_node, id_maker)
    }
}

struct IfStatementCommenter {}

impl Commenter<SolidityAST> for IfStatementCommenter {
    fn insert_comment(
        &self,
        node: &mut SolidityAST,
        node_id: u64,
        comment_node: SolidityAST,
        id_maker: &dyn Id<SolidityAST>,
    ) {
        if insert_comment_into_node_with_key(
            node,
            "trueBody",
            node_id,
            comment_node.clone(),
            id_maker,
        ) {
            return;
        }

        insert_comment_into_node_with_key(node, "falseBody", node_id, comment_node, id_maker);
    }
}

struct ModifierDefinitionCommenter {}

impl Commenter<SolidityAST> for ModifierDefinitionCommenter {
    fn insert_comment(
        &self,
        node: &mut SolidityAST,
        node_id: u64,
        comment_node: SolidityAST,
        id_maker: &dyn Id<SolidityAST>,
    ) {
        insert_comment_into_node_with_key(node, "body", node_id, comment_node, id_maker);
    }
}

struct TryCatchClauseCommenter {}

impl Commenter<SolidityAST> for TryCatchClauseCommenter {
    fn insert_comment(
        &self,
        node: &mut SolidityAST,
        node_id: u64,
        comment_node: SolidityAST,
        id_maker: &dyn Id<SolidityAST>,
    ) {
        insert_comment_into_node_with_key(node, "block", node_id, comment_node, id_maker);
    }
}

struct WhileStatementCommenter {}

impl Commenter<SolidityAST> for WhileStatementCommenter {
    fn insert_comment(
        &self,
        node: &mut SolidityAST,
        node_id: u64,
        comment_node: SolidityAST,
        id_maker: &dyn Id<SolidityAST>,
    ) {
        insert_comment_into_node_with_key(node, "body", node_id, comment_node, id_maker);
    }
}

/// Factory object that can generate [`Commenter<AST>`] objects for [`SolidityAST`] nodes.
pub struct SolidityCommenterFactory {}

impl CommenterFactory<SolidityAST> for SolidityCommenterFactory {
    fn commenter_for(&self, node: &SolidityAST) -> Box<dyn Commenter<SolidityAST>> {
        if let Some(node_type) = node.get_str_for_key("nodeType") {
            match node_type {
                "Block" => Box::new(GenericBlockCommenter {}),
                "ContractDefinition" => Box::new(ContractDefinitionCommenter {}),
                "DoWhileStatement" => Box::new(DoWhileStatementCommenter {}),
                "ForStatement" => Box::new(ForStatementCommenter {}),
                "FunctionDefinition" => Box::new(FunctionDefinitionCommenter {}),
                "IfStatement" => Box::new(IfStatementCommenter {}),
                "ModifierDefinition" => Box::new(ModifierDefinitionCommenter {}),
                "TryCatchClause" => Box::new(TryCatchClauseCommenter {}),
                "UncheckedBlock" => Box::new(GenericBlockCommenter {}),
                "WhileStatement" => Box::new(WhileStatementCommenter {}),
                _ => Box::new(DummyCommenter {}),
            }
        } else {
            Box::new(DummyCommenter {})
        }
    }
}
