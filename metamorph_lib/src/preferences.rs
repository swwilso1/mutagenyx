/// The `preferences` module provides a simple preferences object that supports key/value pairs
/// where the value can vary between numbers, strings, booleans, etc...
use crate::config_file::CompilerDetails;
use std::collections::HashMap;

/// Value type stored in the preferences object.
#[derive(Debug, Clone)]
pub enum PreferenceValue {
    Int(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Object(HashMap<String, PreferenceValue>),
    Array(Vec<PreferenceValue>),
    CompilerDetails(CompilerDetails),
    Null,
}

/// Preferences object that provides a key/value store where the value can have multiple sub-types.
#[derive(Debug, Clone)]
pub struct Preferences {
    map: HashMap<String, PreferenceValue>,
}

impl Preferences {
    /// Create a new [`Preferences`] object.
    pub fn new() -> Preferences {
        Preferences {
            map: HashMap::new(),
        }
    }

    /// Insert a new key/value pair, or update the value for a preexisting key/value pair.
    ///
    /// # Arguments
    ///
    /// * `key` - The string value containing the key.
    /// * `value` - The stored value.
    pub fn set_value_for_key(&mut self, key: &str, value: PreferenceValue) {
        self.map.insert(String::from(key), value);
    }

    /// Get the value for a key.
    ///
    /// # Arguments
    ///
    /// * `key` - The string value containing the key.
    pub fn get_value_for_key(&self, key: &str) -> Option<PreferenceValue> {
        if let Some(value) = self.map.get(key) {
            return Some(value.clone());
        }
        None
    }

    /// Return true if the preferences contains a value for `key`.
    ///
    /// # Arguments
    ///
    /// * `key` - The string value containing the key.
    pub fn contains_key(&self, key: &str) -> bool {
        return self.map.contains_key(key);
    }
}
