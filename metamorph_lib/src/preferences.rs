/// The `preferences` module provides a simple preferences object that supports key/value pairs
/// where the value can vary between numbers, strings, booleans, etc...
use crate::error::MetamorphError;
use crate::json::*;
#[cfg(test)]
use serde_json::json;
use serde_json::{from_str, Map, Value};
use std::collections::HashMap;
use std::convert::TryFrom;

/// Value type stored in the preferences object.
#[derive(Debug, Clone, PartialEq)]
pub enum PreferenceValue {
    Int(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Map(HashMap<String, PreferenceValue>),
    Array(Vec<PreferenceValue>),
    Null,
}

impl TryFrom<Value> for PreferenceValue {
    type Error = MetamorphError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Some(i) = value.as_i64() {
            return Ok(PreferenceValue::Int(i));
        } else if let Some(f) = value.as_f64() {
            return Ok(PreferenceValue::Float(f));
        } else if let Some(s) = value.as_str() {
            return Ok(PreferenceValue::String(String::from(s)));
        } else if let Some(b) = value.as_bool() {
            return Ok(PreferenceValue::Boolean(b));
        } else if let Some(m) = value.as_object() {
            let mut new_map: HashMap<String, PreferenceValue> = HashMap::new();
            for (key, mvalue) in m {
                let new_pvalue = PreferenceValue::try_from(mvalue.clone())?;
                new_map.insert(key.clone(), new_pvalue);
            }
            return Ok(PreferenceValue::Map(new_map));
        } else if let Some(a) = value.as_array() {
            let mut new_array: Vec<PreferenceValue> = Vec::new();
            for avalue in a {
                let new_pvalue = PreferenceValue::try_from(avalue.clone())?;
                new_array.push(new_pvalue);
            }
            return Ok(PreferenceValue::Array(new_array));
        } else if value.is_null() {
            return Ok(PreferenceValue::Null);
        }
        let json_text = format!("{value}");
        Err(MetamorphError::UnrecognizedJSON(json_text))
    }
}

impl TryFrom<PreferenceValue> for Value {
    type Error = MetamorphError;

    fn try_from(value: PreferenceValue) -> Result<Self, Self::Error> {
        match value {
            PreferenceValue::Int(i) => Ok(Value::from(i)),
            PreferenceValue::Float(f) => Ok(Value::from(f)),
            PreferenceValue::String(s) => Ok(Value::String(s)),
            PreferenceValue::Boolean(b) => Ok(Value::from(b)),
            PreferenceValue::Map(o) => {
                let mut jobject: Value = from_str("{}").unwrap();
                for (k, v) in o {
                    let jvalue = Value::try_from(v)?;
                    jobject.set_node_for_key(k.as_str(), jvalue);
                }
                Ok(jobject)
            }
            PreferenceValue::Array(a) => {
                let mut jarray: Value = Value::Array(vec![]);
                for v in a {
                    let jvalue = Value::try_from(v)?;
                    jarray.push_node(jvalue);
                }
                Ok(jarray)
            }
            PreferenceValue::Null => Ok(Value::Null),
        }
    }
}

/// Preferences object that provides a key/value store where the value can have multiple sub-types.
#[derive(Debug, Clone)]
pub struct Preferences {
    map: HashMap<String, PreferenceValue>,
}

impl Preferences {
    /// Create a new preferences object.
    pub fn new() -> Preferences {
        Preferences {
            map: HashMap::new(),
        }
    }

    /// Create a new preferences object from a map.
    ///
    /// # Argument
    ///
    /// * `map` - The [`HashMap<String, PreferenceValue>`] map.
    pub fn new_from_map(map: HashMap<String, PreferenceValue>) -> Preferences {
        Preferences { map }
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
        self.map.contains_key(key)
    }

    /// Return an integer for the value stored for `key` if the value is an integer.
    ///
    /// # Arguments
    ///
    /// * `key` - The string value containing the key.
    pub fn get_int_for_key(&self, key: &str) -> Option<i64> {
        if let Some(value) = self.map.get(key) {
            return match value {
                PreferenceValue::Int(i) => Some(*i),
                _ => None,
            };
        }
        None
    }

    /// Return a float for the value stored for `key` if the value is a float.
    ///
    /// # Arguments
    ///
    /// * `key` - The string value containing the key.
    pub fn get_float_for_key(&self, key: &str) -> Option<f64> {
        if let Some(value) = self.map.get(key) {
            return match value {
                PreferenceValue::Float(f) => Some(*f),
                _ => None,
            };
        }
        None
    }

    /// Return a [`String`] for the value stored for `key` if the value is a [`String`].
    ///
    /// # Arguments
    ///
    /// * `key` - The string value containing the key.
    pub fn get_string_for_key(&self, key: &str) -> Option<String> {
        if let Some(value) = self.map.get(key) {
            return match value {
                PreferenceValue::String(s) => Some(s.clone()),
                _ => None,
            };
        }
        None
    }

    /// Return a [`HashMap<String, PreferenceValue>`] for the value stored for `key` if the value
    /// is a map.
    ///
    /// # Arguments
    ///
    /// * `key` - The string value containing the key.
    pub fn get_map_for_key(&self, key: &str) -> Option<HashMap<String, PreferenceValue>> {
        if let Some(value) = self.map.get(key) {
            return match value {
                PreferenceValue::Map(map) => Some(map.clone()),
                _ => None,
            };
        }
        None
    }

    /// Return a [`Vec<PreferenceValue>`] for the value stored for `key` if the value
    /// is an array.
    ///
    /// # Arguments
    ///
    /// * `key` - The string value containing the key.
    pub fn get_array_for_key(&self, key: &str) -> Option<Vec<PreferenceValue>> {
        if let Some(value) = self.map.get(key) {
            return match value {
                PreferenceValue::Array(array) => Some(array.clone()),
                _ => None,
            };
        }
        None
    }

    /// Return a preferences object for the value stored for `key` if the value
    /// is an map.
    ///
    /// # Arguments
    ///
    /// * `key` - The string value containing the key.
    pub fn get_preferences_for_key(&self, key: &str) -> Option<Preferences> {
        if let Some(value) = self.map.get(key) {
            return match value {
                PreferenceValue::Map(map) => Some(Preferences::new_from_map(map.clone())),
                _ => None,
            };
        }
        None
    }

    /// Insert integer `i` for `key`.
    ///
    /// # Arguments
    ///
    /// * `key` - The string value containing the key.
    /// * `i` - The integer value to store for `key`.
    pub fn set_int_for_key(&mut self, key: &str, i: i64) {
        self.map.insert(String::from(key), PreferenceValue::Int(i));
    }

    /// Insert float `f` for `key`.
    ///
    /// # Arguments
    ///
    /// * `key` - The string value containing the key.
    /// * `f` - The float value to store for `key`.
    pub fn set_float_for_key(&mut self, key: &str, f: f64) {
        self.map
            .insert(String::from(key), PreferenceValue::Float(f));
    }

    /// Insert string `s` for `key`.
    ///
    /// # Arguments
    ///
    /// * `key` - The string value containing the key.
    /// * `s` - The string value to store for `key`.
    pub fn set_string_for_key(&mut self, key: &str, s: &str) {
        self.map
            .insert(String::from(key), PreferenceValue::String(String::from(s)));
    }

    /// Insert bool `b` for `key`.
    ///
    /// # Arguments
    ///
    /// * `key` - The string value containing the key.
    /// * `b` - The bool value to store for `key`.
    pub fn set_bool_for_key(&mut self, key: &str, b: bool) {
        self.map
            .insert(String::from(key), PreferenceValue::Boolean(b));
    }

    /// Insert [`HashMap<String, PreferenceValue>`] `map` for `key`.
    ///
    /// # Arguments
    ///
    /// * `key` - The string value containing the key.
    /// * `map` - The [`HashMap<String, PreferenceValue>`] value to store for `key`.
    pub fn set_map_for_key(&mut self, key: &str, map: HashMap<String, PreferenceValue>) {
        self.map
            .insert(String::from(key), PreferenceValue::Map(map));
    }

    /// Insert [`Vec<PreferenceValue>`] `array` for `key`.
    ///
    /// # Arguments
    ///
    /// * `key` - The string value containing the key.
    /// * `array` - The [`Vec<PreferenceValue>`] value to store for `key`.
    pub fn set_array_for_key(&mut self, key: &str, array: Vec<PreferenceValue>) {
        self.map
            .insert(String::from(key), PreferenceValue::Array(array));
    }

    /// Insert preferences `prefs` for `key`.
    ///
    /// # Arguments
    ///
    /// * `key` - The string value containing the key.
    /// * `prefs` - The preferences value to store for `key`.
    pub fn set_preferences_for_key(&mut self, key: &str, prefs: Preferences) {
        self.map
            .insert(String::from(key), PreferenceValue::Map(prefs.map));
    }
}

impl TryFrom<Value> for Preferences {
    type Error = MetamorphError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if !value.is_object() {
            let json_text = format!("{value}");
            return Err(MetamorphError::UnrecognizedJSON(json_text));
        }

        let mut new_map: HashMap<String, PreferenceValue> = HashMap::new();
        let value_map = value.as_object().unwrap();

        for (k, mvalue) in value_map {
            let new_pvalue = PreferenceValue::try_from(mvalue.clone())?;
            new_map.insert(k.clone(), new_pvalue);
        }

        Ok(Preferences { map: new_map })
    }
}

impl TryFrom<Preferences> for Value {
    type Error = MetamorphError;

    fn try_from(value: Preferences) -> Result<Self, Self::Error> {
        let map: Map<String, Value> = Map::new();
        let mut jvalue = Value::Object(map);
        for (key, value) in &value.map {
            let jsub_value = Value::try_from(value.clone())?;
            jvalue.set_node_for_key(key.as_str(), jsub_value);
        }
        Ok(jvalue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_to_preference_value() {
        let ivalue = json![10];
        let fvalue = json![3.4];
        let svalue = json!["Hello World"];
        let bvalue = json![true];
        let ovalue: Value = from_str(
            "\
        {\
            \"foo\": \"bar\"
        }",
        )
        .unwrap();
        let avalue: Value = from_str("[\"foo\", 3]").unwrap();
        let nvalue = Value::Null;

        assert_eq!(
            PreferenceValue::try_from(ivalue).unwrap(),
            PreferenceValue::Int(10)
        );
        assert_eq!(
            PreferenceValue::try_from(fvalue).unwrap(),
            PreferenceValue::Float(3.4)
        );
        assert_eq!(
            PreferenceValue::try_from(svalue).unwrap(),
            PreferenceValue::String(String::from("Hello World"))
        );
        assert_eq!(
            PreferenceValue::try_from(bvalue).unwrap(),
            PreferenceValue::Boolean(true)
        );
        assert_eq!(
            PreferenceValue::try_from(nvalue).unwrap(),
            PreferenceValue::Null
        );

        let mut map: HashMap<String, PreferenceValue> = HashMap::new();
        map.insert(
            String::from("foo"),
            PreferenceValue::String(String::from("bar")),
        );
        assert_eq!(
            PreferenceValue::try_from(ovalue).unwrap(),
            PreferenceValue::Map(map)
        );

        let mut array: Vec<PreferenceValue> = Vec::new();
        array.push(PreferenceValue::String(String::from("foo")));
        array.push(PreferenceValue::Int(3));
        assert_eq!(
            PreferenceValue::try_from(avalue).unwrap(),
            PreferenceValue::Array(array)
        );
    }

    #[test]
    fn test_json_into_preference_value() {
        let ipvalue: Result<PreferenceValue, MetamorphError> = Value::from(10).try_into();
        assert_eq!(ipvalue.unwrap(), PreferenceValue::Int(10));

        let fpvalue: Result<PreferenceValue, MetamorphError> = Value::from(3.4).try_into();
        assert_eq!(fpvalue.unwrap(), PreferenceValue::Float(3.4));

        let spvalue: Result<PreferenceValue, MetamorphError> = Value::from("foo").try_into();
        assert_eq!(
            spvalue.unwrap(),
            PreferenceValue::String(String::from("foo"))
        );

        let bpvalue: Result<PreferenceValue, MetamorphError> = Value::from(true).try_into();
        assert_eq!(bpvalue.unwrap(), PreferenceValue::Boolean(true));

        let mut actual_map: HashMap<String, PreferenceValue> = HashMap::new();
        actual_map.insert(String::from("foo"), PreferenceValue::Int(1));
        let actual_object = PreferenceValue::Map(actual_map);

        let jobject_value: Value = from_str("{\"foo\":1}").unwrap();
        let opvalue: Result<PreferenceValue, MetamorphError> = jobject_value.try_into();
        assert_eq!(opvalue.unwrap(), actual_object);

        let mut actual_array: Vec<PreferenceValue> = Vec::new();
        actual_array.push(PreferenceValue::Null);
        actual_array.push(PreferenceValue::Float(6.123));
        let actual_array = PreferenceValue::Array(actual_array);

        let jarray_value: Value = from_str("[null, 6.123]").unwrap();
        let apvalue: Result<PreferenceValue, MetamorphError> = jarray_value.try_into();
        assert_eq!(apvalue.unwrap(), actual_array);

        let npvalue: Result<PreferenceValue, MetamorphError> = Value::Null.try_into();
        assert_eq!(npvalue.unwrap(), PreferenceValue::Null);
    }
}
