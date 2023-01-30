use crate::json::*;
use serde_json::{from_str, json, Value};
use std::collections::HashMap;

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

#[derive(Debug)]
pub struct Preferences {
    data: Value,
}

impl Preferences {
    pub fn new() -> Preferences {
        Preferences {
            data: from_str("{}").unwrap(),
        }
    }

    pub fn set_value_for_key(&mut self, key: &str, value: PreferenceValue) {
        let json_value = Preferences::convert_preference_value_to_json(&value);
        self.data.set_node_for_key(key, json_value);
    }

    pub fn get_value_for_key(&self, key: &str) -> Option<PreferenceValue> {
        if let Some(node) = self.data.borrow_value_for_key(key) {
            return Some(Preferences::convert_json_to_preference_value(node));
        }
        None
    }

    pub fn contains_key(&self, key: &str) -> bool {
        if let Some(_node) = self.data.borrow_value_for_key(key) {
            return true;
        }
        false
    }

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

    fn convert_preference_value_to_json(value: &PreferenceValue) -> Value {
        match value {
            PreferenceValue::Int(i) => json![i],
            PreferenceValue::Float(f) => json![f],
            PreferenceValue::String(s) => json![s],
            PreferenceValue::Boolean(b) => json![b],
            PreferenceValue::Null => from_str("").unwrap(),
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
