use crate::error::GambitError;
use crate::json::JSONMutate;
use crate::mutation::{GenericMutation, MutationType, SolidityMutation};
use crate::mutator::{Mutator, MutatorFactory};
use crate::solidity::ast::{SolidityAST, SolidityASTApi};
use crate::solidity::operators::*;
use num::Integer;
use rand::seq::SliceRandom;
use rand::Rng;
use rand_pcg::*;
use rustc_serialize::hex::ToHex;
use serde_json::from_str;
use std::fmt;

fn new_ast_node(text: &str) -> Result<SolidityAST, GambitError> {
    match from_str(text) {
        Ok(n) => Ok(n),
        Err(e) => Err(GambitError::from(e)),
    }
}

fn new_integer_constant_node<I: Integer + fmt::Display>(
    value: I,
) -> Result<SolidityAST, GambitError> {
    let value_str = value.to_string();
    let hex_value_str = value_str.as_bytes().to_hex();
    let node_string = format!(
        "{{
        \"hexValue\": \"{hex_value_str}\",
        \"id\": 999997,
        \"isConstant\": false,
        \"isLValue\": false,
        \"isPure\": true,
        \"kind\": \"number\",
        \"lValueRequested\": false,
        \"nodeType\": \"Literal\",
        \"typeDescriptions\": {{
            \"typeIdentifier\": \"t_rational_{value}_by_1\",
            \"typeString\": \"int_const {value}\"
        }},
        \"value\": \"{value}\"
        }}
    "
    );
    new_ast_node(&node_string)
}

fn new_boolean_literal_node(value: bool) -> Result<SolidityAST, GambitError> {
    let value_str = value.to_string();
    let hex_value_str = value_str.as_bytes().to_hex();
    let node_string = format!(
        "{{\
        \"hexValue\": \"{hex_value_str}\",
        \"id\": 999998,
        \"isConstant\": false,
        \"isLValue\": false,
        \"isPure\": true,
        \"kind\": \"bool\",
        \"lValueRequested\": false,
        \"nodeType\": \"Literal\",
        \"typeDescriptions\": {{
            \"typeIdentifier\": \"t_bool\",
            \"typeString\": \"bool\"
        }},
        \"value\": \"{value}\"
    }}"
    );
    new_ast_node(&node_string)
}

struct BinaryOpMutator {
    operators: Vec<String>,
    mutation_type: MutationType,
}

impl BinaryOpMutator {
    pub fn new(operators: Vec<String>, mutation_type: MutationType) -> BinaryOpMutator {
        BinaryOpMutator {
            operators,
            mutation_type,
        }
    }
}

impl Mutator<SolidityAST> for BinaryOpMutator {
    fn is_mutable_node(&self, node: &SolidityAST) -> bool {
        if let Some(n) = node.get_str_for_key("nodeType") {
            if n == "BinaryOperation" {
                if let Some(op) = node.get_str_for_key("operator") {
                    let op_str = String::from(op);
                    return self.operators.contains(&op_str);
                }
            }
        }
        false
    }

    fn mutate(&self, node: &mut SolidityAST, rand: &mut Pcg64) {
        let chosen_operator = match self.operators.choose(rand) {
            Some(o) => o,
            None => return,
        };
        node.set_str_for_key("operator", &chosen_operator);
    }

    fn implements(&self) -> MutationType {
        self.mutation_type
    }
}

struct UnaryOpMutator {
    prefix_operators: Vec<String>,
    postfix_operators: Vec<String>,
}

impl UnaryOpMutator {
    pub fn new() -> UnaryOpMutator {
        UnaryOpMutator {
            prefix_operators: prefix_operators(),
            postfix_operators: postfix_operators(),
        }
    }
}

impl Mutator<SolidityAST> for UnaryOpMutator {
    fn is_mutable_node(&self, node: &SolidityAST) -> bool {
        if let Some(n) = node.get_str_for_key("nodeType") {
            if n == "UnaryOperation" {
                let operator_list = match node.get_bool_for_key("prefix") {
                    Some(p) => {
                        if p {
                            &self.prefix_operators
                        } else {
                            &self.postfix_operators
                        }
                    }
                    _ => {
                        log::info!(
                            "Found a UnaryOperator node with no prefix information: {:?}",
                            node
                        );
                        return false;
                    }
                };
                if let Some(op) = node.get_str_for_key("operator") {
                    let op_str = String::from(op);
                    return operator_list.contains(&op_str);
                }
            }
        }
        false
    }

    fn mutate(&self, node: &mut SolidityAST, rand: &mut Pcg64) {
        let operator_list = match node.get_bool_for_key("prefix") {
            Some(p) => {
                if p {
                    &self.prefix_operators
                } else {
                    &self.postfix_operators
                }
            }
            _ => {
                log::info!(
                    "Mutate found a UnaryOperator node with no prefix information: {:?}",
                    node
                );
                return;
            }
        };
        let chosen_operator = match operator_list.choose(rand) {
            Some(o) => o,
            None => return,
        };
        node.set_str_for_key("operator", &chosen_operator);
    }

    fn implements(&self) -> MutationType {
        MutationType::Generic(GenericMutation::UnaryOp)
    }
}

struct AssignmentMutator {}

impl AssignmentMutator {
    pub fn new() -> AssignmentMutator {
        AssignmentMutator {}
    }
}

impl Mutator<SolidityAST> for AssignmentMutator {
    fn is_mutable_node(&self, node: &SolidityAST) -> bool {
        if let Some(n) = node.get_str_for_key("nodeType") {
            if n == "Assignment" {
                let type_description_node = match node.borrow_value_for_key("typeDescriptions") {
                    Some(n) => n,
                    _ => {
                        log::info!("Assignment node has no type description object");
                        return false;
                    }
                };

                let type_string = match type_description_node.get_str_for_key("typeString") {
                    Some(s) => s,
                    _ => {
                        log::info!("Assignment node's type description has no type string");
                        return false;
                    }
                };

                if type_string.len() < 3 {
                    return false;
                }

                let first_three_chars = &type_string[..3];

                return match first_three_chars {
                    "uin" => true,
                    "int" => true,
                    "boo" => true,
                    _ => false,
                };
            }
        }
        false
    }

    fn mutate(&self, node: &mut SolidityAST, rand: &mut Pcg64) {
        let type_description_node = match node.borrow_value_for_key("typeDescriptions") {
            Some(n) => n,
            _ => {
                log::info!("Assignment node has no type description object");
                return;
            }
        };

        let type_string = match type_description_node.get_str_for_key("typeString") {
            Some(s) => s,
            _ => {
                log::info!("Assignment node's type description has no type string");
                return;
            }
        };

        let first_three_chars = &type_string[..3];

        match first_three_chars {
            "int" => {
                let type_size_str = &type_string[3..];
                let mut type_size = type_size_str.parse::<u32>().unwrap();

                if type_size > 128 {
                    type_size = 128;
                }

                let lower_bound = -(2_i128.pow(type_size - 1)) as i128;
                let upper_bound = (2_i128.pow(type_size - 1)) as i128;
                let replacement_value = rand.gen_range(lower_bound, upper_bound);

                let new_node = match new_integer_constant_node(replacement_value) {
                    Ok(n) => n,
                    _ => return,
                };

                node.set_node_for_key("rightHandSide", new_node);
            }
            "uin" => {
                let type_size_str = &type_string[4..];
                let mut type_size = type_size_str.parse::<u32>().unwrap();

                if type_size > 128 {
                    type_size = 128;
                }

                let lower_bound = 0 as u128;
                let upper_bound: u128;
                if type_size < 128 {
                    upper_bound = 2_u128.pow(type_size) - 1;
                } else {
                    // 2.pow(128) - 1
                    upper_bound = 340282366920938463463374607431768211455u128;
                }
                let replacement_value = rand.gen_range(lower_bound, upper_bound);

                let new_node = match new_integer_constant_node(replacement_value) {
                    Ok(n) => n,
                    _ => return,
                };

                node.set_node_for_key("rightHandSide", new_node);
            }
            "boo" => {
                assert_eq!(type_string, "bool");
                let replacement_value = rand.gen_range(0, 1);
                let bool_literal: bool;
                if replacement_value == 0 {
                    bool_literal = false;
                } else {
                    bool_literal = true;
                }

                let new_node = match new_boolean_literal_node(bool_literal) {
                    Ok(n) => n,
                    _ => return,
                };

                node.set_node_for_key("rightHandSide", new_node);
            }
            _ => return,
        }
    }

    fn implements(&self) -> MutationType {
        MutationType::Generic(GenericMutation::Assignment)
    }
}

struct FunctionCallMutator {}

impl FunctionCallMutator {
    pub fn new() -> FunctionCallMutator {
        FunctionCallMutator {}
    }
}

impl Mutator<SolidityAST> for FunctionCallMutator {
    fn is_mutable_node(&self, _node: &SolidityAST) -> bool {
        false
    }

    fn mutate(&self, _node: &mut SolidityAST, _rand: &mut Pcg64) {
        return;
    }

    fn implements(&self) -> MutationType {
        MutationType::Generic(GenericMutation::FunctionCall)
    }
}

struct SolidityRequireMutator {}

impl SolidityRequireMutator {
    pub fn new() -> SolidityRequireMutator {
        SolidityRequireMutator {}
    }
}

impl Mutator<SolidityAST> for SolidityRequireMutator {
    fn is_mutable_node(&self, node: &SolidityAST) -> bool {
        return node.get_str_for_key("nodeType").map_or_else(
            || false,
            |n| {
                n == "FunctionCall"
                    && (node.borrow_value_for_key("expression").map_or_else(
                        || false,
                        |n| {
                            n.get_str_for_key("name")
                                .map_or_else(|| false, |name| name == "require")
                        },
                    ))
                    && !node
                        .get_array_for_key("arguments")
                        .map_or_else(|| false, |v| v.is_empty())
            },
        );
    }

    fn mutate(&self, node: &mut SolidityAST, _: &mut Pcg64) {
        let new_node_str = "{\
            \"id\": 99999,
            \"isConstant\": false,
            \"isLValue\": false,
            \"isPure\": true,
            \"lValueRequested\": false,
            \"nodeType\": \"UnaryOperation\",
            \"operator\": \"!\",
            \"prefix\": true,
            \"typeDescriptions\": {
                \"typeIdentifier\": \"t_bool\",
                \"typeString\": \"bool\"
            }
        }";
        let mut new_node = match new_ast_node(new_node_str) {
            Ok(v) => v,
            Err(_) => return,
        };

        let tuple_expression_str = "{\
             \"id\": 99996,
             \"isConstant\": false,
             \"isInlineArray\": false,
             \"isLValue\": false,
             \"isPure\": false,
             \"lValueRequested\": false,
             \"nodeType\": \"TupleExpression\",
             \"typeDescriptions\": {
                 \"typeIdentifier\": \"t_bool\",
                 \"typeString\": \"bool\"
             }
        }";

        let mut tuple_expression_node = match new_ast_node(tuple_expression_str) {
            Ok(v) => v,
            Err(_) => return,
        };

        let components_str = "[]";
        let mut components_node = match new_ast_node(components_str) {
            Ok(v) => v,
            Err(_) => return,
        };

        // Get the node from the arguments list.
        let arg = match node.take_argument(0) {
            Some(n) => n,
            _ => {
                log::info!("Arguments list does not contain valid node");
                return;
            }
        };

        let components_array = match components_node.as_array_mut() {
            Some(v) => v,
            _ => return,
        };

        components_array.push(arg);
        tuple_expression_node["components"] = components_node;

        // Put the node into the generated UnaryOp node.
        new_node["subExpression"] = tuple_expression_node;
        node.set_node_for_key_at_index("arguments", 0, new_node);
    }

    fn implements(&self) -> MutationType {
        MutationType::Solidity(SolidityMutation::Require)
    }
}

pub struct SolidityMutatorFactory {}

impl SolidityMutatorFactory {
    pub fn new() -> SolidityMutatorFactory {
        SolidityMutatorFactory {}
    }
}

impl MutatorFactory<SolidityAST> for SolidityMutatorFactory {
    fn mutator_for(&self, mutation_type: &MutationType) -> Option<Box<dyn Mutator<SolidityAST>>> {
        match mutation_type {
            MutationType::Generic(t) => match t {
                GenericMutation::ArithmeticBinaryOp => Some(Box::new(BinaryOpMutator::new(
                    arithmetic_operators(),
                    MutationType::Generic(GenericMutation::ArithmeticBinaryOp),
                ))),
                GenericMutation::LogicalBinaryOp => Some(Box::new(BinaryOpMutator::new(
                    logical_operators(),
                    MutationType::Generic(GenericMutation::LogicalBinaryOp),
                ))),
                GenericMutation::PredicateBinaryOp => Some(Box::new(BinaryOpMutator::new(
                    predicate_operators(),
                    MutationType::Generic(GenericMutation::PredicateBinaryOp),
                ))),
                GenericMutation::Assignment => Some(Box::new(AssignmentMutator::new())),
                GenericMutation::FunctionCall => Some(Box::new(FunctionCallMutator::new())),
                GenericMutation::UnaryOp => Some(Box::new(UnaryOpMutator::new())),
                _ => None,
            },
            MutationType::Solidity(t) => match t {
                SolidityMutation::Require => Some(Box::new(SolidityRequireMutator::new())),
                _ => None,
            },
        }
    }
}
