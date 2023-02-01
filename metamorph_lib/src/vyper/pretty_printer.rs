//! The `vyper::pretty_printer` module provides the code to traverse a Vyper AST and print out
//! the source code represented in the AST.

use crate::json::*;
use crate::node_printer::{NodePrinter, NodePrinterFactory};
use crate::node_printer_helpers::*;
use crate::pretty_printer::{
    write_flowable_text, write_indent, write_newline, write_space, write_string, write_token,
    write_triple_string, PrettyPrinter,
};
use crate::vyper::ast::VyperAST;
use crate::vyper::operators::get_python_operator_map;
use std::collections::HashMap;
use std::io::Write;

/// Write the `value` element of `node` by traversing the object in `value`.
///
/// # Arguments
///
/// * `printer` - The [`PrettyPrinter`] object that will format the output.
/// * `stream` - The [`Write`] object that will receive the formatted text.
/// * `node` - The node in the Vyper AST.
fn write_simple_value<W: Write>(printer: &mut PrettyPrinter, stream: &mut W, node: &VyperAST) {
    if let Some(value_node) = node.borrow_value_for_key("value") {
        traverse_sub_node_and_print(
            printer,
            stream,
            VyperNodePrinterFactory::default(),
            value_node,
        );
    }
}

/// Write the value of the array contained in the `elements` key.
///
/// # Arguments
///
/// * `printer` - The [`PrettyPrinter`] object that will format the output.
/// * `stream` - The [`Write`] object that will receive the formatted text.
/// * `node` - The node in the Vyper AST.
fn write_elements_array<W: Write>(printer: &mut PrettyPrinter, stream: &mut W, node: &VyperAST) {
    if let Some(elements_node) = node.borrow_value_for_key("elements") {
        if let Some(elements_array) = elements_node.as_array() {
            print_array_helper(
                printer,
                stream,
                VyperNodePrinterFactory::default(),
                elements_array,
            );
        }
    }
}

/// Write the value of the array contained in `key`.
///
/// # Arguments
///
/// * `printer` - The [`PrettyPrinter`] object that will format the output.
/// * `stream` - The [`Write`] object that will receive the formatted text.
/// * `key` - The key string.
/// * `node` - The node in the Vyper AST.
fn write_key_of_node_as_array<W: Write>(
    printer: &mut PrettyPrinter,
    stream: &mut W,
    key: &str,
    node: &VyperAST,
) {
    if let Some(sub_node) = node.borrow_value_for_key(key) {
        if let Some(sub_array) = sub_node.as_array() {
            print_array_helper(
                printer,
                stream,
                VyperNodePrinterFactory::default(),
                sub_array,
            );
        }
    }
}

/// Write the value of the array contained in the 'args' key.
///
/// # Arguments
///
/// * `printer` - The [`PrettyPrinter`] object that will format the output.
/// * `stream` - The [`Write`] object that will receive the formatted text.
/// * `node` - The node in the Vyper AST.
fn write_args_as_array<W: Write>(printer: &mut PrettyPrinter, stream: &mut W, node: &VyperAST) {
    write_key_of_node_as_array(printer, stream, "args", node);
}

/// Write the object in the `value` element of `node` with ' = <value>'.
///
/// # Arguments
///
/// * `printer` - The [`PrettyPrinter`] object that will format the output.
/// * `stream` - The [`Write`] object that will receive the formatted text.
/// * `node` - The node in the Vyper AST.
fn write_value_assignment<W: Write>(printer: &mut PrettyPrinter, stream: &mut W, node: &VyperAST) {
    if let Some(value_node) = node.borrow_value_for_key("value") {
        if !value_node.is_null() {
            write_token(printer, stream, " = ");
            traverse_sub_node_and_print(
                printer,
                stream,
                VyperNodePrinterFactory::default(),
                value_node,
            );
        }
    }
}

/// Write the object in the `body` element as a sequence of indented text followed by a newline.
///
/// # Arguments
///
/// * `printer` - The [`PrettyPrinter`] object that will format the output.
/// * `stream` - The [`Write`] object that will receive the formatted text.
/// * `node` - The node in the Vyper AST.
/// * `settings` - [`VyperPrettyPrinterSettings`] for the node factory to use.
fn write_body_as_nodes<W: Write>(
    printer: &mut PrettyPrinter,
    stream: &mut W,
    node: &VyperAST,
    settings: VyperPrettyPrinterSettings,
) {
    if let Some(body_node) = node.borrow_value_for_key("body") {
        if let Some(body_array) = body_node.as_array() {
            for value in body_array {
                write_indent(printer, stream);
                traverse_sub_node_and_print(
                    printer,
                    stream,
                    VyperNodePrinterFactory::new(settings),
                    value,
                );
                write_newline(printer, stream);
            }
        }
    }
}

/// Write the array in the `body` element as a sequence of indented text followed by a newline, with
/// no newline after the last element in the array.
///
/// # Arguments
///
/// * `printer` - The [`PrettyPrinter`] object that will format the output.
/// * `stream` - The [`Write`] object that will receive the formatted text.
/// * `node` - The node in the Vyper AST.
fn write_indented_body_array<W: Write>(
    printer: &mut PrettyPrinter,
    stream: &mut W,
    node: &VyperAST,
) {
    if let Some(body_node) = node.borrow_value_for_key("body") {
        if let Some(body_array) = body_node.as_array() {
            write_indented_array(printer, stream, body_array);
        }
    }
}

/// Write the contents of the `array` argument as indented text followed by a newline except for
/// no newline after the last element of the array.
///
/// # Arguments
///
/// * `printer` - The [`PrettyPrinter`] object that will format the output.
/// * `stream` - The [`Write`] object that will receive the formatted text.
/// * `array` - The array of [`VyperAST`] nodes.
fn write_indented_array<W: Write>(
    printer: &mut PrettyPrinter,
    stream: &mut W,
    array: &Vec<VyperAST>,
) {
    let mut i = 0;
    while i < array.len() {
        let value = &array[i];
        write_indent(printer, stream);
        traverse_sub_node_and_print(printer, stream, VyperNodePrinterFactory::default(), value);
        if i < (array.len() - 1) {
            write_newline(printer, stream);
        }
        i += 1;
    }
}

/// Write the contents of the node and wrap the contents in parentheses if the node is a BinOp,
/// Compare, or BoolOp.
///
/// # Arguments
///
/// * `printer` - The [`PrettyPrinter`] object that will format the text.
/// * `stream` - The [`Write`] object that will receive the formatted text.
/// * `node` - The Vyper AST node.
fn write_node_with_parens_maybe<W: Write>(
    printer: &mut PrettyPrinter,
    stream: &mut W,
    node: &VyperAST,
) {
    if let Some(ast_type) = node.get_str_for_key("ast_type") {
        let needs_parens = ast_type == "BinOp" || ast_type == "Compare" || ast_type == "BoolOp";

        if needs_parens {
            write_token(printer, stream, "(");
        }
        traverse_sub_node_and_print(printer, stream, VyperNodePrinterFactory::default(), node);

        if needs_parens {
            write_token(printer, stream, ")");
        }
    }
}

/// Write the string element in the value key of `node` as a token.
///
/// # Arguments
///
/// * `printer` - The [`PrettyPrinter`] object that will format the text.
/// * `stream` - The [`Write`] object that will receive the formatted text.
/// * `node` - The Vyper AST node.
fn write_value_string_as_token<W: Write>(
    printer: &mut PrettyPrinter,
    stream: &mut W,
    node: &VyperAST,
) {
    if let Some(value_str) = node.get_str_for_key("value") {
        write_token(printer, stream, value_str);
    }
}

struct DummyNodePrinter {}

impl<W: Write> NodePrinter<W, VyperAST> for DummyNodePrinter {
    fn visit_children(&mut self) -> bool {
        true
    }
}

struct NamePrinter {}

impl<W: Write> NodePrinter<W, VyperAST> for NamePrinter {
    fn print_node(&mut self, stream: &mut W, node: &VyperAST, printer: &mut PrettyPrinter) {
        if let Some(name) = node.get_str_for_key("id") {
            write_token(printer, stream, name);
        }
    }
}

struct VariableDeclPrinter {}

impl<W: Write> NodePrinter<W, VyperAST> for VariableDeclPrinter {
    fn print_node(&mut self, stream: &mut W, node: &VyperAST, printer: &mut PrettyPrinter) {
        if let Some(target_node) = node.borrow_value_for_key("target") {
            traverse_sub_node_and_print(
                printer,
                stream,
                VyperNodePrinterFactory::default(),
                target_node,
            );
            write_token(printer, stream, ":");
            write_space(printer, stream);

            let mut close_parens_needed = 0;
            if let Some(public) = node.get_bool_for_key("is_public") {
                if public {
                    close_parens_needed += 1;
                    write_token(printer, stream, "public");
                    write_token(printer, stream, "(");
                }
            }

            if let Some(constant) = node.get_bool_for_key("is_constant") {
                if constant {
                    close_parens_needed += 1;
                    write_token(printer, stream, "constant");
                    write_token(printer, stream, "(");
                }
            }

            if let Some(immutable) = node.get_bool_for_key("is_immutable") {
                if immutable {
                    close_parens_needed += 1;
                    write_token(printer, stream, "immutable");
                    write_token(printer, stream, "(");
                }
            }

            if let Some(annotation_node) = node.borrow_value_for_key("annotation") {
                let mut printer_settings = VyperPrettyPrinterSettings::default();
                printer_settings.tuples_should_use_parentheses = true;
                traverse_sub_node_and_print(
                    printer,
                    stream,
                    VyperNodePrinterFactory::new(printer_settings),
                    annotation_node,
                );
            }

            for _ in 0..close_parens_needed {
                write_token(printer, stream, ")");
            }

            write_value_assignment(printer, stream, node);
        }
    }
}

struct SubscriptPrinter {}

impl<W: Write> NodePrinter<W, VyperAST> for SubscriptPrinter {
    fn print_node(&mut self, stream: &mut W, node: &VyperAST, printer: &mut PrettyPrinter) {
        if let Some(value_node) = node.borrow_value_for_key("value") {
            traverse_sub_node_and_print(
                printer,
                stream,
                VyperNodePrinterFactory::default(),
                value_node,
            );
        }

        if let Some(slice_node) = node.borrow_value_for_key("slice") {
            traverse_sub_node_and_print(
                printer,
                stream,
                VyperNodePrinterFactory::default(),
                slice_node,
            );
        }
    }
}

struct IndexPrinter {}

impl<W: Write> NodePrinter<W, VyperAST> for IndexPrinter {
    fn on_entry(&mut self, stream: &mut W, _node: &VyperAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, "[");
    }

    fn print_node(&mut self, stream: &mut W, node: &VyperAST, printer: &mut PrettyPrinter) {
        write_simple_value(printer, stream, node);
    }

    fn on_exit(&mut self, stream: &mut W, _node: &VyperAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, "]");
    }
}

struct TuplePrinter {
    pub use_parentheses: bool,
}

impl<W: Write> NodePrinter<W, VyperAST> for TuplePrinter {
    fn print_node(&mut self, stream: &mut W, node: &VyperAST, printer: &mut PrettyPrinter) {
        if self.use_parentheses {
            write_token(printer, stream, "(");
        }
        write_elements_array(printer, stream, node);
        if self.use_parentheses {
            write_token(printer, stream, ")");
        }
    }
}

struct FunctionDefPrinter {
    has_decorators: bool,
    interface_decl_form: bool,
}

impl FunctionDefPrinter {
    fn new(interface_decl_form: bool) -> FunctionDefPrinter {
        FunctionDefPrinter {
            has_decorators: false,
            interface_decl_form,
        }
    }
}

impl<W: Write> NodePrinter<W, VyperAST> for FunctionDefPrinter {
    fn on_entry(&mut self, stream: &mut W, node: &VyperAST, printer: &mut PrettyPrinter) {
        // Process the decorator list.
        if let Some(decorator_list_node) = node.borrow_value_for_key("decorator_list") {
            if let Some(decorator_array) = decorator_list_node.as_array() {
                if decorator_array.len() >= 1 {
                    self.has_decorators = true;
                    write_token(printer, stream, "@");
                    let value = &decorator_array[0];
                    traverse_sub_node_and_print(
                        printer,
                        stream,
                        VyperNodePrinterFactory::default(),
                        value,
                    );
                    write_newline(printer, stream);
                }
                if decorator_array.len() >= 2 {
                    let mut index = 1;
                    while index < decorator_array.len() {
                        let value = &decorator_array[index];
                        write_indent(printer, stream);
                        write_token(printer, stream, "@");
                        traverse_sub_node_and_print(
                            printer,
                            stream,
                            VyperNodePrinterFactory::default(),
                            value,
                        );
                        write_newline(printer, stream);
                        index += 1;
                    }
                }
            }
        }
    }

    fn print_node(&mut self, stream: &mut W, node: &VyperAST, printer: &mut PrettyPrinter) {
        if self.has_decorators {
            write_indent(printer, stream);
        }
        write_token(printer, stream, "def");
        write_space(printer, stream);
        if let Some(name_str) = node.get_str_for_key("name") {
            write_token(printer, stream, name_str);
        }
        write_token(printer, stream, "(");
        if let Some(args_node) = node.borrow_value_for_key("args") {
            traverse_sub_node_and_print(
                printer,
                stream,
                VyperNodePrinterFactory::default(),
                args_node,
            );
        }
        write_token(printer, stream, ")");
        if let Some(returns_node) = node.borrow_value_for_key("returns") {
            if !returns_node.is_null() {
                let mut printer_settings = VyperPrettyPrinterSettings::default();
                printer_settings.tuples_should_use_parentheses = true;
                write_space(printer, stream);
                write_token(printer, stream, "->");
                write_space(printer, stream);
                traverse_sub_node_and_print(
                    printer,
                    stream,
                    VyperNodePrinterFactory::new(printer_settings),
                    returns_node,
                );
            }
        }
        write_token(printer, stream, ":");
        if let Some(doc_string_node) = node.borrow_value_for_key("doc_string") {
            if !doc_string_node.is_null() {
                printer.increase_indent();
                if let Some(doc_string) = doc_string_node.get_str_for_key("value") {
                    write_newline(printer, stream);
                    write_indent(printer, stream);
                    let doc_string_text = String::from(doc_string);
                    if let Some(_newline_index) = doc_string_text.find("\n") {
                        write_triple_string(printer, stream, doc_string);
                    } else {
                        write_token(printer, stream, "\"\"\"");
                        write_flowable_text(printer, stream, doc_string, "");
                        write_token(printer, stream, "\"\"\"");
                    }
                    write_newline(printer, stream);
                }
                printer.decrease_indent();
            }
        }

        if self.interface_decl_form {
            write_space(printer, stream);
            write_key_of_node_as_array(printer, stream, "body", node);
        } else if let Some(body_node) = node.borrow_value_for_key("body") {
            if let Some(body_array) = body_node.as_array() {
                write_newline(printer, stream);
                printer.increase_indent();
                write_indented_array(printer, stream, body_array);
                printer.decrease_indent();
            }
        }
    }
}

struct ArgumentsPrinter {}

impl<W: Write> NodePrinter<W, VyperAST> for ArgumentsPrinter {
    fn print_node(&mut self, stream: &mut W, node: &VyperAST, printer: &mut PrettyPrinter) {
        // Writing the function arguments is a little complicated because of the way the Vyper AST
        // represents default argument values.  The default values live in the node's 'defaults'
        // key.  This key contains an array of default values. Since Vyper function defs can
        // only have default values for the last arguments to the function, the array of defaults
        // array of say length N apply only to the last N arguments of the function.
        if let Some(defaults_node) = node.borrow_value_for_key("defaults") {
            if let Some(defaults_array) = defaults_node.as_array() {
                if defaults_array.len() == 0 {
                    write_args_as_array(printer, stream, node);
                } else {
                    if let Some(args_node) = node.borrow_value_for_key("args") {
                        if let Some(args_array) = args_node.as_array() {
                            let first_default_index = args_array.len() - defaults_array.len();
                            let mut i: usize = 0;
                            while i < args_array.len() {
                                let arg = args_array.get(i).unwrap();
                                traverse_sub_node_and_print(
                                    printer,
                                    stream,
                                    VyperNodePrinterFactory::default(),
                                    arg,
                                );
                                if i >= first_default_index {
                                    let default_index = i - first_default_index;
                                    let default_node = defaults_array.get(default_index).unwrap();
                                    write_token(printer, stream, "=");
                                    traverse_sub_node_and_print(
                                        printer,
                                        stream,
                                        VyperNodePrinterFactory::default(),
                                        default_node,
                                    );
                                }
                                if i < (args_array.len() - 1) {
                                    write_token(printer, stream, ",");
                                    write_space(printer, stream);
                                }
                                i += 1;
                            }
                        }
                    }
                }
            }
        }
    }
}

struct ArgPrinter {}

impl<W: Write> NodePrinter<W, VyperAST> for ArgPrinter {
    fn print_node(&mut self, stream: &mut W, node: &VyperAST, printer: &mut PrettyPrinter) {
        if let Some(arg_str) = node.get_str_for_key("arg") {
            write_token(printer, stream, arg_str);
        }

        if let Some(annotation_node) = node.borrow_value_for_key("annotation") {
            if !annotation_node.is_null() {
                write_token(printer, stream, ":");
                write_space(printer, stream);
                traverse_sub_node_and_print(
                    printer,
                    stream,
                    VyperNodePrinterFactory::default(),
                    annotation_node,
                );
            }
        }
    }
}

struct AssignPrinter {}

impl<W: Write> NodePrinter<W, VyperAST> for AssignPrinter {
    fn print_node(&mut self, stream: &mut W, node: &VyperAST, printer: &mut PrettyPrinter) {
        if let Some(target_node) = node.borrow_value_for_key("target") {
            traverse_sub_node_and_print(
                printer,
                stream,
                VyperNodePrinterFactory::default(),
                target_node,
            );
        }
        write_token(printer, stream, " = ");
        if let Some(value_node) = node.borrow_value_for_key("value") {
            traverse_sub_node_and_print(
                printer,
                stream,
                VyperNodePrinterFactory::default(),
                value_node,
            );
        }
    }
}

struct AttributePrinter {}

impl<W: Write> NodePrinter<W, VyperAST> for AttributePrinter {
    fn print_node(&mut self, stream: &mut W, node: &VyperAST, printer: &mut PrettyPrinter) {
        if let Some(value_node) = node.borrow_value_for_key("value") {
            traverse_sub_node_and_print(
                printer,
                stream,
                VyperNodePrinterFactory::default(),
                value_node,
            );
        }
        write_token(printer, stream, ".");
        if let Some(attribute_str) = node.get_str_for_key("attr") {
            write_token(printer, stream, attribute_str);
        }
    }
}

struct AugAssignPrinter {
    operator_map: HashMap<&'static str, &'static str>,
}

impl AugAssignPrinter {
    fn new() -> AugAssignPrinter {
        AugAssignPrinter {
            operator_map: get_python_operator_map(),
        }
    }
}

impl<W: Write> NodePrinter<W, VyperAST> for AugAssignPrinter {
    fn print_node(&mut self, stream: &mut W, node: &VyperAST, printer: &mut PrettyPrinter) {
        if let Some(target_node) = node.borrow_value_for_key("target") {
            traverse_sub_node_and_print(
                printer,
                stream,
                VyperNodePrinterFactory::default(),
                target_node,
            );
        }
        write_space(printer, stream);
        if let Some(op_node) = node.borrow_value_for_key("op") {
            if let Some(ast_type_str) = op_node.get_str_for_key("ast_type") {
                if self.operator_map.contains_key(ast_type_str) {
                    write_token(printer, stream, &self.operator_map[ast_type_str]);
                }
            }
        }
        write_token(printer, stream, "=");
        write_space(printer, stream);
        if let Some(value_node) = node.borrow_value_for_key("value") {
            traverse_sub_node_and_print(
                printer,
                stream,
                VyperNodePrinterFactory::default(),
                value_node,
            );
        }
    }
}

struct IntPrinter {}

impl<W: Write> NodePrinter<W, VyperAST> for IntPrinter {
    fn print_node(&mut self, stream: &mut W, node: &VyperAST, printer: &mut PrettyPrinter) {
        if let Some(value) = node.get_int_for_key("value") {
            write_token(printer, stream, &value.to_string());
        } else if let Some(value) = node.get_float_for_key("value") {
            write_token(printer, stream, &value.to_string());
        }
    }
}

struct DecimalPrinter {}

impl<W: Write> NodePrinter<W, VyperAST> for DecimalPrinter {
    fn print_node(&mut self, stream: &mut W, node: &VyperAST, printer: &mut PrettyPrinter) {
        write_value_string_as_token(printer, stream, node);
    }
}

struct BinOpPrinter {
    operator_map: HashMap<&'static str, &'static str>,
}

impl BinOpPrinter {
    fn new() -> BinOpPrinter {
        BinOpPrinter {
            operator_map: get_python_operator_map(),
        }
    }
}

impl<W: Write> NodePrinter<W, VyperAST> for BinOpPrinter {
    fn print_node(&mut self, stream: &mut W, node: &VyperAST, printer: &mut PrettyPrinter) {
        if let Some(left_node) = node.borrow_value_for_key("left") {
            write_node_with_parens_maybe(printer, stream, left_node);
        }
        if let Some(op_node) = node.borrow_value_for_key("op") {
            if let Some(ast_type_str) = op_node.get_str_for_key("ast_type") {
                if self.operator_map.contains_key(ast_type_str) {
                    write_space(printer, stream);
                    write_token(printer, stream, &self.operator_map[ast_type_str]);
                    write_space(printer, stream);
                }
            }
        }
        if let Some(right_node) = node.borrow_value_for_key("right") {
            write_node_with_parens_maybe(printer, stream, right_node);
        }
    }
}

struct BoolOpPrinter {}

impl<W: Write> NodePrinter<W, VyperAST> for BoolOpPrinter {
    fn print_node(&mut self, stream: &mut W, node: &VyperAST, printer: &mut PrettyPrinter) {
        if let Some(values_node) = node.borrow_value_for_key("values") {
            if let Some(values_array) = values_node.as_array() {
                if values_array.len() < 2 {
                    return;
                }

                let left = &values_array[0];
                let right = &values_array[1];

                write_node_with_parens_maybe(printer, stream, left);
                write_space(printer, stream);

                if let Some(op_node) = node.borrow_value_for_key("op") {
                    traverse_sub_node_and_print(
                        printer,
                        stream,
                        VyperNodePrinterFactory::default(),
                        op_node,
                    );
                }

                write_space(printer, stream);
                write_node_with_parens_maybe(printer, stream, right);
            }
        }
    }
}

struct AssertPrinter {}

impl<W: Write> NodePrinter<W, VyperAST> for AssertPrinter {
    fn print_node(&mut self, stream: &mut W, node: &VyperAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, "assert");
        write_space(printer, stream);
        if let Some(test_node) = node.borrow_value_for_key("test") {
            traverse_sub_node_and_print(
                printer,
                stream,
                VyperNodePrinterFactory::default(),
                test_node,
            );
        }
        if let Some(msg_node) = node.borrow_value_for_key("msg") {
            if msg_node.is_object() {
                write_token(printer, stream, ",");
                write_space(printer, stream);
                traverse_sub_node_and_print(
                    printer,
                    stream,
                    VyperNodePrinterFactory::default(),
                    msg_node,
                );
            }
        }
    }
}

struct StrPrinter {}

impl<W: Write> NodePrinter<W, VyperAST> for StrPrinter {
    fn print_node(&mut self, stream: &mut W, node: &VyperAST, printer: &mut PrettyPrinter) {
        if let Some(value_str) = node.get_str_for_key("value") {
            write_string(printer, stream, value_str);
        }
    }
}

struct ModulePrinter {}

impl<W: Write> NodePrinter<W, VyperAST> for ModulePrinter {
    fn print_node(&mut self, stream: &mut W, node: &VyperAST, printer: &mut PrettyPrinter) {
        if let Some(doc_string_node) = node.borrow_value_for_key("doc_string") {
            if let Some(doc_string) = doc_string_node.get_str_for_key("value") {
                let doc_text = String::from(doc_string);
                if let Some(_newline_index) = doc_text.find("\n") {
                    write_triple_string(printer, stream, doc_string)
                } else {
                    write_token(printer, stream, "\"\"\"");
                    write_flowable_text(printer, stream, doc_string, "");
                    write_token(printer, stream, "\"\"\"");
                }
                write_newline(printer, stream);
                write_newline(printer, stream);
            }
        }
        if let Some(body_node) = node.borrow_value_for_key("body") {
            if let Some(body_array) = body_node.as_array() {
                for value in body_array {
                    traverse_sub_node_and_print(
                        printer,
                        stream,
                        VyperNodePrinterFactory::default(),
                        value,
                    );
                    write_newline(printer, stream);
                    write_newline(printer, stream);
                }
            }
        }
    }
}

struct AnnAssignPrinter {}

impl<W: Write> NodePrinter<W, VyperAST> for AnnAssignPrinter {
    fn print_node(&mut self, stream: &mut W, node: &VyperAST, printer: &mut PrettyPrinter) {
        if let Some(target_node) = node.borrow_value_for_key("target") {
            traverse_sub_node_and_print(
                printer,
                stream,
                VyperNodePrinterFactory::default(),
                target_node,
            );
        }
        if let Some(annotation_node) = node.borrow_value_for_key("annotation") {
            write_token(printer, stream, ":");
            write_space(printer, stream);
            traverse_sub_node_and_print(
                printer,
                stream,
                VyperNodePrinterFactory::default(),
                annotation_node,
            );
        }
        write_value_assignment(printer, stream, node);
    }
}

struct ExprPrinter {}

impl<W: Write> NodePrinter<W, VyperAST> for ExprPrinter {
    fn print_node(&mut self, stream: &mut W, node: &VyperAST, printer: &mut PrettyPrinter) {
        if let Some(value_node) = node.borrow_value_for_key("value") {
            traverse_sub_node_and_print(
                printer,
                stream,
                VyperNodePrinterFactory::default(),
                value_node,
            );
        }
    }
}

struct CallPrinter {}

impl<W: Write> NodePrinter<W, VyperAST> for CallPrinter {
    fn print_node(&mut self, stream: &mut W, node: &VyperAST, printer: &mut PrettyPrinter) {
        if let Some(func_node) = node.borrow_value_for_key("func") {
            traverse_sub_node_and_print(
                printer,
                stream,
                VyperNodePrinterFactory::default(),
                func_node,
            );
        }
        write_token(printer, stream, "(");
        write_args_as_array(printer, stream, node);
        if let Some(keywords_node) = node.borrow_value_for_key("keywords") {
            if let Some(keywords_array) = keywords_node.as_array() {
                if keywords_array.len() > 0 {
                    write_token(printer, stream, ",");
                    write_space(printer, stream);
                    write_key_of_node_as_array(printer, stream, "keywords", node);
                }
            }
        }
        write_token(printer, stream, ")");
    }
}

struct NameConstantPrinter {}

impl<W: Write> NodePrinter<W, VyperAST> for NameConstantPrinter {
    fn print_node(&mut self, stream: &mut W, node: &VyperAST, printer: &mut PrettyPrinter) {
        if let Some(value_node) = node.borrow_value_for_key("value") {
            if value_node.is_boolean() {
                let value_bool = value_node.as_bool().unwrap();
                if value_bool {
                    write_token(printer, stream, "True");
                } else {
                    write_token(printer, stream, "False");
                }
            } else if value_node.is_null() {
                write_token(printer, stream, "None");
            }
        }
    }
}

struct UnaryOpPrinter {}

impl<W: Write> NodePrinter<W, VyperAST> for UnaryOpPrinter {
    fn print_node(&mut self, stream: &mut W, node: &VyperAST, printer: &mut PrettyPrinter) {
        let mut needs_space = false;
        if let Some(op_node) = node.borrow_value_for_key("op") {
            if let Some(op_str) = op_node.get_str_for_key("ast_type") {
                if op_str == "Not" {
                    write_token(printer, stream, "not");
                    needs_space = true;
                } else if op_str == "Invert" {
                    write_token(printer, stream, "~");
                }
            }
        }
        if let Some(operand_node) = node.borrow_value_for_key("operand") {
            if needs_space {
                write_space(printer, stream);
            }
            write_node_with_parens_maybe(printer, stream, operand_node);
        }
    }
}

struct InterfaceDefPrinter {}

impl<W: Write> NodePrinter<W, VyperAST> for InterfaceDefPrinter {
    fn on_entry(&mut self, stream: &mut W, node: &VyperAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, "interface");
        write_space(printer, stream);
        if let Some(name_str) = node.get_str_for_key("name") {
            write_token(printer, stream, name_str);
            write_token(printer, stream, ":");
            write_newline(printer, stream);
        }
        printer.increase_indent();
    }

    fn print_node(&mut self, stream: &mut W, node: &VyperAST, printer: &mut PrettyPrinter) {
        let mut printer_settings = VyperPrettyPrinterSettings::default();
        printer_settings.function_def_uses_struct_decl_form = true;
        write_body_as_nodes(printer, stream, node, printer_settings);
    }

    fn on_exit(&mut self, _stream: &mut W, _node: &VyperAST, printer: &mut PrettyPrinter) {
        printer.decrease_indent();
    }
}

struct StructDefPrinter {}

impl<W: Write> NodePrinter<W, VyperAST> for StructDefPrinter {
    fn on_entry(&mut self, stream: &mut W, node: &VyperAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, "struct");
        write_space(printer, stream);
        if let Some(name_str) = node.get_str_for_key("name") {
            write_token(printer, stream, name_str);
            write_token(printer, stream, ":");
            write_newline(printer, stream);
        }
        printer.increase_indent();
    }

    fn print_node(&mut self, stream: &mut W, node: &VyperAST, printer: &mut PrettyPrinter) {
        write_body_as_nodes(printer, stream, node, VyperPrettyPrinterSettings::default());
    }

    fn on_exit(&mut self, _stream: &mut W, _node: &VyperAST, printer: &mut PrettyPrinter) {
        printer.decrease_indent();
    }
}

struct ForPrinter {}

impl<W: Write> NodePrinter<W, VyperAST> for ForPrinter {
    fn print_node(&mut self, stream: &mut W, node: &VyperAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, "for");
        write_space(printer, stream);
        if let Some(target_node) = node.borrow_value_for_key("target") {
            traverse_sub_node_and_print(
                printer,
                stream,
                VyperNodePrinterFactory::default(),
                target_node,
            );
        }
        write_space(printer, stream);
        write_token(printer, stream, "in");
        write_space(printer, stream);
        if let Some(iter_node) = node.borrow_value_for_key("iter") {
            traverse_sub_node_and_print(
                printer,
                stream,
                VyperNodePrinterFactory::default(),
                iter_node,
            );
        }
        write_token(printer, stream, ":");
        write_newline(printer, stream);
        printer.increase_indent();
        write_body_as_nodes(printer, stream, node, VyperPrettyPrinterSettings::default());
        printer.decrease_indent();
    }
}

struct ListPrinter {}

impl<W: Write> NodePrinter<W, VyperAST> for ListPrinter {
    fn on_entry(&mut self, stream: &mut W, _node: &VyperAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, "[");
    }

    fn print_node(&mut self, stream: &mut W, node: &VyperAST, printer: &mut PrettyPrinter) {
        write_elements_array(printer, stream, node);
    }

    fn on_exit(&mut self, stream: &mut W, _node: &VyperAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, "]");
    }
}

struct IfPrinter {}

impl<W: Write> NodePrinter<W, VyperAST> for IfPrinter {
    fn print_node(&mut self, stream: &mut W, node: &VyperAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, "if");
        write_space(printer, stream);
        if let Some(test_node) = node.borrow_value_for_key("test") {
            traverse_sub_node_and_print(
                printer,
                stream,
                VyperNodePrinterFactory::default(),
                test_node,
            );
        }
        write_token(printer, stream, ":");
        write_newline(printer, stream);
        printer.increase_indent();
        write_indented_body_array(printer, stream, node);
        printer.decrease_indent();
        if let Some(orelse_node) = node.borrow_value_for_key("orelse") {
            if let Some(orelse_array) = orelse_node.as_array() {
                if orelse_array.len() > 0 {
                    write_newline(printer, stream);
                    write_indent(printer, stream);
                    write_token(printer, stream, "else:");
                    write_newline(printer, stream);
                    printer.increase_indent();
                    write_indented_array(printer, stream, orelse_array);
                    printer.decrease_indent();
                }
            }
        }
    }
}

struct BreakPrinter {}

impl<W: Write> NodePrinter<W, VyperAST> for BreakPrinter {
    fn print_node(&mut self, stream: &mut W, _node: &VyperAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, "break");
    }
}

struct ContinuePrinter {}

impl<W: Write> NodePrinter<W, VyperAST> for ContinuePrinter {
    fn print_node(&mut self, stream: &mut W, _node: &VyperAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, "continue")
    }
}

struct PassPrinter {}

impl<W: Write> NodePrinter<W, VyperAST> for PassPrinter {
    fn print_node(&mut self, stream: &mut W, _node: &VyperAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, "pass");
    }
}

struct ReturnPrinter {}

impl<W: Write> NodePrinter<W, VyperAST> for ReturnPrinter {
    fn print_node(&mut self, stream: &mut W, node: &VyperAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, "return");
        if let Some(value_node) = node.borrow_value_for_key("value") {
            if !value_node.is_null() {
                write_space(printer, stream);
                traverse_sub_node_and_print(
                    printer,
                    stream,
                    VyperNodePrinterFactory::default(),
                    value_node,
                );
            }
        }
    }
}

struct RaisePrinter {}

impl<W: Write> NodePrinter<W, VyperAST> for RaisePrinter {
    fn print_node(&mut self, stream: &mut W, node: &VyperAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, "raise");
        write_space(printer, stream);
        if let Some(exc_node) = node.borrow_value_for_key("exc") {
            traverse_sub_node_and_print(
                printer,
                stream,
                VyperNodePrinterFactory::default(),
                exc_node,
            );
        }
    }
}

struct ImportPrinter {}

impl<W: Write> NodePrinter<W, VyperAST> for ImportPrinter {
    fn print_node(&mut self, stream: &mut W, node: &VyperAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, "import");
        write_space(printer, stream);
        if let Some(name_str) = node.get_str_for_key("name") {
            write_token(printer, stream, name_str);
        }
        if let Some(alias_str) = node.get_str_for_key("alias") {
            write_space(printer, stream);
            write_token(printer, stream, "as");
            write_space(printer, stream);
            write_token(printer, stream, alias_str);
        }
    }
}

struct ImportFromPrinter {}

impl<W: Write> NodePrinter<W, VyperAST> for ImportFromPrinter {
    fn print_node(&mut self, stream: &mut W, node: &VyperAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, "from");
        write_space(printer, stream);
        if let Some(module_node) = node.borrow_value_for_key("module") {
            if !module_node.is_null() {
                if let Some(module_str) = module_node.as_str() {
                    write_token(printer, stream, module_str);
                }
            } else {
                write_token(printer, stream, ".");
            }
            write_space(printer, stream);
        }
        write_token(printer, stream, "import");
        write_space(printer, stream);
        if let Some(name_str) = node.get_str_for_key("name") {
            write_token(printer, stream, name_str);
        }
        if let Some(alias_node) = node.borrow_value_for_key("alias") {
            if !node.is_null() {
                if let Some(alias_str) = alias_node.as_str() {
                    write_space(printer, stream);
                    write_token(printer, stream, "as");
                    write_space(printer, stream);
                    write_token(printer, stream, alias_str);
                }
            }
        }
    }
}

struct EventDefPrinter {}

impl<W: Write> NodePrinter<W, VyperAST> for EventDefPrinter {
    fn print_node(&mut self, stream: &mut W, node: &VyperAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, "event");
        write_space(printer, stream);
        if let Some(name_str) = node.get_str_for_key("name") {
            write_token(printer, stream, name_str);
            write_token(printer, stream, ":");
        }
        write_newline(printer, stream);
        printer.increase_indent();
        write_indented_body_array(printer, stream, node);
        printer.decrease_indent();
    }
}

struct LogPrinter {}

impl<W: Write> NodePrinter<W, VyperAST> for LogPrinter {
    fn print_node(&mut self, stream: &mut W, node: &VyperAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, "log");
        write_space(printer, stream);
        if let Some(value_node) = node.borrow_value_for_key("value") {
            if !value_node.is_null() {
                traverse_sub_node_and_print(
                    printer,
                    stream,
                    VyperNodePrinterFactory::default(),
                    value_node,
                );
            }
        }
    }
}

/// Structure for handling a Comment node.
///
/// **Important**: Note that Comment nodes are not part of the Vyper AST.  We add them in a mutation
/// algorithm.  They are artificial and any AST that has a Comment node no longer conforms to the
/// Vyper AST standard (good luck finding that thought).
struct CommentPrinter {}

impl<W: Write> NodePrinter<W, VyperAST> for CommentPrinter {
    fn print_node(&mut self, stream: &mut W, node: &VyperAST, printer: &mut PrettyPrinter) {
        if let Some(value_str) = node.get_str_for_key("value") {
            if value_str.len() > 0 {
                write_token(printer, stream, "#");
                write_space(printer, stream);
                write_flowable_text(printer, stream, value_str, "# ");
            }
        }
    }
}

struct AndPrinter {}

impl<W: Write> NodePrinter<W, VyperAST> for AndPrinter {
    fn print_node(&mut self, stream: &mut W, node: &VyperAST, printer: &mut PrettyPrinter) {
        if let Some(ast_type) = node.get_str_for_key("ast_type") {
            if ast_type == "And" {
                write_token(printer, stream, "and");
            } else if ast_type == "Or" {
                write_token(printer, stream, "or");
            }
        }
    }
}

struct KeywordPrinter {}

impl<W: Write> NodePrinter<W, VyperAST> for KeywordPrinter {
    fn print_node(&mut self, stream: &mut W, node: &VyperAST, printer: &mut PrettyPrinter) {
        if let Some(arg_str) = node.get_str_for_key("arg") {
            write_token(printer, stream, arg_str);
            write_token(printer, stream, "=");
            if let Some(value_node) = node.borrow_value_for_key("value") {
                traverse_sub_node_and_print(
                    printer,
                    stream,
                    VyperNodePrinterFactory::default(),
                    value_node,
                );
            }
        }
    }
}

struct BytesPrinter {}

impl<W: Write> NodePrinter<W, VyperAST> for BytesPrinter {
    fn print_node(&mut self, stream: &mut W, node: &VyperAST, printer: &mut PrettyPrinter) {
        if let Some(value_str) = node.get_str_for_key("value") {
            let mut byte_string: String = String::new();
            let mut index: usize = 2;
            byte_string += "b'";
            while index < value_str.len() {
                let substr = &value_str[index..index + 2];
                byte_string += "\\x";
                byte_string += substr;
                index += 2;
            }
            byte_string += "'";
            write_token(printer, stream, &byte_string);
        }
    }
}

struct DictPrinter {}

impl<W: Write> NodePrinter<W, VyperAST> for DictPrinter {
    fn on_entry(&mut self, stream: &mut W, _node: &VyperAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, "{");
    }

    fn print_node(&mut self, stream: &mut W, node: &VyperAST, printer: &mut PrettyPrinter) {
        if let Some(keys_node) = node.borrow_value_for_key("keys") {
            if let Some(keys_array) = keys_node.as_array() {
                if let Some(values_node) = node.borrow_value_for_key("values") {
                    if let Some(values_array) = values_node.as_array() {
                        assert_eq!(keys_array.len(), values_array.len());

                        if keys_array.len() > 0 {
                            write_newline(printer, stream);
                            printer.increase_indent();
                            let mut i: usize = 0;
                            while i < keys_array.len() {
                                let key = keys_array.get(i).unwrap();
                                let value = values_array.get(i).unwrap();

                                write_indent(printer, stream);
                                traverse_sub_node_and_print(
                                    printer,
                                    stream,
                                    VyperNodePrinterFactory::default(),
                                    key,
                                );
                                write_token(printer, stream, ":");
                                write_space(printer, stream);
                                traverse_sub_node_and_print(
                                    printer,
                                    stream,
                                    VyperNodePrinterFactory::default(),
                                    value,
                                );

                                if i < (keys_array.len() - 1) {
                                    write_token(printer, stream, ",");
                                    write_newline(printer, stream);
                                }
                                i += 1;
                            }
                            printer.decrease_indent();
                            write_newline(printer, stream);
                            write_indent(printer, stream);
                        }
                    }
                }
            }
        }
    }

    fn on_exit(&mut self, stream: &mut W, _node: &VyperAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, "}");
    }
}

struct HexPrinter {}

impl<W: Write> NodePrinter<W, VyperAST> for HexPrinter {
    fn print_node(&mut self, stream: &mut W, node: &VyperAST, printer: &mut PrettyPrinter) {
        write_value_string_as_token(printer, stream, node);
    }
}

struct EnumDefPrinter {}

impl<W: Write> NodePrinter<W, VyperAST> for EnumDefPrinter {
    fn print_node(&mut self, stream: &mut W, node: &VyperAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, "enum");
        write_space(printer, stream);

        if let Some(name_str) = node.get_str_for_key("name") {
            write_token(printer, stream, name_str);
            write_token(printer, stream, ":");
        }

        write_newline(printer, stream);
        printer.increase_indent();
        write_indented_body_array(printer, stream, node);
        printer.decrease_indent();
    }
}

/// Type that records the preferences for pretty-printing Vyper programs.
#[derive(Clone, Copy)]
pub struct VyperPrettyPrinterSettings {
    /// True if the printer should print tuple expressions enclosed in parentheses.
    pub tuples_should_use_parentheses: bool,

    /// True if the printer should write a function definition in the form:
    /// def name(args,): mutability
    ///
    /// The printer uses this setting when printing a function declaration inside an
    /// interface decl.
    pub function_def_uses_struct_decl_form: bool,
}

impl VyperPrettyPrinterSettings {
    /// Create a default settings object
    pub fn default() -> VyperPrettyPrinterSettings {
        VyperPrettyPrinterSettings {
            tuples_should_use_parentheses: false,
            function_def_uses_struct_decl_form: false,
        }
    }
}

/// Type that implements [`NodePrinterFactory<W,AST>`] for Vyper AST nodes.
///
/// Use this factory object with the [`crate::pretty_print_visitor::PrettyPrintVisitor<W,AST>`] object.
#[derive(Clone)]
pub struct VyperNodePrinterFactory {
    settings: VyperPrettyPrinterSettings,
}

impl VyperNodePrinterFactory {
    /// Create a new node printer factory.
    ///
    /// # Arguments
    ///
    /// * `tuples_use_parens` - True if the factory should generate tuple printers that write
    /// parentheses around the tuple.
    pub fn new(settings: VyperPrettyPrinterSettings) -> VyperNodePrinterFactory {
        VyperNodePrinterFactory { settings }
    }

    /// Create a new node printer factory with default settings.
    pub fn default() -> VyperNodePrinterFactory {
        VyperNodePrinterFactory {
            settings: VyperPrettyPrinterSettings::default(),
        }
    }
}

impl<W: Write> NodePrinterFactory<W, VyperAST> for VyperNodePrinterFactory {
    fn printer_for(&self, node: &VyperAST) -> Box<dyn NodePrinter<W, VyperAST>> {
        if let Some(ast_type) = node.get_str_for_key("ast_type") {
            match ast_type {
                "Name" => Box::new(NamePrinter {}),
                "VariableDecl" => Box::new(VariableDeclPrinter {}),
                "Subscript" => Box::new(SubscriptPrinter {}),
                "Index" => Box::new(IndexPrinter {}),
                "Tuple" => Box::new(TuplePrinter {
                    use_parentheses: self.settings.tuples_should_use_parentheses,
                }),
                "FunctionDef" => Box::new(FunctionDefPrinter::new(
                    self.settings.function_def_uses_struct_decl_form,
                )),
                "arguments" => Box::new(ArgumentsPrinter {}),
                "arg" => Box::new(ArgPrinter {}),
                "Assign" => Box::new(AssignPrinter {}),
                "Attribute" => Box::new(AttributePrinter {}),
                "AugAssign" => Box::new(AugAssignPrinter::new()),
                "Int" => Box::new(IntPrinter {}),
                "Decimal" => Box::new(DecimalPrinter {}),
                "BinOp" => Box::new(BinOpPrinter::new()),
                "BoolOp" => Box::new(BoolOpPrinter {}),
                "Compare" => Box::new(BinOpPrinter::new()),
                "Assert" => Box::new(AssertPrinter {}),
                "Str" => Box::new(StrPrinter {}),
                "Module" => Box::new(ModulePrinter {}),
                "AnnAssign" => Box::new(AnnAssignPrinter {}),
                "Expr" => Box::new(ExprPrinter {}),
                "Call" => Box::new(CallPrinter {}),
                "NameConstant" => Box::new(NameConstantPrinter {}),
                "UnaryOp" => Box::new(UnaryOpPrinter {}),
                "InterfaceDef" => Box::new(InterfaceDefPrinter {}),
                "StructDef" => Box::new(StructDefPrinter {}),
                "For" => Box::new(ForPrinter {}),
                "List" => Box::new(ListPrinter {}),
                "If" => Box::new(IfPrinter {}),
                "Break" => Box::new(BreakPrinter {}),
                "Continue" => Box::new(ContinuePrinter {}),
                "Pass" => Box::new(PassPrinter {}),
                "Return" => Box::new(ReturnPrinter {}),
                "Raise" => Box::new(RaisePrinter {}),
                "Import" => Box::new(ImportPrinter {}),
                "ImportFrom" => Box::new(ImportFromPrinter {}),
                "EventDef" => Box::new(EventDefPrinter {}),
                "Log" => Box::new(LogPrinter {}),
                "Comment" => Box::new(CommentPrinter {}),
                "And" => Box::new(AndPrinter {}),
                "Or" => Box::new(AndPrinter {}),
                "keyword" => Box::new(KeywordPrinter {}),
                "Bytes" => Box::new(BytesPrinter {}),
                "Dict" => Box::new(DictPrinter {}),
                "Hex" => Box::new(HexPrinter {}),
                "EnumDef" => Box::new(EnumDefPrinter {}),
                _ => Box::new(DummyNodePrinter {}),
            }
        } else {
            Box::new(DummyNodePrinter {})
        }
    }
}
