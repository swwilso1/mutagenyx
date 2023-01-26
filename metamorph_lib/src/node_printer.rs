//! The `node_printer` module contains the [`NodePrinter<W,AST>`] and [`NodePrinterFactory<W,AST>`]
//! traits.

use crate::pretty_printer::PrettyPrinter;
use std::io::Write;

/// Trait that provides the functionality needed by objects that will print different nodes in an
/// AST.
///
/// The semantics of this trait shadow the [`crate::visitor::Visitor`] trait where the node traversal algorithm
/// will call `on_entry` to indicate the object can start emitting the output for the first part
/// of the node, next `print_node` to write out the majority of the node, and finally `on_exit`
/// when the finishing the node.
pub trait NodePrinter<W: Write, AST> {
    /// Called when the node traversal first encounters the node in the AST.
    ///
    /// # Arguments
    ///
    /// * `_stream` - The [`Write`] object that will receive formatted output.
    /// * `_node` - The AST node.
    /// * `_printer` - The [`PrettyPrinter`] that will write content to `_stream`
    ///
    /// The default version of this function do not use any of the arguments.
    fn on_entry(&mut self, _stream: &mut W, _node: &AST, _printer: &mut PrettyPrinter) {
        return;
    }

    /// Called when the node printing object should write out the bulk of the node.
    ///
    /// # Arguments
    ///
    /// * `_stream` - The [`Write`] object that will receive formatted output.
    /// * `_node` - The AST node.
    /// * `_printer` - The [`PrettyPrinter`] that will write content to `_stream`.
    ///
    /// The default version of this function do not use any of the arguments.
    fn print_node(&mut self, _stream: &mut W, _node: &AST, _printer: &mut PrettyPrinter) {
        return;
    }

    /// Called when the node traversal algorithm leaves the node.
    ///
    /// # Arguments
    ///
    /// * `_stream` - The [`Write`] object that will receive formatted output.
    /// * `_node` - The AST node.
    /// * `_printer` - The [`PrettyPrinter`] that will write content to `_stream`.
    fn on_exit(&mut self, _stream: &mut W, _node: &AST, _printer: &mut PrettyPrinter) {
        return;
    }

    /// Return true if the traversal should traverse and print children nodes.
    fn visit_children(&mut self) -> bool {
        false
    }
}

/// Trait that describes the functionality for an object that will create [`NodePrinter<W,AST>`]
/// objects for printing a node of an AST.
pub trait NodePrinterFactory<W: Write, AST> {
    fn printer_for(&self, node: &AST) -> Box<dyn NodePrinter<W, AST>>;
}
