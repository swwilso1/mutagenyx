//! The `json_ast` module provides implementation of the SimpleAST trait for [`Value`] objects.

use crate::ast::SimpleAST;
use serde_json::Value;

impl SimpleAST<Value> for Value {
    fn get_node(&self) -> &Value {
        self
    }

    fn get_node_mut(&mut self) -> &mut Value {
        self
    }

    fn get_children(&self) -> Vec<Box<&dyn SimpleAST<Value>>> {
        return if self.is_object() {
            let mut list: Vec<Box<&dyn SimpleAST<Value>>> = vec![];
            let object = self.as_object().unwrap();
            for value in object.values() {
                list.push(Box::new(value));
            }
            list
        } else if self.is_array() {
            let mut node_list: Vec<Box<&dyn SimpleAST<Value>>> = vec![];
            let array = self.as_array().unwrap();
            for value in array {
                node_list.push(Box::new(value));
            }
            node_list
        } else {
            vec![]
        };
    }

    fn get_children_mut(&mut self) -> Vec<Box<&mut dyn SimpleAST<Value>>> {
        return if self.is_object() {
            let mut list: Vec<Box<&mut dyn SimpleAST<Value>>> = vec![];
            let object = self.as_object_mut().unwrap();
            for value in object.values_mut() {
                list.push(Box::new(value));
            }
            list
        } else if self.is_array() {
            let mut list: Vec<Box<&mut dyn SimpleAST<Value>>> = vec![];
            let array = self.as_array_mut().unwrap();
            for value in array {
                list.push(Box::new(value));
            }
            list
        } else {
            vec![]
        };
    }
}
