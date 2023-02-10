//! The `node_printer` module contains the [`NodePrinter<W,AST>`] and [`NodePrinterFactory<W,AST>`]
//! traits.

use crate::preferences::{PreferenceValue, Preferences};
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
    /// * `_printer` - The [`PrettyPrinter`] that will write content to `_stream`
    /// * `_stream` - The [`Write`] object that will receive formatted output.
    /// * `_factory` - The [`NodePrinterFactory<W, AST>`] trait object that will generate node
    /// printers for the nodes of the AST.
    /// * `_node` - The AST node.
    ///
    /// The default version of this function do not use any of the arguments.
    fn on_entry(
        &mut self,
        _printer: &mut PrettyPrinter,
        _stream: &mut W,
        _factory: &dyn NodePrinterFactory<W, AST>,
        _node: &AST,
    ) {
    }

    /// Called when the node printing object should write out the bulk of the node.
    ///
    /// # Arguments
    ///
    /// * `_printer` - The [`PrettyPrinter`] that will write content to `_stream`.
    /// * `_stream` - The [`Write`] object that will receive formatted output.
    /// * `_factory` - The [`NodePrinterFactory<W, AST>`] trait object that will generate node
    /// printers for the nodes of the AST.
    /// * `_node` - The AST node.
    ///
    /// The default version of this function do not use any of the arguments.
    fn print_node(
        &mut self,
        _printer: &mut PrettyPrinter,
        _stream: &mut W,
        _factory: &dyn NodePrinterFactory<W, AST>,
        _node: &AST,
    ) {
    }

    /// Called when the node traversal algorithm leaves the node.
    ///
    /// # Arguments
    ///
    /// * `_printer` - The [`PrettyPrinter`] that will write content to `_stream`.
    /// * `_stream` - The [`Write`] object that will receive formatted output.
    /// * `_factory` - The [`NodePrinterFactory<W, AST>`] trait object that will generate node
    /// printers for the nodes of the AST.
    /// * `_node` - The AST node.
    fn on_exit(
        &mut self,
        _printer: &mut PrettyPrinter,
        _stream: &mut W,
        _factory: &dyn NodePrinterFactory<W, AST>,
        _node: &AST,
    ) {
    }

    /// Return true if the traversal should traverse and print children nodes.
    fn visit_children(&mut self) -> bool {
        false
    }
}

/// Trait that describes the functionality for an object that will create [`NodePrinter<W,AST>`]
/// objects for printing a node of an AST.
pub trait NodePrinterFactory<W: Write, AST> {
    /// Get a node printer for `node`.
    ///
    /// # Arguments
    /// * `node` - The node from the AST.
    fn printer_for(&self, node: &AST) -> Box<dyn NodePrinter<W, AST>>;

    /// Get a true/false value for a settings key.
    ///
    /// # Arguments
    ///
    /// * `key` - The string for the key.
    fn get_preference_value_for_key(&self, key: &str) -> bool {
        if let Some(preference) = self.get_settings().get_value_for_key(key) {
            match preference {
                PreferenceValue::Boolean(b) => b,
                _ => false,
            }
        } else {
            false
        }
    }

    /// Get the settings object for the factory.
    fn get_settings(&self) -> &Preferences;
}
