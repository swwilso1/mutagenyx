//! The `json_comment_helpers` module contains code shared between language specific commenters
//! when the language has a JSON AST representation.

use crate::id::Id;
use crate::json::JSONMutate;
use serde_json::Value;

/// Return true if the function inserts the `comment` node in `array`.
///
/// # Arguments
///
/// * `array` - The list of nodes.
/// * `comment` - The comment node to insert in `array`.
/// * `node_id` - The id of the node that is associated with `comment`.
/// * `id_maker` - The [`Id<AST>`] object that can convert a node to and id value.
pub fn inserted_comment_in_array(
    array: &mut Vec<Value>,
    comment: Value,
    node_id: u64,
    id_maker: &dyn Id<Value>,
) -> bool {
    let mut i: usize = 0;
    while i < array.len() {
        let array_node = &mut array[i];
        if array_node.get_node_with_id(node_id, id_maker).is_some() {
            array.insert(i, comment);
            return true;
        }
        i += 1;
    }
    false
}
