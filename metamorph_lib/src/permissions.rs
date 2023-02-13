//! The `permissions` module provides a simple wrapper around a [`HashMap<String,bool>`] map that
//! provides the syntax to write code that more naturally reads:
//! ```rust.ignore
//! if permissions.have_permission("to.do.something") {
//!     ...
//! }
//! ```

use std::collections::HashMap;

/// Simple object to track permissions according to string keys.
#[derive(Debug, Clone)]
pub struct Permissions {
    /// The underlying map.
    permissions: HashMap<String, bool>,
}

impl Permissions {
    /// Create a new permissions object.
    pub fn new() -> Permissions {
        Permissions {
            permissions: HashMap::new(),
        }
    }

    /// Set `permissions` value for `key`.
    ///
    /// # Arguments
    ///
    /// * `key` - The string for the key.
    /// * `permissions` - True if the object has permission for `key`.
    pub fn set_permission(&mut self, key: &str, permission: bool) {
        self.permissions.insert(String::from(key), permission);
    }

    /// Return true if the object has permission for `key`. Return false if the object does not
    /// have permission or does not have a record for `key`.
    ///
    /// # Arguments
    ///
    /// * `key` - The string for the key.
    pub fn have_permission(&self, key: &str) -> bool {
        if let Some(b) = self.permissions.get(key) {
            *b
        } else {
            false
        }
    }

    /// Return the number of permissions in the object.
    pub fn len(&self) -> usize {
        self.permissions.len()
    }

    /// Return true if the object has no permissions.
    pub fn is_empty(&self) -> bool {
        self.permissions.is_empty()
    }

    /// Return an array of the permission keys.
    pub fn keys(&self) -> Vec<String> {
        let mut key_array: Vec<String> = Vec::new();
        for key in self.permissions.keys() {
            key_array.push(key.clone());
        }
        key_array
    }

    /// Clear all permissions in the object.
    pub fn clear(&mut self) {
        self.permissions.clear();
    }
}

impl Default for Permissions {
    fn default() -> Self {
        Self::new()
    }
}
