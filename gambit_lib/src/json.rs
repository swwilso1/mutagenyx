//! The module contains various functions and traits that simplify working with
//! [`Value`] objects for the purpose of traversing and mutating abstract syntax trees
//! encoded in JSON.
use crate::error::GambitError;
use serde_json::{from_str, json, Value};
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
/// the function may return [`GambitError::IO`] that contains a [`std::io::Error`].
///
/// Also, the function invokes the `serde_json` JSON parser.  The JSON parser may return a
/// [`serde_json::Error`] which the function will return as a [`GambitError::JSON`] enum that
/// wraps the error from `serde_json`.
pub fn load_json_from_file_with_name(file_name: &str) -> Result<Value, GambitError> {
    let file = match File::open(file_name) {
        Ok(f) => f,
        Err(e) => return Err(GambitError::from(e)),
    };

    let reader = BufReader::new(file);

    let v = match serde_json::from_reader(reader) {
        Ok(val) => val,
        Err(e) => return Err(GambitError::from(e)),
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
pub fn new_json_node(text: &str) -> Result<Value, GambitError> {
    match from_str(text) {
        Ok(n) => Ok(n),
        Err(e) => Err(GambitError::from(e)),
    }
}

/// Trait for use with [`Value`] type that adds functionality for accessing and
/// modifying the contents of the JSON node contained in the objects of the type.
pub trait JSONMutate {
    fn as_str(&self) -> Option<&str>;
    fn set_string(&mut self, value: &str);
    fn has_value_for_key(&self, key: &str) -> bool;
    fn take_value_for_key(&mut self, key: &str) -> Option<Value>;
    fn borrow_value_for_key(&self, key: &str) -> Option<&Value>;
    fn set_node_for_key(&mut self, key: &str, node: Value);
    fn set_node_for_key_at_index(&mut self, key: &str, index: usize, node: Value);
    fn get_array_for_key(&self, key: &str) -> Option<&Vec<Value>>;
    fn get_str_for_key(&self, path: &str) -> Option<&str>;
    fn set_str_for_key(&mut self, path: &str, value: &str);
    fn get_bool_for_key(&self, key: &str) -> Option<bool>;
    fn get_int_for_key(&self, key: &str) -> Option<i64>;
}

impl JSONMutate for Value {
    /// If the [`Value`] object contains a string, return a slice referring to the
    /// string.  If the object does not contain a string, the function returns `None`.
    fn as_str(&self) -> Option<&str> {
        self.as_str()
    }

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
        match self.pointer(&json_key) {
            Some(_) => true,
            _ => false,
        }
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
        match self.pointer_mut(&json_key) {
            Some(v) => Some(v.take()),
            _ => None,
        }
    }

    /// Assuming the [`Value`] object represents a JSON dictionary/map object, then
    /// the function will return a reference to the [`Value`] object stored in the
    /// dictionary for `key`.  If the dictionary does not contain a value for `key`, the function
    /// will return `None`.
    ///
    /// The caller should use [`Value::is_object`] to check for a JSON dictionary/map
    /// prior to calling this function.
    ///
    /// # Arguments
    ///
    /// * `key` - The string slice referencing the text comprising the key.
    fn borrow_value_for_key(&self, key: &str) -> Option<&Value> {
        let json_key = json_path(key);
        match self.pointer(&json_key) {
            Some(v) => Some(v),
            _ => None,
        }
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
        }
    }

    /// Assuming the [`Value`] object represents a JSON dictionary/map object, and that
    /// the dictionary contains a JSON array stored for `key`, then insert `node` into the array
    /// at `index`.
    ///
    /// The caller should use [`Value::is_object`] and [`Value::is_array`]
    /// to check for a JSON dicationary/map that contains a JSON array.
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

    /// Assuming the [`Value`] object represents a JSON dictionary/map object, and that
    /// the dictionary contains an entry for `key` that holds a JSON array, return a reference to
    /// that array.
    ///
    /// The caller should use [`Value::is_object`] and [`Value::is_array`]
    /// to check for a JSON dicationary/map that contains a JSON array.
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
            _ => return,
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
    /// * `key` = The string slice referencing the text comprising the key.
    fn get_int_for_key(&self, key: &str) -> Option<i64> {
        let json_path = json_path(key);
        match self.pointer(&json_path) {
            Some(v) => v.as_i64(),
            _ => None,
        }
    }
}
