//! The `permit` module provides a trait that the adds the ability to query an AST node for
//! permission before exercising functionality.

/// Query string that means: 'do I have permission to mutate a node?'
pub static MUTATE: &str = "mutate";

/// Query string that means: 'do I have permission to count a node?'
pub static COUNT: &str = "count";

/// Query string that means: 'do I have permission to visit a node?'
pub static VISIT: &str = "visit";

pub trait Permit<AST> {
    /// Return true if the trait object has permission to do operation `query` on `node`
    ///
    /// # Arguments
    ///
    /// * `query` - A string representing the operation to perform on `node`.
    /// * `node` - The node on which to perform the operation represented by `query`.
    fn has_permission_to(&self, query: &str, node: &AST) -> bool;
}
