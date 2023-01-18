use crate::ast::SimpleAST;
use crate::json::JSONMutate;
use serde_json::Value;

pub type SolidityAST = Value;

pub trait SolidityASTApi {
    fn take_expression(&mut self) -> Option<SolidityAST>;
    fn take_argument(&mut self, index: usize) -> Option<SolidityAST>;
}

impl SolidityASTApi for SolidityAST {
    fn take_expression(&mut self) -> Option<SolidityAST> {
        self.take_value_for_key("expression")
    }

    fn take_argument(&mut self, index: usize) -> Option<SolidityAST> {
        if let Some(v) = self.pointer_mut("/arguments") {
            if let Some(n) = v.get_mut(index) {
                return Some(n.take());
            }
        }
        None
    }
}

impl SimpleAST<SolidityAST> for SolidityAST {
    fn get_node(&self) -> &SolidityAST {
        self
    }

    fn get_node_mut(&mut self) -> &mut SolidityAST {
        self
    }

    fn get_children(&self) -> Vec<Box<&dyn SimpleAST<SolidityAST>>> {
        return if self.is_object() {
            let mut list: Vec<Box<&dyn SimpleAST<SolidityAST>>> = vec![];
            let object = self.as_object().unwrap();
            for value in object.values() {
                list.push(Box::new(value));
            }
            list
        } else if self.is_array() {
            let mut node_list: Vec<Box<&dyn SimpleAST<SolidityAST>>> = vec![];
            let array = self.as_array().unwrap();
            for value in array {
                node_list.push(Box::new(value));
            }
            node_list
        } else {
            vec![]
        };
    }

    fn get_children_mut(&mut self) -> Vec<Box<&mut dyn SimpleAST<SolidityAST>>> {
        return if self.is_object() {
            let mut list: Vec<Box<&mut dyn SimpleAST<SolidityAST>>> = vec![];
            let object = self.as_object_mut().unwrap();
            for value in object.values_mut() {
                list.push(Box::new(value));
            }
            list
        } else if self.is_array() {
            let mut list: Vec<Box<&mut dyn SimpleAST<SolidityAST>>> = vec![];
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
