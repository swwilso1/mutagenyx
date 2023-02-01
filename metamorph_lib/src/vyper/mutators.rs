//! The `vyper::mutators` module provides the objects that implement the mutation algorithms
//! for the Vyper ASTs.  The module also provides the factory that implements [`MutatorFactory<t>`].

use crate::error::MetamorphError;
use crate::json::*;
use crate::mutation::*;
use crate::mutator::*;
use crate::node_printer_helpers::traverse_sub_node_and_print;
use crate::operators::*;
use crate::pretty_printer::PrettyPrinter;
use crate::vyper::ast::VyperAST;
use crate::vyper::operators::get_python_operator_map;
use crate::vyper::pretty_printer::{VyperNodePrinterFactory, VyperPrettyPrinterSettings};
use num::{Float, Integer};
use rand::seq::SliceRandom;
use rand::Rng;
use rand::RngCore;
use rand_pcg::*;
use serde_json::json;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;

/// Return a new integer literal node representing an integer literal number.
///
/// # Arguments
/// * `value` - The number that the node should contain.
fn new_integer_constant_node<I: Integer + fmt::Display>(
    value: I,
) -> Result<VyperAST, MetamorphError> {
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
fn new_float_constant_node<F: Float + fmt::Display>(value: F) -> Result<VyperAST, MetamorphError> {
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
fn new_boolean_constant_node(value: bool) -> Result<VyperAST, MetamorphError> {
    let node_str = format!(
        "{{\
            \"node_id\": 9999997,
            \"ast_type\": \"NameConstant\",
            \"value\": {value}
        }}"
    );
    new_json_node(&node_str)
}

/// Return a new comment node that contains `text`.
///
/// **Important**: The comment node is not a part of the official Vyper AST.
/// We introduce it here in order to have a way to delete nodes from the tree, but
/// still have the node text.  Only the Metamorph Vyper pretty-printing code supports
/// comment nodes.
///
/// # Arguments
///
/// * `text` - The string slice referring to the text to put in the comment.
fn new_comment_node(text: &str) -> Result<VyperAST, MetamorphError> {
    let text_node = json![text];

    let node_str = format!(
        "{{\
            \"node_id\": 9999996,
            \"ast_type\": \"Comment\",
            \"value\": null
        }}"
    );
    let mut node = new_json_node(&node_str)?;
    node.set_node_for_key("value", text_node);
    Ok(node)
}

/// Return a new Pass node.
fn new_pass_node() -> Result<VyperAST, MetamorphError> {
    let node_str = format!(
        "{{\
            \"node_id\": 9999995,
            \"ast_type\": \"Pass\"
        }}"
    );
    new_json_node(&node_str)
}

/// Return a new Str node.
///
/// # Arguments
///
/// * `text` - The reference to the string slice containing the text for the node.
fn new_string_node(text: &str) -> Result<VyperAST, MetamorphError> {
    let node_str = format! {
        "{{\
            \"node_id\": 9999994,
            \"ast_type\": \"Str\",
            \"value\": \"{text}\"
        }}"
    };
    new_json_node(&node_str)
}

/// Return an new `Return` node
///
/// # Arguments
///
/// * `node` - The node to use in the return.
fn new_return_node(node: VyperAST) -> Result<VyperAST, MetamorphError> {
    let node_str = format! {
        "{{\
            \"node_id\": 9999993,
            \"ast_type\": \"Return\",
            \"value\": null
        }}"
    };

    let mut return_node = new_json_node(&node_str)?;
    return_node.set_node_for_key("value", node);
    Ok(return_node)
}

/// Return a new 'UnaryOp' node.
///
/// # Arguments
/// * `operator` - String ref referring to the operator name.  Usually 'Not'.
/// * `operand` - The node that lives inside the UnaryOp (the node to which the node apples the
/// operator).
fn new_unary_op_node(operator: &str, operand: VyperAST) -> Result<VyperAST, MetamorphError> {
    let node_str = format!(
        "{{\
            \"node_id\": 9999992,
            \"operand\": null,
            \"op\": {{\
                \"node_id\": 9999991,
                \"ast_type\": \"{operator}\"
            }},
            \"ast_type\": \"UnaryOp\"
        }}"
    );

    let mut new_node = new_json_node(&node_str)?;
    new_node.set_node_for_key("operand", operand);
    Ok(new_node)
}

enum ListLikeThing {
    Tuple,
    List,
}

impl fmt::Display for ListLikeThing {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let text = match self {
            ListLikeThing::Tuple => "Tuple",
            ListLikeThing::List => "List",
        };
        write!(f, "{}", text)
    }
}

/// Return a `kind` node with `size` members whose value is all None
///
/// # Arguments
///
/// * `size` - The number of None elements in the tuple.
/// * `kind` - The kind of list like thing to make.
fn new_list_like_thing_node(size: u32, kind: ListLikeThing) -> Result<VyperAST, MetamorphError> {
    let mut none_array: Vec<VyperAST> = vec![];
    for _i in 0..size {
        let node_str = format!(
            "{{\
                \"node_id\": 9999992,
                \"ast_type\": \"NameConstant\",
                \"value\": null
            }}"
        );

        let node = new_json_node(&node_str)?;
        none_array.push(node);
    }

    let node_str = format!(
        "{{\
            \"node_id\": 9999991,
            \"ast_type\": \"{kind}\",
            \"elements\": []
        }}"
    );

    let mut tuple_node = new_json_node(&node_str)?;
    let none_array_node = json![none_array];
    tuple_node.set_node_for_key("elements", none_array_node);
    Ok(tuple_node)
}

/// The object that implements mutations for binary expressions.
///
/// Multiple mutation algorithms operate on binary expressions.  Each of those
/// mutation algorithms uses a subset of the available binary operators.  This
/// one structure/class provides the functionality to implement these different
/// binary expression mutations.
struct BinaryOpMutator {
    /// A list of operators valid for the binary expression
    operators: Vec<&'static str>,

    /// A map of names from the Vyper operator name list to the actual operator string.
    operator_map: HashMap<&'static str, &'static str>,

    /// A map of operator strings to the Vyper operator name.
    reverse_operator_map: HashMap<&'static str, &'static str>,

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
    pub fn new(operators: Vec<&'static str>, mutation_type: MutationType) -> BinaryOpMutator {
        let operator_map = get_python_operator_map();
        let mut reverse_operator_map: HashMap<&'static str, &'static str> = HashMap::new();

        for (key, value) in operator_map.iter() {
            reverse_operator_map.insert(value, key);
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

/// Implements the assignment mutation algorithm.
///
/// For a given Assigment expression, the algorithm replaces the right-hand side of the expression
/// with a random value.  The algorithm operates on assignments to integer, unsigned integer,
/// and boolean variables.  For number types, the algorithm will try to generate values in the
/// valid value ranges for each type.
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

/// The type that implements the DeleteStatement mutation algorithm.
///
/// The algorithm finds FunctionDef nodes and attempts to correctly remove
/// a statement in the function definition while retaining correct compilation of the function.
///
/// # Example
///
/// ```python
/// def foo():
///     return 10
/// ```
///
/// might become
///
/// ```python
/// def foo():
///     # return 10
///     pass
/// ```
///
/// Or
///
/// ```python
/// def foo() -> (int128, int128):
///     return 2, 3
/// ```
///
/// might become
///
/// ```python
/// def foo() -> (int128, int128):
///     # return 2, 3
///     return (None, None)
/// ```
struct DeleteStatementMutator {}

impl Mutator<VyperAST> for DeleteStatementMutator {
    fn is_mutable_node(&self, node: &VyperAST) -> bool {
        if let Some(ast_type_str) = node.get_str_for_key("ast_type") {
            if ast_type_str == "FunctionDef" {
                if let Some(body_node) = node.borrow_value_for_key("body") {
                    if let Some(body_array) = body_node.as_array() {
                        if body_array.len() > 0 {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn mutate(&self, node: &mut VyperAST, rand: &mut Pcg64) {
        // We have a FunctionDef node.  We want to randomly delete a node in the body
        // array (assuming a body array is present).
        if let Some(mut body_node) = node.take_value_for_key("body") {
            if let Some(body_array) = body_node.as_array_mut() {
                // Choose an index to replace with a comment.
                let index = (rand.next_u64() % body_array.len() as u64) as usize;
                let value = body_array.remove(index);

                // Pretty-print the node so we can then wrap the string version in a comment
                // node.
                let mut contents = Vec::new();
                let mut printer = PrettyPrinter::new(4, 150);
                traverse_sub_node_and_print(
                    &mut printer,
                    &mut contents,
                    VyperNodePrinterFactory::new(VyperPrettyPrinterSettings::default()),
                    &value,
                );
                let s = core::str::from_utf8(contents.as_slice()).unwrap();

                let new_node = match new_comment_node(s) {
                    Ok(node) => node,
                    Err(_e) => return,
                };

                body_array.insert(index, new_node);
                if body_array.len() == 1 {
                    // We just commented out the only node in the body array.  In this case
                    // We need to check the return type if it exists.  If a return value is needed
                    // We will try and provide a return value.  Otherwise, we need to add a Pass
                    // node so that the function will still compile.
                    if let Some(returns_node) = node.borrow_value_for_key("returns") {
                        if let Some(ast_type_str) = returns_node.get_str_for_key("ast_type") {
                            if ast_type_str == "Name" {
                                if let Some(id_str) = returns_node.get_str_for_key("id") {
                                    match &id_str[..3] {
                                        "boo" => {
                                            let random_boolean = rand.next_u64() % 2 as u64;
                                            let actual_boolean = match random_boolean {
                                                1 => true,
                                                _ => false,
                                            };

                                            let new_node =
                                                match new_boolean_constant_node(actual_boolean) {
                                                    Ok(n) => n,
                                                    _ => return,
                                                };

                                            let return_node = match new_return_node(new_node) {
                                                Ok(node) => node,
                                                Err(_e) => return,
                                            };

                                            body_array.push(return_node);
                                        }
                                        "uin" => {
                                            // We could figure out the numeric range, but instead
                                            // we just return a random value between 0-10.
                                            let number = rand.next_u64() % 10 as u64;
                                            let new_node = match new_integer_constant_node(number) {
                                                Ok(node) => node,
                                                Err(_e) => return,
                                            };
                                            let return_node = match new_return_node(new_node) {
                                                Ok(node) => node,
                                                Err(_e) => return,
                                            };

                                            body_array.push(return_node);
                                        }
                                        "int" => {
                                            // We just pick a number between -10 and 10.  Nothing
                                            // complicated.
                                            let mut number = (rand.next_u64() & 20 as u64) as i64;
                                            number -= 10;
                                            let new_node = match new_integer_constant_node(number) {
                                                Ok(node) => node,
                                                Err(_e) => return,
                                            };
                                            let return_node = match new_return_node(new_node) {
                                                Ok(node) => node,
                                                Err(_e) => return,
                                            };

                                            body_array.push(return_node);
                                        }
                                        "str" => {
                                            // Use our friend lorem ipsum.
                                            let new_node = match new_string_node(
                                                "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua."
                                            ) {
                                                Ok(node) => node,
                                                Err(_e) => return
                                            };
                                            let return_node = match new_return_node(new_node) {
                                                Ok(node) => node,
                                                Err(_e) => return,
                                            };

                                            body_array.push(return_node);
                                        }
                                        _ => return,
                                    }
                                }
                            } else if ast_type_str == "Pass" {
                                // We already have a pass node, just skip out.
                                ();
                            } else if ast_type_str == "Tuple" || ast_type_str == "List" {
                                if let Some(elements_node) =
                                    returns_node.borrow_value_for_key("elements")
                                {
                                    if let Some(elements_array) = elements_node.as_array() {
                                        let node_type = match ast_type_str {
                                            "Tuple" => ListLikeThing::Tuple,
                                            "List" => ListLikeThing::List,
                                            _ => return,
                                        };

                                        let list_like_node = match new_list_like_thing_node(
                                            elements_array.len() as u32,
                                            node_type,
                                        ) {
                                            Ok(node) => node,
                                            Err(_e) => return,
                                        };

                                        let return_node = match new_return_node(list_like_node) {
                                            Ok(node) => node,
                                            Err(_e) => return,
                                        };
                                        body_array.push(return_node);
                                    }
                                }
                            }
                        } else {
                            let new_node = match new_pass_node() {
                                Ok(node) => node,
                                Err(_e) => return,
                            };
                            body_array.push(new_node);
                        }
                    } else {
                        let new_node = match new_pass_node() {
                            Ok(node) => node,
                            Err(_e) => return,
                        };
                        body_array.push(new_node);
                    }
                }

                node.set_node_for_key("body", body_node);
            }
        }
    }

    fn implements(&self) -> MutationType {
        MutationType::Generic(GenericMutation::DeleteStatement)
    }
}

struct FunctionCallMutator {}

impl Mutator<VyperAST> for FunctionCallMutator {
    fn is_mutable_node(&self, node: &VyperAST) -> bool {
        // First check to see if the node is an `Call` node.
        if let Some(n) = node.get_str_for_key("ast_type") {
            if n == "Call" {
                if let Some(args) = node.borrow_value_for_key("args") {
                    if let Some(args_array) = args.as_array() {
                        if args_array.len() > 0 {
                            // We have to have a function call with arguments.
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn mutate(&self, node: &mut VyperAST, rand: &mut Pcg64) {
        if let Some(args_node) = node.borrow_value_for_key("args") {
            if let Some(args_array) = args_node.as_array() {
                loop {
                    // Randomly pick an array member, but avoid Int/Str nodes.
                    let index = (rand.next_u64() % args_array.len() as u64) as usize;
                    let value = &args_array[index];
                    if let Some(node_type) = value.get_str_for_key("nodeType") {
                        if node_type == "Int" || node_type == "Str" {
                            continue;
                        }
                    }
                    *node = value.clone();
                    break;
                }
            }
        }
    }

    fn implements(&self) -> MutationType {
        MutationType::Generic(GenericMutation::FunctionCall)
    }
}

/// Implements the function call argument swap mutation algorithm.
///
/// The mutator should identify function call expressions where the function call contains
/// at least two arguments of the same type.  The mutator will swap two of the arguments.  Since
/// the Vyper AST does not contain type annotations for the arguments to function calls, it is
/// outside the scope of this algorithm to ensure that algorithm only swaps arguments of the same
/// type.
struct SwapFunctionArgumentsMutator {}

impl Mutator<VyperAST> for SwapFunctionArgumentsMutator {
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
        MutationType::Generic(GenericMutation::FunctionSwapArguments)
    }
}

/// Implement the IfStatement mutation algorithm.
///
/// The algorithm will randomly choose between three possible mutations:
/// * The algorithm replaces the condition of the if statement with `true`.
/// * The algorithm replaces the condition of the if statement with `false`.
/// * The algorithm replaces the condition (called c) of the if statement with `!(c)`.
struct IfStatementMutator {}

impl Mutator<VyperAST> for IfStatementMutator {
    fn is_mutable_node(&self, node: &VyperAST) -> bool {
        if let Some(ast_type) = node.get_str_for_key("ast_type") {
            if ast_type == "If" {
                if let Some(_test_node) = node.borrow_value_for_key("test") {
                    return true;
                }
            }
        }
        false
    }

    fn mutate(&self, node: &mut VyperAST, rand: &mut Pcg64) {
        // Randomly choose between three possible mutations:
        // * Replace condition with true.
        // * Replace condition with false.
        // * Replace condition (called c) with !(c) (ie the negation).
        match rand.next_u64() % 3 as u64 {
            0 => {
                // Replace the condition with 'True'
                let new_node = match new_boolean_constant_node(true) {
                    Ok(n) => n,
                    Err(_e) => return,
                };

                node.set_node_for_key("test", new_node);
            }
            1 => {
                // Replace the condition with `False`.
                let new_node = match new_boolean_constant_node(false) {
                    Ok(n) => n,
                    Err(_e) => return,
                };
                node.set_node_for_key("test", new_node);
            }
            2 => {
                // Replace the condition (called c) with !(c).
                if let Some(test_node) = node.take_value_for_key("test") {
                    let new_node = match new_unary_op_node("Not", test_node) {
                        Ok(n) => n,
                        Err(_e) => return,
                    };

                    node.set_node_for_key("test", new_node);
                }
            }
            _ => return,
        }
    }

    fn implements(&self) -> MutationType {
        MutationType::Generic(GenericMutation::IfStatement)
    }
}

/// Implement the Integer mutation algorithm:
///
/// Randomly selects one from the following:
/// * Adds one to integer constant.
/// * Subtracts one from integer constant.
/// * Generates a random value.
struct IntegerMutator {}

impl Mutator<VyperAST> for IntegerMutator {
    fn is_mutable_node(&self, node: &VyperAST) -> bool {
        if let Some(ast_type) = node.get_str_for_key("ast_type") {
            if ast_type == "Int" {
                return true;
            }
        }
        false
    }

    fn mutate(&self, node: &mut VyperAST, rand: &mut Pcg64) {
        match rand.next_u64() % 3 as u64 {
            0 => {
                // Add one to the integer constant.
                if let Some(value_node) = node.borrow_value_for_key("value") {
                    if let Some(mut value) = value_node.as_i64() {
                        value += 1;
                        let json_value = json![value];
                        node.set_node_for_key("value", json_value);
                    }
                }
            }
            1 => {
                // Subtract one from the integer contant.
                if let Some(value_node) = node.borrow_value_for_key("value") {
                    if let Some(mut value) = value_node.as_i64() {
                        value -= 1;
                        let json_value = json![value];
                        node.set_node_for_key("value", json_value);
                    }
                }
            }
            2 => {
                // Generate a random number.
                let value = rand.next_u64();
                let json_value = json![value as i64];
                node.set_node_for_key("value", json_value);
            }
            _ => return,
        }
    }

    fn implements(&self) -> MutationType {
        MutationType::Generic(GenericMutation::Integer)
    }
}

/// Implement the operator swap mutation algorithm
///
/// The algorithm swaps the left and right hand sides of the arguments
/// to a BinOp/BoolOp/Compare.  The operator of the BinOp/BoolOp/Compare must
/// be in the list of non-commutative operators: [-, /, %, **, >, <, <=, >=, <<, >>]
struct OperatorSwapArgumentsMutator {
    valid_operators: Vec<&'static str>,
    operator_map: HashMap<&'static str, &'static str>,
}

impl OperatorSwapArgumentsMutator {
    /// Create a new mutator
    fn new() -> OperatorSwapArgumentsMutator {
        OperatorSwapArgumentsMutator {
            valid_operators: non_commutative_operators(),
            operator_map: get_python_operator_map(),
        }
    }
}

impl Mutator<VyperAST> for OperatorSwapArgumentsMutator {
    fn is_mutable_node(&self, node: &VyperAST) -> bool {
        if let Some(ast_type) = node.get_str_for_key("ast_type") {
            if ast_type == "BinOp" || ast_type == "BoolOp" || ast_type == "Compare" {
                if let Some(op_node) = node.borrow_value_for_key("op") {
                    if let Some(op_string) = op_node.get_str_for_key("ast_type") {
                        let converted_operator = self.operator_map[&op_string];
                        if self.valid_operators.contains(&converted_operator) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn mutate(&self, node: &mut VyperAST, _rand: &mut Pcg64) {
        if let Some(ast_type) = node.get_str_for_key("ast_type") {
            if ast_type == "BinOp" || ast_type == "Compare" {
                if let Some(left_node) = node.take_value_for_key("left") {
                    if let Some(right_node) = node.take_value_for_key("right") {
                        node.set_node_for_key("left", right_node);
                        node.set_node_for_key("right", left_node);
                    }
                }
            } else if ast_type == "BoolOp" {
                if let Some(mut values_node) = node.take_value_for_key("values") {
                    if let Some(values_array) = values_node.as_array_mut() {
                        let right = values_array.remove(1);
                        let left = values_array.remove(0);
                        values_array.push(right);
                        values_array.push(left);
                    }
                }
            }
        }
    }

    fn implements(&self) -> MutationType {
        MutationType::Generic(GenericMutation::OperatorSwapArguments)
    }
}

/// Implement line swap mutation algorithm.
///
/// The algorithm chooses two lines from a block of code and attempts to randomly swap two of
/// the lines.  Since function return statements affect how a program compiles, the algorithm
/// will explicitly not swap lines with return statements.
struct LinesSwapMutator {}

impl Mutator<VyperAST> for LinesSwapMutator {
    fn is_mutable_node(&self, node: &VyperAST) -> bool {
        // We need a function definition with at least two body statements.
        if let Some(ast_type) = node.get_str_for_key("ast_type") {
            if ast_type == "FunctionDef" || ast_type == "For" || ast_type == "If" {
                if let Some(body_node) = node.borrow_value_for_key("body") {
                    if let Some(body_array) = body_node.as_array() {
                        if body_array.len() >= 2 {
                            let mut found_return_statement = false;
                            for value in body_array {
                                if let Some(value_ast_type) = value.get_str_for_key("ast_type") {
                                    if value_ast_type == "Return" {
                                        found_return_statement = true;
                                        break;
                                    }
                                }
                            }

                            // If the body of the function def has a return statement, then there
                            // must be at least 3 statements in the body in order to swap (but not
                            // swap a return statement) statements.  If there are no return statements
                            // then we can just go ahead and swap.
                            if (found_return_statement && body_array.len() >= 3)
                                || (!found_return_statement)
                            {
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
        if let Some(mut body_node) = node.take_value_for_key("body") {
            if let Some(body_array) = body_node.as_array_mut() {
                // Randomly pick a first node.
                let mut first_index: usize;
                loop {
                    first_index = (rand.next_u64() % body_array.len() as u64) as usize;
                    if let Some(ast_type) = body_array[first_index].get_str_for_key("ast_type") {
                        if ast_type == "Return" {
                            continue;
                        }
                    }
                    break;
                }

                // Try to randomly pick a second node.
                let mut second_index: usize;
                loop {
                    second_index = (rand.next_u64() % body_array.len() as u64) as usize;
                    if second_index == first_index {
                        continue;
                    }
                    if let Some(ast_type) = body_array[second_index].get_str_for_key("ast_type") {
                        if ast_type == "Return" {
                            continue;
                        }
                    }
                    break;
                }

                let larger_index = if first_index >= second_index {
                    first_index
                } else {
                    second_index
                };

                let smaller_index = if first_index >= second_index {
                    second_index
                } else {
                    first_index
                };

                let larger_node = body_array.remove(larger_index);
                let smaller_node = body_array.remove(smaller_index);
                body_array.insert(smaller_index, larger_node);
                body_array.insert(larger_index, smaller_node);
                node.set_node_for_key("body", body_node);
            }
        }
    }

    fn implements(&self) -> MutationType {
        MutationType::Generic(GenericMutation::LinesSwap)
    }
}

/// Implement a form of the unary operator mutation algorithm.
///
/// As of this writing, Vyper has only two valid unary operations:
/// * `not a`
/// * `~a`
///
/// Since these unary operations occur in expressions with different semantic meaning,
/// (ie one a logical operation and one a bitwise negation) the algorithm will not
/// interchange `not` for `~`, but instead, drop the prefix operator leaving just the operand.
struct UnaryOpMutator {}

impl Mutator<VyperAST> for UnaryOpMutator {
    fn is_mutable_node(&self, node: &VyperAST) -> bool {
        if let Some(ast_type) = node.get_str_for_key("ast_type") {
            if ast_type == "UnaryOp" {
                return true;
            }
        }
        false
    }

    fn mutate(&self, node: &mut VyperAST, _rand: &mut Pcg64) {
        if let Some(operand_node) = node.take_value_for_key("operand") {
            *node = operand_node;
        }
    }

    fn implements(&self) -> MutationType {
        MutationType::Generic(GenericMutation::UnaryOp)
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
                GenericMutation::DeleteStatement => Some(Box::new(DeleteStatementMutator {})),
                GenericMutation::FunctionCall => Some(Box::new(FunctionCallMutator {})),
                GenericMutation::FunctionSwapArguments => {
                    Some(Box::new(SwapFunctionArgumentsMutator {}))
                }
                GenericMutation::IfStatement => Some(Box::new(IfStatementMutator {})),
                GenericMutation::Integer => Some(Box::new(IntegerMutator {})),
                GenericMutation::OperatorSwapArguments => {
                    Some(Box::new(OperatorSwapArgumentsMutator::new()))
                }
                GenericMutation::LinesSwap => Some(Box::new(LinesSwapMutator {})),
                GenericMutation::UnaryOp => Some(Box::new(UnaryOpMutator {})),
            },
            _ => None,
        }
    }
}
