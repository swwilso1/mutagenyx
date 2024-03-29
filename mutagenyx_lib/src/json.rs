//! The module contains various functions and traits that simplify working with
//! [`Value`] objects for the purpose of traversing and mutating abstract syntax trees
//! encoded in JSON.
use crate::error::MutagenyxError;
use crate::id::Id;
#[cfg(test)]
use crate::json_ast_id_maker::JSONIDMaker;
use serde_json::{from_str, json, Map, Value};
use std::fs::File;
use std::io::BufReader;

/// Reads the contents of the JSON in the file named by `file_name`.
///
/// # Arguments
///
/// * `file_name` - The string slice containing the path to the file in the file system.
///
/// # Errors
///
/// This function interacts with lower-level code to read JSON data from a file. As a result,
/// the function may return [`MutagenyxError::IO`] that contains a [`std::io::Error`].
///
/// Also, the function invokes the `serde_json` JSON parser.  The JSON parser may return a
/// [`serde_json::Error`] which the function will return as a [`MutagenyxError::JSON`] enum that
/// wraps the error from `serde_json`.
pub fn load_json_from_file_with_name(file_name: &str) -> Result<Value, MutagenyxError> {
    let file = match File::open(file_name) {
        Ok(f) => f,
        Err(e) => return Err(MutagenyxError::from(e)),
    };

    let reader = BufReader::new(file);

    let v = match serde_json::from_reader(reader) {
        Ok(val) => val,
        Err(e) => return Err(MutagenyxError::from(e)),
    };

    Ok(v)
}

/// Add a '/' character as a prefix to a string slice.
///
/// The function exists to allow an `json` module function to take a [`String`] or
/// [`&str`], and convert the string into a JSON path format for use with
/// the [`Value::pointer`] and [`Value::pointer_mut`] functions.
///
/// # Arguments
///
/// * `path` - The string slice containing the name to decorate.
fn json_path(path: &str) -> String {
    let mut jp = String::from("/");
    jp += path;
    jp
}

/// Return a JSON node by creating the node from `text`.
///
/// # Arguments
///
/// * `text` - The string slice referring to the text that contains JSON.
pub fn new_json_node(text: &str) -> Result<Value, MutagenyxError> {
    match from_str(text) {
        Ok(n) => Ok(n),
        Err(e) => Err(MutagenyxError::from(e)),
    }
}

/// Trait for use with [`Value`] type that adds functionality for accessing and
/// modifying the contents of the JSON node contained in the objects of the type.
pub trait JSONMutate {
    fn set_string(&mut self, value: &str);
    fn has_value_for_key(&self, key: &str) -> bool;
    fn take_value_for_key(&mut self, key: &str) -> Option<Value>;
    fn set_node_for_key(&mut self, key: &str, node: Value);
    fn set_node_for_key_at_index(&mut self, key: &str, index: usize, node: Value);
    fn push_node(&mut self, node: Value);
    fn get_array_for_key(&self, key: &str) -> Option<&Vec<Value>>;
    fn get_array_for_key_mut(&mut self, key: &str) -> Option<&mut Vec<Value>>;
    fn get_map_for_key(&self, key: &str) -> Option<&Map<String, Value>>;
    fn get_map_for_key_mut(&mut self, key: &str) -> Option<&mut Map<String, Value>>;
    fn get_str_for_key(&self, path: &str) -> Option<&str>;
    fn set_str_for_key(&mut self, path: &str, value: &str);
    fn get_bool_for_key(&self, key: &str) -> Option<bool>;
    fn get_int_for_key(&self, key: &str) -> Option<i64>;
    fn get_float_for_key(&self, key: &str) -> Option<f64>;
    fn contains_key(&self, key: &str) -> bool;
    fn get_node_with_id(&self, id: u64, id_maker: &dyn Id<Value>) -> Option<&Value>;
    fn get_node_with_id_mut(&mut self, id: u64, id_maker: &dyn Id<Value>) -> Option<&mut Value>;
}

impl JSONMutate for Value {
    /// Change the current [`Value`] object so that it now contains a [`String`] copy
    /// of the argument `value`.
    ///
    /// # Arguments
    ///
    /// * `value` - A string slice to insert into the [`Value`] object.
    fn set_string(&mut self, value: &str) {
        *self = json![value];
    }

    /// Return true if the [`Value`] object has key/value pair indexed
    /// by `key`
    ///
    /// # Arguments
    ///
    /// * `key` - A string slice referring to the text of the key.
    fn has_value_for_key(&self, key: &str) -> bool {
        let json_key = json_path(key);
        matches!(self.pointer(&json_key), Some(_))
    }

    /// Assuming the [`Value`] object represents a JSON dictionary/map object, then
    /// the function will return the [`Value`] object stored in the dictionary for
    /// `key`. If the dictionary does not contain a value for `key`, the function will
    /// return `None`.  This function returns ownership of the [`Value`] object.
    ///
    /// The caller should use [`Value::is_object`] to check for a JSON dictionary/map
    /// prior to calling this function.
    ///
    /// # Arguments
    ///
    /// * `key` - The string slice referencing the text comprising the key.
    fn take_value_for_key(&mut self, key: &str) -> Option<Value> {
        let json_key = json_path(key);
        self.pointer_mut(&json_key).map(|v| v.take())
    }

    /// Assuming the [`Value`] object represents a JSON dictionary/map object, then
    /// the function will store the value in `node` for `key` in the dictionary/map object.
    ///
    /// The caller should use [`Value::is_object`] to check for a JSON dictionary/map
    /// prior to calling this function.
    ///
    /// # Arguments
    ///
    /// * `key` - The string slice referencing the text comprising the key.
    /// * `node` - The [`Value`] object to use as the new value for `key` in the JSON
    /// dictionary.
    fn set_node_for_key(&mut self, key: &str, node: Value) {
        let json_path = json_path(key);
        if let Some(v) = self.pointer_mut(&json_path) {
            *v = node;
        } else {
            // The node does not have an existing value for `key`.  We need to get
            // the lower level map and insert the key/value pair.
            if let Some(node_map) = self.as_object_mut() {
                node_map.insert(String::from(key), node);
            }
        }
    }

    /// Assuming the [`Value`] object represents a JSON dictionary/map object, and that
    /// the dictionary contains a JSON array stored for `key`, then insert `node` into the array
    /// at `index`.
    ///
    /// The caller should use [`Value::is_object`] and [`Value::is_array`]
    /// to check for a JSON dictionary/map that contains a JSON array.
    ///
    /// # Arguments
    ///
    /// * `key` - The string slice referencing the text comprising the key.
    /// * `index` - The index in the JSON array that will contain `node`.
    /// * `node` - The [`Value`] object to insert into the array.
    fn set_node_for_key_at_index(&mut self, key: &str, index: usize, node: Value) {
        let json_path = json_path(key);
        if let Some(v) = self.pointer_mut(&json_path) {
            if let Some(a) = v.get_mut(index) {
                *a = node;
            }
        }
    }

    /// Assume the ['Value'] object represents a JSON array object, append the value of `node`
    /// to the array.
    ///
    /// The caller should use [`Value::is_array`] to check for a JSON array.
    ///
    /// # Arguments
    ///
    /// * `node` - The [`Value`] object to append to the array.
    fn push_node(&mut self, node: Value) {
        if let Some(array) = self.as_array_mut() {
            array.push(node);
        }
    }

    /// Assuming the [`Value`] object represents a JSON dictionary/map object, and that
    /// the dictionary contains an entry for `key` that holds a JSON array, return a reference to
    /// that array.
    ///
    /// The caller should use [`Value::is_object`] and [`Value::is_array`]
    /// to check for a JSON dictionary/map that contains a JSON array.
    ///
    /// # Arguments
    ///
    /// * `key` - The string slice referencing the text comprising the key.
    fn get_array_for_key(&self, key: &str) -> Option<&Vec<Value>> {
        let json_path = json_path(key);
        match self.pointer(&json_path) {
            Some(v) => v.as_array(),
            _ => None,
        }
    }

    /// Assuming the [`Value`] object represents a JSON dictionary/map object, and that
    /// the dictionary contains an entry for `key` that holds a JSON array, return a mutable
    /// reference to that array.
    ///
    /// The caller should use [`Value::is_object`] and [`Value::is_array`]
    /// to check for a JSON dictionary/map that contains a JSON array.
    ///
    /// # Arguments
    ///
    /// * `key` - The string slice referencing the text comprising the key.
    fn get_array_for_key_mut(&mut self, key: &str) -> Option<&mut Vec<Value>> {
        let json_path = json_path(key);
        match self.pointer_mut(&json_path) {
            Some(v) => v.as_array_mut(),
            _ => None,
        }
    }

    /// Assuming the [`Value`] object represents a JSON dictionary/map object, and that
    /// the dictionary contains an entry for `key` that holds a JSON map, return a reference
    /// to that map.
    ///
    /// The caller should use [`Value::is_object`] to check for a JSON dictionary/map that contains
    /// a JSON map.
    ///
    /// # Arguments
    ///
    /// * `key` - The string slice referencing the text comprising the key.
    fn get_map_for_key(&self, key: &str) -> Option<&Map<String, Value>> {
        let json_path = json_path(key);
        match self.pointer(&json_path) {
            Some(v) => v.as_object(),
            _ => None,
        }
    }

    /// Assuming the [`Value`] object represents a JSON dictionary/map object, and that
    /// the dictionary contains an entry for `key` that holds a JSON map, return a mutable reference
    /// to that map.
    ///
    /// The caller should use [`Value::is_object`] to check for a JSON dictionary/map that contains
    /// a JSON map.
    ///
    /// # Arguments
    ///
    /// * `key` - The string slice referencing the text comprising the key.
    fn get_map_for_key_mut(&mut self, key: &str) -> Option<&mut Map<String, Value>> {
        let json_path = json_path(key);
        match self.pointer_mut(&json_path) {
            Some(v) => v.as_object_mut(),
            _ => None,
        }
    }

    /// Assuming the [`Value`] object represents a JSON dictionary/map object, and that
    /// the dictionary contains an entry for `key` that holds a String object, return a string slice
    /// referencing the text for that string.
    ///
    /// The caller should use [`Value::is_object`] and [`Value::is_string`]
    /// to check for a JSON dictionary/map and for a string object.
    ///
    /// # Arguments
    ///
    /// * `key` - The string slice referencing the text comprising the key.
    fn get_str_for_key(&self, key: &str) -> Option<&str> {
        let json_path = json_path(key);
        match self.pointer(&json_path) {
            Some(v) => v.as_str(),
            _ => None,
        }
    }

    /// Assuming the [`Value`] object represents a JSON dictionary/map object, the
    /// function will add/update the string content from `value` for the `key` member of the
    /// dictionary.
    ///
    /// The caller should use [`Value::is_object`] to check for a JSON dictionary/map.
    ///
    /// # Arguments
    ///
    /// * `key` - The string slice referencing the text comprising the key.
    /// * `value` - The string slice referencing the text comprising the value.
    fn set_str_for_key(&mut self, key: &str, value: &str) {
        let json_path = json_path(key);
        match self.pointer_mut(&json_path) {
            Some(v) => v.set_string(value),
            _ => {
                // The json object has no existing value for 'key'.  Get the map and insert it.
                if let Some(map) = self.as_object_mut() {
                    map.insert(String::from(key), Value::from(value));
                }
            }
        }
    }

    /// Assuming the [`Value`] object represents a JSON dictionary/map object, the
    /// function will return the boolean value stored in the object for `key`.
    ///
    /// The caller should use [`Value::is_object`] and [`Value::is_boolean`] to
    /// check for a JSON dictionary/map and for a boolean value.
    ///
    /// # Arguments
    ///
    /// * `key` - The string slice referencing the text comprising the key.
    fn get_bool_for_key(&self, key: &str) -> Option<bool> {
        let json_path = json_path(key);
        match self.pointer(&json_path) {
            Some(v) => v.as_bool(),
            _ => None,
        }
    }

    /// Assuming the [`Value`] object represents a JSON dictionary/map object,
    /// the function will return the i64 value stored in the object for `key`.
    ///
    /// The caller should use [`Value::is_object`] and [`Value::is_i64`] to
    /// check for a JSON dictionary/map and for an i64 value.
    ///
    /// # Arguments
    ///
    /// * `key` - The string slice referencing the text comprising the key.
    fn get_int_for_key(&self, key: &str) -> Option<i64> {
        let json_path = json_path(key);
        match self.pointer(&json_path) {
            Some(v) => v.as_i64(),
            _ => None,
        }
    }

    /// Assuming the [`Value`] object represents a JSON dictionary/map object,
    /// the function will return the f64 value stored in the object for `key`.
    ///
    /// The caller should use [`Value::is_object`] and [`Value::is_f64`] to
    /// check for a JSON dictionary/man and for an f64 value.
    ///
    /// # Arguments
    ///
    /// * `key` - The string slice referencing the text comprising the key.
    fn get_float_for_key(&self, key: &str) -> Option<f64> {
        let json_path = json_path(key);
        match self.pointer(&json_path) {
            Some(v) => v.as_f64(),
            _ => None,
        }
    }

    /// Return true if the JSON object is an object and it has a key/value pair
    /// for `key`.
    ///
    /// # Arguments
    ///
    /// * `key` - The key for which to search.
    fn contains_key(&self, key: &str) -> bool {
        if self.is_object() {
            if let Some(_node) = self.get(key) {
                return true;
            }
        }
        false
    }

    /// If the node contains another node with id `id`, return a reference to that node.
    ///
    /// # Arguments
    ///
    /// * `id` - The id code value.
    /// * `id_maker` - A [`Id<AST>`] trait object capable of returning the id of an AST node.
    fn get_node_with_id(&self, id: u64, id_maker: &dyn Id<Value>) -> Option<&Value> {
        if let Some(node_id) = id_maker.get_id(self) {
            if node_id == id {
                return Some(self);
            }
        }

        if self.is_array() {
            for value in self.as_array().unwrap() {
                if let Some(sub_node) = value.get_node_with_id(id, id_maker) {
                    return Some(sub_node);
                }
            }
            None
        } else if self.is_object() {
            for (_key, value) in self.as_object().unwrap() {
                if let Some(sub_node) = value.get_node_with_id(id, id_maker) {
                    return Some(sub_node);
                }
            }
            None
        } else {
            None
        }
    }

    /// If the node contains another node with the id `id`, return a mutable reference to that
    /// node.
    ///
    /// # Arguments
    ///
    /// * `id` - The node id to find.
    /// * `id_maker` - The [`Id<AST>`] object that can convert an AST node to an id value.
    fn get_node_with_id_mut(&mut self, id: u64, id_maker: &dyn Id<Value>) -> Option<&mut Value> {
        if let Some(node_id) = id_maker.get_id(self) {
            if node_id == id {
                return Some(self);
            }
        }

        if self.is_array() {
            for value in self.as_array_mut().unwrap() {
                if let Some(sub_node) = value.get_node_with_id_mut(id, id_maker) {
                    return Some(sub_node);
                }
            }
            None
        } else if self.is_object() {
            for (_key, value) in self.as_object_mut().unwrap() {
                if let Some(sub_node) = value.get_node_with_id_mut(id, id_maker) {
                    return Some(sub_node);
                }
            }
            None
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_mutate_set_string() {
        let mut value = Value::Null;
        value.set_string("foo");
        assert!(value.is_string());
        assert_eq!(value.as_str().unwrap(), "foo");
    }

    #[test]
    fn test_json_mutate_has_value_for_key() {
        let value: Value = from_str(
            "{\
            \"dog\": \"bark\",
            \"cat\": \"meow\"
        }",
        )
        .unwrap();

        assert!(value.has_value_for_key("dog"));
        assert!(value.has_value_for_key("cat"));
        assert!(!value.has_value_for_key("horse"));
    }

    #[test]
    fn test_json_mutate_take_value_for_key() {
        let mut value: Value = from_str(
            "{\
            \"first\": 1,
            \"second\": 2
        }",
        )
        .unwrap();

        if let Some(first_node) = value.take_value_for_key("first") {
            assert_eq!(first_node.as_i64().unwrap(), 1);
        }

        if let Some(first_node) = value.get("first") {
            assert!(first_node.is_null());
        }
    }

    #[test]
    fn test_json_mutate_set_node_for_key() {
        let mut value: Value = from_str(
            "{\
            \"first\": 1
        }",
        )
        .unwrap();

        value.set_node_for_key("second", Value::from(2));

        match value.get("second") {
            Some(n) => {
                assert_eq!(n.as_i64().unwrap(), 2);
            }
            None => assert!(false, "Failed to set node for key 'second'"),
        }
    }

    #[test]
    fn test_json_mutate_set_node_for_key_at_index() {
        let mut value: Value = from_str(
            "{\
            \"one\": [\"a\", \"b\", \"c\"]
        }",
        )
        .unwrap();

        let number: Value = json![47];

        value.set_node_for_key_at_index("one", 1, number);

        if let Some(one_node) = value.get("one") {
            if let Some(one_array) = one_node.as_array() {
                assert_eq!(one_array[1].as_i64().unwrap(), 47);
                assert_eq!(one_array.len(), 3);
            } else {
                assert!(false, "Node for key 'one' not an array");
            }
        } else {
            assert!(false, "Node for key 'one' not found");
        }
    }

    #[test]
    fn test_push_node() {
        let mut value: Value = from_str("[1, 2]").unwrap();

        value.push_node(Value::from(10));

        if let Some(array) = value.as_array() {
            assert_eq!(array.len(), 3);
            assert_eq!(array[0].as_i64().unwrap(), 1);
            assert_eq!(array[1].as_i64().unwrap(), 2);
            assert_eq!(array[2].as_i64().unwrap(), 10);
        } else {
            assert!(false, "Value was not a JSON array.");
        }
    }

    #[test]
    fn test_json_mutate_get_array_for_key() {
        let value: Value = from_str(
            "{\
            \"node\": [1, 3, 5]
        }",
        )
        .unwrap();

        if let Some(array) = value.get_array_for_key("node") {
            assert_eq!(array.len(), 3);
            if let Some(node) = array.get(0) {
                assert_eq!(node.as_i64().unwrap(), 1);
            } else {
                assert!(false, "Array did not have anything at index 0");
            }
            if let Some(node) = array.get(1) {
                assert_eq!(node.as_i64().unwrap(), 3);
            } else {
                assert!(false, "Array did not have anything at index 1");
            }
            if let Some(node) = array.get(2) {
                assert_eq!(node.as_i64().unwrap(), 5);
            } else {
                assert!(false, "Array did not have anything at index 2");
            }
        } else {
            assert!(false, "Cannot find value for key 'node'");
        }
    }

    #[test]
    fn test_json_mutate_get_array_for_key_mut() {
        let mut value: Value = from_str(
            "{\
            \"node\": [1, 3, 5]
        }",
        )
        .unwrap();

        if let Some(array) = value.get_array_for_key_mut("node") {
            assert_eq!(array.len(), 3);
            if let Some(node) = array.get(0) {
                assert_eq!(node.as_i64().unwrap(), 1);
            } else {
                assert!(false, "Array did not have anything at index 0");
            }
            if let Some(node) = array.get(1) {
                assert_eq!(node.as_i64().unwrap(), 3);
            } else {
                assert!(false, "Array did not have anything at index 1");
            }
            if let Some(node) = array.get(2) {
                assert_eq!(node.as_i64().unwrap(), 5);
            } else {
                assert!(false, "Array did not have anything at index 2");
            }

            array.push(json![65]);

            if let Some(node) = array.get(3) {
                assert_eq!(node.as_i64().unwrap(), 65);
            } else {
                assert!(false, "Array did not have the newly inserted value 65");
            }
        } else {
            assert!(false, "Cannot find value for key 'node'");
        }
    }

    #[test]
    fn test_json_mutate_get_map_for_key() {
        let value: Value = from_str(
            "{\
            \"node\": {\
                \"one\": 1,
                \"two\": 2
            }
        }",
        )
        .unwrap();

        if let Some(map) = value.get_map_for_key("node") {
            assert_eq!(map.len(), 2);
            if let Some(node) = map.get("one") {
                assert_eq!(node.as_i64().unwrap(), 1);
            } else {
                assert!(false, "map did not contain key/value for key 'one'");
            }
            if let Some(node) = map.get("two") {
                assert_eq!(node.as_i64().unwrap(), 2);
            } else {
                assert!(false, "map did not contain key/value for key 'two'");
            }
        } else {
            assert!(false, "Cannot find value for key 'node'");
        }
    }

    #[test]
    fn test_json_mutate_get_str_for_key() {
        let value: Value = from_str(
            "{\
            \"str_key\": \"str_value\"
        }",
        )
        .unwrap();

        if let Some(s) = value.get_str_for_key("str_key") {
            assert_eq!(s, "str_value");
        } else {
            assert!(false, "Unable to retrieve string for key: 'str_key'");
        }
    }

    #[test]
    fn test_json_mutate_set_str_for_key() {
        let mut value: Value = from_str(
            "{\
            \"first\": 1
        }",
        )
        .unwrap();

        value.set_str_for_key("first", "one");

        if let Some(s) = value.get_str_for_key("first") {
            assert_eq!(s, "one");
        } else {
            assert!(false, "Unable to get string for key 'first'");
        }

        value = from_str("{}").unwrap();

        value.set_str_for_key("first", "two");

        if let Some(s) = value.get_str_for_key("first") {
            assert_eq!(s, "two");
        } else {
            assert!(false, "Unable to get string for key 'first'");
        }
    }

    #[test]
    fn test_json_mutate_get_book_for_key() {
        let value: Value = from_str(
            "{\
            \"first\": false
        }",
        )
        .unwrap();

        if let Some(b) = value.get_bool_for_key("first") {
            assert!(!b);
        } else {
            assert!(false, "Unable to get bool for key 'first'");
        }
    }

    #[test]
    fn test_json_mutate_get_int_for_key() {
        let value: Value = from_str(
            "{\
            \"one\": 100
        }",
        )
        .unwrap();

        if let Some(i) = value.get_int_for_key("one") {
            assert_eq!(i, 100);
        } else {
            assert!(false, "Unable to get int for key 'one'");
        }
    }

    #[test]
    fn test_json_mutate_get_float_for_key() {
        let value: Value = from_str(
            "{\
            \"f\": 1.23
        }",
        )
        .unwrap();

        if let Some(f) = value.get_float_for_key("f") {
            assert_eq!(f, 1.23);
        } else {
            assert!(false, "Unable to get float for key `f`");
        }
    }

    #[test]
    fn test_json_mutate_contains_key() {
        let value: Value = from_str(
            "{\
            \"foo\": 1
        }",
        )
        .unwrap();

        assert!(value.contains_key("foo"));

        let value2 = json![[]];

        assert!(!value2.contains_key("bar"));
    }

    #[test]
    fn test_json_mutate_get_node_with_id() {
        let node: Value = from_str(
            "{\
                \"id\": 10,
                \"thing\": [{
                    \"id\":11
                }]
            }",
        )
        .unwrap();

        let id_maker = JSONIDMaker::new(|n| {
            if let Some(id) = n.get_int_for_key("id") {
                Some(id as u64)
            } else {
                None
            }
        });

        if let Some(found_node) = node.get_node_with_id(11, &id_maker) {
            assert_eq!(id_maker.get_id(found_node), Some(11));
        } else {
            assert!(false, "Get_node_with_id failed to locate node with id=11.");
        }
    }

    #[test]
    fn test_json_mutate_get_node_with_id_mut() {
        let mut node: Value = from_str(
            "{\
                \"id\": 10,
                \"thing\": [{
                    \"id\":11
                }]
            }",
        )
        .unwrap();

        let id_maker = JSONIDMaker::new(|n| {
            if let Some(id) = n.get_int_for_key("id") {
                Some(id as u64)
            } else {
                None
            }
        });

        if let Some(found_node) = node.get_node_with_id_mut(11, &id_maker) {
            assert_eq!(id_maker.get_id(found_node), Some(11));
        } else {
            assert!(false, "Get_node_with_id failed to locate node with id=11.");
        }
    }
}
