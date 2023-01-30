use crate::json::*;
use serde_json::{from_str, json, Value};
use std::collections::HashMap;

/// Value type stored in the preferences object.
#[derive(Debug)]
pub enum PreferenceValue<'a> {
    Int(i64),
    Float(f64),
    String(&'a str),
    Boolean(bool),
    Object(HashMap<String, PreferenceValue<'a>>),
    Array(Vec<PreferenceValue<'a>>),
    Null,
}

/// Preferences object that provides a key/value store where the value can have multiple sub-types.
#[derive(Debug)]
pub struct Preferences {
    /// A JSON object.
    data: Value,
}

impl Preferences {
    /// Create a new [`Preferences`] object.
    pub fn new() -> Preferences {
        Preferences {
            data: from_str("{}").unwrap(),
        }
    }

    /// Insert a new key/value pair, or update the value for a preexisting key/value pair.
    ///
    /// # Arguments
    ///
    /// * `key` - The string value containing the key.
    /// * `value` - The stored value.
    pub fn set_value_for_key(&mut self, key: &str, value: PreferenceValue) {
        let json_value = Preferences::convert_preference_value_to_json(&value);
        self.data.set_node_for_key(key, json_value);
    }

    /// Get the value for a key.
    ///
    /// # Arguments
    ///
    /// * `key` - The string value containing the key.
    pub fn get_value_for_key(&self, key: &str) -> Option<PreferenceValue> {
        if let Some(node) = self.data.borrow_value_for_key(key) {
            return Some(Preferences::convert_json_to_preference_value(node));
        }
        None
    }

    /// Return true if the preferences contains a value for `key`.
    ///
    /// # Arguments
    ///
    /// * `key` - The string value containing the key.
    pub fn contains_key(&self, key: &str) -> bool {
        if let Some(_node) = self.data.borrow_value_for_key(key) {
            return true;
        }
        false
    }

    /// Helper function to convert a JSON [`Value`] object to a [`PreferenceValue`]
    /// analogue.
    fn convert_json_to_preference_value(jvalue: &Value) -> PreferenceValue {
        if jvalue.is_i64() {
            PreferenceValue::Int(jvalue.as_i64().unwrap())
        } else if jvalue.is_f64() {
            PreferenceValue::Float(jvalue.as_f64().unwrap())
        } else if jvalue.is_boolean() {
            PreferenceValue::Boolean(jvalue.as_bool().unwrap())
        } else if jvalue.is_string() {
            PreferenceValue::String(jvalue.as_str().unwrap())
        } else if jvalue.is_object() {
            let mut map: HashMap<String, PreferenceValue> = HashMap::new();
            if let Some(json_map) = jvalue.as_object() {
                for (key, value) in json_map {
                    let pvalue = Preferences::convert_json_to_preference_value(value);
                    map.insert(key.clone(), pvalue);
                }
            }
            PreferenceValue::Object(map)
        } else if jvalue.is_array() {
            let mut array: Vec<PreferenceValue> = Vec::new();
            if let Some(json_array) = jvalue.as_array() {
                for value in json_array {
                    let pvalue = Preferences::convert_json_to_preference_value(value);
                    array.push(pvalue);
                }
            }
            PreferenceValue::Array(array)
        } else if jvalue.is_null() {
            PreferenceValue::Null
        } else {
            // For now return a Null value.
            PreferenceValue::Null
        }
    }

    /// Helper function to convert a [`PreferenceValue`] to a JSON [`Value`] object.
    fn convert_preference_value_to_json(value: &PreferenceValue) -> Value {
        match value {
            PreferenceValue::Int(i) => json![i],
            PreferenceValue::Float(f) => json![f],
            PreferenceValue::String(s) => json![s],
            PreferenceValue::Boolean(b) => json![b],
            PreferenceValue::Null => Value::Null,
            PreferenceValue::Object(map) => {
                let mut json_object: Value = from_str("{}").unwrap();
                for (key, value) in map {
                    let json_value = Preferences::convert_preference_value_to_json(value);
                    json_object.set_node_for_key(key, json_value);
                }
                json_object
            }
            PreferenceValue::Array(a) => {
                let mut json_array: Value = from_str("[]").unwrap();
                if let Some(json_vector) = json_array.as_array_mut() {
                    for value in a {
                        let json_value = Preferences::convert_preference_value_to_json(value);
                        json_vector.push(json_value);
                    }
                }
                json_array
            }
        }
    }
}
