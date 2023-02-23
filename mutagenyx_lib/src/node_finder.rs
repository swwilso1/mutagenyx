//! The `node_finder` module provides traits for finding nodes in an AST that have a
//! particular node id.  The traits in this module allow the developer to provide
//! different behaviors for different nodes in the AST that might need to locate
//! child objects in unique ways.

use crate::id::Id;

pub trait NodeFinder<AST> {
    /// Return true if a node with `node_id` is a direct member/child of `node`.
    ///
    /// A direct member/child in this case means an immediate child of `node`, not
    /// a grandchild, etc...
    ///
    /// # Arguments
    ///
    /// * `node` - The node in the AST to check for children with `node_id`.
    /// * `node_id` - The id of the node to find.
    /// * `id_maker` - An [`Id<AST>`] trait object that can turn nodes in the AST into node ids.
    fn node_is_a_direct_member(
        &self,
        _node: &mut AST,
        _node_id: u64,
        _id_maker: &dyn Id<AST>,
    ) -> bool {
        false
    }
}

pub trait NodeFinderFactory<AST> {
    /// Given `node` a node in the AST, return a [`NodeFinder<AST>`] trait object that can search
    /// `node` for a particulare node id.
    fn node_finder_for(&self, node: &AST) -> Box<dyn NodeFinder<AST>>;
}
