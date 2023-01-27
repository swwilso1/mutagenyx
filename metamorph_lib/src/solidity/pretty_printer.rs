//! The `solidity::pretty_printer` module provides the code to traverse a SolidityAST and print out
//! the source code represented in the AST.

use crate::json::JSONMutate;
use crate::node_printer::{NodePrinter, NodePrinterFactory};
use crate::node_printer_helpers::*;
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
/// * `printer` - The [`PrettyPrinter`] object that will write to `stream`.
fn print_documentation_helper<W: Write>(
    printer: &mut PrettyPrinter,
    stream: &mut W,
    node: &SolidityAST,
) {
    if let Some(docs) = node.borrow_value_for_key("documentation") {
        traverse_sub_node_and_print(printer, stream, SolidityNodePrinterFactory {}, docs);
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
    node: &SolidityAST,
) {
    if let Some(array) = node.as_array() {
        if array.len() > 0 {
            write_space(printer, stream);
            print_array_helper(printer, stream, SolidityNodePrinterFactory {}, array);
        }
    }
}

/// For Block and UncheckedBlock nodes, print out the statements array.
///
/// # Arguments
///
/// * `printer` - The [`PrettyPrinter`] object that writes formatted text to `stream`.
/// * `stream` - The [`Write`] object that receives the formatted text.
/// * `node` - The [`SolidityAST`] node containing statements.
fn print_statements_helper<W: Write>(
    printer: &mut PrettyPrinter,
    stream: &mut W,
    node: &SolidityAST,
) {
    if let Some(statements) = node.borrow_value_for_key("statements") {
        if let Some(statements_array) = statements.as_array() {
            if statements_array.len() > 0 {
                printer.increase_indent();
                write_newline(printer, stream);

                let mut i = 0;
                while i < statements_array.len() {
                    if let Some(statement) = statements_array.get(i) {
                        write_indent(printer, stream);
                        traverse_sub_node_and_print(
                            printer,
                            stream,
                            SolidityNodePrinterFactory {},
                            statement,
                        );
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

/// Default node printer for unsupported nodes.
struct DummyNodePrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for DummyNodePrinter {
    fn visit_children(&mut self) -> bool {
        true
    }
}

struct SourceUnitPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for SourceUnitPrinter {
    fn on_entry(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
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

    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        if let Some(nodes) = node.borrow_value_for_key("nodes") {
            if let Some(node_array) = nodes.as_array() {
                let mut i = 0;
                while i < node_array.len() {
                    if let Some(node) = node_array.get(i) {
                        write_indent(printer, stream);
                        traverse_sub_node_and_print(
                            printer,
                            stream,
                            SolidityNodePrinterFactory {},
                            node,
                        );
                        if i < (node_array.len() - 1) {
                            write_newline(printer, stream);
                            write_newline(printer, stream);
                        }
                    }
                    i += 1;
                }
            }
        }
    }

    fn on_exit(&mut self, stream: &mut W, _node: &SolidityAST, printer: &mut PrettyPrinter) {
        write_newline(printer, stream);
    }
}

struct PragmaDirectivePrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for PragmaDirectivePrinter {
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
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

    fn on_exit(&mut self, stream: &mut W, _node: &SolidityAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, ";");
    }
}

struct ContractDefinitionPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for ContractDefinitionPrinter {
    fn on_entry(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        print_documentation_helper(printer, stream, node);

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
                    print_array_helper(
                        printer,
                        stream,
                        SolidityNodePrinterFactory {},
                        contract_array,
                    );
                }
            }
        }

        write_space(printer, stream);
        write_token(printer, stream, "{");
        printer.increase_indent();
        write_newline(printer, stream);
    }

    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        if let Some(nodes) = node.borrow_value_for_key("nodes") {
            if let Some(node_array) = nodes.as_array() {
                let mut i = 0;
                while i < node_array.len() {
                    if let Some(contract_node) = node_array.get(i) {
                        write_indent(printer, stream);
                        traverse_sub_node_and_print(
                            printer,
                            stream,
                            SolidityNodePrinterFactory {},
                            contract_node,
                        );
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
            }
        }
    }

    fn on_exit(&mut self, stream: &mut W, _node: &SolidityAST, printer: &mut PrettyPrinter) {
        close_block_exit_helper(printer, stream);
    }
}

struct StructDefinitionPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for StructDefinitionPrinter {
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
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
                        traverse_sub_node_and_print(
                            printer,
                            stream,
                            SolidityNodePrinterFactory {},
                            member,
                        );
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

    fn on_exit(&mut self, stream: &mut W, _node: &SolidityAST, printer: &mut PrettyPrinter) {
        close_block_exit_helper(printer, stream);
    }
}

struct ElementaryTypeNamePrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for ElementaryTypeNamePrinter {
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
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
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, "mapping");
        write_token(printer, stream, "(");

        if let Some(key) = node.borrow_value_for_key("keyType") {
            traverse_sub_node_and_print(printer, stream, SolidityNodePrinterFactory {}, key);
        }

        write_space(printer, stream);
        write_token(printer, stream, "=>");
        write_space(printer, stream);

        if let Some(value) = node.borrow_value_for_key("valueType") {
            traverse_sub_node_and_print(printer, stream, SolidityNodePrinterFactory {}, value);
        }

        write_token(printer, stream, ")");
    }
}

struct UserDefinedTypeNamePrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for UserDefinedTypeNamePrinter {
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        if let Some(path_node) = node.borrow_value_for_key("pathNode") {
            print_name_helper(printer, stream, path_node);
        }
    }
}

struct VariableDeclarationPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for VariableDeclarationPrinter {
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        print_documentation_helper(printer, stream, node);
        if let Some(type_name_node) = node.borrow_value_for_key("typeName") {
            traverse_sub_node_and_print(
                printer,
                stream,
                SolidityNodePrinterFactory {},
                type_name_node,
            );
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
    }

    fn on_exit(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        if let Some(name) = node.get_str_for_key("name") {
            if name.len() > 0 {
                write_space(printer, stream);

                if let Some(location) = node.get_str_for_key("storageLocation") {
                    if location == "memory" || location == "storage" {
                        write_token(printer, stream, location);
                        write_space(printer, stream);
                    }
                }

                write_token(printer, stream, name);
            }
        }

        if let Some(value) = node.borrow_value_for_key("value") {
            write_space(printer, stream);
            write_token(printer, stream, "=");
            write_space(printer, stream);
            traverse_sub_node_and_print(printer, stream, SolidityNodePrinterFactory {}, value);
        }
    }
}

struct FunctionDefinitionPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for FunctionDefinitionPrinter {
    fn on_entry(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        if let Some(documentation) = node.borrow_value_for_key("documentation") {
            traverse_sub_node_and_print(
                printer,
                stream,
                SolidityNodePrinterFactory {},
                documentation,
            );
            write_newline(printer, stream);
            write_indent(printer, stream);
        }

        if let Some(kind) = node.get_str_for_key("kind") {
            if kind == "function" {
                write_token(printer, stream, "function");
                write_space(printer, stream);

                if let Some(name) = node.get_str_for_key("name") {
                    write_token(printer, stream, name);
                }
            } else if kind == "constructor" {
                write_token(printer, stream, "constructor");
            } else if kind == "receive" {
                write_token(printer, stream, "receive");
            }
        }
    }

    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        if let Some(parameters) = node.borrow_value_for_key("parameters") {
            traverse_sub_node_and_print(printer, stream, SolidityNodePrinterFactory {}, parameters);
            write_space(printer, stream);
        }

        if let Some(visibility) = node.get_str_for_key("visibility") {
            write_token(printer, stream, visibility);
        }

        if let Some(state_mutability) = node.get_str_for_key("stateMutability") {
            write_space(printer, stream);
            write_token(printer, stream, state_mutability);
        }

        if let Some(overrides) = node.borrow_value_for_key("overrides") {
            traverse_sub_node_and_print(printer, stream, SolidityNodePrinterFactory {}, overrides);
        }

        if let Some(modifiers) = node.borrow_value_for_key("modifiers") {
            print_space_and_array_helper(printer, stream, modifiers);
        }

        if let Some(return_parameters) = node.borrow_value_for_key("returnParameters") {
            if let Some(parameters) = return_parameters.borrow_value_for_key("parameters") {
                if let Some(parameter_array) = parameters.as_array() {
                    if parameter_array.len() > 0 {
                        write_space(printer, stream);
                        write_token(printer, stream, "returns");
                        write_space(printer, stream);
                        traverse_sub_node_and_print(
                            printer,
                            stream,
                            SolidityNodePrinterFactory {},
                            return_parameters,
                        );
                    }
                }
            }
        }

        if let Some(body) = node.borrow_value_for_key("body") {
            write_space(printer, stream);
            traverse_sub_node_and_print(printer, stream, SolidityNodePrinterFactory {}, body);
        }
    }
}

struct ParameterListPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for ParameterListPrinter {
    fn on_entry(&mut self, stream: &mut W, _node: &SolidityAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, "(");
    }

    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        if let Some(parameters) = node.borrow_value_for_key("parameters") {
            if let Some(parameters_array) = parameters.as_array() {
                print_array_helper(
                    printer,
                    stream,
                    SolidityNodePrinterFactory {},
                    parameters_array,
                );
            }
        }
    }

    fn on_exit(&mut self, stream: &mut W, _node: &SolidityAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, ")");
    }
}

struct BlockPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for BlockPrinter {
    fn on_entry(&mut self, stream: &mut W, _node: &SolidityAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, "{");
    }

    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        print_statements_helper(printer, stream, node);
    }

    fn on_exit(&mut self, stream: &mut W, _node: &SolidityAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, "}");
    }
}

struct VariableDeclarationStatementPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for VariableDeclarationStatementPrinter {
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        if let Some(declarations) = node.borrow_value_for_key("declarations") {
            if let Some(declarations_array) = declarations.as_array() {
                let mut i = 0;
                while i < declarations_array.len() {
                    if let Some(declaration) = declarations_array.get(i) {
                        traverse_sub_node_and_print(
                            printer,
                            stream,
                            SolidityNodePrinterFactory {},
                            declaration,
                        );
                        if i < (declarations_array.len() - 1) {
                            write_space(printer, stream);
                        }
                    }
                    i += 1;
                }
            }
        }

        if let Some(initial_value) = node.borrow_value_for_key("initialValue") {
            write_space(printer, stream);
            write_token(printer, stream, "=");
            write_space(printer, stream);
            traverse_sub_node_and_print(
                printer,
                stream,
                SolidityNodePrinterFactory {},
                initial_value,
            );
        }
    }

    fn on_exit(&mut self, stream: &mut W, _node: &SolidityAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, ";");
    }
}

struct BinaryOperationPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for BinaryOperationPrinter {
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        if let Some(le) = node.borrow_value_for_key("leftExpression") {
            traverse_sub_node_and_print(printer, stream, SolidityNodePrinterFactory {}, le);
        }

        print_operator_helper(printer, stream, node);

        if let Some(re) = node.borrow_value_for_key("rightExpression") {
            traverse_sub_node_and_print(printer, stream, SolidityNodePrinterFactory {}, re);
        }
    }
}

struct LiteralPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for LiteralPrinter {
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        if let Some(value) = node.get_str_for_key("value") {
            if let Some(kind) = node.get_str_for_key("kind") {
                if kind == "string" {
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

struct ExpressionStatementPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for ExpressionStatementPrinter {
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        if let Some(expression) = node.borrow_value_for_key("expression") {
            traverse_sub_node_and_print(printer, stream, SolidityNodePrinterFactory {}, expression);
        }
    }

    fn on_exit(&mut self, stream: &mut W, _node: &SolidityAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, ";");
    }
}

struct AssignmentPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for AssignmentPrinter {
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        if let Some(lhs) = node.borrow_value_for_key("leftHandSide") {
            traverse_sub_node_and_print(printer, stream, SolidityNodePrinterFactory {}, lhs);
        }

        print_operator_helper(printer, stream, node);

        if let Some(rhs) = node.borrow_value_for_key("rightHandSide") {
            traverse_sub_node_and_print(printer, stream, SolidityNodePrinterFactory {}, rhs);
        }
    }
}

struct IdentifierPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for IdentifierPrinter {
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        if let Some(name) = node.get_str_for_key("name") {
            write_token(printer, stream, name);
        }
    }
}

struct FunctionCallPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for FunctionCallPrinter {
    fn on_entry(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        if let Some(expression) = node.borrow_value_for_key("expression") {
            traverse_sub_node_and_print(printer, stream, SolidityNodePrinterFactory {}, expression);
            write_token(printer, stream, "(");
        }
    }

    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        if let Some(arguments) = node.borrow_value_for_key("arguments") {
            if let Some(arguments_array) = arguments.as_array() {
                print_array_helper(
                    printer,
                    stream,
                    SolidityNodePrinterFactory {},
                    arguments_array,
                );
            }
        }
    }

    fn on_exit(&mut self, stream: &mut W, _node: &SolidityAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, ")");
    }
}

struct UnaryOperationPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for UnaryOperationPrinter {
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
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
                traverse_sub_node_and_print(
                    printer,
                    stream,
                    SolidityNodePrinterFactory {},
                    sub_expression,
                );
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
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        if let Some(expression) = node.borrow_value_for_key("expression") {
            traverse_sub_node_and_print(printer, stream, SolidityNodePrinterFactory {}, expression);
        }

        write_token(printer, stream, ".");

        if let Some(member_name) = node.get_str_for_key("memberName") {
            write_token(printer, stream, member_name);
        }
    }
}

struct IndexAccessPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for IndexAccessPrinter {
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        if let Some(base_expression) = node.borrow_value_for_key("baseExpression") {
            traverse_sub_node_and_print(
                printer,
                stream,
                SolidityNodePrinterFactory {},
                base_expression,
            );
        }

        write_token(printer, stream, "[");

        if let Some(index_expression) = node.borrow_value_for_key("indexExpression") {
            traverse_sub_node_and_print(
                printer,
                stream,
                SolidityNodePrinterFactory {},
                index_expression,
            );
        }

        write_token(printer, stream, "]");
    }
}

struct IfStatementPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for IfStatementPrinter {
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, "if");
        write_token(printer, stream, "(");

        if let Some(condition) = node.borrow_value_for_key("condition") {
            traverse_sub_node_and_print(printer, stream, SolidityNodePrinterFactory {}, condition);
        }

        write_token(printer, stream, ")");
        write_space(printer, stream);

        if let Some(true_body) = node.borrow_value_for_key("trueBody") {
            traverse_sub_node_and_print(printer, stream, SolidityNodePrinterFactory {}, true_body);
        }

        if let Some(false_body) = node.borrow_value_for_key("falseBody") {
            write_space(printer, stream);
            write_token(printer, stream, "else");
            write_space(printer, stream);
            traverse_sub_node_and_print(printer, stream, SolidityNodePrinterFactory {}, false_body);
        }
    }
}

struct ImportDirectivePrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for ImportDirectivePrinter {
    fn on_entry(&mut self, stream: &mut W, _node: &SolidityAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, "import");
        write_space(printer, stream);
    }

    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        if let Some(symbol_aliases) = node.borrow_value_for_key("symbolAliases") {
            if let Some(aliases_array) = symbol_aliases.as_array() {
                write_token(printer, stream, "{");
                let mut i = 0;
                while i < aliases_array.len() {
                    if let Some(alias) = aliases_array.get(i) {
                        if let Some(foreign_symbol) = alias.borrow_value_for_key("foreign") {
                            traverse_sub_node_and_print(
                                printer,
                                stream,
                                SolidityNodePrinterFactory {},
                                foreign_symbol,
                            );
                        }
                        if i < (aliases_array.len() - 1) {
                            write_token(printer, stream, ",");
                            write_space(printer, stream);
                        }
                    }
                    i += 1;
                }
                write_token(printer, stream, "}");
            }
        }

        write_space(printer, stream);
        write_token(printer, stream, "from");
        write_space(printer, stream);

        if let Some(file) = node.get_str_for_key("file") {
            write_string(printer, stream, file);
        }
    }

    fn on_exit(&mut self, stream: &mut W, _node: &SolidityAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, ";");
    }
}

struct InheritanceSpecifierPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for InheritanceSpecifierPrinter {
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        if let Some(base_name) = node.borrow_value_for_key("baseName") {
            traverse_sub_node_and_print(printer, stream, SolidityNodePrinterFactory {}, base_name);
        }
    }
}

struct IdentifierPathPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for IdentifierPathPrinter {
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        print_name_helper(printer, stream, node);
    }
}

struct ModifierInvocationPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for ModifierInvocationPrinter {
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        if let Some(modifier_name) = node.borrow_value_for_key("modifierName") {
            traverse_sub_node_and_print(
                printer,
                stream,
                SolidityNodePrinterFactory {},
                modifier_name,
            );
        }

        if let Some(arguments) = node.borrow_value_for_key("arguments") {
            if let Some(arguments_array) = arguments.as_array() {
                if arguments_array.len() > 0 {
                    write_token(printer, stream, "(");
                    print_array_helper(
                        printer,
                        stream,
                        SolidityNodePrinterFactory {},
                        arguments_array,
                    );
                    write_token(printer, stream, ")");
                }
            }
        }
    }
}

struct UsingForDirectivePrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for UsingForDirectivePrinter {
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, "using");
        if let Some(library_name) = node.borrow_value_for_key("libraryName") {
            write_space(printer, stream);
            traverse_sub_node_and_print(
                printer,
                stream,
                SolidityNodePrinterFactory {},
                library_name,
            );
        }
        write_space(printer, stream);
        write_token(printer, stream, "for");
        write_space(printer, stream);
        if let Some(type_name) = node.borrow_value_for_key("typeName") {
            traverse_sub_node_and_print(printer, stream, SolidityNodePrinterFactory {}, type_name);
        }
    }

    fn on_exit(&mut self, stream: &mut W, _node: &SolidityAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, ";");
    }
}

struct ModifierDefinitionPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for ModifierDefinitionPrinter {
    fn on_entry(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        print_documentation_helper(printer, stream, node);
        write_token(printer, stream, "modifier");
        write_space(printer, stream);
        print_name_helper(printer, stream, node);
        if let Some(parameters) = node.borrow_value_for_key("parameters") {
            traverse_sub_node_and_print(printer, stream, SolidityNodePrinterFactory {}, parameters);
        }
    }

    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        if let Some(body) = node.borrow_value_for_key("body") {
            write_space(printer, stream);
            traverse_sub_node_and_print(printer, stream, SolidityNodePrinterFactory {}, body);
        }
    }
}

struct StructuredDocumentationPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for StructuredDocumentationPrinter {
    fn on_entry(&mut self, stream: &mut W, _node: &SolidityAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, "/**");
    }

    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
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

    fn on_exit(&mut self, stream: &mut W, _node: &SolidityAST, printer: &mut PrettyPrinter) {
        write_newline(printer, stream);
        write_indent(printer, stream);
        write_space(printer, stream);
        write_token(printer, stream, "*/");
    }
}

struct PlaceholderStatementPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for PlaceholderStatementPrinter {
    fn print_node(&mut self, stream: &mut W, _node: &SolidityAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, "_");
        write_token(printer, stream, ";");
    }
}

struct ReturnPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for ReturnPrinter {
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        if let Some(expression) = node.borrow_value_for_key("expression") {
            write_token(printer, stream, "return");
            write_space(printer, stream);
            traverse_sub_node_and_print(printer, stream, SolidityNodePrinterFactory {}, expression);
            write_token(printer, stream, ";");
        }
    }
}

struct EmitStatementPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for EmitStatementPrinter {
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        if let Some(event_call) = node.borrow_value_for_key("eventCall") {
            write_token(printer, stream, "emit");
            write_space(printer, stream);
            traverse_sub_node_and_print(printer, stream, SolidityNodePrinterFactory {}, event_call);
            write_token(printer, stream, ";");
        }
    }
}

struct OverrideSpecifierPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for OverrideSpecifierPrinter {
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        write_space(printer, stream);
        write_token(printer, stream, "override");
        if let Some(overrides) = node.borrow_value_for_key("overrides") {
            print_space_and_array_helper(printer, stream, overrides);
        }
    }
}

struct ElementaryTypeNameExpressionPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for ElementaryTypeNameExpressionPrinter {
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        // Not sure if this is the correct way to handle this node.
        if let Some(pure) = node.get_bool_for_key("isPure") {
            if pure {
                if let Some(type_name) = node.borrow_value_for_key("typeName") {
                    if let Some(mutability) = type_name.get_str_for_key("stateMutability") {
                        write_token(printer, stream, mutability);
                    } else {
                        traverse_sub_node_and_print(
                            printer,
                            stream,
                            SolidityNodePrinterFactory {},
                            type_name,
                        );
                    }
                }
            }
        }
    }
}

struct TupleExpressionPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for TupleExpressionPrinter {
    fn on_entry(&mut self, stream: &mut W, _node: &SolidityAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, "(");
    }

    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        if let Some(components) = node.borrow_value_for_key("components") {
            if let Some(components_array) = components.as_array() {
                for component in components_array {
                    traverse_sub_node_and_print(
                        printer,
                        stream,
                        SolidityNodePrinterFactory {},
                        component,
                    );
                }
            }
        }
    }

    fn on_exit(&mut self, stream: &mut W, _node: &SolidityAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, ")");
    }
}

/// Provide code for pretty-printing comments.
///
/// **Important**: Comment nodes do not naturally occur in Solidity AST files (not yet) and so
/// any AST with a comment node no longer conforms to the standard AST format generated by the
/// Solidity compiler.
struct CommentPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for CommentPrinter {
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        if let Some(comment_text) = node.get_str_for_key("text") {
            if comment_text.len() > 0 {
                write_token(printer, stream, "//");
                write_space(printer, stream);
                write_flowable_text(printer, stream, comment_text, "// ");
            }
        }
    }
}

struct UncheckedBlockPrinter {}

impl<W: Write> NodePrinter<W, SolidityAST> for UncheckedBlockPrinter {
    fn on_entry(&mut self, stream: &mut W, _node: &SolidityAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, "unchecked");
        write_token(printer, stream, "{");
    }

    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        print_statements_helper(printer, stream, node);
    }

    fn on_exit(&mut self, stream: &mut W, _node: &SolidityAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, "}");
    }
}

/// Type that implements [`NodePrinterFactory<W,AST>`] for Solidity nodes.
///
/// Use this factory object with the [`crate::pretty_print_visitor::PrettyPrintVisitor<W,AST>`] object.
#[derive(Clone)]
pub struct SolidityNodePrinterFactory {}

impl<W: Write> NodePrinterFactory<W, SolidityAST> for SolidityNodePrinterFactory {
    fn printer_for(&self, node: &SolidityAST) -> Box<dyn NodePrinter<W, SolidityAST>> {
        if let Some(node_type) = node.get_str_for_key("nodeType") {
            match node_type {
                "SourceUnit" => Box::new(SourceUnitPrinter {}),
                "PragmaDirective" => Box::new(PragmaDirectivePrinter {}),
                "ContractDefinition" => Box::new(ContractDefinitionPrinter {}),
                "StructDefinition" => Box::new(StructDefinitionPrinter {}),
                "VariableDeclaration" => Box::new(VariableDeclarationPrinter {}),
                "ElementaryTypeName" => Box::new(ElementaryTypeNamePrinter {}),
                "Mapping" => Box::new(MappingPrinter {}),
                "UserDefinedTypeName" => Box::new(UserDefinedTypeNamePrinter {}),
                "FunctionDefinition" => Box::new(FunctionDefinitionPrinter {}),
                "ParameterList" => Box::new(ParameterListPrinter {}),
                "Block" => Box::new(BlockPrinter {}),
                "VariableDeclarationStatement" => Box::new(VariableDeclarationStatementPrinter {}),
                "BinaryOperation" => Box::new(BinaryOperationPrinter {}),
                "Literal" => Box::new(LiteralPrinter {}),
                "ExpressionStatement" => Box::new(ExpressionStatementPrinter {}),
                "Assignment" => Box::new(AssignmentPrinter {}),
                "Identifier" => Box::new(IdentifierPrinter {}),
                "FunctionCall" => Box::new(FunctionCallPrinter {}),
                "UnaryOperation" => Box::new(UnaryOperationPrinter {}),
                "MemberAccess" => Box::new(MemberAccessPrinter {}),
                "IndexAccess" => Box::new(IndexAccessPrinter {}),
                "IfStatement" => Box::new(IfStatementPrinter {}),
                "ImportDirective" => Box::new(ImportDirectivePrinter {}),
                "InheritanceSpecifier" => Box::new(InheritanceSpecifierPrinter {}),
                "IdentifierPath" => Box::new(IdentifierPathPrinter {}),
                "ModifierInvocation" => Box::new(ModifierInvocationPrinter {}),
                "UsingForDirective" => Box::new(UsingForDirectivePrinter {}),
                "ModifierDefinition" => Box::new(ModifierDefinitionPrinter {}),
                "StructuredDocumentation" => Box::new(StructuredDocumentationPrinter {}),
                "PlaceholderStatement" => Box::new(PlaceholderStatementPrinter {}),
                "Return" => Box::new(ReturnPrinter {}),
                "EmitStatement" => Box::new(EmitStatementPrinter {}),
                "OverrideSpecifier" => Box::new(OverrideSpecifierPrinter {}),
                "ElementaryTypeNameExpression" => Box::new(ElementaryTypeNameExpressionPrinter {}),
                "TupleExpression" => Box::new(TupleExpressionPrinter {}),
                "Comment" => Box::new(CommentPrinter {}),
                "UncheckedBlock" => Box::new(UncheckedBlockPrinter {}),
                _ => Box::new(DummyNodePrinter {}),
            }
        } else {
            Box::new(DummyNodePrinter {})
        }
    }
}
