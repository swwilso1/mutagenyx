use crate::visitor::*;

/// The trait that describes the functionality that an object must possess in order to
/// traverse an abstract syntax tree.
pub trait AST<N> {
    /// Traverse a language abstract syntax tree allowing the visitor object access to each node
    /// during the traversal.  The traversal algorithm does not mutate the AST.
    fn traverse<V: Visitor<N> + ?Sized>(&self, visitor: &mut V) -> bool;

    /// Traverse a language abstract syntax tree allowing the visitor object access to each node
    /// during the traversal.  The visitor object may mutate each node.
    fn traverse_mut<V: VisitorMut<N> + ?Sized>(&mut self, visitor: &mut V) -> bool;
}
