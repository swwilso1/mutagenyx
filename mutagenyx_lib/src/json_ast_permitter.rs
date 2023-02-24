//! The `json_ast_permitter` object provides an implementation of a permitter object for
//! JSON based abstract syntax trees.

use crate::permissions::*;
use crate::permit::*;
use serde_json::Value;

/// An object that tracks permissions for accessing node types.
pub struct JSONPermitter<'a> {
    /// The permissions object.
    permissions: &'a Permissions,
}

impl<'a> JSONPermitter<'a> {
    /// Create a new permitter object.
    ///
    /// # Arguments
    ///
    /// * `permissions` - The permissions that control the behavior of the permitter.
    pub fn new(permissions: &'a Permissions) -> JSONPermitter<'a> {
        JSONPermitter { permissions }
    }
}

impl<'a> Permit<Value> for JSONPermitter<'a> {
    fn has_permission_to(&self, action: PermissionAction) -> bool {
        self.permissions.have_permission(action)
    }
}
