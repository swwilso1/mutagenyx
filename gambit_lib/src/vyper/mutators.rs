//! The `vyper::mutators` module provides the objects that implement the mutation algorithms
//! for the Vyper ASTs.  The module also provides the factory that implements [`MutatorFactory<t>`].

use crate::error::GambitError;
use crate::json::*;
use crate::mutation::*;
use crate::mutator::*;
use crate::operators::*;
use crate::vyper::ast::VyperAST;
use crate::vyper::operators::get_python_operator_map;
use num::{Float, Integer};
use rand::seq::SliceRandom;
use rand::Rng;
use rand::RngCore;
use rand_pcg::*;
use std::collections::HashMap;
use std::fmt;

/// Return a new integer literal node representing an integer literal number.
///
/// # Arguments
/// * `value` - The number that the node should contain.
fn new_integer_constant_node<I: Integer + fmt::Display>(value: I) -> Result<VyperAST, GambitError> {
    let node_str = format!(
        "{{\
            \"node_id\": 9999999,
            \"ast_type\": \"Int\",
            \"value\": {value}
        }}"
    );
    new_json_node(&node_str)
}

/// Return a new float literal node representing an float literal number.
///
/// # Arguments
///
/// * `value` - The number that the node should contain.
fn new_float_constant_node<F: Float + fmt::Display>(value: F) -> Result<VyperAST, GambitError> {
    let node_str = format!(
        "{{\
            \"node_id\": 9999998,
            \"ast_type\": \"Decimal\",
            \"value\": \"{value}\"
        }}"
    );
    new_json_node(&node_str)
}

/// Return a new boolean literal node representing a boolean value.
///
/// # Arguments
///
/// * `value` - The boolean value that the node should contain.
fn new_boolean_constant_node(value: bool) -> Result<VyperAST, GambitError> {
    let value_str = match value {
        true => "True",
        false => "False",
    };

    let node_str = format!(
        "{{\
            \"node_id\": 9999997,
            \"ast_type\": \"NameConstant\",
            \"value\": {value_str}
        }}"
    );
    new_json_node(&node_str)
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

    /// A map of names from the Vyper operator name list to the actual operator string.
    operator_map: HashMap<String, String>,

    /// A map of operator strings to the Vyper operator name.
    reverse_operator_map: HashMap<String, String>,

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
        let operator_map = get_python_operator_map();
        let mut reverse_operator_map: HashMap<String, String> = HashMap::new();

        for (key, value) in operator_map.iter() {
            reverse_operator_map.insert(value.clone(), key.clone());
        }

        BinaryOpMutator {
            operators,
            operator_map,
            reverse_operator_map,
            mutation_type,
        }
    }
}

impl Mutator<VyperAST> for BinaryOpMutator {
    fn is_mutable_node(&self, node: &VyperAST) -> bool {
        // First check to see if the node in the AST is a "BinOp" node.
        if let Some(n) = node.get_str_for_key("ast_type") {
            if n == "BinOp" || n == "BoolOp" || n == "Compare" {
                // Get the operator from the node and see if the operator is in the mutator's
                // list of supported operators. The mutator can mutate the node if it supports
                // the node's operator.
                if let Some(op_node) = node.borrow_value_for_key("op") {
                    if let Some(op_type_str) = op_node.get_str_for_key("ast_type") {
                        if let Some(op_form_str) = self.operator_map.get(op_type_str) {
                            return self.operators.contains(op_form_str);
                        }
                    }
                }
            }
        }
        false
    }

    fn mutate(&self, node: &mut VyperAST, rand: &mut Pcg64) {
        if let Some(mut op) = node.take_value_for_key("op") {
            if let Some(op_type_str) = op.get_str_for_key("ast_type") {
                // Get the original operator so that we can use it to compare for the
                // randomly chosen new operator. We do not want to replace the original operator
                // with itself, just by randomly selecting the same operator from the operator list.
                let original_operator = self.operator_map.get(op_type_str).unwrap();

                // Choose a new operator.
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

                // Now recover the Vyper operator name from the operator string form.
                let vyper_chosen_operator = self.reverse_operator_map.get(chosen_operator).unwrap();

                // Insert the new operator into the node.
                op.set_str_for_key("ast_type", &vyper_chosen_operator);

                node.set_node_for_key("op", op);
            }
        }
    }

    fn implements(&self) -> MutationType {
        self.mutation_type
    }
}

struct AssignmentMutator {}

impl Mutator<VyperAST> for AssignmentMutator {
    fn is_mutable_node(&self, node: &VyperAST) -> bool {
        // First check to see if the node is an `Assign` node.
        if let Some(n) = node.get_str_for_key("ast_type") {
            if n == "Assign" {
                return true;
            }
        }
        false
    }

    fn mutate(&self, node: &mut VyperAST, rand: &mut Pcg64) {
        // We need to replace the 'value' node in the tree with a
        // node that contains an int, unsigned int, float, or a boolean value.
        // Vyper is Python based and as such nodes in the tree do not carry
        // type annotations.  So, we can basically assign any value to the
        // target in the node.

        // Randomly pick the which subtype of value to assign.
        let mutation_kind = rand.next_u64() % 4;

        match mutation_kind {
            0 => {
                // Generate an integer
                let lower_bound = -(2_i128.pow(127)) as i128;
                let upper_bound = (2_i128.pow(127)) as i128;
                let replacement_value = rand.gen_range(lower_bound, upper_bound);

                let new_node = match new_integer_constant_node(replacement_value) {
                    Ok(n) => n,
                    _ => return,
                };

                node.set_node_for_key("value", new_node);
            }
            1 => {
                // Generate and unsigned integer
                let lower_bound = 0 as u128;
                let upper_bound = 340282366920938463463374607431768211455u128;
                let replacement_value = rand.gen_range(lower_bound, upper_bound);

                let new_node = match new_integer_constant_node(replacement_value) {
                    Ok(n) => n,
                    _ => return,
                };

                node.set_node_for_key("value", new_node);
            }
            2 => {
                // Generate a boolean
                let random_boolean = rand.next_u64() % 2 as u64;
                let actual_boolean = match random_boolean {
                    1 => true,
                    _ => false,
                };

                let new_node = match new_boolean_constant_node(actual_boolean) {
                    Ok(n) => n,
                    _ => return,
                };

                node.set_node_for_key("value", new_node);
            }
            3 => {
                // Generate a float
                // Vyper floating point numbers support values in the range:
                // [-2^167 / 10^10, (2^167 - 1) / 10^10]
                let lower_bound = -18707220957835557353007165858768422651595.9365500928;
                let upper_bound = 18707220957835557353007165858768422651595.9365500927;
                let random_float = rand.gen_range(lower_bound, upper_bound);

                let new_node = match new_float_constant_node(random_float) {
                    Ok(n) => n,
                    _ => return,
                };

                node.set_node_for_key("value", new_node);
            }
            _ => return,
        }
    }

    fn implements(&self) -> MutationType {
        MutationType::Generic(GenericMutation::Assignment)
    }
}

struct FunctionCallMutator {}

impl Mutator<VyperAST> for FunctionCallMutator {
    fn is_mutable_node(&self, node: &VyperAST) -> bool {
        // First check to see if the node is an `Call` node.
        if let Some(n) = node.get_str_for_key("ast_type") {
            if n == "Call" {
                if let Some(args) = node.borrow_value_for_key("args") {
                    if args.is_array() {
                        if let Some(args_array) = args.as_array() {
                            if args_array.len() >= 2 {
                                // We can only meaningfully mutate arguments if the function call
                                // has two or more arguments.
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    fn mutate(&self, node: &mut VyperAST, rand: &mut Pcg64) {
        if let Some(args) = node.take_value_for_key("args") {
            if let Some(args_array) = node.as_array_mut() {
                if args_array.len() == 2 {
                    // Just swap the two arguments
                    let arg1 = args_array[0].clone();
                    let arg2 = args_array[1].clone();
                    args_array[0] = arg2;
                    args_array[1] = arg1;

                    node.set_node_for_key("args", args);
                } else if args_array.len() > 2 {
                    // Pick two arguments randomly and swap them.
                    let random_index1 = (rand.next_u64() % args_array.len() as u64) as usize;
                    let mut random_index2 = (rand.next_u64() % args_array.len() as u64) as usize;

                    // The indexes may be the same, so just iterate until we get a different index.
                    while random_index1 == random_index2 {
                        random_index2 = (rand.next_u64() % args_array.len() as u64) as usize;
                    }

                    let arg1 = args_array[random_index1].clone();
                    let arg2 = args_array[random_index2].clone();
                    args_array[random_index1] = arg2;
                    args_array[random_index2] = arg1;

                    node.set_node_for_key("args", args);
                }
            }
        }
    }

    fn implements(&self) -> MutationType {
        MutationType::Generic(GenericMutation::FunctionCall)
    }
}

/// Implement the [`MutatorFactory<T>`] trait to have an interface for getting mutators for requested
/// mutation algorithms.
pub struct VyperMutatorFactory {}

impl MutatorFactory<VyperAST> for VyperMutatorFactory {
    fn mutator_for(&self, mutation_type: &MutationType) -> Option<Box<dyn Mutator<VyperAST>>> {
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
                GenericMutation::BitshiftBinaryOp => Some(Box::new(BinaryOpMutator::new(
                    bitshift_operators(),
                    MutationType::Generic(GenericMutation::BitshiftBinaryOp),
                ))),
                GenericMutation::ComparisonBinaryOp => Some(Box::new(BinaryOpMutator::new(
                    comparison_operators(),
                    MutationType::Generic(GenericMutation::ComparisonBinaryOp),
                ))),
                GenericMutation::Assignment => Some(Box::new(AssignmentMutator {})),
                GenericMutation::FunctionCall => Some(Box::new(FunctionCallMutator {})),
                _ => None,
            },
            _ => None,
        }
    }
}
