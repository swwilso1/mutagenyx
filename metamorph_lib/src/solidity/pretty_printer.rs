//! The `solidity::pretty_printer` module provides the code to traverse a SolidityAST and print out
//! the source code represented in the AST.

use crate::json::JSONMutate;
use crate::node_printer::{NodePrinter, NodePrinterFactory};
use crate::node_printer_helpers::*;
use crate::preferences::{PreferenceValue, Preferences};
use crate::pretty_printer::{
    write_flowable_text, write_indent, write_newline, write_space, write_string, write_token,
    PrettyPrinter,
};
use crate::solidity::ast::SolidityAST;
use std::io::Write;

/// Helper function for printing out documentation sub-nodes from a node.
///
/// # Arguments
///
/// * `stream` - The [`Write`] object that will receive formatted output.
/// * `node` - The Solidity AST node to check for a documentation sub-node.
/// * `factory` - The [`SolidityNodePrinterFactory`] object that generates node printers for AST
/// nodes.
/// * `printer` - The [`PrettyPrinter`] object that will write to `stream`.
fn print_documentation_helper<W: Write>(
    printer: &mut PrettyPrinter,
    stream: &mut W,
    factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
    node: &SolidityAST,
) {
    if let Some(docs) = node.borrow_value_for_key("documentation") {
        traverse_sub_node_and_print(printer, stream, factory, docs);
        write_newline(printer, stream);
        write_indent(printer, stream);
    }
}

/// Helper function to write out the end text for a block of statements.
///
/// # Arguments
///
/// * `stream` - The [`Write`] object that will receive the formatted output.
/// * `printer` - The [`PrettyPrinter`] object that will write to `stream`.
fn close_block_exit_helper<W: Write>(printer: &mut PrettyPrinter, stream: &mut W) {
    printer.decrease_indent();
    write_newline(printer, stream);
    write_indent(printer, stream);
    write_token(printer, stream, "}");
}

/// Helper function to write out the text for the `name` element of the node.
///
/// # Arguments
///
/// * `stream` - The [`Write`] object that will receive the formatted output.
/// * `node` - The [`SolidityAST`] node.
/// * `printer` - The [`PrettyPrinter`] object that will write to `stream`.
fn print_name_helper<W: Write>(printer: &mut PrettyPrinter, stream: &mut W, node: &SolidityAST) {
    if let Some(name) = node.get_str_for_key("name") {
        write_token(printer, stream, name);
    }
}

/// Helper function to print an operator if a node has a value for the 'operator' key.
///
/// # Abstract
///
/// * `stream` - The [`Write`] object that will receive formatted output.
/// * `node` - The node from the syntax tree.
/// * `printer` - The [`PrettyPrinter`] that will write formatted output to `stream`.
fn print_operator_helper<W: Write>(
    printer: &mut PrettyPrinter,
    stream: &mut W,
    node: &SolidityAST,
) {
    if let Some(operator) = node.get_str_for_key("operator") {
        write_space(printer, stream);
        write_token(printer, stream, operator);
        write_space(printer, stream);
    }
}

/// Helper function to write an array preceded by a space.
///
/// # Abstract
///
/// * `printer` - The [`PrettyPrinter`] that will write formatted output to `stream`.
/// * `node` - The [`Write`] object that will receive formatted output.
/// * `factory` - The [`NodePrinterFactory<W,AST>`] object to generate node printers.
/// * `node` - The [`SolidityAST`] node.
fn print_space_and_array_helper<W: Write>(
    printer: &mut PrettyPrinter,
    stream: &mut W,
    factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
    node: &SolidityAST,
) {
    if let Some(array) = node.as_array() {
        if array.len() > 0 {
            write_space(printer, stream);
            print_array_helper(printer, stream, factory, array);
        }
    }
}

/// For Block, UncheckedBlock, and YulBlock nodes, print out the statements array.
///
/// # Arguments
///
/// * `printer` - The [`PrettyPrinter`] object that writes formatted text to `stream`.
/// * `stream` - The [`Write`] object that receives the formatted text.
/// * `factory` - The [`SolidityNodePrinterFactory`] object that generates printer nodes.
/// * `node` - The [`SolidityAST`] node containing statements.
/// * `single_element_on_same_line` - True of the function should write a single element array
/// without writing newlines and indents.
fn print_statements_helper<W: Write>(
    printer: &mut PrettyPrinter,
    stream: &mut W,
    factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
    node: &SolidityAST,
    single_element_on_same_line: bool,
) {
    if let Some(statements) = node.borrow_value_for_key("statements") {
        if let Some(statements_array) = statements.as_array() {
            if statements_array.len() == 1 && single_element_on_same_line {
                if let Some(statement) = statements_array.get(0) {
                    write_space(printer, stream);
                    traverse_sub_node_and_print(printer, stream, factory, statement);
                    write_space(printer, stream);
                }
            } else if statements_array.len() > 0 {
                printer.increase_indent();
                write_newline(printer, stream);

                let mut i = 0;
                while i < statements_array.len() {
                    if let Some(statement) = statements_array.get(i) {
                        write_indent(printer, stream);
                        traverse_sub_node_and_print(printer, stream, factory, statement);
                        if i < (statements_array.len() - 1) {
                            write_newline(printer, stream);
                        }
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

/// Write the array in the `arguments` key.
///
/// # Arguments
///
/// * `printer` - The [`PrettyPrinter`] object that will format the output text.
/// * `stream` - The [`Write`] object that will received the formatted text.
/// * `factory` - The [`SolidityNodePrinterFactory`] that generates printers for AST nodes.
/// * `node` - The node in the Solidity AST.
fn print_arguments_helper<W: Write>(
    printer: &mut PrettyPrinter,
    stream: &mut W,
    factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
    node: &SolidityAST,
) {
    if let Some(arguments_node) = node.borrow_value_for_key("arguments") {
        if let Some(arguments_array) = arguments_node.as_array() {
            print_array_helper(printer, stream, factory, arguments_array);
        }
    }
}

/// Change any non-ASCII characters into a Solidity escaped character form.
///
/// # Arguments
///
/// * `s` - The string to un-encode.
fn unencode_string(s: &str) -> String {
    let mut unencoded_s = String::new();

    for c in s.chars() {
        let c_code = c as u32;

        if c_code < 32 {
            match c_code {
                8 => {
                    // backspace
                    unencoded_s.push('\\');
                    unencoded_s.push('b');
                }
                9 => {
                    // horizontal tab
                    unencoded_s.push('\\');
                    unencoded_s.push('t');
                }
                10 => {
                    // new line/line feed
                    unencoded_s.push('\\');
                    unencoded_s.push('n');
                }
                11 => {
                    // vertical tab
                    unencoded_s.push('\\');
                    unencoded_s.push('v');
                }
                12 => {
                    // form feed
                    unencoded_s.push('\\');
                    unencoded_s.push('f');
                }
                13 => {
                    // carriage return
                    unencoded_s.push('\\');
                    unencoded_s.push('r');
                }
                _ => {
                    let converted_char = format!("\\x{:x}", c_code);
                    unencoded_s += converted_char.as_str();
                }
            }
        } else if c_code >= 32 && c_code < 127 {
            unencoded_s.push(c);
        } else {
            let converted_char = format!("\\u{:x}", c_code);
            unencoded_s += converted_char.as_str();
        }
    }

    unencoded_s
}

/// Default node printer for unsupported nodes.
struct DummyNodePrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for DummyNodePrinter {
    fn visit_children(&mut self) -> bool {
        true
    }
}

struct SourceUnitPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for SourceUnitPrinter {
    fn on_entry(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        _factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        if let Some(license) = node.get_str_for_key("license") {
            write_token(printer, stream, "//");
            write_space(printer, stream);
            write_token(printer, stream, "SPDX-License-Identifier");
            write_token(printer, stream, ":");
            write_space(printer, stream);

            write_token(printer, stream, &license);
            write_newline(printer, stream);
        }
    }

    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        if let Some(nodes) = node.borrow_value_for_key("nodes") {
            if let Some(node_array) = nodes.as_array() {
                let mut i = 0;
                while i < node_array.len() {
                    if let Some(node) = node_array.get(i) {
                        write_indent(printer, stream);
                        traverse_sub_node_and_print(printer, stream, factory, node);
                        if i < (node_array.len() - 1) {
                            write_newline(printer, stream);
                            write_newline(printer, stream);
                        }
                    }
                    i += 1;
                }
                write_newline(printer, stream);
            }
        }
    }

    fn on_exit(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        _factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        _node: &SolidityAST,
    ) {
        write_newline(printer, stream);
    }
}

struct PragmaDirectivePrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for PragmaDirectivePrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        _factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        if let Some(l) = node.borrow_value_for_key("literals") {
            if let Some(v) = l.as_array() {
                write_token(printer, stream, "pragma");
                write_space(printer, stream);

                for val in v {
                    if let Some(s) = val.as_str() {
                        write_token(printer, stream, s);
                        if s == "solidity" {
                            write_space(printer, stream);
                        }
                    }
                }
            }
        }
    }

    fn on_exit(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        _factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        _node: &SolidityAST,
    ) {
        write_token(printer, stream, ";");
    }
}

struct ContractDefinitionPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for ContractDefinitionPrinter {
    fn on_entry(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        print_documentation_helper(printer, stream, factory, node);

        if let Some(abs) = node.get_bool_for_key("abstract") {
            if abs {
                write_token(printer, stream, "abstract");
                write_space(printer, stream);
            }
        }

        if let Some(s) = node.get_str_for_key("contractKind") {
            write_token(printer, stream, s);
        } else {
            write_token(printer, stream, "contract");
        }

        write_space(printer, stream);

        if let Some(n) = node.get_str_for_key("name") {
            write_token(printer, stream, n);
        }

        if let Some(base_contracts) = node.borrow_value_for_key("baseContracts") {
            if let Some(contract_array) = base_contracts.as_array() {
                if contract_array.len() > 0 {
                    write_space(printer, stream);
                    write_token(printer, stream, "is");
                    write_space(printer, stream);
                    print_array_helper(printer, stream, factory, contract_array);
                }
            }
        }
    }

    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        write_space(printer, stream);
        write_token(printer, stream, "{");
        if let Some(node_array) = node.get_array_for_key("nodes") {
            if node_array.len() > 0 {
                printer.increase_indent();
                write_newline(printer, stream);

                let mut i = 0;
                while i < node_array.len() {
                    if let Some(contract_node) = node_array.get(i) {
                        write_indent(printer, stream);
                        traverse_sub_node_and_print(printer, stream, factory, contract_node);
                        if let Some(node_type) = contract_node.get_str_for_key("nodeType") {
                            if node_type == "VariableDeclaration" {
                                write_token(printer, stream, ";");
                            }
                        }
                        if i < (node_array.len() - 1) {
                            write_newline(printer, stream);
                            write_newline(printer, stream);
                        }
                    }
                    i += 1;
                }
                write_newline(printer, stream);
                printer.decrease_indent();
                write_indent(printer, stream);
            }
        }
        write_token(printer, stream, "}");
    }
}

struct StructDefinitionPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for StructDefinitionPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        write_token(printer, stream, "struct");
        write_space(printer, stream);

        if let Some(name) = node.get_str_for_key("name") {
            write_token(printer, stream, name);
        }

        write_space(printer, stream);

        write_token(printer, stream, "{");
        write_newline(printer, stream);
        printer.increase_indent();

        if let Some(members) = node.borrow_value_for_key("members") {
            if let Some(members_array) = members.as_array() {
                let mut i = 0;
                while i < members_array.len() {
                    if let Some(member) = members_array.get(i) {
                        write_indent(printer, stream);
                        traverse_sub_node_and_print(printer, stream, factory, member);
                        write_token(printer, stream, ";");
                        if i < (members_array.len() - 1) {
                            write_newline(printer, stream);
                        }
                        i += 1;
                    }
                }
            }
        }
    }

    fn on_exit(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        _factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        _node: &SolidityAST,
    ) {
        close_block_exit_helper(printer, stream);
    }
}

struct ElementaryTypeNamePrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for ElementaryTypeNamePrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        _factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        print_name_helper(printer, stream, node);

        if let Some(mutability) = node.get_str_for_key("stateMutability") {
            if mutability == "payable" {
                write_space(printer, stream);
                write_token(printer, stream, mutability);
            }
        }
    }
}

struct MappingPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for MappingPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        write_token(printer, stream, "mapping");
        write_token(printer, stream, "(");

        if let Some(key) = node.borrow_value_for_key("keyType") {
            traverse_sub_node_and_print(printer, stream, factory, key);
        }

        write_space(printer, stream);
        write_token(printer, stream, "=>");
        write_space(printer, stream);

        if let Some(value) = node.borrow_value_for_key("valueType") {
            traverse_sub_node_and_print(printer, stream, factory, value);
        }

        write_token(printer, stream, ")");
    }
}

struct UserDefinedTypeNamePrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for UserDefinedTypeNamePrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        _factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        if let Some(path_node) = node.borrow_value_for_key("pathNode") {
            print_name_helper(printer, stream, path_node);
        }
    }
}

struct VariableDeclarationPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for VariableDeclarationPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        print_documentation_helper(printer, stream, factory, node);
        if let Some(type_name_node) = node.borrow_value_for_key("typeName") {
            traverse_sub_node_and_print(printer, stream, factory, type_name_node);
        }

        if let Some(visibility) = node.get_str_for_key("visibility") {
            if visibility != "internal" {
                write_space(printer, stream);
                write_token(printer, stream, visibility);
            }
        }

        if let Some(constant) = node.get_bool_for_key("constant") {
            if constant {
                write_space(printer, stream);
                write_token(printer, stream, "constant");
            }
        }

        if let Some(indexed) = node.get_bool_for_key("indexed") {
            if indexed {
                write_space(printer, stream);
                write_token(printer, stream, "indexed");
            }
        }

        if let Some(override_node) = node.borrow_value_for_key("overrides") {
            traverse_sub_node_and_print(printer, stream, factory, override_node);
        }

        if let Some(mutability_str) = node.get_str_for_key("mutability") {
            if mutability_str == "immutable" {
                write_space(printer, stream);
                write_token(printer, stream, mutability_str);
            }
        }
    }

    fn on_exit(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        if let Some(location) = node.get_str_for_key("storageLocation") {
            if location == "memory" || location == "storage" || location == "calldata" {
                write_space(printer, stream);
                write_token(printer, stream, location);
            }
        }

        if let Some(name) = node.get_str_for_key("name") {
            if name.len() > 0 {
                write_space(printer, stream);
                write_token(printer, stream, name);
            }
        }

        if let Some(value) = node.borrow_value_for_key("value") {
            write_space(printer, stream);
            write_token(printer, stream, "=");
            write_space(printer, stream);
            traverse_sub_node_and_print(printer, stream, factory, value);
        }
    }
}

struct FunctionDefinitionPrinter {
    write_nonpayable_state_mutability: bool,
}

impl FunctionDefinitionPrinter {
    /// Create a new function definition printer.
    ///
    /// # Arguments
    ///
    /// * `write_nonpayable_state_mutability` - True if the printer should write nonpayable.
    pub fn new(write_nonpayable_state_mutability: bool) -> FunctionDefinitionPrinter {
        FunctionDefinitionPrinter {
            write_nonpayable_state_mutability,
        }
    }
}

impl<W: Write> NodePrinter<W, SolidityAST> for FunctionDefinitionPrinter {
    fn on_entry(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        if let Some(documentation) = node.borrow_value_for_key("documentation") {
            traverse_sub_node_and_print(printer, stream, factory, documentation);
            write_newline(printer, stream);
            write_indent(printer, stream);
        }

        if let Some(kind) = node.get_str_for_key("kind") {
            if kind == "function" || kind == "freeFunction" {
                write_token(printer, stream, "function");
                write_space(printer, stream);

                if let Some(name) = node.get_str_for_key("name") {
                    write_token(printer, stream, name);
                }
            } else if kind == "constructor" || kind == "receive" || kind == "fallback" {
                write_token(printer, stream, kind);
            }
        }
    }

    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        if let Some(parameters) = node.borrow_value_for_key("parameters") {
            traverse_sub_node_and_print(printer, stream, factory, parameters);
        }

        if let Some(visibility) = node.get_str_for_key("visibility") {
            if visibility != "internal" {
                write_space(printer, stream);
                write_token(printer, stream, visibility);
            }
        }

        if let Some(virtual_bool) = node.get_bool_for_key("virtual") {
            if virtual_bool {
                write_space(printer, stream);
                write_token(printer, stream, "virtual");
            }
        }

        if let Some(state_mutability) = node.get_str_for_key("stateMutability") {
            let do_mutability: bool = match state_mutability {
                "nonpayable" => {
                    if self.write_nonpayable_state_mutability {
                        true
                    } else {
                        false
                    }
                }
                _ => true,
            };
            if do_mutability {
                write_space(printer, stream);
                write_token(printer, stream, state_mutability);
            }
        }

        if let Some(overrides) = node.borrow_value_for_key("overrides") {
            write_space(printer, stream);
            traverse_sub_node_and_print(printer, stream, factory, overrides);
        }

        if let Some(modifiers) = node.borrow_value_for_key("modifiers") {
            print_space_and_array_helper(printer, stream, factory, modifiers);
        }

        if let Some(return_parameters) = node.borrow_value_for_key("returnParameters") {
            if let Some(parameters) = return_parameters.borrow_value_for_key("parameters") {
                if let Some(parameter_array) = parameters.as_array() {
                    if parameter_array.len() > 0 {
                        write_space(printer, stream);
                        write_token(printer, stream, "returns");
                        write_space(printer, stream);
                        traverse_sub_node_and_print(printer, stream, factory, return_parameters);
                    }
                }
            }
        }

        if let Some(body) = node.borrow_value_for_key("body") {
            write_space(printer, stream);
            traverse_sub_node_and_print(printer, stream, factory, body);
        }

        if let Some(implemented) = node.get_bool_for_key("implemented") {
            if !implemented {
                write_token(printer, stream, ";");
            }
        }
    }
}

struct ParameterListPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for ParameterListPrinter {
    fn on_entry(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        _factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        _node: &SolidityAST,
    ) {
        write_token(printer, stream, "(");
    }

    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        if let Some(parameters) = node.borrow_value_for_key("parameters") {
            if let Some(parameters_array) = parameters.as_array() {
                print_array_helper(printer, stream, factory, parameters_array);
            }
        }
    }

    fn on_exit(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        _factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        _node: &SolidityAST,
    ) {
        write_token(printer, stream, ")");
    }
}

struct BlockPrinter {
    single_statement_on_same_line: bool,
}

impl BlockPrinter {
    /// Create a new block printer.
    ///
    /// # Arguments
    ///
    /// * `single_statements_on_same_line` - If true, the printer will write blocks with one statement
    /// without using indents or newlines.
    fn new(single_statement_on_same_line: bool) -> BlockPrinter {
        BlockPrinter {
            single_statement_on_same_line,
        }
    }
}

impl<W: Write> NodePrinter<W, SolidityAST> for BlockPrinter {
    fn on_entry(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        _factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        _node: &SolidityAST,
    ) {
        write_token(printer, stream, "{");
    }

    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        print_statements_helper(
            printer,
            stream,
            factory,
            node,
            self.single_statement_on_same_line,
        );
    }

    fn on_exit(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        _factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        _node: &SolidityAST,
    ) {
        write_token(printer, stream, "}");
    }
}

struct VariableDeclarationStatementPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for VariableDeclarationStatementPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        if let Some(declarations) = node.borrow_value_for_key("declarations") {
            if let Some(declarations_array) = declarations.as_array() {
                if declarations_array.len() > 1 {
                    write_token(printer, stream, "(");
                }
                let mut i = 0;
                while i < declarations_array.len() {
                    if let Some(declaration) = declarations_array.get(i) {
                        traverse_sub_node_and_print(printer, stream, factory, declaration);
                        if i < (declarations_array.len() - 1) {
                            write_token(printer, stream, ",");
                            if let Some(value) = declarations_array.get(i + 1) {
                                if !value.is_null() {
                                    write_space(printer, stream);
                                }
                            }
                        }
                    }
                    i += 1;
                }
                if declarations_array.len() > 1 {
                    write_token(printer, stream, ")");
                }
            }
        }

        if let Some(initial_value) = node.borrow_value_for_key("initialValue") {
            write_space(printer, stream);
            write_token(printer, stream, "=");
            write_space(printer, stream);
            traverse_sub_node_and_print(printer, stream, factory, initial_value);
        }
    }

    fn on_exit(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        _factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        _node: &SolidityAST,
    ) {
        write_token(printer, stream, ";");
    }
}

struct BinaryOperationPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for BinaryOperationPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        if let Some(le) = node.borrow_value_for_key("leftExpression") {
            traverse_sub_node_and_print(printer, stream, factory, le);
        }

        print_operator_helper(printer, stream, node);

        if let Some(re) = node.borrow_value_for_key("rightExpression") {
            traverse_sub_node_and_print(printer, stream, factory, re);
        }
    }
}

struct LiteralPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for LiteralPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        _factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        if let Some(value) = node.get_str_for_key("value") {
            if let Some(kind) = node.get_str_for_key("kind") {
                if kind == "string" {
                    let unencoded_string = unencode_string(value);
                    write_string(printer, stream, &unencoded_string);
                } else if kind == "unicodeString" {
                    write_token(printer, stream, "unicode");
                    write_string(printer, stream, value);
                } else {
                    write_token(printer, stream, value);
                }
            } else {
                write_token(printer, stream, value);
            }
        }

        if let Some(sub_denomination) = node.get_str_for_key("subdenomination") {
            write_space(printer, stream);
            write_token(printer, stream, sub_denomination);
        }
    }
}

struct ExpressionStatementPrinter {
    write_statement_semicolon: bool,
}

impl ExpressionStatementPrinter {
    pub fn new(write_statement_semicolon: bool) -> ExpressionStatementPrinter {
        ExpressionStatementPrinter {
            write_statement_semicolon,
        }
    }
}

impl<W: Write> NodePrinter<W, SolidityAST> for ExpressionStatementPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        if let Some(expression) = node.borrow_value_for_key("expression") {
            traverse_sub_node_and_print(printer, stream, factory, expression);
        }
    }

    fn on_exit(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        _factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        _node: &SolidityAST,
    ) {
        if self.write_statement_semicolon {
            write_token(printer, stream, ";");
        }
    }
}

struct AssignmentPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for AssignmentPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        if let Some(lhs) = node.borrow_value_for_key("leftHandSide") {
            traverse_sub_node_and_print(printer, stream, factory, lhs);
        }

        print_operator_helper(printer, stream, node);

        if let Some(rhs) = node.borrow_value_for_key("rightHandSide") {
            traverse_sub_node_and_print(printer, stream, factory, rhs);
        }
    }
}

struct IdentifierPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for IdentifierPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        _factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        if let Some(name) = node.get_str_for_key("name") {
            write_token(printer, stream, name);
        }
    }
}

struct FunctionCallPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for FunctionCallPrinter {
    fn on_entry(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        if let Some(expression) = node.borrow_value_for_key("expression") {
            traverse_sub_node_and_print(printer, stream, factory, expression);
            write_token(printer, stream, "(");
        }
    }

    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        if let Some(names_array) = node.get_array_for_key("names") {
            if names_array.len() > 0 {
                if let Some(arguments_array) = node.get_array_for_key("arguments") {
                    assert_eq!(
                        names_array.len(),
                        arguments_array.len(),
                        "names and arguments arrays have different lenghts in FunctionCallPrinter"
                    );
                    write_token(printer, stream, "{");
                    printer.increase_indent();
                    let mut i: usize = 0;
                    while i < names_array.len() {
                        let name = names_array.get(i).unwrap().as_str().unwrap();
                        let arg = arguments_array.get(i).unwrap();
                        write_newline(printer, stream);
                        write_indent(printer, stream);
                        write_token(printer, stream, name);
                        write_token(printer, stream, ":");
                        write_space(printer, stream);
                        traverse_sub_node_and_print(printer, stream, factory, arg);
                        if i < (names_array.len() - 1) {
                            write_token(printer, stream, ",");
                        }
                        i += 1;
                    }
                    printer.decrease_indent();
                    write_newline(printer, stream);
                    write_indent(printer, stream);
                    write_token(printer, stream, "}");
                }
            } else {
                print_arguments_helper(printer, stream, factory, node);
            }
        } else {
            print_arguments_helper(printer, stream, factory, node);
        }
    }

    fn on_exit(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        _factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        write_token(printer, stream, ")");

        if let Some(try_call_bool) = node.get_bool_for_key("tryCall") {
            if try_call_bool {
                write_space(printer, stream);
                write_token(printer, stream, "returns");
            }
        }
    }
}

struct UnaryOperationPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for UnaryOperationPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        if let Some(prefix) = node.get_bool_for_key("prefix") {
            if prefix {
                if let Some(operator) = node.get_str_for_key("operator") {
                    if operator == "delete" {
                        write_token(printer, stream, "delete");
                        write_space(printer, stream);
                    } else {
                        write_token(printer, stream, operator);
                    }
                }
            }

            if let Some(sub_expression) = node.borrow_value_for_key("subExpression") {
                traverse_sub_node_and_print(printer, stream, factory, sub_expression);
            }

            if !prefix {
                if let Some(operator) = node.get_str_for_key("operator") {
                    write_token(printer, stream, operator);
                }
            }
        }
    }
}

struct MemberAccessPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for MemberAccessPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        if let Some(expression) = node.borrow_value_for_key("expression") {
            traverse_sub_node_and_print(printer, stream, factory, expression);
        }

        write_token(printer, stream, ".");

        if let Some(member_name) = node.get_str_for_key("memberName") {
            write_token(printer, stream, member_name);
        }
    }
}

struct IndexAccessPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for IndexAccessPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        if let Some(base_expression) = node.borrow_value_for_key("baseExpression") {
            traverse_sub_node_and_print(printer, stream, factory, base_expression);
        }

        write_token(printer, stream, "[");

        if let Some(index_expression) = node.borrow_value_for_key("indexExpression") {
            traverse_sub_node_and_print(printer, stream, factory, index_expression);
        }

        write_token(printer, stream, "]");
    }
}

struct IfStatementPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for IfStatementPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        write_token(printer, stream, "if");
        write_token(printer, stream, "(");

        if let Some(condition) = node.borrow_value_for_key("condition") {
            traverse_sub_node_and_print(printer, stream, factory, condition);
        }

        write_token(printer, stream, ")");
        write_space(printer, stream);

        if let Some(true_body) = node.borrow_value_for_key("trueBody") {
            if let Some(node_type_str) = true_body.get_str_for_key("nodeType") {
                if node_type_str == "Block" {
                    traverse_sub_node_and_print(printer, stream, factory, true_body);
                } else {
                    write_newline(printer, stream);
                    printer.increase_indent();
                    write_indent(printer, stream);
                    traverse_sub_node_and_print(printer, stream, factory, true_body);
                    printer.decrease_indent();
                }
            }
        }

        if let Some(false_body) = node.borrow_value_for_key("falseBody") {
            if let Some(node_type_str) = false_body.get_str_for_key("nodeType") {
                if node_type_str == "Block" {
                    write_space(printer, stream);
                    write_token(printer, stream, "else");
                    write_space(printer, stream);
                    traverse_sub_node_and_print(printer, stream, factory, false_body);
                } else {
                    write_newline(printer, stream);
                    write_indent(printer, stream);
                    write_token(printer, stream, "else");
                    write_newline(printer, stream);
                    printer.increase_indent();
                    write_indent(printer, stream);
                    traverse_sub_node_and_print(printer, stream, factory, false_body);
                    printer.decrease_indent();
                }
            }
        }
    }
}

struct ImportDirectivePrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for ImportDirectivePrinter {
    fn on_entry(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        _factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        _node: &SolidityAST,
    ) {
        write_token(printer, stream, "import");
    }

    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        let mut wrote_symbol_aliases = false;
        if let Some(symbol_aliases) = node.borrow_value_for_key("symbolAliases") {
            if let Some(aliases_array) = symbol_aliases.as_array() {
                if aliases_array.len() > 0 {
                    write_space(printer, stream);
                    write_token(printer, stream, "{");
                    let mut i = 0;
                    while i < aliases_array.len() {
                        if let Some(alias) = aliases_array.get(i) {
                            if let Some(foreign_symbol) = alias.borrow_value_for_key("foreign") {
                                traverse_sub_node_and_print(
                                    printer,
                                    stream,
                                    factory,
                                    foreign_symbol,
                                );
                            }
                            if let Some(local_str) = alias.get_str_for_key("local") {
                                write_space(printer, stream);
                                write_token(printer, stream, "as");
                                write_space(printer, stream);
                                write_token(printer, stream, local_str);
                            }
                            if i < (aliases_array.len() - 1) {
                                write_token(printer, stream, ",");
                                write_space(printer, stream);
                            }
                        }
                        i += 1;
                    }
                    write_token(printer, stream, "}");
                    wrote_symbol_aliases = true;
                }
            }
        }

        write_space(printer, stream);

        if wrote_symbol_aliases {
            write_token(printer, stream, "from");
            write_space(printer, stream);
        }

        if let Some(file) = node.get_str_for_key("file") {
            write_string(printer, stream, file);
        }

        if let Some(unit_alias_str) = node.get_str_for_key("unitAlias") {
            if unit_alias_str.len() > 0 {
                write_space(printer, stream);
                write_token(printer, stream, "as");
                write_space(printer, stream);
                write_token(printer, stream, unit_alias_str);
            }
        }
    }

    fn on_exit(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        _factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        _node: &SolidityAST,
    ) {
        write_token(printer, stream, ";");
    }
}

struct InheritanceSpecifierPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for InheritanceSpecifierPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        if let Some(base_name) = node.borrow_value_for_key("baseName") {
            traverse_sub_node_and_print(printer, stream, factory, base_name);
        }
        if let Some(arguments_array) = node.get_array_for_key("arguments") {
            write_token(printer, stream, "(");
            print_array_helper(printer, stream, factory, arguments_array);
            write_token(printer, stream, ")");
        }
    }
}

struct IdentifierPathPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for IdentifierPathPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        _factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        print_name_helper(printer, stream, node);
    }
}

struct ModifierInvocationPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for ModifierInvocationPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        if let Some(modifier_name) = node.borrow_value_for_key("modifierName") {
            traverse_sub_node_and_print(printer, stream, factory, modifier_name);
        }

        if let Some(arguments) = node.borrow_value_for_key("arguments") {
            if let Some(arguments_array) = arguments.as_array() {
                write_token(printer, stream, "(");
                if arguments_array.len() > 0 {
                    print_array_helper(printer, stream, factory, arguments_array);
                }
                write_token(printer, stream, ")");
            }
        }
    }
}

struct UsingForDirectivePrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for UsingForDirectivePrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        write_token(printer, stream, "using");
        if let Some(library_name) = node.borrow_value_for_key("libraryName") {
            write_space(printer, stream);
            traverse_sub_node_and_print(printer, stream, factory, library_name);
        }
        if let Some(function_list_node) = node.borrow_value_for_key("functionList") {
            if let Some(function_list_array) = function_list_node.as_array() {
                if function_list_array.len() > 0 {
                    write_space(printer, stream);
                    write_token(printer, stream, "{");
                    print_array_helper(printer, stream, factory, function_list_array);
                    write_token(printer, stream, "}");
                }
            }
        }
        write_space(printer, stream);
        write_token(printer, stream, "for");
        write_space(printer, stream);
        if let Some(type_name) = node.borrow_value_for_key("typeName") {
            traverse_sub_node_and_print(printer, stream, factory, type_name);
        }

        if let Some(global_bool) = node.get_bool_for_key("global") {
            if global_bool {
                write_space(printer, stream);
                write_token(printer, stream, "global");
            }
        }
    }

    fn on_exit(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        _factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        _node: &SolidityAST,
    ) {
        write_token(printer, stream, ";");
    }
}

struct ModifierDefinitionPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for ModifierDefinitionPrinter {
    fn on_entry(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        print_documentation_helper(printer, stream, factory, node);
        write_token(printer, stream, "modifier");
        write_space(printer, stream);
        print_name_helper(printer, stream, node);
        if let Some(parameters) = node.borrow_value_for_key("parameters") {
            traverse_sub_node_and_print(printer, stream, factory, parameters);
        }
    }

    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        if let Some(virtual_bool) = node.get_bool_for_key("virtual") {
            if virtual_bool {
                write_space(printer, stream);
                write_token(printer, stream, "virtual");
            }
        }

        if let Some(override_node) = node.borrow_value_for_key("overrides") {
            traverse_sub_node_and_print(printer, stream, factory, override_node);
        }

        if let Some(body) = node.borrow_value_for_key("body") {
            write_space(printer, stream);
            traverse_sub_node_and_print(printer, stream, factory, body);
        }
    }
}

struct StructuredDocumentationPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for StructuredDocumentationPrinter {
    fn on_entry(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        _factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        _node: &SolidityAST,
    ) {
        write_token(printer, stream, "/**");
    }

    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        _factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        if let Some(text) = node.get_str_for_key("text") {
            let text_array: Vec<&str> = text.split("\n").collect();
            for sub_text in text_array {
                write_newline(printer, stream);
                write_indent(printer, stream);
                write_space(printer, stream);
                write_token(printer, stream, "*");
                write_space(printer, stream);
                write_flowable_text(printer, stream, sub_text, " *   ");
            }
        }
    }

    fn on_exit(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        _factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        _node: &SolidityAST,
    ) {
        write_newline(printer, stream);
        write_indent(printer, stream);
        write_space(printer, stream);
        write_token(printer, stream, "*/");
    }
}

struct PlaceholderStatementPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for PlaceholderStatementPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        _factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        _node: &SolidityAST,
    ) {
        write_token(printer, stream, "_");
        write_token(printer, stream, ";");
    }
}

struct ReturnPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for ReturnPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        write_token(printer, stream, "return");
        if let Some(expression) = node.borrow_value_for_key("expression") {
            write_space(printer, stream);
            traverse_sub_node_and_print(printer, stream, factory, expression);
        }
        write_token(printer, stream, ";");
    }
}

struct EmitStatementPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for EmitStatementPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        if let Some(event_call) = node.borrow_value_for_key("eventCall") {
            write_token(printer, stream, "emit");
            write_space(printer, stream);
            traverse_sub_node_and_print(printer, stream, factory, event_call);
            write_token(printer, stream, ";");
        }
    }
}

struct OverrideSpecifierPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for OverrideSpecifierPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        write_token(printer, stream, "override");
        if let Some(overrides_array) = node.get_array_for_key("overrides") {
            if overrides_array.len() > 0 {
                write_token(printer, stream, "(");
                print_array_helper(printer, stream, factory, overrides_array);
                write_token(printer, stream, ")");
            }
        }
    }
}

struct ElementaryTypeNameExpressionPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for ElementaryTypeNameExpressionPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        // Not sure if this is the correct way to handle this node.
        if let Some(pure) = node.get_bool_for_key("isPure") {
            if pure {
                if let Some(type_name) = node.borrow_value_for_key("typeName") {
                    if let Some(mutability) = type_name.get_str_for_key("stateMutability") {
                        write_token(printer, stream, mutability);
                    } else {
                        traverse_sub_node_and_print(printer, stream, factory, type_name);
                    }
                }
            }
        }
    }
}

struct TupleExpressionPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for TupleExpressionPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        if let Some(components) = node.borrow_value_for_key("components") {
            if let Some(components_array) = components.as_array() {
                if components_array.len() > 0 {
                    write_token(printer, stream, "(");
                    let mut i: usize = 0;
                    while i < components_array.len() {
                        let value = components_array.get(i).unwrap();
                        traverse_sub_node_and_print(printer, stream, factory, value);
                        if i < (components_array.len() - 1) {
                            write_token(printer, stream, ",");
                            let next_node = components_array.get(i + 1).unwrap();
                            if !next_node.is_null() {
                                write_space(printer, stream);
                            }
                        }
                        i += 1;
                    }
                    write_token(printer, stream, ")");
                }
            }
        }
    }
}

/// Provide code for pretty-printing comments.
///
/// **Important**: Comment nodes do not naturally occur in Solidity AST files (not yet) and so
/// any AST with a comment node no longer conforms to the standard AST format generated by the
/// Solidity compiler.
struct CommentPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for CommentPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        _factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        if let Some(comment_text) = node.get_str_for_key("text") {
            if comment_text.len() > 0 {
                write_token(printer, stream, "//");
                write_space(printer, stream);
                write_flowable_text(printer, stream, comment_text, "// ");
            }
        }
    }
}

struct UncheckedBlockPrinter {
    single_statement_on_same_line: bool,
}

impl UncheckedBlockPrinter {
    /// Create a new UncheckedBlock printer.
    ///
    /// # Arguments
    ///
    /// * `single_statement_on_same_line` - If true, the printer will write blocks with one
    /// statement on the same line without writing newlines and indents.
    pub fn new(single_statement_on_same_line: bool) -> UncheckedBlockPrinter {
        UncheckedBlockPrinter {
            single_statement_on_same_line,
        }
    }
}

impl<W: Write> NodePrinter<W, SolidityAST> for UncheckedBlockPrinter {
    fn on_entry(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        _factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        _node: &SolidityAST,
    ) {
        write_token(printer, stream, "unchecked");
        write_token(printer, stream, "{");
    }

    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        print_statements_helper(
            printer,
            stream,
            factory,
            node,
            self.single_statement_on_same_line,
        );
    }

    fn on_exit(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        _factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        _node: &SolidityAST,
    ) {
        write_token(printer, stream, "}");
    }
}

struct ErrorDefinitionPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for ErrorDefinitionPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        write_token(printer, stream, "error");
        write_space(printer, stream);
        if let Some(name_str) = node.get_str_for_key("name") {
            write_token(printer, stream, name_str);
        }
        if let Some(parameters_node) = node.borrow_value_for_key("parameters") {
            traverse_sub_node_and_print(printer, stream, factory, parameters_node);
        }
        write_token(printer, stream, ";");
    }
}

struct ForStatementPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for ForStatementPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        // Include the '(' in the token so it doesn't line break to the next line.
        write_token(printer, stream, "for(");
        if let Some(initialization_node) = node.borrow_value_for_key("initializationExpression") {
            traverse_sub_node_and_print(printer, stream, factory, initialization_node);
        }
        if let Some(condition_node) = node.borrow_value_for_key("condition") {
            write_space(printer, stream);
            traverse_sub_node_and_print(printer, stream, factory, condition_node);
            write_token(printer, stream, ";");
        }
        if let Some(loop_expression_node) = node.borrow_value_for_key("loopExpression") {
            let mut printer_settings = factory.get_settings().clone();
            printer_settings.set_value_for_key(
                WRITE_EXPRESSION_STATEMENT_SEMICOLON,
                PreferenceValue::Boolean(false),
            );
            let sub_factory: Box<dyn NodePrinterFactory<W, SolidityAST>> =
                Box::new(SolidityNodePrinterFactory::new(printer_settings));
            write_space(printer, stream);
            traverse_sub_node_and_print(printer, stream, &sub_factory, loop_expression_node);
        }
        write_token(printer, stream, ")");
        if let Some(body_node) = node.borrow_value_for_key("body") {
            if let Some(node_type_str) = body_node.get_str_for_key("nodeType") {
                if node_type_str == "Block" {
                    traverse_sub_node_and_print(printer, stream, factory, body_node);
                } else {
                    write_newline(printer, stream);
                    printer.increase_indent();
                    write_indent(printer, stream);
                    traverse_sub_node_and_print(printer, stream, factory, body_node);
                    printer.decrease_indent();
                }
            }
        }
    }
}

struct ArrayTypeNamePrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for ArrayTypeNamePrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        if let Some(base_type_node) = node.borrow_value_for_key("baseType") {
            traverse_sub_node_and_print(printer, stream, factory, base_type_node);
        }
        write_token(printer, stream, "[");
        if let Some(length_node) = node.borrow_value_for_key("length") {
            traverse_sub_node_and_print(printer, stream, factory, length_node);
        }
        write_token(printer, stream, "]");
    }
}

struct InlineAssemblyPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for InlineAssemblyPrinter {
    fn on_entry(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        _factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        if let Some(documentation_str) = node.get_str_for_key("documentation") {
            write_token(printer, stream, "///");
            write_space(printer, stream);
            write_flowable_text(printer, stream, documentation_str, "/// ");
            write_newline(printer, stream);
            write_indent(printer, stream);
        }
    }

    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        write_token(printer, stream, "assembly");
        write_space(printer, stream);
        if let Some(flags_node) = node.borrow_value_for_key("flags") {
            if let Some(flags_array) = flags_node.as_array() {
                if flags_array.len() > 0 {
                    write_token(printer, stream, "(");
                    let mut i: usize = 0;
                    while i < flags_array.len() {
                        if let Some(value) = flags_array.get(i) {
                            if let Some(value_str) = value.as_str() {
                                let unencoded_string = unencode_string(value_str);
                                write_string(printer, stream, &unencoded_string);
                            }
                        }

                        if i < (flags_array.len() - 1) {
                            write_token(printer, stream, ",");
                            write_space(printer, stream);
                        }
                        i += 1;
                    }
                    write_token(printer, stream, ")");
                    write_space(printer, stream);
                }
            }
        }
        if let Some(ast_node) = node.borrow_value_for_key("AST") {
            traverse_sub_node_and_print(printer, stream, factory, ast_node);
        }
    }
}

struct YulBlockPrinter {
    single_statement_on_same_line: bool,
}

impl YulBlockPrinter {
    /// Create a new YulBlock printer.
    ///
    /// # Arguments
    ///
    /// * `single_statement_on_same_line` - If true, the printer will write blocks with only one
    /// statement on the same line without using newlines and indents.
    pub fn new(single_statement_on_same_line: bool) -> YulBlockPrinter {
        YulBlockPrinter {
            single_statement_on_same_line,
        }
    }
}

impl<W: Write> NodePrinter<W, SolidityAST> for YulBlockPrinter {
    fn on_entry(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        _factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        _node: &SolidityAST,
    ) {
        write_token(printer, stream, "{");
    }

    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        print_statements_helper(
            printer,
            stream,
            factory,
            node,
            self.single_statement_on_same_line,
        );
    }

    fn on_exit(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        _factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        _node: &SolidityAST,
    ) {
        write_token(printer, stream, "}");
    }
}

struct YulVariableDeclarationPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for YulVariableDeclarationPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        write_token(printer, stream, "let");
        write_space(printer, stream);
        if let Some(variables_node) = node.borrow_value_for_key("variables") {
            if let Some(variables_array) = variables_node.as_array() {
                print_array_helper(printer, stream, factory, variables_array);
                write_space(printer, stream);
            }
        }
        write_token(printer, stream, ":=");
        write_space(printer, stream);
        if let Some(value_node) = node.borrow_value_for_key("value") {
            traverse_sub_node_and_print(printer, stream, factory, value_node);
        }
    }
}

struct YulFunctionCallPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for YulFunctionCallPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        if let Some(function_name_node) = node.borrow_value_for_key("functionName") {
            traverse_sub_node_and_print(printer, stream, factory, function_name_node);
        }
        write_token(printer, stream, "(");
        print_arguments_helper(printer, stream, factory, node);
        write_token(printer, stream, ")");
    }
}

struct YulIdentifierPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for YulIdentifierPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        _factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        print_name_helper(printer, stream, node);
    }
}

struct YulTypedNamePrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for YulTypedNamePrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        _factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        print_name_helper(printer, stream, node);
    }
}

struct YulAssignmentPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for YulAssignmentPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        if let Some(variable_names_node) = node.borrow_value_for_key("variableNames") {
            if let Some(variable_names_array) = variable_names_node.as_array() {
                print_array_helper(printer, stream, factory, variable_names_array);
                write_space(printer, stream);
            }
        }
        write_token(printer, stream, ":=");
        write_space(printer, stream);
        if let Some(value_node) = node.borrow_value_for_key("value") {
            traverse_sub_node_and_print(printer, stream, factory, value_node);
        }
    }
}

struct YulLiteralPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for YulLiteralPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        _factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        if let Some(kind_str) = node.get_str_for_key("kind") {
            if kind_str == "number" {
                if let Some(value_str) = node.get_str_for_key("value") {
                    write_token(printer, stream, value_str);
                }
            }
        }
    }
}

struct YulForLoopPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for YulForLoopPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        write_token(printer, stream, "for");
        printer.increase_indent();

        let mut printing_settings = factory.get_settings().clone();
        printing_settings.set_value_for_key(
            SINGLE_BLOCK_STATEMENTS_ON_SAME_LINE,
            PreferenceValue::Boolean(true),
        );
        let sub_factory: Box<dyn NodePrinterFactory<W, SolidityAST>> =
            Box::new(SolidityNodePrinterFactory::new(printing_settings));

        if let Some(pre_node) = node.borrow_value_for_key("pre") {
            write_newline(printer, stream);
            write_indent(printer, stream);
            traverse_sub_node_and_print(printer, stream, &sub_factory, pre_node);
        }
        if let Some(condition_node) = node.borrow_value_for_key("condition") {
            write_newline(printer, stream);
            write_indent(printer, stream);
            traverse_sub_node_and_print(printer, stream, factory, condition_node);
        }
        if let Some(post_node) = node.borrow_value_for_key("post") {
            write_newline(printer, stream);
            write_indent(printer, stream);
            traverse_sub_node_and_print(printer, stream, &sub_factory, post_node);
        }
        printer.decrease_indent();
        write_newline(printer, stream);
        write_indent(printer, stream);
        if let Some(body_node) = node.borrow_value_for_key("body") {
            traverse_sub_node_and_print(printer, stream, factory, body_node);
        }
    }
}

struct FunctionTypeNamePrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for FunctionTypeNamePrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        write_token(printer, stream, "function");
        if let Some(parameter_types_node) = node.borrow_value_for_key("parameterTypes") {
            traverse_sub_node_and_print(printer, stream, factory, parameter_types_node);
        }
        if let Some(visibility_str) = node.get_str_for_key("visibility") {
            if visibility_str == "external" {
                write_space(printer, stream);
                write_token(printer, stream, visibility_str);
            }
        }
        if let Some(return_parameters_node) = node.borrow_value_for_key("returnParameterTypes") {
            if let Some(parameters_node) = return_parameters_node.borrow_value_for_key("parameters")
            {
                if let Some(parameters_array) = parameters_node.as_array() {
                    if parameters_array.len() > 0 {
                        write_space(printer, stream);
                        write_token(printer, stream, "returns");
                        write_space(printer, stream);
                        traverse_sub_node_and_print(
                            printer,
                            stream,
                            factory,
                            return_parameters_node,
                        );
                    }
                }
            }
        }
    }
}

struct EventDefinitionPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for EventDefinitionPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        write_token(printer, stream, "event");
        write_space(printer, stream);
        print_name_helper(printer, stream, node);
        if let Some(parameters_node) = node.borrow_value_for_key("parameters") {
            traverse_sub_node_and_print(printer, stream, factory, parameters_node);
        }
        if let Some(anonymous) = node.get_bool_for_key("anonymous") {
            if anonymous {
                write_space(printer, stream);
                write_token(printer, stream, "anonymous");
            }
        }
        write_token(printer, stream, ";");
    }
}

struct RevertStatementPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for RevertStatementPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        write_token(printer, stream, "revert");
        write_space(printer, stream);
        if let Some(error_call_node) = node.borrow_value_for_key("errorCall") {
            traverse_sub_node_and_print(printer, stream, factory, error_call_node);
        }
        write_token(printer, stream, ";");
    }
}

struct EnumDefinitionPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for EnumDefinitionPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        write_token(printer, stream, "enum");
        write_space(printer, stream);
        print_name_helper(printer, stream, node);
        if let Some(members_node) = node.borrow_value_for_key("members") {
            if let Some(members_array) = members_node.as_array() {
                if members_array.len() > 0 {
                    write_space(printer, stream);
                    write_token(printer, stream, "{");
                    printer.increase_indent();

                    let mut i: usize = 0;
                    while i < members_array.len() {
                        write_newline(printer, stream);
                        write_indent(printer, stream);
                        if let Some(value) = members_array.get(i) {
                            traverse_sub_node_and_print(printer, stream, factory, value);
                        }

                        if i < (members_array.len() - 1) {
                            write_token(printer, stream, ",");
                            write_space(printer, stream);
                        }
                        i += 1;
                    }

                    printer.decrease_indent();
                    write_newline(printer, stream);
                    write_indent(printer, stream);
                    write_token(printer, stream, "}");
                }
            }
        }
    }
}

struct EnumValuePrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for EnumValuePrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        _factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        print_name_helper(printer, stream, node);
    }
}

struct UserDefinedValueTypeDefinitionPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for UserDefinedValueTypeDefinitionPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        write_token(printer, stream, "type");
        write_space(printer, stream);
        print_name_helper(printer, stream, node);
        write_space(printer, stream);
        write_token(printer, stream, "is");
        write_space(printer, stream);
        if let Some(underlying_node) = node.borrow_value_for_key("underlyingType") {
            traverse_sub_node_and_print(printer, stream, factory, underlying_node);
        }
        write_token(printer, stream, ";");
    }
}

struct NewExpressionPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for NewExpressionPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        write_token(printer, stream, "new");
        if let Some(type_name_node) = node.borrow_value_for_key("typeName") {
            write_space(printer, stream);
            traverse_sub_node_and_print(printer, stream, factory, type_name_node);
        }
    }
}

struct ConditionalPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for ConditionalPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        if let Some(condition_node) = node.borrow_value_for_key("condition") {
            traverse_sub_node_and_print(printer, stream, factory, condition_node);
        }
        write_space(printer, stream);
        write_token(printer, stream, "?");
        write_space(printer, stream);

        if let Some(true_expr_node) = node.borrow_value_for_key("trueExpression") {
            traverse_sub_node_and_print(printer, stream, factory, true_expr_node);
        }

        write_space(printer, stream);
        write_token(printer, stream, ":");
        write_space(printer, stream);

        if let Some(false_expr_node) = node.borrow_value_for_key("falseExpression") {
            traverse_sub_node_and_print(printer, stream, factory, false_expr_node);
        }
    }
}

struct FunctionCallOptionsPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for FunctionCallOptionsPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        if let Some(expr_node) = node.borrow_value_for_key("expression") {
            traverse_sub_node_and_print(printer, stream, factory, expr_node);
        }

        if let Some(names_array) = node.get_array_for_key("names") {
            if let Some(options_array) = node.get_array_for_key("options") {
                assert_eq!(
                    names_array.len(),
                    options_array.len(),
                    "names and options array lengths differ in FunctionCallOptionsPrinter"
                );

                if names_array.len() > 0 {
                    write_token(printer, stream, "{");
                    let mut i: usize = 0;
                    while i < names_array.len() {
                        let name = names_array.get(i).unwrap().as_str().unwrap();
                        let option = options_array.get(i).unwrap();

                        write_token(printer, stream, name);
                        write_token(printer, stream, ":");
                        write_space(printer, stream);

                        traverse_sub_node_and_print(printer, stream, factory, option);

                        if i < (names_array.len() - 1) {
                            write_token(printer, stream, ",");
                            write_space(printer, stream);
                        }
                        i += 1;
                    }
                    write_token(printer, stream, "}");
                }
            }
        }
    }
}

struct TryStatementPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for TryStatementPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        write_token(printer, stream, "try");
        write_space(printer, stream);
        if let Some(external_call_node) = node.borrow_value_for_key("externalCall") {
            traverse_sub_node_and_print(printer, stream, factory, external_call_node);
            write_space(printer, stream);
        }
        if let Some(clauses_array) = node.get_array_for_key("clauses") {
            let mut i: usize = 0;
            while i < clauses_array.len() {
                let clause = clauses_array.get(i).unwrap();
                if i >= 1 {
                    write_token(printer, stream, "catch");
                    write_space(printer, stream);
                }
                traverse_sub_node_and_print(printer, stream, factory, clause);
                if i < (clauses_array.len()) {
                    write_space(printer, stream);
                }
                i += 1;
            }
        }
    }
}

struct TryCatchClausePrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for TryCatchClausePrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut W,
        factory: &Box<dyn NodePrinterFactory<W, SolidityAST>>,
        node: &SolidityAST,
    ) {
        if let Some(error_name_str) = node.get_str_for_key("errorName") {
            if error_name_str.len() > 0 {
                write_token(printer, stream, error_name_str);
            }
        }

        if let Some(parameters_node) = node.borrow_value_for_key("parameters") {
            traverse_sub_node_and_print(printer, stream, factory, parameters_node);
            write_space(printer, stream);
        }

        if let Some(block_node) = node.borrow_value_for_key("block") {
            traverse_sub_node_and_print(printer, stream, factory, block_node);
        }
    }
}

static WRITE_EXPRESSION_STATEMENT_SEMICOLON: &str = "write_expression_statement_semicolon";
static SINGLE_BLOCK_STATEMENTS_ON_SAME_LINE: &str = "single_block_statements_on_same_line";
static WRITE_NONPAYABLE_STATE_MUTABILITY: &str = "write_nonpayable_state_mutability";

/// Type that implements [`NodePrinterFactory<W,AST>`] for Solidity nodes.
///
/// Use this factory object with the [`crate::pretty_print_visitor::PrettyPrintVisitor<W,AST>`] object.
#[derive(Clone)]
pub struct SolidityNodePrinterFactory {
    pub settings: Preferences,
}

impl SolidityNodePrinterFactory {
    /// Create a new node printer factory using `settings`.
    ///
    /// # Arguments
    ///
    /// * `settings` - The pretty printer settings.
    pub fn new(settings: Preferences) -> SolidityNodePrinterFactory {
        SolidityNodePrinterFactory { settings }
    }

    /// Create a new node printer factory using default pretty-printer settings.
    pub fn default() -> SolidityNodePrinterFactory {
        let mut preferences = Preferences::new();
        preferences.set_value_for_key(
            WRITE_EXPRESSION_STATEMENT_SEMICOLON,
            PreferenceValue::Boolean(true),
        );
        preferences.set_value_for_key(
            SINGLE_BLOCK_STATEMENTS_ON_SAME_LINE,
            PreferenceValue::Boolean(false),
        );
        preferences.set_value_for_key(
            WRITE_NONPAYABLE_STATE_MUTABILITY,
            PreferenceValue::Boolean(false),
        );

        SolidityNodePrinterFactory {
            settings: preferences,
        }
    }

    fn get_preference_value_for_key(&self, key: &str) -> bool {
        if let Some(preference) = self.settings.get_value_for_key(key) {
            match preference {
                PreferenceValue::Boolean(b) => b,
                _ => false,
            }
        } else {
            false
        }
    }
}

impl<W: Write> NodePrinterFactory<W, SolidityAST> for SolidityNodePrinterFactory {
    fn printer_for(&self, node: &SolidityAST) -> Box<dyn NodePrinter<W, SolidityAST>> {
        if let Some(node_type) = node.get_str_for_key("nodeType") {
            match node_type {
                "ArrayTypeName" => Box::new(ArrayTypeNamePrinter {}),
                "Assignment" => Box::new(AssignmentPrinter {}),
                "BinaryOperation" => Box::new(BinaryOperationPrinter {}),
                "Block" => Box::new(BlockPrinter::new(
                    self.get_preference_value_for_key(SINGLE_BLOCK_STATEMENTS_ON_SAME_LINE),
                )),
                "Comment" => Box::new(CommentPrinter {}),
                "Conditional" => Box::new(ConditionalPrinter {}),
                "ContractDefinition" => Box::new(ContractDefinitionPrinter {}),
                "ElementaryTypeName" => Box::new(ElementaryTypeNamePrinter {}),
                "ElementaryTypeNameExpression" => Box::new(ElementaryTypeNameExpressionPrinter {}),
                "EmitStatement" => Box::new(EmitStatementPrinter {}),
                "EnumDefinition" => Box::new(EnumDefinitionPrinter {}),
                "EnumValue" => Box::new(EnumValuePrinter {}),
                "ErrorDefinition" => Box::new(ErrorDefinitionPrinter {}),
                "EventDefinition" => Box::new(EventDefinitionPrinter {}),
                "ExpressionStatement" => Box::new(ExpressionStatementPrinter::new(
                    self.get_preference_value_for_key(WRITE_EXPRESSION_STATEMENT_SEMICOLON),
                )),
                "ForStatement" => Box::new(ForStatementPrinter {}),
                "FunctionCall" => Box::new(FunctionCallPrinter {}),
                "FunctionCallOptions" => Box::new(FunctionCallOptionsPrinter {}),
                "FunctionDefinition" => Box::new(FunctionDefinitionPrinter::new(
                    self.get_preference_value_for_key(WRITE_NONPAYABLE_STATE_MUTABILITY),
                )),
                "FunctionTypeName" => Box::new(FunctionTypeNamePrinter {}),
                "Identifier" => Box::new(IdentifierPrinter {}),
                "IdentifierPath" => Box::new(IdentifierPathPrinter {}),
                "IfStatement" => Box::new(IfStatementPrinter {}),
                "ImportDirective" => Box::new(ImportDirectivePrinter {}),
                "IndexAccess" => Box::new(IndexAccessPrinter {}),
                "InheritanceSpecifier" => Box::new(InheritanceSpecifierPrinter {}),
                "InlineAssembly" => Box::new(InlineAssemblyPrinter {}),
                "Literal" => Box::new(LiteralPrinter {}),
                "Mapping" => Box::new(MappingPrinter {}),
                "MemberAccess" => Box::new(MemberAccessPrinter {}),
                "ModifierDefinition" => Box::new(ModifierDefinitionPrinter {}),
                "ModifierInvocation" => Box::new(ModifierInvocationPrinter {}),
                "NewExpression" => Box::new(NewExpressionPrinter {}),
                "OverrideSpecifier" => Box::new(OverrideSpecifierPrinter {}),
                "ParameterList" => Box::new(ParameterListPrinter {}),
                "PlaceholderStatement" => Box::new(PlaceholderStatementPrinter {}),
                "PragmaDirective" => Box::new(PragmaDirectivePrinter {}),
                "Return" => Box::new(ReturnPrinter {}),
                "RevertStatement" => Box::new(RevertStatementPrinter {}),
                "SourceUnit" => Box::new(SourceUnitPrinter {}),
                "StructDefinition" => Box::new(StructDefinitionPrinter {}),
                "StructuredDocumentation" => Box::new(StructuredDocumentationPrinter {}),
                "TryCatchClause" => Box::new(TryCatchClausePrinter {}),
                "TryStatement" => Box::new(TryStatementPrinter {}),
                "TupleExpression" => Box::new(TupleExpressionPrinter {}),
                "UserDefinedTypeName" => Box::new(UserDefinedTypeNamePrinter {}),
                "VariableDeclaration" => Box::new(VariableDeclarationPrinter {}),
                "VariableDeclarationStatement" => Box::new(VariableDeclarationStatementPrinter {}),
                "UnaryOperation" => Box::new(UnaryOperationPrinter {}),
                "UncheckedBlock" => Box::new(UncheckedBlockPrinter::new(
                    self.get_preference_value_for_key(SINGLE_BLOCK_STATEMENTS_ON_SAME_LINE),
                )),
                "UserDefinedValueTypeDefinition" => {
                    Box::new(UserDefinedValueTypeDefinitionPrinter {})
                }
                "UsingForDirective" => Box::new(UsingForDirectivePrinter {}),
                "YulAssignment" => Box::new(YulAssignmentPrinter {}),
                "YulBlock" => Box::new(YulBlockPrinter::new(
                    self.get_preference_value_for_key(SINGLE_BLOCK_STATEMENTS_ON_SAME_LINE),
                )),
                "YulForLoop" => Box::new(YulForLoopPrinter {}),
                "YulFunctionCall" => Box::new(YulFunctionCallPrinter {}),
                "YulIdentifier" => Box::new(YulIdentifierPrinter {}),
                "YulLiteral" => Box::new(YulLiteralPrinter {}),
                "YulTypedName" => Box::new(YulTypedNamePrinter {}),
                "YulVariableDeclaration" => Box::new(YulVariableDeclarationPrinter {}),
                _ => Box::new(DummyNodePrinter {}),
            }
        } else {
            Box::new(DummyNodePrinter {})
        }
    }

    fn get_settings(&self) -> &Preferences {
        return &self.settings;
    }
}
