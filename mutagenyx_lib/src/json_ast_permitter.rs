//! The `json_ast_permitter` object provides an implementation of a permitter object for
//! JSON based abstract syntax trees.

use crate::json::JSONMutate;
use crate::permissions::Permissions;
use crate::permit::*;
use serde_json::Value;

/// An object that tracks permissions for accessing node types.
pub struct JSONPermitter<'a> {
    /// The permissions object.
    permissions: &'a Permissions,

    /// The key from the JSON AST to check.  The permitter will look in a node for this key and
    /// compare the associated string value with `node_name_value`.
    node_type_key: &'a str,

    /// The string to compare with the string value from the `node_type_key` element.
    node_name_value: &'a str,
}

impl<'a> JSONPermitter<'a> {
    /// Create a new permitter object.
    ///
    /// # Arguments
    ///
    /// * `permissions` - The permissions that control the behavior of the permitter.
    /// * `node_type_key` - The string containing the AST node key that contains the type of AST
    /// node.
    /// * `node_name_value` - The value to match from the string in the `node_type_key`.
    pub fn new(
        permissions: &'a Permissions,
        node_type_key: &'a str,
        node_name_value: &'a str,
    ) -> JSONPermitter<'a> {
        JSONPermitter {
            permissions,
            node_type_key,
            node_name_value,
        }
    }
}

impl<'a> Permit<Value> for JSONPermitter<'a> {
    fn has_permission_to(&self, query: &str, node: &Value) -> bool {
        // If we do not have any permissions then we can allow access to all nodes.
        if self.permissions.is_empty() {
            return true;
        }
        if query == MUTATE || query == COUNT || query == VISIT {
            // Now look for a node_name_value node.
            if let Some(ast_type) = node.get_str_for_key(self.node_type_key) {
                if ast_type == self.node_name_value {
                    // See if the node has a 'name' element.
                    return if let Some(name) = node.get_str_for_key("name") {
                        // It does have a 'name', so see if we have permissions to access the node.
                        let permission_str = String::from("mutate.") + name;
                        self.permissions.have_permission(&permission_str)
                    } else {
                        false
                    };
                }
            }
        }
        true
    }
}
