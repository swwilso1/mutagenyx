//! The `node_printer_helper` module contains common algorithms used while traversing an AST.

use crate::ast::{ASTTraverser, SimpleAST};
use crate::node_printer::NodePrinterFactory;
use crate::pretty_print_visitor::PrettyPrintVisitor;
use crate::pretty_printer::{write_space, write_token, PrettyPrinter};
use std::io::Write;

/// Helper function to traverse a child node
///
/// # Arguments
///
/// * `node` - The node in the AST to traverse.
/// * `stream` - The [`Write`] object that will receive formatted output
/// * `printer` - The [`PrettyPrinter`] object that will write to `stream`.
/// * `factory` - The [`NodePrinterFactory<AST>`] object for generating node printers.
pub fn traverse_sub_node_and_print<AST: SimpleAST<AST>>(
    printer: &mut PrettyPrinter,
    stream: &mut dyn Write,
    factory: &dyn NodePrinterFactory<AST>,
    node: &AST,
) {
    let mut visitor = PrettyPrintVisitor::<AST>::new(stream, printer, factory);
    ASTTraverser::traverse(node, &mut visitor);
}

/// Helper function to output the contents of an array of nodes
///
/// # Arguments
///
/// * `printer` - The [`PrettyPrinter`] that will send formatted output to `stream`.
/// * `stream` - The [`Write`] object that will receive the formatted output.
/// * `factory` - The reference to a [`NodePrinterFactory<AST>`] trait object to use for generating
/// [`crate::node_printer::NodePrinter<AST>`] trait objects to print AST nodes.
/// * `array` - The [`Vec`] of nodes
pub fn print_array_helper<AST: SimpleAST<AST>>(
    printer: &mut PrettyPrinter,
    stream: &mut dyn Write,
    factory: &dyn NodePrinterFactory<AST>,
    array: &Vec<AST>,
) {
    let mut i = 0;
    while i < array.len() {
        if let Some(n) = array.get(i) {
            traverse_sub_node_and_print(printer, stream, factory, n);
            if i < (array.len() - 1) {
                write_token(printer, stream, ",");
                write_space(printer, stream);
            }
        }
        i += 1;
    }
}
