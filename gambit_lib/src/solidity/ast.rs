use crate::ast::AST;
use crate::json::JSONMutate;
use crate::visitor::{Visitor, VisitorMut};
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

impl AST<SolidityAST> for SolidityAST {
    fn traverse<V: Visitor<SolidityAST> + ?Sized>(&self, visitor: &mut V) -> bool {
        visitor.on_enter(self);
        let should_stop = visitor.visit(self);
        if should_stop {
            visitor.on_exit(self);
            return true;
        }
        if visitor.visit_children(self) {
            if self.is_object() {
                let object = self.as_object().unwrap();
                for value in object.values() {
                    let should_stop = value.traverse(visitor);
                    if should_stop {
                        visitor.on_exit(self);
                        return true;
                    }
                }
            } else if self.is_array() {
                let array = self.as_array().unwrap();
                for value in array {
                    let should_stop = value.traverse(visitor);
                    if should_stop {
                        visitor.on_exit(self);
                        return true;
                    }
                }
            }
        }
        visitor.on_exit(self);
        false
    }

    fn traverse_mut<V: VisitorMut<SolidityAST> + ?Sized>(&mut self, visitor: &mut V) -> bool {
        visitor.on_enter(self);
        let should_stop = visitor.visit_mut(self);
        if should_stop {
            visitor.on_exit(self);
            return true;
        }
        if visitor.visit_children(self) {
            if self.is_object() {
                let object = self.as_object_mut().unwrap();
                for value in object.values_mut() {
                    let should_stop = value.traverse_mut(visitor);
                    if should_stop {
                        visitor.on_exit(self);
                        return true;
                    }
                }
            } else if self.is_array() {
                let array = self.as_array_mut().unwrap();
                for value in array {
                    let should_stop = value.traverse_mut(visitor);
                    if should_stop {
                        visitor.on_exit(self);
                        return true;
                    }
                }
            }
        }
        visitor.on_exit(self);
        false
    }
}
