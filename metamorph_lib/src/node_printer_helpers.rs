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
/// * `node` - The node in the Vyper AST to traverse.
/// * `stream` - The [`Write`] object that will receive formatted output
/// * `printer` - The [`PrettyPrinter`] object that will write to `stream`.
/// * `factory` - The [`NodePrinterFactory<W,AST>`] object for generating node printers.
pub fn traverse_sub_node<W: Write, AST: SimpleAST<AST>, F: NodePrinterFactory<W, AST> + 'static>(
    printer: &mut PrettyPrinter,
    stream: &mut W,
    factory: F,
    node: &AST,
) {
    let mut visitor = PrettyPrintVisitor::<W, AST>::new(stream, printer, Box::new(factory));
    ASTTraverser::traverse(node, &mut visitor);
}

/// Helper function to output the contents of an array of nodes
///
/// # Arguments
///
/// * `stream` - The [`Write`] object that will receive the formatted output.
/// * `array` - The [`Vec`] of nodes
/// * `printer` - The [`PrettyPrinter`] that will send formatted output to `stream`.
pub fn print_array_helper<
    W: Write,
    AST: SimpleAST<AST>,
    F: NodePrinterFactory<W, AST> + Clone + 'static,
>(
    printer: &mut PrettyPrinter,
    stream: &mut W,
    factory: F,
    array: &Vec<AST>,
) {
    let mut i = 0;
    while i < array.len() {
        if let Some(n) = array.get(i) {
            traverse_sub_node(printer, stream, factory.clone(), n);
            if i < (array.len() - 1) {
                write_token(printer, stream, ",");
                write_space(printer, stream);
            }
        }
        i += 1;
    }
}
