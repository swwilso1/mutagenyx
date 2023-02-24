//! The `permissions` module provides a simple wrapper around a map that
//! stores permissions for the tool to perform specific actions on nodes.
//! ```rust.ignore
//! if permissions.have_permission(action, node) {
//!     ...
//! }
//! ```

/// The Permit trait will allow you to query for permissions to perform an operation
/// on a node of an AST.  Using strings for example, we can query for the permissions in
/// the following way:
///
/// `action.scope.name`
///
/// * Verb represents the operation the caller wants to perform on the node.
/// * Scope represents the kind of node to which the operation will apply.
/// * Name represents the id value of the node to which the Verb/Scope should apply.
///
/// We explicitly do not use strings to represent permission queries.  Programmers can
/// easily make mistakes trying to do string manipulation to get the query syntax correct
/// and so instead we leverage Rust's enum facilities to encode the permission query.

/// The type representing the action portion of a query.
#[derive(Debug, Clone, PartialEq)]
pub enum PermissionAction {
    /// Indicates a request to mutate a node with the specific scope.
    Mutate(PermissionScope),

    /// Indicates a request to visit a node with the specific scope during an AST traversal.
    Visit(PermissionScope),
}

impl PermissionAction {
    /// Return true if this permission action matches `action`
    ///
    /// # Arguments
    ///
    /// * `action` - The other [`PermissionAction`] object to match.
    pub fn matches(&self, action: &PermissionAction) -> bool {
        match self {
            PermissionAction::Mutate(scope) => match action {
                PermissionAction::Mutate(ascope) => scope.matches(ascope),
                PermissionAction::Visit(_) => false,
            },
            PermissionAction::Visit(scope) => match action {
                PermissionAction::Mutate(_) => false,
                PermissionAction::Visit(ascope) => scope.matches(ascope),
            },
        }
    }
}

/// The type representing the scope of a permissions query.
#[derive(Debug, Clone, PartialEq)]
pub enum PermissionScope {
    /// Type refers to the node type.  ASTs often have node types describing the purpose of the
    /// node in the AST.  Eg FunctionDefinition in Solidity or FunctionDef in Vyper indicate a
    /// function definition in these respective language.  A scope of Type indicates that the
    /// query will apply to `types` of nodes in the AST.
    Type(PermissionObject),

    /// Name refers to the node's name. ASTs often have a name value for a node type (although
    /// the actual key for the name may differ from AST to AST).  For example, function definition
    /// nodes usually have the name of the function as its 'name'.  A scope of Name indicates that
    /// the permission query applies to nodes in the AST that have particular names.
    Name(PermissionObject),

    /// Children refers to a node's child nodes. The query here might look like:
    /// `mutate.children.function_name.allow`.  This scope allows the tool to allow mutation of the
    /// children of specific, named nodes.
    Children(PermissionObject),

    /// Any allows the PermissionAction scope to apply to any valid sub-scope.  In string form:
    /// `action.*`.
    Any(Permission),
}

impl PermissionScope {
    /// Return true if this scope matches the scope in `other`.
    ///
    /// # Arguments
    ///
    /// * `other` - The other scope object.
    pub fn matches(&self, other: &PermissionScope) -> bool {
        match self {
            PermissionScope::Type(object) => match other {
                PermissionScope::Type(other_object) => object.matches(other_object),
                _ => false,
            },
            PermissionScope::Name(object) => match other {
                PermissionScope::Name(other_object) => object.matches(other_object),
                _ => false,
            },
            PermissionScope::Children(object) => match other {
                PermissionScope::Children(other_object) => object.matches(other_object),
                _ => false,
            },
            PermissionScope::Any(perm) => match other {
                PermissionScope::Type(object) => match object {
                    PermissionObject::Value(_, other_perm) => perm.matches(other_perm),
                    PermissionObject::Any(other_perm) => perm.matches(other_perm),
                },
                PermissionScope::Name(object) => match object {
                    PermissionObject::Value(_, other_perm) => perm.matches(other_perm),
                    PermissionObject::Any(other_perm) => perm.matches(other_perm),
                },
                PermissionScope::Children(object) => match object {
                    PermissionObject::Value(_, other_perm) => perm.matches(other_perm),
                    PermissionObject::Any(other_perm) => perm.matches(other_perm),
                },
                PermissionScope::Any(other_perm) => perm.matches(other_perm),
            },
        }
    }
}

/// The type representing the focus of the permission query.
#[derive(Debug, Clone, PartialEq)]
pub enum PermissionObject {
    /// Value refers to a specific moniker for the scope of a query.  For example, `mutate.name.foo`
    /// would request permission to mutate nodes that have the name foo.  We use
    /// PermissionVerb::Mutate(PermissionScope::Name(PermissionObject::Value("foo"))) to represent
    /// foo in this query.
    Value(String, Permission),

    /// Any refers to a special wildcard kind of focus.  In string form: `mutate.name.*`.
    Any(Permission),
}

impl PermissionObject {
    /// Return true if this object matches the object in `other`.
    ///
    /// # Arguments
    ///
    /// * `other` - The other object.
    pub fn matches(&self, other: &PermissionObject) -> bool {
        match self {
            PermissionObject::Value(s, perm) => match other {
                PermissionObject::Value(other_s, other_perm) => {
                    s == other_s && perm.matches(other_perm)
                }
                _ => false,
            },
            PermissionObject::Any(perm) => match other {
                PermissionObject::Value(_, other_perm) => perm.matches(other_perm),
                PermissionObject::Any(other_perm) => perm.matches(other_perm),
            },
        }
    }
}

/// The type representing the actual permission, the proverbial 'yes' or 'no'.
#[derive(Debug, Clone, PartialEq)]
pub enum Permission {
    /// Permit the operation to continue.
    Allow,

    /// Do not permit the operation to continue.
    Deny,
    // /// The Permission operation is not assigning a permission, but querying for permission.
    // Query,
}

impl Permission {
    pub fn matches(&self, other: &Permission) -> bool {
        match self {
            Permission::Allow => match other {
                Permission::Allow => true,
                _ => false,
            },
            Permission::Deny => match other {
                Permission::Deny => true,
                _ => false,
            },
            // Permission::Query => true,
        }
    }
}

/// Simple object to track permissions according to string keys.
#[derive(Debug, Clone)]
pub struct Permissions {
    /// The underlying map.
    permissions: Vec<PermissionAction>,
}

impl Permissions {
    /// Create a new permissions object.
    pub fn new() -> Permissions {
        Permissions {
            permissions: Vec::new(),
        }
    }

    /// Set `permissions` value for `key`.
    ///
    /// # Arguments
    ///
    /// * `key` - The string for the key.
    /// * `permissions` - True if the object has permission for `key`.
    pub fn add_permission(&mut self, action: PermissionAction) {
        self.permissions.push(action)
    }

    /// Remove a permission setting from the Permissions object.
    ///
    /// # Arguments
    ///
    /// * `action` - The [`PermissionAction`] to remove, if present.
    pub fn remove_permission(&mut self, action: PermissionAction) {
        for (index, value) in self.permissions.iter().enumerate() {
            if *value == action {
                // This removal has O(n) performance, but that should work; we do not heavily use
                // permissions.  We should also preserve order because permissions may get added so
                // that fall through permissions occur towards the end of the permissions array.
                // Using something like .swap_remove() would break any ordering behavior.
                self.permissions.remove(index);
                break;
            }
        }
    }

    /// Return true if the object has permission for `action`.
    ///
    /// # Arguments
    ///
    /// * `action` - The requested action permission.
    pub fn have_permission(&self, action: PermissionAction) -> bool {
        for stored_action in &self.permissions {
            if stored_action.matches(&action) {
                return true;
            }
        }
        false
    }

    /// Return the number of permissions in the object.
    pub fn len(&self) -> usize {
        self.permissions.len()
    }

    /// Return true if the object has no permissions.
    pub fn is_empty(&self) -> bool {
        self.permissions.is_empty()
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
