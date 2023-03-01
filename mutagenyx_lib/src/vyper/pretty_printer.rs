//! The `vyper::pretty_printer` module provides the code to traverse a Vyper AST and print out
//! the source code represented in the AST.

use crate::json::*;
use crate::node_printer::{NodePrinter, NodePrinterFactory};
use crate::node_printer_helpers::*;
use crate::preferences::*;
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
/// * `factory` - The [`NodePrinterFactory<AST>`] object that generates node printers for AST
/// nodes.
/// * `node` - The node in the Vyper AST.
fn write_simple_value(
    printer: &mut PrettyPrinter,
    stream: &mut dyn Write,
    factory: &dyn NodePrinterFactory<VyperAST>,
    node: &VyperAST,
) {
    if let Some(value_node) = node.get("value") {
        traverse_sub_node_and_print(printer, stream, factory, value_node);
    }
}

/// Write the value of the array contained in the `elements` key.
///
/// # Arguments
///
/// * `printer` - The [`PrettyPrinter`] object that will format the output.
/// * `stream` - The [`Write`] object that will receive the formatted text.
/// * `factory` - The [`NodePrinterFactory<AST>`] object that generates node printers for AST
/// nodes.
/// * `node` - The node in the Vyper AST.
fn write_elements_array(
    printer: &mut PrettyPrinter,
    stream: &mut dyn Write,
    factory: &dyn NodePrinterFactory<VyperAST>,
    node: &VyperAST,
) {
    if let Some(elements_node) = node.get("elements") {
        if let Some(elements_array) = elements_node.as_array() {
            print_array_helper(printer, stream, factory, elements_array);
        }
    }
}

/// Write the value of the array contained in `key`.
///
/// # Arguments
///
/// * `printer` - The [`PrettyPrinter`] object that will format the output.
/// * `stream` - The [`Write`] object that will receive the formatted text.
/// * `factory` - The [`NodePrinterFactory<AST>`] object that generates node printers for AST
/// nodes.
/// * `key` - The key string.
/// * `node` - The node in the Vyper AST.
fn write_key_of_node_as_array(
    printer: &mut PrettyPrinter,
    stream: &mut dyn Write,
    factory: &dyn NodePrinterFactory<VyperAST>,
    key: &str,
    node: &VyperAST,
) {
    if let Some(sub_node) = node.get(key) {
        if let Some(sub_array) = sub_node.as_array() {
            print_array_helper(printer, stream, factory, sub_array);
        }
    }
}

/// Write the value of the array contained in the 'args' key.
///
/// # Arguments
///
/// * `printer` - The [`PrettyPrinter`] object that will format the output.
/// * `stream` - The [`Write`] object that will receive the formatted text.
/// * `factory` - The [`NodePrinterFactory<AST>`] object that generates node printers for AST
/// nodes.
/// * `node` - The node in the Vyper AST.
fn write_args_as_array(
    printer: &mut PrettyPrinter,
    stream: &mut dyn Write,
    factory: &dyn NodePrinterFactory<VyperAST>,
    node: &VyperAST,
) {
    write_key_of_node_as_array(printer, stream, factory, "args", node);
}

/// Write the object in the `value` element of `node` with `something = <value>`.
///
/// # Arguments
///
/// * `printer` - The [`PrettyPrinter`] object that will format the output.
/// * `stream` - The [`Write`] object that will receive the formatted text.
/// * `factory` - The [`NodePrinterFactory<AST>`] object that generates node printers for AST
/// nodes.
/// * `node` - The node in the Vyper AST.
fn write_value_assignment(
    printer: &mut PrettyPrinter,
    stream: &mut dyn Write,
    factory: &dyn NodePrinterFactory<VyperAST>,
    node: &VyperAST,
) {
    if let Some(value_node) = node.get("value") {
        if !value_node.is_null() {
            write_token(printer, stream, " = ");
            traverse_sub_node_and_print(printer, stream, factory, value_node);
        }
    }
}

/// Write the object in the `body` element as a sequence of indented text followed by a newline.
///
/// # Arguments
///
/// * `printer` - The [`PrettyPrinter`] object that will format the output.
/// * `stream` - The [`Write`] object that will receive the formatted text.
/// * `factory` - The [`NodePrinterFactory<AST>`] trait object that will generate printers for
/// AST nodes.
/// * `node` - The node in the Vyper AST.
fn write_body_as_nodes(
    printer: &mut PrettyPrinter,
    stream: &mut dyn Write,
    factory: &dyn NodePrinterFactory<VyperAST>,
    node: &VyperAST,
) {
    if let Some(body_node) = node.get("body") {
        if let Some(body_array) = body_node.as_array() {
            for value in body_array {
                write_indent(printer, stream);
                traverse_sub_node_and_print(printer, stream, factory, value);
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
/// * `factory` - The [`NodePrinterFactory<AST>`] object that generates node printers for AST
/// nodes.
/// * `node` - The node in the Vyper AST.
fn write_indented_body_array(
    printer: &mut PrettyPrinter,
    stream: &mut dyn Write,
    factory: &dyn NodePrinterFactory<VyperAST>,
    node: &VyperAST,
) {
    if let Some(body_node) = node.get("body") {
        if let Some(body_array) = body_node.as_array() {
            write_indented_array(printer, stream, factory, body_array);
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
/// * `factory` - The [`NodePrinterFactory<AST>`] object that generates node printers for AST
/// nodes.
/// * `array` - The array of [`VyperAST`] nodes.
fn write_indented_array(
    printer: &mut PrettyPrinter,
    stream: &mut dyn Write,
    factory: &dyn NodePrinterFactory<VyperAST>,
    array: &Vec<VyperAST>,
) {
    let mut i = 0;
    while i < array.len() {
        let value = &array[i];
        write_indent(printer, stream);
        traverse_sub_node_and_print(printer, stream, factory, value);
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
/// * `factory` - The [`NodePrinterFactory<AST>`] object that generates node printers for AST
/// nodes.
/// * `node` - The Vyper AST node.
fn write_node_with_parens_maybe(
    printer: &mut PrettyPrinter,
    stream: &mut dyn Write,
    factory: &dyn NodePrinterFactory<VyperAST>,
    node: &VyperAST,
) {
    if let Some(ast_type) = node.get_str_for_key("ast_type") {
        let needs_parens = ast_type == "BinOp" || ast_type == "Compare" || ast_type == "BoolOp";

        if needs_parens {
            write_token(printer, stream, "(");
        }
        traverse_sub_node_and_print(printer, stream, factory, node);

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
fn write_value_string_as_token(
    printer: &mut PrettyPrinter,
    stream: &mut dyn Write,
    node: &VyperAST,
) {
    if let Some(value_str) = node.get_str_for_key("value") {
        write_token(printer, stream, value_str);
    }
}

/// Return the string containing the pretty-printed form of `node`.
///
/// # Arguments
///
/// * `node` - The node to pretty-print.
/// * `factory` - The node printer factory.
fn pretty_print_node(node: &VyperAST, factory: &dyn NodePrinterFactory<VyperAST>) -> String {
    let mut node_contents = Vec::new();
    let mut printer = PrettyPrinter::new(4, 150);
    traverse_sub_node_and_print(&mut printer, &mut node_contents, factory, node);

    // s now contains the pretty-printed node.
    let s = core::str::from_utf8(node_contents.as_slice()).unwrap();
    String::from(s)
}

struct DummyNodePrinter {}

impl NodePrinter<VyperAST> for DummyNodePrinter {
    fn visit_children(&mut self) -> bool {
        true
    }
}

struct NamePrinter {}

impl NodePrinter<VyperAST> for NamePrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<VyperAST>,
        node: &VyperAST,
    ) {
        if let Some(name) = node.get_str_for_key("id") {
            write_token(printer, stream, name);
        }
    }
}

struct VariableDeclPrinter {}

impl NodePrinter<VyperAST> for VariableDeclPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<VyperAST>,
        node: &VyperAST,
    ) {
        if let Some(target_node) = node.get("target") {
            traverse_sub_node_and_print(printer, stream, factory, target_node);
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

            if let Some(annotation_node) = node.get("annotation") {
                let mut printer_settings = factory.get_settings().clone();
                printer_settings.set_value_for_key(
                    TUPLES_SHOULD_USE_PARENTHESES,
                    PreferenceValue::Boolean(true),
                );
                let sub_factory = VyperNodePrinterFactory::new(printer_settings);
                traverse_sub_node_and_print(printer, stream, &sub_factory, annotation_node);
            }

            for _ in 0..close_parens_needed {
                write_token(printer, stream, ")");
            }

            write_value_assignment(printer, stream, factory, node);
        }
    }
}

struct SubscriptPrinter {}

impl NodePrinter<VyperAST> for SubscriptPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<VyperAST>,
        node: &VyperAST,
    ) {
        if let Some(value_node) = node.get("value") {
            traverse_sub_node_and_print(printer, stream, factory, value_node);
        }

        if let Some(slice_node) = node.get("slice") {
            traverse_sub_node_and_print(printer, stream, factory, slice_node);
        }
    }
}

struct IndexPrinter {}

impl NodePrinter<VyperAST> for IndexPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<VyperAST>,
        node: &VyperAST,
    ) {
        let mut printer_settings = factory.get_settings().clone();
        printer_settings.set_value_for_key(
            TUPLES_SHOULD_USE_PARENTHESES,
            PreferenceValue::Boolean(false),
        );
        let sub_factory = VyperNodePrinterFactory::new(printer_settings);
        write_token(printer, stream, "[");
        write_simple_value(printer, stream, &sub_factory, node);
        write_token(printer, stream, "]");
    }
}

struct TuplePrinter {
    pub use_parentheses: bool,
}

impl NodePrinter<VyperAST> for TuplePrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<VyperAST>,
        node: &VyperAST,
    ) {
        if self.use_parentheses {
            write_token(printer, stream, "(");
        }
        write_elements_array(printer, stream, factory, node);
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

impl NodePrinter<VyperAST> for FunctionDefPrinter {
    fn on_entry(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<VyperAST>,
        node: &VyperAST,
    ) {
        // Process the decorator list.
        if let Some(decorator_list_node) = node.get("decorator_list") {
            if let Some(decorator_array) = decorator_list_node.as_array() {
                if !decorator_array.is_empty() {
                    self.has_decorators = true;
                    write_token(printer, stream, "@");
                    let value = &decorator_array[0];
                    traverse_sub_node_and_print(printer, stream, factory, value);
                    write_newline(printer, stream);
                }
                if decorator_array.len() >= 2 {
                    let mut index = 1;
                    while index < decorator_array.len() {
                        let value = &decorator_array[index];
                        write_indent(printer, stream);
                        write_token(printer, stream, "@");
                        traverse_sub_node_and_print(printer, stream, factory, value);
                        write_newline(printer, stream);
                        index += 1;
                    }
                }
            }
        }
    }

    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<VyperAST>,
        node: &VyperAST,
    ) {
        if self.has_decorators {
            write_indent(printer, stream);
        }
        write_token(printer, stream, "def");
        write_space(printer, stream);
        if let Some(name_str) = node.get_str_for_key("name") {
            write_token(printer, stream, name_str);
        }
        write_token(printer, stream, "(");
        if let Some(args_node) = node.get("args") {
            traverse_sub_node_and_print(printer, stream, factory, args_node);
        }
        write_token(printer, stream, ")");
        if let Some(returns_node) = node.get("returns") {
            if !returns_node.is_null() {
                let mut printer_settings = factory.get_settings().clone();
                printer_settings.set_value_for_key(
                    TUPLES_SHOULD_USE_PARENTHESES,
                    PreferenceValue::Boolean(true),
                );
                // let sub_factory: Box<dyn NodePrinterFactory<VyperAST>> =
                //     Box::new(VyperNodePrinterFactory::new(printer_settings));
                let sub_factory = VyperNodePrinterFactory::new(printer_settings);
                write_space(printer, stream);
                write_token(printer, stream, "->");
                write_space(printer, stream);
                traverse_sub_node_and_print(printer, stream, &sub_factory, returns_node);
            }
        }
        write_token(printer, stream, ":");
        if let Some(doc_string_node) = node.get("doc_string") {
            if !doc_string_node.is_null() {
                printer.increase_indent();
                if let Some(doc_string) = doc_string_node.get_str_for_key("value") {
                    write_newline(printer, stream);
                    write_indent(printer, stream);
                    let doc_string_text = String::from(doc_string);
                    if let Some(_newline_index) = doc_string_text.find('\n') {
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
            write_key_of_node_as_array(printer, stream, factory, "body", node);
        } else if let Some(body_node) = node.get("body") {
            if let Some(body_array) = body_node.as_array() {
                write_newline(printer, stream);
                printer.increase_indent();
                write_indented_array(printer, stream, factory, body_array);
                printer.decrease_indent();
            }
        }
    }
}

struct ArgumentsPrinter {}

impl NodePrinter<VyperAST> for ArgumentsPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<VyperAST>,
        node: &VyperAST,
    ) {
        // Writing the function arguments is a little complicated because of the way the Vyper AST
        // represents default argument values.  The default values live in the node's 'defaults'
        // key.  This key contains an array of default values. Since Vyper function defs can
        // only have default values for the last arguments to the function, the array of defaults
        // array of say length N apply only to the last N arguments of the function.
        if let Some(defaults_node) = node.get("defaults") {
            if let Some(defaults_array) = defaults_node.as_array() {
                if defaults_array.is_empty() {
                    write_args_as_array(printer, stream, factory, node);
                } else if let Some(args_node) = node.get("args") {
                    if let Some(args_array) = args_node.as_array() {
                        let first_default_index = args_array.len() - defaults_array.len();
                        let mut i: usize = 0;
                        while i < args_array.len() {
                            let arg = args_array.get(i).unwrap();
                            traverse_sub_node_and_print(printer, stream, factory, arg);
                            if i >= first_default_index {
                                let default_index = i - first_default_index;
                                let default_node = defaults_array.get(default_index).unwrap();
                                write_token(printer, stream, "=");
                                traverse_sub_node_and_print(printer, stream, factory, default_node);
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

struct ArgPrinter {}

impl NodePrinter<VyperAST> for ArgPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<VyperAST>,
        node: &VyperAST,
    ) {
        if let Some(arg_str) = node.get_str_for_key("arg") {
            write_token(printer, stream, arg_str);
        }

        if let Some(annotation_node) = node.get("annotation") {
            if !annotation_node.is_null() {
                write_token(printer, stream, ":");
                write_space(printer, stream);
                traverse_sub_node_and_print(printer, stream, factory, annotation_node);
            }
        }
    }
}

struct AssignPrinter {}

impl NodePrinter<VyperAST> for AssignPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<VyperAST>,
        node: &VyperAST,
    ) {
        if let Some(target_node) = node.get("target") {
            traverse_sub_node_and_print(printer, stream, factory, target_node);
        }
        write_token(printer, stream, " = ");
        if let Some(value_node) = node.get("value") {
            traverse_sub_node_and_print(printer, stream, factory, value_node);
        }
    }
}

struct AttributePrinter {}

impl NodePrinter<VyperAST> for AttributePrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<VyperAST>,
        node: &VyperAST,
    ) {
        if let Some(value_node) = node.get("value") {
            traverse_sub_node_and_print(printer, stream, factory, value_node);
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

impl NodePrinter<VyperAST> for AugAssignPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<VyperAST>,
        node: &VyperAST,
    ) {
        if let Some(target_node) = node.get("target") {
            traverse_sub_node_and_print(printer, stream, factory, target_node);
        }
        write_space(printer, stream);
        if let Some(op_node) = node.get("op") {
            if let Some(ast_type_str) = op_node.get_str_for_key("ast_type") {
                if self.operator_map.contains_key(ast_type_str) {
                    write_token(printer, stream, self.operator_map[ast_type_str]);
                }
            }
        }
        write_token(printer, stream, "=");
        write_space(printer, stream);
        if let Some(value_node) = node.get("value") {
            traverse_sub_node_and_print(printer, stream, factory, value_node);
        }
    }
}

struct IntPrinter {}

impl NodePrinter<VyperAST> for IntPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<VyperAST>,
        node: &VyperAST,
    ) {
        if let Some(value) = node.get_int_for_key("value") {
            write_token(printer, stream, &value.to_string());
        } else if let Some(value) = node.get_float_for_key("value") {
            write_token(printer, stream, &value.to_string());
        }
    }
}

/// Used to print Hex and Decimal nodes.
struct ValuePrinter {}

impl NodePrinter<VyperAST> for ValuePrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<VyperAST>,
        node: &VyperAST,
    ) {
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

impl NodePrinter<VyperAST> for BinOpPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<VyperAST>,
        node: &VyperAST,
    ) {
        if let Some(left_node) = node.get("left") {
            write_node_with_parens_maybe(printer, stream, factory, left_node);
        }
        if let Some(op_node) = node.get("op") {
            if let Some(ast_type_str) = op_node.get_str_for_key("ast_type") {
                if self.operator_map.contains_key(ast_type_str) {
                    write_space(printer, stream);
                    write_token(printer, stream, self.operator_map[ast_type_str]);
                    write_space(printer, stream);
                }
            }
        }
        if let Some(right_node) = node.get("right") {
            write_node_with_parens_maybe(printer, stream, factory, right_node);
        }
    }
}

struct BoolOpPrinter {}

impl NodePrinter<VyperAST> for BoolOpPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<VyperAST>,
        node: &VyperAST,
    ) {
        if let Some(values_node) = node.get("values") {
            if let Some(values_array) = values_node.as_array() {
                if values_array.len() < 2 {
                    return;
                }

                let left = &values_array[0];
                let right = &values_array[1];

                write_node_with_parens_maybe(printer, stream, factory, left);
                write_space(printer, stream);

                if let Some(op_node) = node.get("op") {
                    traverse_sub_node_and_print(printer, stream, factory, op_node);
                }

                write_space(printer, stream);
                write_node_with_parens_maybe(printer, stream, factory, right);
            }
        }
    }
}

struct AssertPrinter {}

impl NodePrinter<VyperAST> for AssertPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<VyperAST>,
        node: &VyperAST,
    ) {
        write_token(printer, stream, "assert");
        write_space(printer, stream);
        if let Some(test_node) = node.get("test") {
            traverse_sub_node_and_print(printer, stream, factory, test_node);
        }
        if let Some(msg_node) = node.get("msg") {
            if msg_node.is_object() {
                write_token(printer, stream, ",");
                write_space(printer, stream);
                traverse_sub_node_and_print(printer, stream, factory, msg_node);
            }
        }
    }
}

struct StrPrinter {}

impl NodePrinter<VyperAST> for StrPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<VyperAST>,
        node: &VyperAST,
    ) {
        if let Some(value_str) = node.get_str_for_key("value") {
            write_string(printer, stream, value_str);
        }
    }
}

struct ModulePrinter {}

impl NodePrinter<VyperAST> for ModulePrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<VyperAST>,
        node: &VyperAST,
    ) {
        if let Some(doc_string_node) = node.get("doc_string") {
            if let Some(doc_string) = doc_string_node.get_str_for_key("value") {
                let doc_text = String::from(doc_string);
                if let Some(_newline_index) = doc_text.find('\n') {
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
        if let Some(body_node) = node.get("body") {
            if let Some(body_array) = body_node.as_array() {
                for value in body_array {
                    traverse_sub_node_and_print(printer, stream, factory, value);
                    write_newline(printer, stream);
                    write_newline(printer, stream);
                }
            }
        }
    }
}

struct AnnAssignPrinter {}

impl NodePrinter<VyperAST> for AnnAssignPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<VyperAST>,
        node: &VyperAST,
    ) {
        if let Some(target_node) = node.get("target") {
            traverse_sub_node_and_print(printer, stream, factory, target_node);
        }
        if let Some(annotation_node) = node.get("annotation") {
            write_token(printer, stream, ":");
            write_space(printer, stream);
            traverse_sub_node_and_print(printer, stream, factory, annotation_node);
        }
        write_value_assignment(printer, stream, factory, node);
    }
}

struct ExprPrinter {}

impl NodePrinter<VyperAST> for ExprPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<VyperAST>,
        node: &VyperAST,
    ) {
        if let Some(value_node) = node.get("value") {
            traverse_sub_node_and_print(printer, stream, factory, value_node);
        }
    }
}

struct CallPrinter {}

impl NodePrinter<VyperAST> for CallPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<VyperAST>,
        node: &VyperAST,
    ) {
        if let Some(func_node) = node.get("func") {
            traverse_sub_node_and_print(printer, stream, factory, func_node);
        }
        write_token(printer, stream, "(");
        write_args_as_array(printer, stream, factory, node);
        if let Some(keywords_node) = node.get("keywords") {
            if let Some(keywords_array) = keywords_node.as_array() {
                if !keywords_array.is_empty() {
                    write_token(printer, stream, ",");
                    write_space(printer, stream);
                    write_key_of_node_as_array(printer, stream, factory, "keywords", node);
                }
            }
        }
        write_token(printer, stream, ")");
    }
}

struct NameConstantPrinter {}

impl NodePrinter<VyperAST> for NameConstantPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<VyperAST>,
        node: &VyperAST,
    ) {
        if let Some(value_node) = node.get("value") {
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

impl NodePrinter<VyperAST> for UnaryOpPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<VyperAST>,
        node: &VyperAST,
    ) {
        let mut needs_space = false;
        if let Some(op_node) = node.get("op") {
            if let Some(op_str) = op_node.get_str_for_key("ast_type") {
                if op_str == "Not" {
                    write_token(printer, stream, "not");
                    needs_space = true;
                } else if op_str == "Invert" {
                    write_token(printer, stream, "~");
                }
            }
        }
        if let Some(operand_node) = node.get("operand") {
            if needs_space {
                write_space(printer, stream);
            }
            write_node_with_parens_maybe(printer, stream, factory, operand_node);
        }
    }
}

struct InterfaceDefPrinter {}

impl NodePrinter<VyperAST> for InterfaceDefPrinter {
    fn on_entry(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<VyperAST>,
        node: &VyperAST,
    ) {
        write_token(printer, stream, "interface");
        write_space(printer, stream);
        if let Some(name_str) = node.get_str_for_key("name") {
            write_token(printer, stream, name_str);
            write_token(printer, stream, ":");
            write_newline(printer, stream);
        }
        printer.increase_indent();
    }

    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<VyperAST>,
        node: &VyperAST,
    ) {
        let mut printer_settings = factory.get_settings().clone();
        printer_settings.set_value_for_key(
            FUNCTION_DEF_USES_STRUCT_DECL_FORM,
            PreferenceValue::Boolean(true),
        );
        let sub_factory = VyperNodePrinterFactory::new(printer_settings);
        write_body_as_nodes(printer, stream, &sub_factory, node);
    }

    fn on_exit(
        &mut self,
        printer: &mut PrettyPrinter,
        _stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<VyperAST>,
        _node: &VyperAST,
    ) {
        printer.decrease_indent();
    }
}

struct StructDefPrinter {}

impl NodePrinter<VyperAST> for StructDefPrinter {
    fn on_entry(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<VyperAST>,
        node: &VyperAST,
    ) {
        write_token(printer, stream, "struct");
        write_space(printer, stream);
        if let Some(name_str) = node.get_str_for_key("name") {
            write_token(printer, stream, name_str);
            write_token(printer, stream, ":");
            write_newline(printer, stream);
        }
        printer.increase_indent();
    }

    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<VyperAST>,
        node: &VyperAST,
    ) {
        write_body_as_nodes(printer, stream, factory, node);
    }

    fn on_exit(
        &mut self,
        printer: &mut PrettyPrinter,
        _stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<VyperAST>,
        _node: &VyperAST,
    ) {
        printer.decrease_indent();
    }
}

struct ForPrinter {}

impl NodePrinter<VyperAST> for ForPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<VyperAST>,
        node: &VyperAST,
    ) {
        write_token(printer, stream, "for");
        write_space(printer, stream);
        if let Some(target_node) = node.get("target") {
            traverse_sub_node_and_print(printer, stream, factory, target_node);
        }
        write_space(printer, stream);
        write_token(printer, stream, "in");
        write_space(printer, stream);
        if let Some(iter_node) = node.get("iter") {
            traverse_sub_node_and_print(printer, stream, factory, iter_node);
        }
        write_token(printer, stream, ":");
        write_newline(printer, stream);
        printer.increase_indent();
        write_body_as_nodes(printer, stream, factory, node);
        printer.decrease_indent();
    }
}

struct ListPrinter {}

impl NodePrinter<VyperAST> for ListPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<VyperAST>,
        node: &VyperAST,
    ) {
        write_token(printer, stream, "[");
        write_elements_array(printer, stream, factory, node);
        write_token(printer, stream, "]");
    }
}

struct IfPrinter {}

impl NodePrinter<VyperAST> for IfPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<VyperAST>,
        node: &VyperAST,
    ) {
        write_token(printer, stream, "if");
        write_space(printer, stream);
        if let Some(test_node) = node.get("test") {
            traverse_sub_node_and_print(printer, stream, factory, test_node);
        }
        write_token(printer, stream, ":");
        write_newline(printer, stream);
        printer.increase_indent();
        write_indented_body_array(printer, stream, factory, node);
        printer.decrease_indent();
        if let Some(orelse_node) = node.get("orelse") {
            if let Some(orelse_array) = orelse_node.as_array() {
                if !orelse_array.is_empty() {
                    write_newline(printer, stream);
                    write_indent(printer, stream);
                    write_token(printer, stream, "else:");
                    write_newline(printer, stream);
                    printer.increase_indent();
                    write_indented_array(printer, stream, factory, orelse_array);
                    printer.decrease_indent();
                }
            }
        }
    }
}

struct BreakPrinter {}

impl NodePrinter<VyperAST> for BreakPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<VyperAST>,
        _node: &VyperAST,
    ) {
        write_token(printer, stream, "break");
    }
}

struct ContinuePrinter {}

impl NodePrinter<VyperAST> for ContinuePrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<VyperAST>,
        _node: &VyperAST,
    ) {
        write_token(printer, stream, "continue")
    }
}

struct PassPrinter {}

impl NodePrinter<VyperAST> for PassPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<VyperAST>,
        _node: &VyperAST,
    ) {
        write_token(printer, stream, "pass");
    }
}

struct ReturnPrinter {}

impl NodePrinter<VyperAST> for ReturnPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<VyperAST>,
        node: &VyperAST,
    ) {
        write_token(printer, stream, "return");
        if let Some(value_node) = node.get("value") {
            if !value_node.is_null() {
                write_space(printer, stream);
                traverse_sub_node_and_print(printer, stream, factory, value_node);
            }
        }
    }
}

struct RaisePrinter {}

impl NodePrinter<VyperAST> for RaisePrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<VyperAST>,
        node: &VyperAST,
    ) {
        write_token(printer, stream, "raise");
        write_space(printer, stream);
        if let Some(exc_node) = node.get("exc") {
            traverse_sub_node_and_print(printer, stream, factory, exc_node);
        }
    }
}

struct ImportPrinter {}

impl NodePrinter<VyperAST> for ImportPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<VyperAST>,
        node: &VyperAST,
    ) {
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

impl NodePrinter<VyperAST> for ImportFromPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<VyperAST>,
        node: &VyperAST,
    ) {
        write_token(printer, stream, "from");
        write_space(printer, stream);
        if let Some(module_node) = node.get("module") {
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
        if let Some(alias_node) = node.get("alias") {
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

impl NodePrinter<VyperAST> for EventDefPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<VyperAST>,
        node: &VyperAST,
    ) {
        write_token(printer, stream, "event");
        write_space(printer, stream);
        if let Some(name_str) = node.get_str_for_key("name") {
            write_token(printer, stream, name_str);
            write_token(printer, stream, ":");
        }
        write_newline(printer, stream);
        printer.increase_indent();
        write_indented_body_array(printer, stream, factory, node);
        printer.decrease_indent();
    }
}

struct LogPrinter {}

impl NodePrinter<VyperAST> for LogPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<VyperAST>,
        node: &VyperAST,
    ) {
        write_token(printer, stream, "log");
        write_space(printer, stream);
        if let Some(value_node) = node.get("value") {
            if !value_node.is_null() {
                traverse_sub_node_and_print(printer, stream, factory, value_node);
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

impl NodePrinter<VyperAST> for CommentPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<VyperAST>,
        node: &VyperAST,
    ) {
        if let Some(value_node) = node.get("value") {
            if value_node.is_string() {
                if let Some(value_str) = value_node.as_str() {
                    write_token(printer, stream, "#");
                    write_space(printer, stream);
                    write_flowable_text(printer, stream, value_str, "# ");
                }
            } else if value_node.is_object() {
                write_token(printer, stream, "#");
                write_space(printer, stream);
                let s = pretty_print_node(value_node, factory);
                write_flowable_text(printer, stream, &s, "# ");
            }
        }
    }
}

struct AndPrinter {}

impl NodePrinter<VyperAST> for AndPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<VyperAST>,
        node: &VyperAST,
    ) {
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

impl NodePrinter<VyperAST> for KeywordPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<VyperAST>,
        node: &VyperAST,
    ) {
        if let Some(arg_str) = node.get_str_for_key("arg") {
            write_token(printer, stream, arg_str);
            write_token(printer, stream, "=");
            if let Some(value_node) = node.get("value") {
                traverse_sub_node_and_print(printer, stream, factory, value_node);
            }
        }
    }
}

struct BytesPrinter {}

impl NodePrinter<VyperAST> for BytesPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<VyperAST>,
        node: &VyperAST,
    ) {
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

impl NodePrinter<VyperAST> for DictPrinter {
    fn on_entry(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<VyperAST>,
        _node: &VyperAST,
    ) {
        write_token(printer, stream, "{");
    }

    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<VyperAST>,
        node: &VyperAST,
    ) {
        if let Some(keys_node) = node.get("keys") {
            if let Some(keys_array) = keys_node.as_array() {
                if let Some(values_node) = node.get("values") {
                    if let Some(values_array) = values_node.as_array() {
                        assert_eq!(keys_array.len(), values_array.len());

                        if !keys_array.is_empty() {
                            write_newline(printer, stream);
                            printer.increase_indent();
                            let mut i: usize = 0;
                            while i < keys_array.len() {
                                let key = keys_array.get(i).unwrap();
                                let value = values_array.get(i).unwrap();

                                write_indent(printer, stream);
                                traverse_sub_node_and_print(printer, stream, factory, key);
                                write_token(printer, stream, ":");
                                write_space(printer, stream);
                                traverse_sub_node_and_print(printer, stream, factory, value);

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

    fn on_exit(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<VyperAST>,
        _node: &VyperAST,
    ) {
        write_token(printer, stream, "}");
    }
}

struct EnumDefPrinter {}

impl NodePrinter<VyperAST> for EnumDefPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<VyperAST>,
        node: &VyperAST,
    ) {
        write_token(printer, stream, "enum");
        write_space(printer, stream);

        if let Some(name_str) = node.get_str_for_key("name") {
            write_token(printer, stream, name_str);
            write_token(printer, stream, ":");
        }

        write_newline(printer, stream);
        printer.increase_indent();
        write_indented_body_array(printer, stream, factory, node);
        printer.decrease_indent();
    }
}

static TUPLES_SHOULD_USE_PARENTHESES: &str = "tuples_should_use_parentheses";
static FUNCTION_DEF_USES_STRUCT_DECL_FORM: &str = "function_def_uses_struct_decl_form";

/// Type that implements [`NodePrinterFactory<AST>`] for Vyper AST nodes.
///
/// Use this factory object with the [`crate::pretty_print_visitor::PrettyPrintVisitor<AST>`] object.
#[derive(Clone)]
pub struct VyperNodePrinterFactory {
    settings: Preferences,
}

impl VyperNodePrinterFactory {
    /// Create a new node printer factory.
    ///
    /// # Arguments
    ///
    /// * `tuples_use_parens` - True if the factory should generate tuple printers that write
    /// parentheses around the tuple.
    pub fn new(settings: Preferences) -> VyperNodePrinterFactory {
        VyperNodePrinterFactory { settings }
    }

    /// Create a new node printer factory with default settings.
    pub fn default() -> VyperNodePrinterFactory {
        let mut preferences = Preferences::new();
        preferences.set_value_for_key(
            TUPLES_SHOULD_USE_PARENTHESES,
            PreferenceValue::Boolean(false),
        );
        preferences.set_value_for_key(
            FUNCTION_DEF_USES_STRUCT_DECL_FORM,
            PreferenceValue::Boolean(false),
        );
        VyperNodePrinterFactory {
            settings: preferences,
        }
    }
}

impl NodePrinterFactory<VyperAST> for VyperNodePrinterFactory {
    fn printer_for(&self, node: &VyperAST) -> Box<dyn NodePrinter<VyperAST>> {
        if let Some(ast_type) = node.get_str_for_key("ast_type") {
            match ast_type {
                "Name" => Box::new(NamePrinter {}),
                "VariableDecl" => Box::new(VariableDeclPrinter {}),
                "Subscript" => Box::new(SubscriptPrinter {}),
                "Index" => Box::new(IndexPrinter {}),
                "Tuple" => {
                    Box::new(TuplePrinter {
                        use_parentheses: <VyperNodePrinterFactory as NodePrinterFactory<
                            VyperAST,
                        >>::get_preference_value_for_key(
                            self, TUPLES_SHOULD_USE_PARENTHESES
                        ),
                    })
                }
                "FunctionDef" => Box::new(
                    FunctionDefPrinter::new(<VyperNodePrinterFactory as NodePrinterFactory<
                        VyperAST,
                    >>::get_preference_value_for_key(
                        self, FUNCTION_DEF_USES_STRUCT_DECL_FORM
                    )),
                ),
                "arguments" => Box::new(ArgumentsPrinter {}),
                "arg" => Box::new(ArgPrinter {}),
                "Assign" => Box::new(AssignPrinter {}),
                "Attribute" => Box::new(AttributePrinter {}),
                "AugAssign" => Box::new(AugAssignPrinter::new()),
                "Int" => Box::new(IntPrinter {}),
                "Decimal" => Box::new(ValuePrinter {}),
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
                "Hex" => Box::new(ValuePrinter {}),
                "EnumDef" => Box::new(EnumDefPrinter {}),
                _ => Box::new(DummyNodePrinter {}),
            }
        } else {
            Box::new(DummyNodePrinter {})
        }
    }

    fn get_settings(&self) -> &Preferences {
        &self.settings
    }
}
