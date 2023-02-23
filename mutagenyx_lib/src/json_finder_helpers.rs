//! The `json_finder_helpers` module contains code shared between language specific finders
//! when the language has a JSON AST representation.

use crate::id::Id;
use crate::json::JSONMutate;
use serde_json::Value;

/// Return true if `node_id` was found in a node that is a direct member of `array`.
///
/// # Arguments
///
/// * `array` - The array of nodes.
/// * `node_id` - The id number of the node of interest.
/// * `id_maker` - The [`Id<AST>`] object that can convert nodes to node ids.
pub fn found_node_id_in_array(
    array: &mut Vec<Value>,
    node_id: u64,
    id_maker: &dyn Id<Value>,
) -> bool {
    let mut i: usize = 0;
    while i < array.len() {
        let array_node = &array[i];
        if array_node.get_node_with_id(node_id, id_maker).is_some() {
            return true;
        }
        i += 1;
    }
    false
}
