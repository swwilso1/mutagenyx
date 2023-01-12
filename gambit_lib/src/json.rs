use crate::error::GambitError;
use serde_json::{json, Value};
use std::fs::File;
use std::io::BufReader;

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

pub fn json_path(path: &str) -> String {
    let mut jp = String::from("/");
    jp += path;
    jp
}

pub trait JSONAPI {
    fn as_str(&self) -> Option<&str>;
    fn set_string(&mut self, value: &str);
    fn take_value_for_key(&mut self, key: &str) -> Option<Value>;
    fn borrow_value_for_key(&self, key: &str) -> Option<&Value>;
    fn set_node_for_key(&mut self, key: &str, node: Value);
    fn set_node_for_key_at_index(&mut self, key: &str, index: usize, node: Value);
    fn get_array_for_key(&self, key: &str) -> Option<&Vec<Value>>;
    fn get_str_for_key(&self, path: &str) -> Option<&str>;
    fn set_str_for_key(&mut self, path: &str, value: &str);
    fn get_bool_for_key(&self, key: &str) -> Option<bool>;
}

impl JSONAPI for Value {
    fn as_str(&self) -> Option<&str> {
        self.as_str()
    }

    fn set_string(&mut self, value: &str) {
        *self = json![value];
    }

    fn take_value_for_key(&mut self, key: &str) -> Option<Value> {
        let json_key = json_path(key);
        match self.pointer_mut(&json_key) {
            Some(v) => Some(v.take()),
            _ => None,
        }
    }

    fn borrow_value_for_key(&self, key: &str) -> Option<&Value> {
        let json_key = json_path(key);
        match self.pointer(&json_key) {
            Some(v) => Some(v),
            _ => None,
        }
    }

    fn set_node_for_key(&mut self, key: &str, node: Value) {
        let json_path = json_path(key);
        if let Some(v) = self.pointer_mut(&json_path) {
            *v = node;
        }
    }

    fn set_node_for_key_at_index(&mut self, key: &str, index: usize, node: Value) {
        let json_path = json_path(key);
        if let Some(v) = self.pointer_mut(&json_path) {
            if let Some(a) = v.get_mut(index) {
                *a = node;
            }
        }
    }

    fn get_array_for_key(&self, key: &str) -> Option<&Vec<Value>> {
        let json_path = json_path(key);
        match self.pointer(&json_path) {
            Some(v) => v.as_array(),
            _ => None,
        }
    }

    fn get_str_for_key(&self, path: &str) -> Option<&str> {
        let json_path = json_path(path);
        match self.pointer(&json_path) {
            Some(v) => v.as_str(),
            _ => None,
        }
    }

    fn set_str_for_key(&mut self, path: &str, value: &str) {
        let json_path = json_path(path);
        match self.pointer_mut(&json_path) {
            Some(v) => v.set_string(value),
            _ => return,
        }
    }

    fn get_bool_for_key(&self, key: &str) -> Option<bool> {
        let json_path = json_path(key);
        match self.pointer(&json_path) {
            Some(v) => v.as_bool(),
            _ => None,
        }
    }
}
