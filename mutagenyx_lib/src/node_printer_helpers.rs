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
    print_array_helper_with_node_handler_and_space_formatter(
        printer,
        stream,
        factory,
        array,
        |p, s, f, n| {
            traverse_sub_node_and_print(p, s, f, n);
        },
        |p, s, _f| {
            write_token(p, s, ",");
            write_space(p, s);
        },
    )
}

/// Helper function to print out the elements of an array using a closure to actually do the
/// formatting of the node and a closure to do write any inter-element custom formatting.
///
/// # Arguments
///
/// * `printer` - The [`PrettyPrinter`] that will send formatted output to `stream`.
/// * `stream` - The [`Write`] object that will receive the formatted output.
/// * `factory` - The reference to a [`NodePrinterFactory<AST>`] trait object to use for generating
/// [`crate::node_printer::NodePrinter<AST>`] trait objects to print AST nodes.
/// * `array` - The [`Vec`] of nodes
/// * `node_printer` - The closure that directly formats/prints each element of `array`.
/// * `inter_node_formatter` - The closure that formats/prints the content that should appear between
/// elements of the array.
pub fn print_array_helper_with_node_handler_and_space_formatter<AST: SimpleAST<AST>, F, I>(
    printer: &mut PrettyPrinter,
    stream: &mut dyn Write,
    factory: &dyn NodePrinterFactory<AST>,
    array: &Vec<AST>,
    node_printer: F,
    inter_node_formatter: I,
) where
    F: Fn(&mut PrettyPrinter, &mut dyn Write, &dyn NodePrinterFactory<AST>, &AST) -> (),
    I: Fn(&mut PrettyPrinter, &mut dyn Write, &dyn NodePrinterFactory<AST>) -> (),
{
    let mut i: usize = 0;
    while i < array.len() {
        if let Some(n) = array.get(i) {
            // Call the custom node printer closure.
            node_printer(printer, stream, factory, n);
            if i < (array.len() - 1) {
                // Call the custom space formatter for content between the nodes.
                inter_node_formatter(printer, stream, factory);
            }
        }
        i += 1;
    }
}
