//! The `visitor` module provides the traits that describe the functionality an object must have
//! to traverse an abstract syntax tree.

/// Trait that describes the behavior required from objects that traverse an abstract syntax
/// tree, but the object does not mutate the syntax tree.
///
/// The trait provides three visitation methods to enable the visitor to process the node with
/// more precision:
///
/// * `on_enter` - called as the visitor starts to examine the node.
/// * `visit` - called when the visitor should fully handle the node.
/// * `on_exit` - called when the visitor should stop examining the node.
///
/// These three methods allow for more flexibility in processing a node.  For instance, when
/// trying to pretty-print a node, the syntax tree may need to behave slightly differently for
/// each of these function calls. For some languages, the `visit` method will traverse children
/// of a node in a special way and then need to provide some closing behavior when the visitor
/// finishes with the node in `on_exit`.
///
/// The trait's generic type refers to an abstract syntax tree of a particular language.
pub trait Visitor<N: ?Sized> {
    /// The traversal algorithm calls this function when the visitor enters a node.
    ///
    /// # Arguments
    ///
    /// * `node` - A reference to the node that the visitor has started visiting.
    fn on_enter(&mut self, _node: &N) {}

    /// Return true if the visitor can visit a node.
    ///
    /// During traversal, the traversal algorithm will enter the node.  Then, prior to calling
    /// visit, the algorithm will see if the visitor has permission to enter the node.  If it does
    /// not have permission, the traversal algorithm will skip directly to the node exit call.
    ///
    /// # Arguments
    ///
    /// * `node` - A reference to the node that the visitor wants to visit.
    fn have_permission_to_visit(&self, _node: &N) -> bool {
        true
    }

    /// The traversal algorithm calls this function when the visitor should fully process `node`.
    /// This function should return true if the visitor needs to prematurely terminate the traversal
    /// of the syntax tree.
    ///
    /// # Arguments
    ///
    /// * `node` - A reference to the node that the visitor should should process.
    fn visit(&mut self, _node: &N) -> bool {
        false
    }

    /// Return true if the main traversal algorithm should process the nodes children.
    ///
    /// Depending on the purpose of the visitor, the visitor may internally traverse a node's
    /// children to more correctly handle parts of the syntax tree.  In these cases, the main
    /// traversal algorithm should not traverse the children a second time.
    ///
    /// # Arguments
    ///
    /// * `node` - A reference to the node.
    fn visit_children(&mut self, _node: &N) -> bool {
        true
    }

    /// Alert the visitor that the traversal algorithm will now traverse children.
    ///
    /// # Arguments
    ///
    /// * `node` - A reference to the node.
    fn on_start_visit_children(&mut self, _node: &N) {}

    /// Alert the visitor that the traversal algorithm will stop traversing children.
    ///
    /// # Arguments
    ///
    /// * `node` - A reference to the node.
    fn on_end_visit_children(&mut self, _node: &N) {}

    /// The traversal algorithm calls this function just after processing the children of `node`
    /// and before moving on to the next node in the syntax tree.
    ///
    /// # Arguments
    ///
    /// * `node` - A reference to the current node.
    fn on_exit(&mut self, _node: &N) {}
}

/// Trait that describes the behavior required from objects that traverse an abstract syntax
/// tree for the object mutates the syntax tree.
///
/// This trait behaves in the same manner as [`Visitor`] but operates on mutable nodes of the
/// syntax tree so that the object that implements this trait can mutate the tree.
pub trait VisitorMut<N: ?Sized> {
    /// The traversal algorithm calls this function when the visitor enters a node.
    ///
    /// # Arguments
    ///
    /// * `node` - A mutable reference to the node that the visitor has started visiting.
    fn on_enter(&mut self, _node: &mut N) {}

    /// Return true if the visitor can visit a node.
    ///
    /// During traversal, the traversal algorithm will enter the node.  Then, prior to calling
    /// visit, the algorithm will see if the visitor has permission to enter the node.  If it does
    /// not have permission, the traversal algorithm will skip directly to the node exit call.
    ///
    /// # Arguments
    ///
    /// * `node` - A reference to the node that the visitor wants to visit.
    fn have_permission_to_visit(&self, _node: &N) -> bool {
        true
    }

    /// The traversal algorithm calls this function when the visitor should fully process `node`.
    /// This function should return true if the visitor needs to prematurely terminate the traversal
    /// of the syntax tree.
    ///
    /// # Arguments
    ///
    /// * `node` - A mutable reference to the node that the visitor should should process.
    fn visit_mut(&mut self, _node: &mut N) -> bool {
        false
    }

    /// Return true if the main traversal algorithm should process the nodes children.
    ///
    /// Depending on the purpose of the visitor, the visitor may internally traverse a node's
    /// children to more correctly handle parts of the syntax tree.  In these cases, the main
    /// traversal algorithm should not traverse the children a second time.
    ///
    /// # Arguments
    ///
    /// * `node` - A mutable reference to the node.
    fn visit_children(&mut self, _node: &mut N) -> bool {
        true
    }

    /// Alert the visitor that the traversal algorithm will now traverse children.
    ///
    /// # Arguments
    ///
    /// * `node` - A reference to the node.
    fn on_start_visit_children(&mut self, _node: &N) {}

    /// Alert the visitor that the traversal algorithm will stop traversing children.
    ///
    /// # Arguments
    ///
    /// * `node` - A reference to the node.
    fn on_end_visit_children(&mut self, _node: &N) {}

    /// The traversal algorithm calls this function just after processing the children of `node`
    /// and before moving on to the next node in the syntax tree.
    ///
    /// # Arguments
    ///
    /// * `node` - A reference to the current node.
    fn on_exit(&mut self, _node: &mut N) {}
}
