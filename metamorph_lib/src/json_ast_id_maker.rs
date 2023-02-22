//! The `json_ast_id_maker` module provides the [`JSONIDMaker<F>`] object that can be used
//! to convert JSON [`Value`] nodes to an id value.

use crate::id::Id;
use serde_json::Value;

/// Object that supports converting a [`Value`] node to an id value.
pub struct JSONIDMaker<F>
where
    F: Fn(&Value) -> Option<u64>,
{
    /// The closure function that receives a [`Value`] node object and converts the node to an
    /// id value.
    f: F,
}

impl<F> JSONIDMaker<F>
where
    F: Fn(&Value) -> Option<u64>,
{
    /// Create a new json id maker object.
    ///
    /// # Arguments
    ///
    /// * `f` - The closure function.
    pub fn new(f: F) -> JSONIDMaker<F> {
        JSONIDMaker { f }
    }
}

impl<F> Id<Value> for JSONIDMaker<F>
where
    F: Fn(&Value) -> Option<u64>,
{
    fn get_id(&self, node: &Value) -> Option<u64> {
        (self.f)(node)
    }
}
