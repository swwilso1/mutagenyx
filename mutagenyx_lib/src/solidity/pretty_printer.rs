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
use serde_json::Value;
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
fn print_documentation_helper(
    printer: &mut PrettyPrinter,
    stream: &mut dyn Write,
    factory: &dyn NodePrinterFactory<SolidityAST>,
    node: &SolidityAST,
) {
    if let Some(docs) = node.get("documentation") {
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
fn close_block_exit_helper(printer: &mut PrettyPrinter, stream: &mut dyn Write) {
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
fn print_name_helper(printer: &mut PrettyPrinter, stream: &mut dyn Write, node: &SolidityAST) {
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
fn print_operator_helper(printer: &mut PrettyPrinter, stream: &mut dyn Write, node: &SolidityAST) {
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
/// * `factory` - The [`NodePrinterFactory<AST>`] object to generate node printers.
/// * `node` - The [`SolidityAST`] node.
fn print_space_and_array_helper(
    printer: &mut PrettyPrinter,
    stream: &mut dyn Write,
    factory: &dyn NodePrinterFactory<SolidityAST>,
    node: &SolidityAST,
) {
    if let Some(array) = node.as_array() {
        if !array.is_empty() {
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
fn print_statements_helper(
    printer: &mut PrettyPrinter,
    stream: &mut dyn Write,
    factory: &dyn NodePrinterFactory<SolidityAST>,
    node: &SolidityAST,
    single_element_on_same_line: bool,
) {
    if let Some(statements) = node.get("statements") {
        if let Some(statements_array) = statements.as_array() {
            if statements_array.len() == 1 && single_element_on_same_line {
                if let Some(statement) = statements_array.get(0) {
                    write_space(printer, stream);
                    traverse_sub_node_and_print(printer, stream, factory, statement);
                    write_space(printer, stream);
                }
            } else if !statements_array.is_empty() {
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
fn print_arguments_helper(
    printer: &mut PrettyPrinter,
    stream: &mut dyn Write,
    factory: &dyn NodePrinterFactory<SolidityAST>,
    node: &SolidityAST,
) {
    if let Some(arguments_node) = node.get("arguments") {
        if let Some(arguments_array) = arguments_node.as_array() {
            print_array_helper(printer, stream, factory, arguments_array);
        }
    }
}

/// Write the returnParameters node with a returns statement.
///
/// # Arguments
///
/// * `printer` - The [`PrettyPrinter`] object that will format the output text.
/// * `stream` - The [`Write`] object that will received the formatted text.
/// * `factory` - The [`SolidityNodePrinterFactory`] that generates printers for AST nodes.
/// * `node` - The node in the Solidity AST.
fn print_return_parameters_helper(
    printer: &mut PrettyPrinter,
    stream: &mut dyn Write,
    factory: &dyn NodePrinterFactory<SolidityAST>,
    node: &SolidityAST,
) {
    if let Some(return_parameters_node) = node.get("returnParameters") {
        if let Some(parameters_array) = return_parameters_node.get_array_for_key("parameters") {
            if !parameters_array.is_empty() {
                write_space(printer, stream);
                write_token(printer, stream, "returns");
                write_space(printer, stream);
                traverse_sub_node_and_print(printer, stream, factory, return_parameters_node);
            }
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
        } else if (32..127).contains(&c_code) {
            unencoded_s.push(c);
        } else {
            let converted_char = format!("\\u{:x}", c_code);
            unencoded_s += converted_char.as_str();
        }
    }

    unencoded_s
}

/// Return true if the string only contains ASCII digits and/or the '.' character.
///
/// # Arguments
///
/// * `s` - The input string slice.
fn contains_digits_and_or_dots(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    for c in s.chars() {
        if !c.is_ascii_digit() && c != '.' {
            return false;
        }
    }
    true
}

/// Default node printer for unsupported nodes.
struct DummyNodePrinter {}

impl NodePrinter<SolidityAST> for DummyNodePrinter {
    fn visit_children(&mut self) -> bool {
        true
    }
}

struct SourceUnitPrinter {}

impl NodePrinter<SolidityAST> for SourceUnitPrinter {
    fn on_entry(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        if let Some(license) = node.get_str_for_key("license") {
            write_token(printer, stream, "//");
            write_space(printer, stream);
            write_token(printer, stream, "SPDX-License-Identifier");
            write_token(printer, stream, ":");
            write_space(printer, stream);

            write_token(printer, stream, license);
            write_newline(printer, stream);
        }
    }

    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        if let Some(nodes) = node.get("nodes") {
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
        stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<SolidityAST>,
        _node: &SolidityAST,
    ) {
        write_newline(printer, stream);
    }
}

struct PragmaDirectivePrinter {}

impl NodePrinter<SolidityAST> for PragmaDirectivePrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        if let Some(l) = node.get("literals") {
            if let Some(v) = l.as_array() {
                let operators = vec!["<", "<=", ">=", ">", "^"];

                write_token(printer, stream, "pragma");

                let mut previous_token = "pragma";
                let mut i: usize = 0;
                while i < v.len() {
                    let val = &v[i];
                    let mut needs_space: bool = true;

                    if let Some(s) = val.as_str() {
                        if operators.contains(&previous_token) {
                            needs_space = false;
                        } else if contains_digits_and_or_dots(previous_token)
                            && contains_digits_and_or_dots(s)
                        {
                            needs_space = false;
                        }

                        if i < (v.len() - 1) && needs_space {
                            write_space(printer, stream);
                        }
                        write_token(printer, stream, s);

                        previous_token = s;
                    }
                    i += 1;
                }

                write_token(printer, stream, ";");
            }
        }
    }
}

struct ContractDefinitionPrinter {}

impl NodePrinter<SolidityAST> for ContractDefinitionPrinter {
    fn on_entry(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
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

        if let Some(base_contracts) = node.get("baseContracts") {
            if let Some(contract_array) = base_contracts.as_array() {
                if !contract_array.is_empty() {
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
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        write_space(printer, stream);
        write_token(printer, stream, "{");
        if let Some(node_array) = node.get_array_for_key("nodes") {
            if !node_array.is_empty() {
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

impl NodePrinter<SolidityAST> for StructDefinitionPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
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

        if let Some(members) = node.get("members") {
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
        stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<SolidityAST>,
        _node: &SolidityAST,
    ) {
        close_block_exit_helper(printer, stream);
    }
}

struct ElementaryTypeNamePrinter {}

impl NodePrinter<SolidityAST> for ElementaryTypeNamePrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<SolidityAST>,
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

impl NodePrinter<SolidityAST> for MappingPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        write_token(printer, stream, "mapping");
        write_token(printer, stream, "(");

        if let Some(key) = node.get("keyType") {
            traverse_sub_node_and_print(printer, stream, factory, key);
        }

        write_space(printer, stream);
        write_token(printer, stream, "=>");
        write_space(printer, stream);

        if let Some(value) = node.get("valueType") {
            traverse_sub_node_and_print(printer, stream, factory, value);
        }

        write_token(printer, stream, ")");
    }
}

struct UserDefinedTypeNamePrinter {}

impl NodePrinter<SolidityAST> for UserDefinedTypeNamePrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        if let Some(path_node) = node.get("pathNode") {
            print_name_helper(printer, stream, path_node);
        } else {
            print_name_helper(printer, stream, node);
        }
    }
}

struct VariableDeclarationPrinter {}

impl NodePrinter<SolidityAST> for VariableDeclarationPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        print_documentation_helper(printer, stream, factory, node);
        if let Some(type_name_node) = node.get("typeName") {
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

        if let Some(override_node) = node.get("overrides") {
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
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        if let Some(location) = node.get_str_for_key("storageLocation") {
            if location == "memory" || location == "storage" || location == "calldata" {
                write_space(printer, stream);
                write_token(printer, stream, location);
            }
        }

        if let Some(name) = node.get_str_for_key("name") {
            if !name.is_empty() {
                write_space(printer, stream);
                write_token(printer, stream, name);
            }
        }

        if let Some(value) = node.get("value") {
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

impl NodePrinter<SolidityAST> for FunctionDefinitionPrinter {
    fn on_entry(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        if let Some(documentation) = node.get("documentation") {
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
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        if let Some(parameters) = node.get("parameters") {
            traverse_sub_node_and_print(printer, stream, factory, parameters);
        }

        if let Some(visibility) = node.get_str_for_key("visibility") {
            write_space(printer, stream);
            write_token(printer, stream, visibility);
            // if visibility != "internal" {
            //     write_space(printer, stream);
            //     write_token(printer, stream, visibility);
            // }
        }

        if let Some(virtual_bool) = node.get_bool_for_key("virtual") {
            if virtual_bool {
                write_space(printer, stream);
                write_token(printer, stream, "virtual");
            }
        }

        if let Some(state_mutability) = node.get_str_for_key("stateMutability") {
            let do_mutability: bool = match state_mutability {
                "nonpayable" => self.write_nonpayable_state_mutability,
                _ => true,
            };
            if do_mutability {
                write_space(printer, stream);
                write_token(printer, stream, state_mutability);
            }
        }

        if let Some(overrides) = node.get("overrides") {
            write_space(printer, stream);
            traverse_sub_node_and_print(printer, stream, factory, overrides);
        }

        if let Some(modifiers) = node.get("modifiers") {
            print_space_and_array_helper(printer, stream, factory, modifiers);
        }

        print_return_parameters_helper(printer, stream, factory, node);

        if let Some(body) = node.get("body") {
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

impl NodePrinter<SolidityAST> for ParameterListPrinter {
    fn on_entry(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<SolidityAST>,
        _node: &SolidityAST,
    ) {
        write_token(printer, stream, "(");
    }

    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        if let Some(parameters) = node.get("parameters") {
            if let Some(parameters_array) = parameters.as_array() {
                print_array_helper(printer, stream, factory, parameters_array);
            }
        }
    }

    fn on_exit(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<SolidityAST>,
        _node: &SolidityAST,
    ) {
        write_token(printer, stream, ")");
    }
}

/// Used for both Block, UncheckedBlock, and YulBlock nodes.
struct BlockPrinter {
    /// When true, write a block with only one statement on the same line:
    ///
    /// `{ <statement>; }`
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

impl NodePrinter<SolidityAST> for BlockPrinter {
    fn on_entry(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        if let Some(node_type_str) = node.get_str_for_key("nodeType") {
            if node_type_str == "UncheckedBlock" {
                write_token(printer, stream, "unchecked");
                write_space(printer, stream);
            }
        }
        write_token(printer, stream, "{");
    }

    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
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
        stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<SolidityAST>,
        _node: &SolidityAST,
    ) {
        write_token(printer, stream, "}");
    }
}

struct VariableDeclarationStatementPrinter {}

impl NodePrinter<SolidityAST> for VariableDeclarationStatementPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        if let Some(declarations) = node.get("declarations") {
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

        if let Some(initial_value) = node.get("initialValue") {
            write_space(printer, stream);
            write_token(printer, stream, "=");
            write_space(printer, stream);
            traverse_sub_node_and_print(printer, stream, factory, initial_value);
        }

        write_token(printer, stream, ";");
    }
}

struct BinaryOperationPrinter {}

impl NodePrinter<SolidityAST> for BinaryOperationPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        if let Some(le) = node.get("leftExpression") {
            traverse_sub_node_and_print(printer, stream, factory, le);
        }

        print_operator_helper(printer, stream, node);

        if let Some(re) = node.get("rightExpression") {
            traverse_sub_node_and_print(printer, stream, factory, re);
        }
    }
}

struct LiteralPrinter {}

impl NodePrinter<SolidityAST> for LiteralPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<SolidityAST>,
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

impl NodePrinter<SolidityAST> for ExpressionStatementPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        if let Some(expression) = node.get("expression") {
            traverse_sub_node_and_print(printer, stream, factory, expression);
        }
    }

    fn on_exit(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<SolidityAST>,
        _node: &SolidityAST,
    ) {
        if self.write_statement_semicolon {
            write_token(printer, stream, ";");
        }
    }
}

struct AssignmentPrinter {}

impl NodePrinter<SolidityAST> for AssignmentPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        if let Some(lhs) = node.get("leftHandSide") {
            traverse_sub_node_and_print(printer, stream, factory, lhs);
        }

        print_operator_helper(printer, stream, node);

        if let Some(rhs) = node.get("rightHandSide") {
            traverse_sub_node_and_print(printer, stream, factory, rhs);
        }
    }
}

struct IdentifierPrinter {}

impl NodePrinter<SolidityAST> for IdentifierPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        if let Some(name) = node.get_str_for_key("name") {
            write_token(printer, stream, name);
        }
    }
}

struct FunctionCallPrinter {}

impl NodePrinter<SolidityAST> for FunctionCallPrinter {
    fn on_entry(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        if let Some(expression) = node.get("expression") {
            traverse_sub_node_and_print(printer, stream, factory, expression);
            write_token(printer, stream, "(");
        }
    }

    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        if let Some(names_array) = node.get_array_for_key("names") {
            if !names_array.is_empty() {
                if let Some(arguments_array) = node.get_array_for_key("arguments") {
                    assert_eq!(
                        names_array.len(),
                        arguments_array.len(),
                        "names and arguments arrays have different lengths in FunctionCallPrinter"
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
        stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<SolidityAST>,
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

impl NodePrinter<SolidityAST> for UnaryOperationPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
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

            if let Some(sub_expression) = node.get("subExpression") {
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

impl NodePrinter<SolidityAST> for MemberAccessPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        if let Some(expression) = node.get("expression") {
            traverse_sub_node_and_print(printer, stream, factory, expression);
        }

        write_token(printer, stream, ".");

        if let Some(member_name) = node.get_str_for_key("memberName") {
            write_token(printer, stream, member_name);
        }
    }
}

struct IndexAccessPrinter {}

impl NodePrinter<SolidityAST> for IndexAccessPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        if let Some(base_expression) = node.get("baseExpression") {
            traverse_sub_node_and_print(printer, stream, factory, base_expression);
        }

        write_token(printer, stream, "[");

        if let Some(index_expression) = node.get("indexExpression") {
            traverse_sub_node_and_print(printer, stream, factory, index_expression);
        }

        write_token(printer, stream, "]");
    }
}

struct IfStatementPrinter {}

impl NodePrinter<SolidityAST> for IfStatementPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        write_token(printer, stream, "if");
        write_token(printer, stream, "(");

        if let Some(condition) = node.get("condition") {
            traverse_sub_node_and_print(printer, stream, factory, condition);
        }

        write_token(printer, stream, ")");
        write_space(printer, stream);

        if let Some(true_body) = node.get("trueBody") {
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

        if let Some(false_body) = node.get("falseBody") {
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

impl NodePrinter<SolidityAST> for ImportDirectivePrinter {
    fn on_entry(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<SolidityAST>,
        _node: &SolidityAST,
    ) {
        write_token(printer, stream, "import");
    }

    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        let mut wrote_symbol_aliases = false;
        if let Some(symbol_aliases) = node.get("symbolAliases") {
            if let Some(aliases_array) = symbol_aliases.as_array() {
                if !aliases_array.is_empty() {
                    write_space(printer, stream);
                    write_token(printer, stream, "{");
                    let mut i = 0;
                    while i < aliases_array.len() {
                        if let Some(alias) = aliases_array.get(i) {
                            if let Some(foreign_symbol) = alias.get("foreign") {
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
            if !unit_alias_str.is_empty() {
                write_space(printer, stream);
                write_token(printer, stream, "as");
                write_space(printer, stream);
                write_token(printer, stream, unit_alias_str);
            }
        }

        write_token(printer, stream, ";");
    }
}

struct InheritanceSpecifierPrinter {}

impl NodePrinter<SolidityAST> for InheritanceSpecifierPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        if let Some(base_name) = node.get("baseName") {
            traverse_sub_node_and_print(printer, stream, factory, base_name);
        }
        if let Some(arguments_array) = node.get_array_for_key("arguments") {
            write_token(printer, stream, "(");
            print_array_helper(printer, stream, factory, arguments_array);
            write_token(printer, stream, ")");
        }
    }
}

struct ModifierInvocationPrinter {}

impl NodePrinter<SolidityAST> for ModifierInvocationPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        if let Some(modifier_name) = node.get("modifierName") {
            traverse_sub_node_and_print(printer, stream, factory, modifier_name);
        }

        if let Some(arguments) = node.get("arguments") {
            if let Some(arguments_array) = arguments.as_array() {
                write_token(printer, stream, "(");
                if !arguments_array.is_empty() {
                    print_array_helper(printer, stream, factory, arguments_array);
                }
                write_token(printer, stream, ")");
            }
        }
    }
}

struct UsingForDirectivePrinter {}

impl NodePrinter<SolidityAST> for UsingForDirectivePrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        write_token(printer, stream, "using");
        if let Some(library_name) = node.get("libraryName") {
            write_space(printer, stream);
            traverse_sub_node_and_print(printer, stream, factory, library_name);
        }
        if let Some(function_list_node) = node.get("functionList") {
            if let Some(function_list_array) = function_list_node.as_array() {
                if !function_list_array.is_empty() {
                    write_space(printer, stream);
                    write_token(printer, stream, "{");
                    printer.increase_indent();
                    write_newline(printer, stream);
                    write_indent(printer, stream);
                    print_array_helper_with_node_handler_and_space_formatter(
                        printer,
                        stream,
                        factory,
                        function_list_array,
                        |p, s, _f, n| {
                            if let Some(definition) = n.get("definition") {
                                if let Some(name) = definition.get_str_for_key("name") {
                                    write_token(p, s, name);
                                }
                            }
                            if let Some(operator) = n.get_str_for_key("operator") {
                                write_space(p, s);
                                write_token(p, s, "as");
                                write_space(p, s);
                                write_token(p, s, operator);
                            }
                        },
                        |p, s, _f| {
                            write_token(p, s, ",");
                            write_newline(p, s);
                            write_indent(p, s);
                        },
                    );
                    write_newline(printer, stream);
                    printer.decrease_indent();
                    write_indent(printer, stream);
                    write_token(printer, stream, "}");
                }
            }
        }
        write_space(printer, stream);
        write_token(printer, stream, "for");
        write_space(printer, stream);
        if let Some(type_name) = node.get("typeName") {
            traverse_sub_node_and_print(printer, stream, factory, type_name);
        }

        if let Some(global_bool) = node.get_bool_for_key("global") {
            if global_bool {
                write_space(printer, stream);
                write_token(printer, stream, "global");
            }
        }

        write_token(printer, stream, ";");
    }
}

struct ModifierDefinitionPrinter {}

impl NodePrinter<SolidityAST> for ModifierDefinitionPrinter {
    fn on_entry(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        print_documentation_helper(printer, stream, factory, node);
        write_token(printer, stream, "modifier");
        write_space(printer, stream);
        print_name_helper(printer, stream, node);
        if let Some(parameters) = node.get("parameters") {
            traverse_sub_node_and_print(printer, stream, factory, parameters);
        }
    }

    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        if let Some(virtual_bool) = node.get_bool_for_key("virtual") {
            if virtual_bool {
                write_space(printer, stream);
                write_token(printer, stream, "virtual");
            }
        }

        if let Some(override_node) = node.get("overrides") {
            traverse_sub_node_and_print(printer, stream, factory, override_node);
        }

        if let Some(body) = node.get("body") {
            write_space(printer, stream);
            traverse_sub_node_and_print(printer, stream, factory, body);
        }
    }
}

struct StructuredDocumentationPrinter {}

impl NodePrinter<SolidityAST> for StructuredDocumentationPrinter {
    fn on_entry(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<SolidityAST>,
        _node: &SolidityAST,
    ) {
        write_token(printer, stream, "/**");
    }

    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        if let Some(text) = node.get_str_for_key("text") {
            let text_array: Vec<&str> = text.split('\n').collect();
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
        stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<SolidityAST>,
        _node: &SolidityAST,
    ) {
        write_newline(printer, stream);
        write_indent(printer, stream);
        write_space(printer, stream);
        write_token(printer, stream, "*/");
    }
}

struct PlaceholderStatementPrinter {}

impl NodePrinter<SolidityAST> for PlaceholderStatementPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<SolidityAST>,
        _node: &SolidityAST,
    ) {
        write_token(printer, stream, "_");
        write_token(printer, stream, ";");
    }
}

struct ReturnPrinter {}

impl NodePrinter<SolidityAST> for ReturnPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        write_token(printer, stream, "return");
        if let Some(expression) = node.get("expression") {
            write_space(printer, stream);
            traverse_sub_node_and_print(printer, stream, factory, expression);
        }
        write_token(printer, stream, ";");
    }
}

struct EmitStatementPrinter {}

impl NodePrinter<SolidityAST> for EmitStatementPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        if let Some(event_call) = node.get("eventCall") {
            write_token(printer, stream, "emit");
            write_space(printer, stream);
            traverse_sub_node_and_print(printer, stream, factory, event_call);
            write_token(printer, stream, ";");
        }
    }
}

struct OverrideSpecifierPrinter {}

impl NodePrinter<SolidityAST> for OverrideSpecifierPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        write_token(printer, stream, "override");
        if let Some(overrides_array) = node.get_array_for_key("overrides") {
            if !overrides_array.is_empty() {
                write_token(printer, stream, "(");
                print_array_helper(printer, stream, factory, overrides_array);
                write_token(printer, stream, ")");
            }
        }
    }
}

struct ElementaryTypeNameExpressionPrinter {}

impl NodePrinter<SolidityAST> for ElementaryTypeNameExpressionPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        // Not sure if this is the correct way to handle this node.
        if let Some(pure) = node.get_bool_for_key("isPure") {
            if pure {
                if let Some(type_name) = node.get("typeName") {
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

impl NodePrinter<SolidityAST> for TupleExpressionPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        if let Some(components) = node.get("components") {
            if let Some(components_array) = components.as_array() {
                if !components_array.is_empty() {
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

impl NodePrinter<SolidityAST> for CommentPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        write_token(printer, stream, "//");
        write_space(printer, stream);

        if let Some(comment_text) = node.get_str_for_key("text") {
            if !comment_text.is_empty() {
                write_flowable_text(printer, stream, comment_text, "// ");
            }
        }

        if let Some(value_node) = node.get("value") {
            traverse_sub_node_and_print(printer, stream, factory, value_node);
        }
    }
}

struct ErrorDefinitionPrinter {}

impl NodePrinter<SolidityAST> for ErrorDefinitionPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        write_token(printer, stream, "error");
        write_space(printer, stream);
        if let Some(name_str) = node.get_str_for_key("name") {
            write_token(printer, stream, name_str);
        }
        if let Some(parameters_node) = node.get("parameters") {
            traverse_sub_node_and_print(printer, stream, factory, parameters_node);
        }
        write_token(printer, stream, ";");
    }
}

struct ForStatementPrinter {}

impl NodePrinter<SolidityAST> for ForStatementPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        // Include the '(' in the token so it doesn't line break to the next line.
        write_token(printer, stream, "for(");
        if let Some(initialization_node) = node.get("initializationExpression") {
            traverse_sub_node_and_print(printer, stream, factory, initialization_node);
        }
        if let Some(condition_node) = node.get("condition") {
            write_space(printer, stream);
            traverse_sub_node_and_print(printer, stream, factory, condition_node);
            write_token(printer, stream, ";");
        }
        if let Some(loop_expression_node) = node.get("loopExpression") {
            let mut printer_settings = factory.get_settings().clone();
            printer_settings.set_value_for_key(
                WRITE_EXPRESSION_STATEMENT_SEMICOLON,
                PreferenceValue::Boolean(false),
            );
            let sub_factory = SolidityNodePrinterFactory::new(printer_settings);
            write_space(printer, stream);
            traverse_sub_node_and_print(printer, stream, &sub_factory, loop_expression_node);
        }
        write_token(printer, stream, ")");
        if let Some(body_node) = node.get("body") {
            if let Some(node_type_str) = body_node.get_str_for_key("nodeType") {
                if node_type_str == "Block" {
                    write_space(printer, stream);
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

impl NodePrinter<SolidityAST> for ArrayTypeNamePrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        if let Some(base_type_node) = node.get("baseType") {
            traverse_sub_node_and_print(printer, stream, factory, base_type_node);
        }
        write_token(printer, stream, "[");
        if let Some(length_node) = node.get("length") {
            traverse_sub_node_and_print(printer, stream, factory, length_node);
        }
        write_token(printer, stream, "]");
    }
}

struct InlineAssemblyPrinter {}

impl NodePrinter<SolidityAST> for InlineAssemblyPrinter {
    fn on_entry(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<SolidityAST>,
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
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        write_token(printer, stream, "assembly");
        write_space(printer, stream);
        if let Some(flags_node) = node.get("flags") {
            if let Some(flags_array) = flags_node.as_array() {
                if !flags_array.is_empty() {
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
        if let Some(ast_node) = node.get("AST") {
            traverse_sub_node_and_print(printer, stream, factory, ast_node);
        }
    }
}

struct YulVariableDeclarationPrinter {}

impl NodePrinter<SolidityAST> for YulVariableDeclarationPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        write_token(printer, stream, "let");
        write_space(printer, stream);
        if let Some(variables_node) = node.get("variables") {
            if let Some(variables_array) = variables_node.as_array() {
                print_array_helper(printer, stream, factory, variables_array);
                write_space(printer, stream);
            }
        }
        write_token(printer, stream, ":=");
        write_space(printer, stream);
        if let Some(value_node) = node.get("value") {
            traverse_sub_node_and_print(printer, stream, factory, value_node);
        }
    }
}

struct YulFunctionCallPrinter {}

impl NodePrinter<SolidityAST> for YulFunctionCallPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        if let Some(function_name_node) = node.get("functionName") {
            traverse_sub_node_and_print(printer, stream, factory, function_name_node);
        }
        write_token(printer, stream, "(");
        print_arguments_helper(printer, stream, factory, node);
        write_token(printer, stream, ")");
    }
}

/// Used for nodes that need to just print the string in the 'name' parameter.
/// * EnumValue
/// * IdentifierPath
/// * YulIdentifier
/// * YulTypedName
struct NamePrinter {}

impl NodePrinter<SolidityAST> for NamePrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        print_name_helper(printer, stream, node);
    }
}

struct YulAssignmentPrinter {}

impl NodePrinter<SolidityAST> for YulAssignmentPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        if let Some(variable_names_node) = node.get("variableNames") {
            if let Some(variable_names_array) = variable_names_node.as_array() {
                print_array_helper(printer, stream, factory, variable_names_array);
                write_space(printer, stream);
            }
        }
        write_token(printer, stream, ":=");
        write_space(printer, stream);
        if let Some(value_node) = node.get("value") {
            traverse_sub_node_and_print(printer, stream, factory, value_node);
        }
    }
}

struct YulLiteralPrinter {}

impl NodePrinter<SolidityAST> for YulLiteralPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        if let Some(kind_str) = node.get_str_for_key("kind") {
            if kind_str == "number" || kind_str == "bool" {
                if let Some(value_str) = node.get_str_for_key("value") {
                    write_token(printer, stream, value_str);
                }
            } else if kind_str == "string" {
                if let Some(value_str) = node.get_str_for_key("value") {
                    write_string(printer, stream, value_str);
                }
            }
        }
    }
}

struct YulForLoopPrinter {}

impl NodePrinter<SolidityAST> for YulForLoopPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        write_token(printer, stream, "for");
        printer.increase_indent();

        let mut printing_settings = factory.get_settings().clone();
        printing_settings.set_value_for_key(
            SINGLE_BLOCK_STATEMENTS_ON_SAME_LINE,
            PreferenceValue::Boolean(true),
        );
        let sub_factory = SolidityNodePrinterFactory::new(printing_settings);

        if let Some(pre_node) = node.get("pre") {
            write_newline(printer, stream);
            write_indent(printer, stream);
            traverse_sub_node_and_print(printer, stream, &sub_factory, pre_node);
        }
        if let Some(condition_node) = node.get("condition") {
            write_newline(printer, stream);
            write_indent(printer, stream);
            traverse_sub_node_and_print(printer, stream, factory, condition_node);
        }
        if let Some(post_node) = node.get("post") {
            write_newline(printer, stream);
            write_indent(printer, stream);
            traverse_sub_node_and_print(printer, stream, &sub_factory, post_node);
        }
        printer.decrease_indent();
        write_newline(printer, stream);
        write_indent(printer, stream);
        if let Some(body_node) = node.get("body") {
            traverse_sub_node_and_print(printer, stream, factory, body_node);
        }
    }
}

struct FunctionTypeNamePrinter {}

impl NodePrinter<SolidityAST> for FunctionTypeNamePrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        write_token(printer, stream, "function");
        if let Some(parameter_types_node) = node.get("parameterTypes") {
            traverse_sub_node_and_print(printer, stream, factory, parameter_types_node);
        }
        if let Some(visibility_str) = node.get_str_for_key("visibility") {
            if visibility_str == "external" {
                write_space(printer, stream);
                write_token(printer, stream, visibility_str);
            }
        }

        print_return_parameters_helper(printer, stream, factory, node);
    }
}

struct EventDefinitionPrinter {}

impl NodePrinter<SolidityAST> for EventDefinitionPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        write_token(printer, stream, "event");
        write_space(printer, stream);
        print_name_helper(printer, stream, node);
        if let Some(parameters_node) = node.get("parameters") {
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

impl NodePrinter<SolidityAST> for RevertStatementPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        write_token(printer, stream, "revert");
        write_space(printer, stream);
        if let Some(error_call_node) = node.get("errorCall") {
            traverse_sub_node_and_print(printer, stream, factory, error_call_node);
        }
        write_token(printer, stream, ";");
    }
}

struct EnumDefinitionPrinter {}

impl NodePrinter<SolidityAST> for EnumDefinitionPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        write_token(printer, stream, "enum");
        write_space(printer, stream);
        print_name_helper(printer, stream, node);
        if let Some(members_node) = node.get("members") {
            if let Some(members_array) = members_node.as_array() {
                if !members_array.is_empty() {
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

struct UserDefinedValueTypeDefinitionPrinter {}

impl NodePrinter<SolidityAST> for UserDefinedValueTypeDefinitionPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        write_token(printer, stream, "type");
        write_space(printer, stream);
        print_name_helper(printer, stream, node);
        write_space(printer, stream);
        write_token(printer, stream, "is");
        write_space(printer, stream);
        if let Some(underlying_node) = node.get("underlyingType") {
            traverse_sub_node_and_print(printer, stream, factory, underlying_node);
        }
        write_token(printer, stream, ";");
    }
}

struct NewExpressionPrinter {}

impl NodePrinter<SolidityAST> for NewExpressionPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        write_token(printer, stream, "new");
        if let Some(type_name_node) = node.get("typeName") {
            write_space(printer, stream);
            traverse_sub_node_and_print(printer, stream, factory, type_name_node);
        }
    }
}

struct ConditionalPrinter {}

impl NodePrinter<SolidityAST> for ConditionalPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        if let Some(condition_node) = node.get("condition") {
            traverse_sub_node_and_print(printer, stream, factory, condition_node);
        }
        write_space(printer, stream);
        write_token(printer, stream, "?");
        write_space(printer, stream);

        if let Some(true_expr_node) = node.get("trueExpression") {
            traverse_sub_node_and_print(printer, stream, factory, true_expr_node);
        }

        write_space(printer, stream);
        write_token(printer, stream, ":");
        write_space(printer, stream);

        if let Some(false_expr_node) = node.get("falseExpression") {
            traverse_sub_node_and_print(printer, stream, factory, false_expr_node);
        }
    }
}

struct FunctionCallOptionsPrinter {}

impl NodePrinter<SolidityAST> for FunctionCallOptionsPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        if let Some(expr_node) = node.get("expression") {
            traverse_sub_node_and_print(printer, stream, factory, expr_node);
        }

        if let Some(names_array) = node.get_array_for_key("names") {
            if let Some(options_array) = node.get_array_for_key("options") {
                assert_eq!(
                    names_array.len(),
                    options_array.len(),
                    "names and options array lengths differ in FunctionCallOptionsPrinter"
                );

                if !names_array.is_empty() {
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

impl NodePrinter<SolidityAST> for TryStatementPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        write_token(printer, stream, "try");
        write_space(printer, stream);
        if let Some(external_call_node) = node.get("externalCall") {
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

impl NodePrinter<SolidityAST> for TryCatchClausePrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        if let Some(error_name_str) = node.get_str_for_key("errorName") {
            if !error_name_str.is_empty() {
                write_token(printer, stream, error_name_str);
            }
        }

        if let Some(parameters_node) = node.get("parameters") {
            traverse_sub_node_and_print(printer, stream, factory, parameters_node);
            write_space(printer, stream);
        }

        if let Some(block_node) = node.get("block") {
            traverse_sub_node_and_print(printer, stream, factory, block_node);
        }
    }
}

struct WhileStatementPrinter {}

impl NodePrinter<SolidityAST> for WhileStatementPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        write_token(printer, stream, "while");
        write_space(printer, stream);
        if let Some(condition_node) = node.get("condition") {
            write_token(printer, stream, "(");
            traverse_sub_node_and_print(printer, stream, factory, condition_node);
            write_token(printer, stream, ")");
        }
        if let Some(body_node) = node.get("body") {
            write_space(printer, stream);
            traverse_sub_node_and_print(printer, stream, factory, body_node);
        }
    }
}

struct DoWhileStatementPrinter {}

impl NodePrinter<SolidityAST> for DoWhileStatementPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        write_token(printer, stream, "do");
        if let Some(body_node) = node.get("body") {
            write_space(printer, stream);
            traverse_sub_node_and_print(printer, stream, factory, body_node);
        }
        if let Some(condition_node) = node.get("condition") {
            write_space(printer, stream);
            write_token(printer, stream, "while");
            write_space(printer, stream);
            write_token(printer, stream, "(");
            traverse_sub_node_and_print(printer, stream, factory, condition_node);
            write_token(printer, stream, ")");
        }
        write_token(printer, stream, ";");
    }
}

struct ContinuePrinter {}

impl NodePrinter<SolidityAST> for ContinuePrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<SolidityAST>,
        _node: &SolidityAST,
    ) {
        write_token(printer, stream, "continue");
        write_token(printer, stream, ";");
    }
}

struct BreakPrinter {}

impl NodePrinter<SolidityAST> for BreakPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<SolidityAST>,
        _node: &SolidityAST,
    ) {
        write_token(printer, stream, "break");
        write_token(printer, stream, ";");
    }
}

struct YulIfPrinter {}

impl NodePrinter<SolidityAST> for YulIfPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        write_token(printer, stream, "if");
        write_space(printer, stream);
        if let Some(condition_node) = node.get("condition") {
            traverse_sub_node_and_print(printer, stream, factory, condition_node);
        }
        if let Some(body_node) = node.get("body") {
            write_space(printer, stream);
            traverse_sub_node_and_print(printer, stream, factory, body_node);
        }
    }
}

struct YulSwitchPrinter {}

impl NodePrinter<SolidityAST> for YulSwitchPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        write_token(printer, stream, "switch");
        write_space(printer, stream);
        if let Some(expression_node) = node.get("expression") {
            traverse_sub_node_and_print(printer, stream, factory, expression_node);
            write_space(printer, stream);
        }
        if let Some(cases_array) = node.get_array_for_key("cases") {
            if !cases_array.is_empty() {
                printer.increase_indent();
                for value in cases_array {
                    write_newline(printer, stream);
                    write_indent(printer, stream);
                    traverse_sub_node_and_print(printer, stream, factory, value);
                }
                printer.decrease_indent();
            }
        }
    }
}

struct YulCasePrinter {}

impl NodePrinter<SolidityAST> for YulCasePrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        if let Some(value_node) = node.get("value") {
            if value_node.is_object() {
                write_token(printer, stream, "case");
                write_space(printer, stream);
                traverse_sub_node_and_print(printer, stream, factory, value_node);
                write_space(printer, stream);
            } else if let Some(value_string) = value_node.as_str() {
                if value_string == "default" {
                    write_token(printer, stream, value_string);
                    write_space(printer, stream);
                }
            }
        }
        if let Some(body_node) = node.get("body") {
            traverse_sub_node_and_print(printer, stream, factory, body_node);
        }
    }
}

struct YulFunctionDefinitionPrinter {}

impl NodePrinter<SolidityAST> for YulFunctionDefinitionPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        factory: &dyn NodePrinterFactory<SolidityAST>,
        node: &SolidityAST,
    ) {
        write_token(printer, stream, "function");
        write_space(printer, stream);
        print_name_helper(printer, stream, node);
        write_token(printer, stream, "(");
        if let Some(parameters_array) = node.get_array_for_key("parameters") {
            print_array_helper(printer, stream, factory, parameters_array);
        }
        write_token(printer, stream, ")");
        write_space(printer, stream);
        if let Some(return_vars_array) = node.get_array_for_key("returnVariables") {
            if !return_vars_array.is_empty() {
                write_token(printer, stream, "->");
                write_space(printer, stream);
                if return_vars_array.len() > 1 {
                    write_token(printer, stream, "(");
                }

                print_array_helper(printer, stream, factory, return_vars_array);

                if return_vars_array.len() > 1 {
                    write_token(printer, stream, ")");
                }
                write_space(printer, stream);
            }
        }
        if let Some(body_node) = node.get("body") {
            traverse_sub_node_and_print(printer, stream, factory, body_node);
        }
    }
}

struct YulBreakPrinter {}

impl NodePrinter<SolidityAST> for YulBreakPrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<SolidityAST>,
        _node: &SolidityAST,
    ) {
        write_token(printer, stream, "break");
    }
}

struct YulContinuePrinter {}

impl NodePrinter<SolidityAST> for YulContinuePrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<SolidityAST>,
        _node: &SolidityAST,
    ) {
        write_token(printer, stream, "continue");
    }
}

struct YulLeavePrinter {}
impl NodePrinter<SolidityAST> for YulLeavePrinter {
    fn print_node(
        &mut self,
        printer: &mut PrettyPrinter,
        stream: &mut dyn Write,
        _factory: &dyn NodePrinterFactory<SolidityAST>,
        _node: &SolidityAST,
    ) {
        write_token(printer, stream, "leave");
    }
}

static WRITE_EXPRESSION_STATEMENT_SEMICOLON: &str = "write_expression_statement_semicolon";
static SINGLE_BLOCK_STATEMENTS_ON_SAME_LINE: &str = "single_block_statements_on_same_line";
static WRITE_NONPAYABLE_STATE_MUTABILITY: &str = "write_nonpayable_state_mutability";

/// Type that implements [`NodePrinterFactory<AST>`] for Solidity nodes.
///
/// Use this factory object with the [`crate::pretty_print_visitor::PrettyPrintVisitor<AST>`] object.
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
}

impl NodePrinterFactory<SolidityAST> for SolidityNodePrinterFactory {
    fn printer_for(&self, node: &SolidityAST) -> Box<dyn NodePrinter<SolidityAST>> {
        if let Some(node_type) = node.get_str_for_key("nodeType") {
            match node_type {
                "ArrayTypeName" => Box::new(ArrayTypeNamePrinter {}),
                "Assignment" => Box::new(AssignmentPrinter {}),
                "BinaryOperation" => Box::new(BinaryOperationPrinter {}),
                "Block" => Box::new(BlockPrinter::new(
                    <SolidityNodePrinterFactory as NodePrinterFactory<Value>>::get_preference_value_for_key(self, SINGLE_BLOCK_STATEMENTS_ON_SAME_LINE),
                )),
                "Break" => Box::new(BreakPrinter {}),
                "Comment" => Box::new(CommentPrinter {}),
                "Conditional" => Box::new(ConditionalPrinter {}),
                "Continue" => Box::new(ContinuePrinter {}),
                "ContractDefinition" => Box::new(ContractDefinitionPrinter {}),
                "DoWhileStatement" => Box::new(DoWhileStatementPrinter {}),
                "ElementaryTypeName" => Box::new(ElementaryTypeNamePrinter {}),
                "ElementaryTypeNameExpression" => Box::new(ElementaryTypeNameExpressionPrinter {}),
                "EmitStatement" => Box::new(EmitStatementPrinter {}),
                "EnumDefinition" => Box::new(EnumDefinitionPrinter {}),
                "EnumValue" => Box::new(NamePrinter {}),
                "ErrorDefinition" => Box::new(ErrorDefinitionPrinter {}),
                "EventDefinition" => Box::new(EventDefinitionPrinter {}),
                "ExpressionStatement" => Box::new(ExpressionStatementPrinter::new(
                    <SolidityNodePrinterFactory as NodePrinterFactory<Value>>::get_preference_value_for_key(self, WRITE_EXPRESSION_STATEMENT_SEMICOLON),
                )),
                "ForStatement" => Box::new(ForStatementPrinter {}),
                "FunctionCall" => Box::new(FunctionCallPrinter {}),
                "FunctionCallOptions" => Box::new(FunctionCallOptionsPrinter {}),
                "FunctionDefinition" => Box::new(FunctionDefinitionPrinter::new(
                    <SolidityNodePrinterFactory as NodePrinterFactory<Value>>::get_preference_value_for_key(self, WRITE_NONPAYABLE_STATE_MUTABILITY),
                )),
                "FunctionTypeName" => Box::new(FunctionTypeNamePrinter {}),
                "Identifier" => Box::new(IdentifierPrinter {}),
                "IdentifierPath" => Box::new(NamePrinter {}),
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
                "UnaryOperation" => Box::new(UnaryOperationPrinter {}),
                "UncheckedBlock" => Box::new(BlockPrinter::new(
                    <SolidityNodePrinterFactory as NodePrinterFactory<Value>>::get_preference_value_for_key(self, SINGLE_BLOCK_STATEMENTS_ON_SAME_LINE),
                )),
                "UserDefinedTypeName" => Box::new(UserDefinedTypeNamePrinter {}),
                "UserDefinedValueTypeDefinition" => {
                    Box::new(UserDefinedValueTypeDefinitionPrinter {})
                }
                "UsingForDirective" => Box::new(UsingForDirectivePrinter {}),
                "VariableDeclaration" => Box::new(VariableDeclarationPrinter {}),
                "VariableDeclarationStatement" => Box::new(VariableDeclarationStatementPrinter {}),
                "WhileStatement" => Box::new(WhileStatementPrinter {}),
                "YulAssignment" => Box::new(YulAssignmentPrinter {}),
                "YulBlock" => Box::new(BlockPrinter::new(
                    <SolidityNodePrinterFactory as NodePrinterFactory<Value>>::get_preference_value_for_key(self, SINGLE_BLOCK_STATEMENTS_ON_SAME_LINE),
                )),
                "YulBreak" => Box::new(YulBreakPrinter {}),
                "YulCase" => Box::new(YulCasePrinter {}),
                "YulContinue" => Box::new(YulContinuePrinter {}),
                "YulForLoop" => Box::new(YulForLoopPrinter {}),
                "YulFunctionCall" => Box::new(YulFunctionCallPrinter {}),
                "YulFunctionDefinition" => Box::new(YulFunctionDefinitionPrinter {}),
                "YulIdentifier" => Box::new(NamePrinter {}),
                "YulIf" => Box::new(YulIfPrinter {}),
                "YulLeave" => Box::new(YulLeavePrinter {}),
                "YulLiteral" => Box::new(YulLiteralPrinter {}),
                "YulSwitch" => Box::new(YulSwitchPrinter {}),
                "YulTypedName" => Box::new(NamePrinter {}),
                "YulVariableDeclaration" => Box::new(YulVariableDeclarationPrinter {}),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_digits_and_or_dots() {
        let s1 = "0.2.345";
        assert_eq!(contains_digits_and_or_dots(s1), true);
        assert_eq!(contains_digits_and_or_dots("23.f.234"), false);
        assert_eq!(contains_digits_and_or_dots(""), false);
    }
}
