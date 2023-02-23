//! The `commenter` module contains traits that describe functionality to insert comments into
//! nodes.  The traits allow the developer to provide commenter functionality specific to each
//! node.

use crate::id::Id;

pub trait Commenter<AST> {
    /// Insert `comment_node` in `node` if `node` contains a direct child that has id `node_id`.
    ///
    /// # Arguments
    ///
    /// * `node` - The AST node that will receive `comment_node`.
    /// * `node_id` The id of the AST node that was mutated (ie the one to find).
    /// * `comment_node` - The AST node containing the comment that goes with the node with id
    /// `node_id`.
    /// * `id_maker` - The [`Id<AST>`] trait object that can convert AST nodes to node ids.
    fn insert_comment(
        &self,
        _node: &mut AST,
        _node_id: u64,
        _comment_node: AST,
        _id_maker: &dyn Id<AST>,
    ) {
    }
}

pub trait CommenterFactory<AST> {
    /// Return a [`Commenter<AST>`] trait object that can insert a comment in `node`.
    ///
    /// # Arguments
    ///
    /// * `node` - The AST node for which the function should return a [`Commenter<AST>`] trait object.
    fn commenter_for(&self, node: &AST) -> Box<dyn Commenter<AST>>;
}
