//! The `json_comment_inserter` module provides [`JSONCommentInserter`], a type that can be used
//! for ASTs that use JSON to insert a comment node into the AST JSON.

use crate::commenter::CommenterFactory;
use crate::id::Id;
use crate::json::JSONMutate;
use crate::mutation_visitor::NodePath;
use crate::node_finder::NodeFinderFactory;
use serde_json::Value;

/// The type that wraps the insertion algorithm for JSON based ASTs.
pub struct JSONCommentInserter {
    /// The [`NodeFinderFactory<Value>`] trait object that returns node finder trait objects
    /// which can determine if a node is the correct node for inserting a comment.
    finder_factory: Box<dyn NodeFinderFactory<Value>>,

    /// The [`CommenterFactory<Value>`] trait object that returns commenter trait objects
    /// which can actually insert a comment into a node.
    commenter_factory: Box<dyn CommenterFactory<Value>>,

    /// The [`Id<Value>`] trait object that can convert [`Value`] objects (nodes) to node ids.
    id_maker: Box<dyn Id<Value>>,
}

impl JSONCommentInserter {
    /// Create a new comment inserter object.
    ///
    /// # Arguments
    ///
    /// * `finder` - The [`NodeFinderFactory<Value>`] trait object.
    /// * `commenter` - The [`CommenterFactory<Value>`] trait object.
    /// * `id_maker` - The [`Id<Value>`] trait object.
    pub fn new(
        finder: Box<dyn NodeFinderFactory<Value>>,
        commenter: Box<dyn CommenterFactory<Value>>,
        id_maker: Box<dyn Id<Value>>,
    ) -> JSONCommentInserter {
        JSONCommentInserter {
            finder_factory: finder,
            commenter_factory: commenter,
            id_maker,
        }
    }

    /// Using `node_path`, insert `comment_node` into `ast`.
    ///
    /// # Arguments
    /// * `ast` - The mutated AST that needs a comment.
    /// * `comment_node` - The comment node to insert into `ast`.
    /// * `node_path` - The path object that contains the node id path of the nodes from the
    /// top node in the AST to the mutated node in the AST.
    pub fn insert_comment_by_path(
        &self,
        ast: &mut Value,
        comment_node: Value,
        node_path: &NodePath,
    ) {
        if node_path.is_empty() {
            log::info!("Cannot insert comment node into AST, path to mutated node too short.");
            return;
        }

        let mut reversed_path: NodePath = node_path.clone();

        // The path goes from the top of the AST to the mutated node.  We need the path in reverse
        // order.
        reversed_path.reverse();

        let id_maker_ref = self.id_maker.as_ref();

        // We start the iteration at 1 because the first element of reversed_path is actually
        // the mutated node itself, that node will not have the statement array that we need.
        let mut i: usize = 1;

        while i < reversed_path.len() {
            if let Some(node) = ast.get_node_with_id_mut(reversed_path[i], id_maker_ref) {
                let finder = self.finder_factory.node_finder_for(node);
                if finder.node_is_a_direct_member(node, reversed_path[0], id_maker_ref) {
                    let commenter = self.commenter_factory.commenter_for(node);
                    commenter.insert_comment(node, reversed_path[0], comment_node, id_maker_ref);
                    return;
                }
            }
            i += 1;
        }
    }
}
