use crate::visitor::*;

/// Trait used to inspect a conforming AST object for information necessary to travers the syntax
/// tree.
pub trait SimpleAST<N> {
    /// Return a reference to the node of the syntax tree.
    fn get_node(&self) -> &N;

    /// Return a mutable reference to the node of the syntax tree.
    fn get_node_mut(&mut self) -> &mut N;

    /// Return a list of references to the children of the node.
    fn get_children(&self) -> Vec<Box<&dyn SimpleAST<N>>>;

    /// Return a list of mutable references to the children of the node.
    fn get_children_mut(&mut self) -> Vec<Box<&mut dyn SimpleAST<N>>>;
}

/// Type used to traverse an abstract syntax tree of a language.
///
/// Includes traversal functions for mutable and non-mutable traversals.
pub struct ASTTraverser {}

impl ASTTraverser {
    /// Traverse an abstract syntax tree, but do not mutate the syntax tree.  Return true if the
    /// traversal should abort early.
    ///
    /// # Trait Bounds
    ///
    /// * `N` - Represents the language specific syntax tree type.
    /// * `A` - The type that conforms to the SimpleAST trait.
    /// * `V` - The visitor type for visiting nodes of type `N`.
    ///
    /// # Arguments
    ///
    /// * `tree_node` - The node of the syntax tree to traverse.
    /// * `visitor` - The visitor object that implements the algorithm needed to produce the desired
    /// result from traversing the syntax tree.
    pub fn traverse<N, A: SimpleAST<N> + ?Sized, V: Visitor<N> + ?Sized>(
        tree_node: &A,
        visitor: &mut V,
    ) -> bool {
        let node = tree_node.get_node();

        // Let the visitor know that we are starting to process the node.
        visitor.on_enter(node);

        // Have the visitor process the node.
        let should_stop = visitor.visit(node);
        if should_stop {
            visitor.on_exit(node);
            return true;
        }

        // Check to see if this visitor needs this algorithm to traverse the node's children.  Some
        // visitors will traverse child nodes in order to correctly in-order process the syntax
        // tree. If the visitor does not traverse the children then we will traverse the children
        // in this algorithm.
        if visitor.visit_children(node) {
            let children = tree_node.get_children();
            for child in children {
                // Traverse each child of the node.
                let should_stop = ASTTraverser::traverse(*child, visitor);
                if should_stop {
                    // Inform the visitor that we will leave this node.
                    visitor.on_exit(node);
                    return true;
                }
            }
        }
        // Inform the visitor that we will leave this node.
        visitor.on_exit(node);
        false
    }

    /// Traverse an abstract syntax tree, possibly mutating the syntax tree.  Return true if the
    /// traversal should abort early.
    ///
    /// # Trait Bounds
    ///
    /// * `N` - Represents the language specific syntax tree type.
    /// * `A` - The type that conforms to the SimpleAST trait.
    /// * `V` - The visitor type for visiting nodes of type `N`.
    ///
    /// # Arguments
    ///
    /// * `tree_node` - The node of the syntax tree to traverse.
    /// * `visitor` - The visitor object that implements the algorithm needed to produce the desired
    /// result from traversing the syntax tree.
    pub fn traverse_mut<N, A: SimpleAST<N> + ?Sized, V: VisitorMut<N> + ?Sized>(
        tree_node: &mut A,
        visitor: &mut V,
    ) -> bool {
        // We do not extract the node from tree_node here.  Extracting the node mutably borrows a reference
        // that we would hold on to for the duration of the function.  Later, this function needs to borrow
        // another mutable reference in order to traverse the children. Rust (of course) does not allow
        // more than one mutable reference to an object at any given time, so we instead make use of
        // the tree_node.get_node_mut() function to borrow mutably only when absolutely necessary.

        // Inform the visitor that we have started processing the node.
        visitor.on_enter(tree_node.get_node_mut());

        // Have the visitor fully process the node.
        let mut should_stop = visitor.visit_mut(tree_node.get_node_mut());
        if should_stop {
            visitor.on_exit(tree_node.get_node_mut());
            return true;
        }

        // Check to see if this visitor needs this algorithm to traverse the node's children.  Some
        // visitors will traverse child nodes in order to correctly in-order process the syntax
        // tree. If the visitor does not traverse the children then we will traverse the children
        // in this algorithm.
        if visitor.visit_children(tree_node.get_node_mut()) {
            let children = tree_node.get_children_mut();
            for child in children {
                // Traverse each child node.
                should_stop = ASTTraverser::traverse_mut(*child, visitor);
                if should_stop {
                    // We do not call visitor.on_exit() here because that would involve a second
                    // mutable borrow from tree_node in this scope.
                    break;
                }
            }
        }
        // Inform the visitor that we will leave this node.
        visitor.on_exit(tree_node.get_node_mut());
        if should_stop {
            return true;
        }
        false
    }
}
