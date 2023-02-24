//! The `permit` module provides a trait that the adds the ability to query an AST node for
//! permission before exercising functionality.

use crate::permissions::PermissionAction;

/// Trait that gives an implementing object the ability to answer permission questions about
/// requested actions.
pub trait Permit<AST> {
    /// Return true if the trait object has permission to do operation `action` on `node`
    ///
    /// # Arguments
    ///
    /// * `action` - A [`PermissionAction`] object representing the requested permission for `node`.
    /// * `node` - The node on which to perform the operation represented by `action`.
    fn has_permission_to(&self, action: PermissionAction) -> bool;
}
