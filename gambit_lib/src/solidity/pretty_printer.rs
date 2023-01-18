use crate::ast::ASTTraverser;
use crate::json::JSONMutate;
use crate::pretty_printer::{
    write_flowable_text, write_indent, write_newline, write_space, write_string, write_token,
    PrettyPrinter,
};
use crate::solidity::ast::SolidityAST;
use crate::visitor::Visitor;
use std::io::Write;

fn traverse_sub_node<W: Write>(node: &SolidityAST, stream: &mut W, printer: &mut PrettyPrinter) {
    let mut visitor = SolidityPrettyPrintVisitor::new(stream, printer);
    ASTTraverser::traverse(node, &mut visitor);
}

fn print_documentation_helper<W: Write>(
    stream: &mut W,
    node: &SolidityAST,
    printer: &mut PrettyPrinter,
) {
    if let Some(docs) = node.borrow_value_for_key("documentation") {
        traverse_sub_node(docs, stream, printer);
        write_newline(printer, stream);
        write_indent(printer, stream);
    }
}

trait NodePrinter<W: Write> {
    fn on_entry(&mut self, _stream: &mut W, _node: &SolidityAST, _printer: &mut PrettyPrinter) {
        return;
    }

    fn print_node(&mut self, _stream: &mut W, _node: &SolidityAST, _printer: &mut PrettyPrinter) {
        return;
    }

    fn on_exit(&mut self, _stream: &mut W, _node: &SolidityAST, _printer: &mut PrettyPrinter) {
        return;
    }

    fn visit_children(&mut self) -> bool {
        false
    }
}

struct DummyNodePrinter {}

impl<W: Write> NodePrinter<W> for DummyNodePrinter {
    fn visit_children(&mut self) -> bool {
        true
    }
}

struct SourceUnitPrinter {}

impl<W: Write> NodePrinter<W> for SourceUnitPrinter {
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
                        traverse_sub_node(node, stream, printer);
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

impl<W: Write> NodePrinter<W> for PragmaDirectivePrinter {
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

fn close_block_exit_helper<W: Write>(stream: &mut W, printer: &mut PrettyPrinter) {
    printer.decrease_indent();
    write_newline(printer, stream);
    write_indent(printer, stream);
    write_token(printer, stream, "}");
}

struct ContractDefinitionPrinter {}

impl<W: Write> NodePrinter<W> for ContractDefinitionPrinter {
    fn on_entry(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        print_documentation_helper(stream, node, printer);

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
                    print_array_helper(stream, contract_array, printer);
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
                        traverse_sub_node(contract_node, stream, printer);
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
        close_block_exit_helper(stream, printer);
    }
}

struct StructDefinitionPrinter {}

impl<W: Write> NodePrinter<W> for StructDefinitionPrinter {
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
                        traverse_sub_node(member, stream, printer);
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
        close_block_exit_helper(stream, printer);
    }
}

fn print_name_helper<W: Write>(stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
    if let Some(name) = node.get_str_for_key("name") {
        write_token(printer, stream, name);
    }
}

struct ElementaryTypeNamePrinter {}

impl<W: Write> NodePrinter<W> for ElementaryTypeNamePrinter {
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        print_name_helper(stream, node, printer);

        if let Some(mutability) = node.get_str_for_key("stateMutability") {
            if mutability == "payable" {
                write_space(printer, stream);
                write_token(printer, stream, mutability);
            }
        }
    }
}

struct MappingPrinter {}

impl<W: Write> NodePrinter<W> for MappingPrinter {
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, "mapping");
        write_token(printer, stream, "(");

        if let Some(key) = node.borrow_value_for_key("keyType") {
            traverse_sub_node(key, stream, printer);
        }

        write_space(printer, stream);
        write_token(printer, stream, "=>");
        write_space(printer, stream);

        if let Some(value) = node.borrow_value_for_key("valueType") {
            traverse_sub_node(value, stream, printer);
        }

        write_token(printer, stream, ")");
    }
}

struct UserDefinedTypeNamePrinter {}

impl<W: Write> NodePrinter<W> for UserDefinedTypeNamePrinter {
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        if let Some(path_node) = node.borrow_value_for_key("pathNode") {
            print_name_helper(stream, path_node, printer);
        }
    }
}

struct VariableDeclarationPrinter {}

impl<W: Write> NodePrinter<W> for VariableDeclarationPrinter {
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        print_documentation_helper(stream, node, printer);
        if let Some(type_name_node) = node.borrow_value_for_key("typeName") {
            traverse_sub_node(type_name_node, stream, printer);
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
            traverse_sub_node(value, stream, printer);
        }
    }
}

struct FunctionDefinitionPrinter {}

impl<W: Write> NodePrinter<W> for FunctionDefinitionPrinter {
    fn on_entry(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        if let Some(documentation) = node.borrow_value_for_key("documentation") {
            traverse_sub_node(documentation, stream, printer);
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
            traverse_sub_node(parameters, stream, printer);
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
            traverse_sub_node(overrides, stream, printer);
        }

        if let Some(modifiers) = node.borrow_value_for_key("modifiers") {
            if let Some(modifier_array) = modifiers.as_array() {
                if modifier_array.len() > 0 {
                    write_space(printer, stream);
                    print_array_helper(stream, modifier_array, printer);
                }
            }
        }

        if let Some(return_parameters) = node.borrow_value_for_key("returnParameters") {
            if let Some(parameters) = return_parameters.borrow_value_for_key("parameters") {
                if let Some(parameter_array) = parameters.as_array() {
                    if parameter_array.len() > 0 {
                        write_space(printer, stream);
                        write_token(printer, stream, "returns");
                        write_space(printer, stream);
                        traverse_sub_node(return_parameters, stream, printer);
                    }
                }
            }
        }

        if let Some(body) = node.borrow_value_for_key("body") {
            write_space(printer, stream);
            traverse_sub_node(body, stream, printer);
        }
    }
}

fn print_array_helper<W: Write>(
    stream: &mut W,
    array: &Vec<SolidityAST>,
    printer: &mut PrettyPrinter,
) {
    let mut i = 0;
    while i < array.len() {
        if let Some(n) = array.get(i) {
            traverse_sub_node(n, stream, printer);
            if i < (array.len() - 1) {
                write_token(printer, stream, ",");
                write_space(printer, stream);
            }
        }
        i += 1;
    }
}

struct ParameterListPrinter {}

impl<W: Write> NodePrinter<W> for ParameterListPrinter {
    fn on_entry(&mut self, stream: &mut W, _node: &SolidityAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, "(");
    }

    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        if let Some(parameters) = node.borrow_value_for_key("parameters") {
            if let Some(parameters_array) = parameters.as_array() {
                print_array_helper(stream, parameters_array, printer);
            }
        }
    }

    fn on_exit(&mut self, stream: &mut W, _node: &SolidityAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, ")");
    }
}

struct BlockPrinter {}

impl<W: Write> NodePrinter<W> for BlockPrinter {
    fn on_entry(&mut self, stream: &mut W, _node: &SolidityAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, "{");
    }

    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        if let Some(statements) = node.borrow_value_for_key("statements") {
            if let Some(statements_array) = statements.as_array() {
                if statements_array.len() > 0 {
                    printer.increase_indent();
                    write_newline(printer, stream);

                    let mut i = 0;
                    while i < statements_array.len() {
                        if let Some(statement) = statements_array.get(i) {
                            write_indent(printer, stream);
                            traverse_sub_node(statement, stream, printer);
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

    fn on_exit(&mut self, stream: &mut W, _node: &SolidityAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, "}");
    }
}

struct VariableDeclarationStatementPrinter {}

impl<W: Write> NodePrinter<W> for VariableDeclarationStatementPrinter {
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        if let Some(declarations) = node.borrow_value_for_key("declarations") {
            if let Some(declarations_array) = declarations.as_array() {
                let mut i = 0;
                while i < declarations_array.len() {
                    if let Some(declaration) = declarations_array.get(i) {
                        traverse_sub_node(declaration, stream, printer);
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
            traverse_sub_node(initial_value, stream, printer);
        }
    }

    fn on_exit(&mut self, stream: &mut W, _node: &SolidityAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, ";");
    }
}

fn print_operator_helper<W: Write>(
    stream: &mut W,
    node: &SolidityAST,
    printer: &mut PrettyPrinter,
) {
    if let Some(operator) = node.get_str_for_key("operator") {
        write_space(printer, stream);
        write_token(printer, stream, operator);
        write_space(printer, stream);
    }
}

struct BinaryOperationPrinter {}

impl<W: Write> NodePrinter<W> for BinaryOperationPrinter {
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        if let Some(le) = node.borrow_value_for_key("leftExpression") {
            traverse_sub_node(le, stream, printer);
        }

        print_operator_helper(stream, node, printer);

        if let Some(re) = node.borrow_value_for_key("rightExpression") {
            traverse_sub_node(re, stream, printer);
        }
    }
}

struct LiteralPrinter {}

impl<W: Write> NodePrinter<W> for LiteralPrinter {
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

impl<W: Write> NodePrinter<W> for ExpressionStatementPrinter {
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        if let Some(expression) = node.borrow_value_for_key("expression") {
            traverse_sub_node(expression, stream, printer);
        }
    }

    fn on_exit(&mut self, stream: &mut W, _node: &SolidityAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, ";");
    }
}

struct AssignmentPrinter {}

impl<W: Write> NodePrinter<W> for AssignmentPrinter {
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        if let Some(lhs) = node.borrow_value_for_key("leftHandSide") {
            traverse_sub_node(lhs, stream, printer);
        }

        print_operator_helper(stream, node, printer);

        if let Some(rhs) = node.borrow_value_for_key("rightHandSide") {
            traverse_sub_node(rhs, stream, printer);
        }
    }
}

struct IdentifierPrinter {}

impl<W: Write> NodePrinter<W> for IdentifierPrinter {
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        if let Some(name) = node.get_str_for_key("name") {
            write_token(printer, stream, name);
        }
    }
}

struct FunctionCallPrinter {}

impl<W: Write> NodePrinter<W> for FunctionCallPrinter {
    fn on_entry(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        if let Some(expression) = node.borrow_value_for_key("expression") {
            traverse_sub_node(expression, stream, printer);
            write_token(printer, stream, "(");
        }
    }

    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        if let Some(arguments) = node.borrow_value_for_key("arguments") {
            if let Some(arguments_array) = arguments.as_array() {
                print_array_helper(stream, arguments_array, printer);
            }
        }
    }

    fn on_exit(&mut self, stream: &mut W, _node: &SolidityAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, ")");
    }
}

struct UnaryOperationPrinter {}

impl<W: Write> NodePrinter<W> for UnaryOperationPrinter {
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
                traverse_sub_node(sub_expression, stream, printer);
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

impl<W: Write> NodePrinter<W> for MemberAccessPrinter {
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        if let Some(expression) = node.borrow_value_for_key("expression") {
            traverse_sub_node(expression, stream, printer);
        }

        write_token(printer, stream, ".");

        if let Some(member_name) = node.get_str_for_key("memberName") {
            write_token(printer, stream, member_name);
        }
    }
}

struct IndexAccessPrinter {}

impl<W: Write> NodePrinter<W> for IndexAccessPrinter {
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        if let Some(base_expression) = node.borrow_value_for_key("baseExpression") {
            traverse_sub_node(base_expression, stream, printer);
        }

        write_token(printer, stream, "[");

        if let Some(index_expression) = node.borrow_value_for_key("indexExpression") {
            traverse_sub_node(index_expression, stream, printer);
        }

        write_token(printer, stream, "]");
    }
}

struct IfStatementPrinter {}

impl<W: Write> NodePrinter<W> for IfStatementPrinter {
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, "if");
        write_token(printer, stream, "(");

        if let Some(condition) = node.borrow_value_for_key("condition") {
            traverse_sub_node(condition, stream, printer);
        }

        write_token(printer, stream, ")");
        write_space(printer, stream);

        if let Some(true_body) = node.borrow_value_for_key("trueBody") {
            traverse_sub_node(true_body, stream, printer);
        }

        if let Some(false_body) = node.borrow_value_for_key("falseBody") {
            write_space(printer, stream);
            write_token(printer, stream, "else");
            write_space(printer, stream);
            traverse_sub_node(false_body, stream, printer);
        }
    }
}

struct ImportDirectivePrinter {}

impl<W: Write> NodePrinter<W> for ImportDirectivePrinter {
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
                            traverse_sub_node(foreign_symbol, stream, printer);
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

impl<W: Write> NodePrinter<W> for InheritanceSpecifierPrinter {
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        if let Some(base_name) = node.borrow_value_for_key("baseName") {
            traverse_sub_node(base_name, stream, printer);
        }
    }
}

struct IdentifierPathPrinter {}

impl<W: Write> NodePrinter<W> for IdentifierPathPrinter {
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        print_name_helper(stream, node, printer);
    }
}

struct ModifierInvocationPrinter {}

impl<W: Write> NodePrinter<W> for ModifierInvocationPrinter {
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        if let Some(modifier_name) = node.borrow_value_for_key("modifierName") {
            traverse_sub_node(modifier_name, stream, printer);
        }

        if let Some(arguments) = node.borrow_value_for_key("arguments") {
            if let Some(arguments_array) = arguments.as_array() {
                if arguments_array.len() > 0 {
                    write_token(printer, stream, "(");
                    print_array_helper(stream, arguments_array, printer);
                    write_token(printer, stream, ")");
                }
            }
        }
    }
}

struct UsingForDirectivePrinter {}

impl<W: Write> NodePrinter<W> for UsingForDirectivePrinter {
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, "using");
        if let Some(library_name) = node.borrow_value_for_key("libraryName") {
            write_space(printer, stream);
            traverse_sub_node(library_name, stream, printer);
        }
        write_space(printer, stream);
        write_token(printer, stream, "for");
        write_space(printer, stream);
        if let Some(type_name) = node.borrow_value_for_key("typeName") {
            traverse_sub_node(type_name, stream, printer);
        }
    }

    fn on_exit(&mut self, stream: &mut W, _node: &SolidityAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, ";");
    }
}

struct ModifierDefinitionPrinter {}

impl<W: Write> NodePrinter<W> for ModifierDefinitionPrinter {
    fn on_entry(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        print_documentation_helper(stream, node, printer);
        write_token(printer, stream, "modifier");
        write_space(printer, stream);
        print_name_helper(stream, node, printer);
        if let Some(parameters) = node.borrow_value_for_key("parameters") {
            traverse_sub_node(parameters, stream, printer);
        }
    }

    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        if let Some(body) = node.borrow_value_for_key("body") {
            write_space(printer, stream);
            traverse_sub_node(body, stream, printer);
        }
    }
}

struct StructuredDocumentationPrinter {}

impl<W: Write> NodePrinter<W> for StructuredDocumentationPrinter {
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

impl<W: Write> NodePrinter<W> for PlaceholderStatementPrinter {
    fn print_node(&mut self, stream: &mut W, _node: &SolidityAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, "_");
        write_token(printer, stream, ";");
    }
}

struct ReturnPrinter {}

impl<W: Write> NodePrinter<W> for ReturnPrinter {
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        if let Some(expression) = node.borrow_value_for_key("expression") {
            write_token(printer, stream, "return");
            write_space(printer, stream);
            traverse_sub_node(expression, stream, printer);
            write_token(printer, stream, ";");
        }
    }
}

struct EmitStatementPrinter {}

impl<W: Write> NodePrinter<W> for EmitStatementPrinter {
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        if let Some(event_call) = node.borrow_value_for_key("eventCall") {
            write_token(printer, stream, "emit");
            write_space(printer, stream);
            traverse_sub_node(event_call, stream, printer);
            write_token(printer, stream, ";");
        }
    }
}

struct OverrideSpecifierPrinter {}

impl<W: Write> NodePrinter<W> for OverrideSpecifierPrinter {
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        write_space(printer, stream);
        write_token(printer, stream, "override");
        if let Some(overrides) = node.borrow_value_for_key("overrides") {
            if let Some(overrides_array) = overrides.as_array() {
                if overrides_array.len() > 0 {
                    write_space(printer, stream);
                    print_array_helper(stream, overrides_array, printer);
                }
            }
        }
    }
}

struct ElementaryTypeNameExpressionPrinter {}

impl<W: Write> NodePrinter<W> for ElementaryTypeNameExpressionPrinter {
    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        // Not sure if this is the correct way to handle this node.
        if let Some(pure) = node.get_bool_for_key("isPure") {
            if pure {
                if let Some(type_name) = node.borrow_value_for_key("typeName") {
                    if let Some(mutability) = type_name.get_str_for_key("stateMutability") {
                        write_token(printer, stream, mutability);
                    } else {
                        traverse_sub_node(type_name, stream, printer);
                    }
                }
            }
        }
    }
}

struct TupleExpressionPrinter {}

impl<W: Write> NodePrinter<W> for TupleExpressionPrinter {
    fn on_entry(&mut self, stream: &mut W, _node: &SolidityAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, "(");
    }

    fn print_node(&mut self, stream: &mut W, node: &SolidityAST, printer: &mut PrettyPrinter) {
        if let Some(components) = node.borrow_value_for_key("components") {
            if let Some(components_array) = components.as_array() {
                for component in components_array {
                    traverse_sub_node(component, stream, printer);
                }
            }
        }
    }

    fn on_exit(&mut self, stream: &mut W, _node: &SolidityAST, printer: &mut PrettyPrinter) {
        write_token(printer, stream, ")");
    }
}

struct PrinterFactory {}

impl PrinterFactory {
    fn new() -> PrinterFactory {
        PrinterFactory {}
    }

    fn printer_for<W: Write>(&self, node: &SolidityAST) -> Box<dyn NodePrinter<W>> {
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
                _ => Box::new(DummyNodePrinter {}),
            }
        } else {
            Box::new(DummyNodePrinter {})
        }
    }
}

pub struct SolidityPrettyPrintVisitor<'a, W: Write> {
    stack: Vec<Box<dyn NodePrinter<W>>>,
    pretty_printer: &'a mut PrettyPrinter,
    out_stream: &'a mut W,
}

impl<'a, W: Write> SolidityPrettyPrintVisitor<'a, W> {
    pub fn new(
        stream: &'a mut W,
        printer: &'a mut PrettyPrinter,
    ) -> SolidityPrettyPrintVisitor<'a, W> {
        SolidityPrettyPrintVisitor {
            stack: vec![],
            pretty_printer: printer,
            out_stream: stream,
        }
    }
}

impl<'a, W: Write> Visitor<SolidityAST> for SolidityPrettyPrintVisitor<'a, W> {
    fn on_enter(&mut self, node: &SolidityAST) {
        let printer_factory = PrinterFactory::new();

        let printer = printer_factory.printer_for(node);
        self.stack.push(printer);

        if let Some(p) = self.stack.last_mut() {
            p.on_entry(&mut self.out_stream, node, self.pretty_printer);
        }
    }

    fn visit(&mut self, node: &SolidityAST) -> bool {
        if let Some(p) = self.stack.last_mut() {
            p.print_node(&mut self.out_stream, node, self.pretty_printer);
        }
        false
    }

    fn visit_children(&mut self, _node: &SolidityAST) -> bool {
        if let Some(p) = self.stack.last_mut() {
            return p.visit_children();
        }
        true
    }

    fn on_exit(&mut self, node: &SolidityAST) {
        if let Some(p) = self.stack.last_mut() {
            p.on_exit(&mut self.out_stream, node, self.pretty_printer);
        }
        self.stack.pop();
    }
}
