//! The `json_namer` module provides a JSON AST specific implementation of an object with the
//! [`Namer<T>`] trait. ASTs represented in JSON can use this generic namer type and provide any
//! AST specific conversion code from a node to a string in the closure.

use crate::namer::Namer;
use serde_json::Value;

/// Generic JSON [`Namer<Value>`] object that can convert a node in a JSON AST to a [`String`]
/// representing the node.
pub struct JSONNamer<F>
where
    F: Fn(&Value) -> Option<String>,
{
    /// A closure that implements AST specific code to convert an AST node for a particular language
    /// to a name string.
    value_to_name_converter: F,
}

impl<F> JSONNamer<F>
where
    F: Fn(&Value) -> Option<String>,
{
    /// Create a new JSON namer object.
    ///
    /// # Arguments
    ///
    /// * `f` - The closure that invokes AST specific transformations to convert an AST node for
    /// a particular language to a string.
    pub fn new(f: F) -> JSONNamer<F> {
        JSONNamer {
            value_to_name_converter: f,
        }
    }
}

impl<F> Namer<Value> for JSONNamer<F>
where
    F: Fn(&Value) -> Option<String>,
{
    fn get_name(&self, thing: &Value) -> Option<String> {
        (self.value_to_name_converter)(thing)
    }
}
