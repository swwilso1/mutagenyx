//! The `solidity::mutators` module provides the objects that implement the mutation algorithms
//! for the Solidity ASTs.  The module also provides the factory that implements [`MutatorFactory<t>`].

use crate::error::GambitError;
use crate::json::JSONMutate;
use crate::mutation::{GenericMutation, MutationType, SolidityMutation};
use crate::mutator::{Mutator, MutatorFactory};
use crate::operators::*;
use crate::solidity::ast::{SolidityAST, SolidityASTApi};
use num::Integer;
use rand::seq::SliceRandom;
use rand::Rng;
use rand_pcg::*;
use rustc_serialize::hex::ToHex;
use serde_json::from_str;
use std::fmt;

/// Return a SolidityAST node by creating the node from `text`.
///
/// # Arguments
///
/// * `text` - The string slice referring to the text that contains JSON.
fn new_ast_node(text: &str) -> Result<SolidityAST, GambitError> {
    match from_str(text) {
        Ok(n) => Ok(n),
        Err(e) => Err(GambitError::from(e)),
    }
}

/// Return a new integer literal node represengint an integer literal number.
///
/// # Argument
/// * `value` - The number that the node should contain.
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

/// Return a new boolean literal node.
///
/// # Argument
///
/// * `value` - The boolean value to store in the node.
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

/// The object that implements mutations for binary expressions.
///
/// Multiple mutation algorithms operate on binary expressions.  Each of those
/// mutation algorithms uses a subset of the available binary operators.  This
/// one structure/class provides the functionality to implement these different
/// binary expression mutations.
struct BinaryOpMutator {
    /// A list of operators valid for the binary expression
    operators: Vec<String>,

    /// The mutation algorithm implemented by the mutator.
    mutation_type: MutationType,
}

impl BinaryOpMutator {
    /// Return a new instance of the mutator.
    ///
    /// # Arguments
    ///
    /// * `operators` - the list of operators for the mutator
    /// * `mutation_type` - the mutation algorithm implemented by the mutator
    pub fn new(operators: Vec<String>, mutation_type: MutationType) -> BinaryOpMutator {
        BinaryOpMutator {
            operators,
            mutation_type,
        }
    }
}

impl Mutator<SolidityAST> for BinaryOpMutator {
    fn is_mutable_node(&self, node: &SolidityAST) -> bool {
        // First check to see if the node in the AST is a "BinaryOperation" node.
        if let Some(n) = node.get_str_for_key("nodeType") {
            if n == "BinaryOperation" {
                // Get the operator from the node and see if the operator is in the mutator's
                // list of supported operators. The mutator can mutate the node if it supports
                // the node's operator.
                if let Some(op) = node.get_str_for_key("operator") {
                    let op_str = String::from(op);
                    return self.operators.contains(&op_str);
                }
            }
        }
        false
    }

    fn mutate(&self, node: &mut SolidityAST, rand: &mut Pcg64) {
        if let Some(original_operator) = node.get_str_for_key("operator") {
            // Get the original operator so that we can use it to compare for the
            // randomly chosen new operator. We do not want to replace the original operator
            // with itself, just by randomly selecting the same operator from the operator list.
            let mut chosen_operator = match self.operators.choose(rand) {
                Some(o) => o,
                None => return,
            };

            // If we chose the original operator, keep choosing until we get a different operator.
            while original_operator == chosen_operator {
                chosen_operator = match self.operators.choose(rand) {
                    Some(o) => o,
                    None => return,
                };
            }

            // Insert the new operator into the node.
            node.set_str_for_key("operator", &chosen_operator);
        }
    }

    fn implements(&self) -> MutationType {
        self.mutation_type
    }
}

/// The structure/class that implements mutations for unary expressions.
struct UnaryOpMutator {
    /// A list of operators usable as prefix operators.
    prefix_operators: Vec<String>,

    /// A list of operators usable as postfix operators.
    postfix_operators: Vec<String>,
}

impl UnaryOpMutator {
    /// Create a new unary expression mutator.
    pub fn new() -> UnaryOpMutator {
        UnaryOpMutator {
            prefix_operators: prefix_operators(),
            postfix_operators: postfix_operators(),
        }
    }
}

impl Mutator<SolidityAST> for UnaryOpMutator {
    fn is_mutable_node(&self, node: &SolidityAST) -> bool {
        // First check to see if the node is a 'UnaryOperation' node.
        if let Some(n) = node.get_str_for_key("nodeType") {
            if n == "UnaryOperation" {
                // Now check the 'prefix' member of the node and if the node is a prefix operation
                // use the prefix operator list and otherwise use the postifix operator list.
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

                // Check to see if the operator in the node is in the operator list for the
                // mutator
                if let Some(op) = node.get_str_for_key("operator") {
                    let op_str = String::from(op);
                    return operator_list.contains(&op_str);
                }
            }
        }
        false
    }

    fn mutate(&self, node: &mut SolidityAST, rand: &mut Pcg64) {
        // Determine if the node is a prefix/postfix operation and then use the prefix or
        // postfix operator list.
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

        // Get the original operator that we can use to compare to the new operator that we select.
        // If the operator choice selects the same operator as the original, then we want to keep
        // selecting operators until we have a different operator.
        let original_operator = match node.get_str_for_key("operator") {
            Some(o) => o,
            None => "",
        };

        // Choose a new operator.
        let mut chosen_operator = match operator_list.choose(rand) {
            Some(o) => o,
            None => return,
        };

        // If the operators match, choose another operator until they no longer match.
        while original_operator == chosen_operator {
            chosen_operator = match operator_list.choose(rand) {
                Some(o) => o,
                None => return,
            };
        }

        node.set_str_for_key("operator", &chosen_operator);
    }

    fn implements(&self) -> MutationType {
        MutationType::Generic(GenericMutation::UnaryOp)
    }
}

/// Implements the assignment mutation algorithm.
///
/// For a given Assigment expression, the algorithm replaces the right-hand side of the expression
/// with a type correct random value.  Specifically the algorithm operates on assignments to
/// integer, unsigned integer, and boolean variables.  For signed and unsigned integers, the
/// algorithm will recognize the range of the type and generate random numbers that lie in the
/// range.  Since rust only supports a maximum of 128-bit signed and unsigned integers, Solidity
/// types larger than that will only receive a random number in the 128-bit range.
struct AssignmentMutator {}

impl AssignmentMutator {
    /// Create the new mutator.
    pub fn new() -> AssignmentMutator {
        AssignmentMutator {}
    }
}

impl Mutator<SolidityAST> for AssignmentMutator {
    fn is_mutable_node(&self, node: &SolidityAST) -> bool {
        // First check to see if the node is an `Assignment` node.
        if let Some(n) = node.get_str_for_key("nodeType") {
            if n == "Assignment" {
                // Now recover the type information from the node.
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
        // Recover the type descriptions for the node.
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

/// Implements the function call mutation algorithm.
///
/// The mutator should identify function call expressions where the function call contains
/// at least two arguments of the same type.  The mutator will swap the two arguments.
struct FunctionCallMutator {}

impl FunctionCallMutator {
    /// Create the mutator object.
    pub fn new() -> FunctionCallMutator {
        FunctionCallMutator {}
    }
}

impl Mutator<SolidityAST> for FunctionCallMutator {
    fn is_mutable_node(&self, _node: &SolidityAST) -> bool {
        // TODO: Finish the implementation.
        false
    }

    fn mutate(&self, _node: &mut SolidityAST, _rand: &mut Pcg64) {
        // TODO: Finish the implementation.
        return;
    }

    fn implements(&self) -> MutationType {
        MutationType::Generic(GenericMutation::FunctionCall)
    }
}

/// Implements the Solidity require function mutation algorithm.
///
/// This mutator will replace the expression in the argument to the Solidity `require` function
/// with the logical negation of the expression.
///
/// # Example
///
/// ```solidity
/// require(a > b);
/// ```
///
/// would become
///
/// ```solidity
/// require(!(a > b))
/// ```
struct SolidityRequireMutator {}

impl SolidityRequireMutator {
    /// Create the new require mutator.
    pub fn new() -> SolidityRequireMutator {
        SolidityRequireMutator {}
    }
}

impl Mutator<SolidityAST> for SolidityRequireMutator {
    fn is_mutable_node(&self, node: &SolidityAST) -> bool {
        // Check that the node is a 'FunctionCall' node, that the function call is the
        // `require` function, and that the function has an argument.
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
        // First create a Unary ! operation node.
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

        // Create a Tuple node to hold the function argument
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

/// Implement the [`MutatorFactory<T>`] trait to have an interface for getting mutators for requested
/// mutation algorithms.
pub struct SolidityMutatorFactory {}

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
                GenericMutation::BitwiseBinaryOp => Some(Box::new(BinaryOpMutator::new(
                    bitwise_operators(),
                    MutationType::Generic(GenericMutation::BitwiseBinaryOp),
                ))),
                GenericMutation::ComparisonBinaryOp => Some(Box::new(BinaryOpMutator::new(
                    comparison_operators(),
                    MutationType::Generic(GenericMutation::ComparisonBinaryOp),
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
