//! The `id` module provides trait [`Id`] for generating a unique u64 value for an AST node.

pub trait Id<AST> {
    /// Return a unique u64 number for `node`.
    ///
    /// # Arguments
    ///
    /// * `node` - The node in the AST.
    fn get_id(&self, node: &AST) -> Option<u64>;
}
