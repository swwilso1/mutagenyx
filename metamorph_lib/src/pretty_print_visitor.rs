//! The `pretty_print_visitor` module provides a common pretty-print [`Visitor`] object that can
//! traverse an AST. In order to provide language specific behavior, the user must supply an object
//! that can conform to [`NodePrinterFactory<W,AST>`] and that will produce [`NodePrinter<W,AST>`]
//! trait objects that can handle instances of AST nodes.

use crate::node_printer::*;
use crate::pretty_printer::PrettyPrinter;
use crate::visitor::Visitor;
use std::io::Write;

/// [`Visitor<T>`] conforming object that will emit the source code version of a Solidity program.
pub struct PrettyPrintVisitor<'a, W: Write, AST> {
    /// Stack of [`NodePrinter<W,AST>`] objects.
    stack: Vec<Box<dyn NodePrinter<W, AST>>>,
    /// A reference to [`PrettyPrinter`] object that will write formatted output to a [`Write`] object.
    pretty_printer: &'a mut PrettyPrinter,
    /// A reference to a [`Write`] object that will receive formatted output.
    out_stream: &'a mut W,
    /// A [`NodePrinterFactory<W,AST>`] object that will produce [`NodePrinter<W,AST>`] trait
    /// objects
    node_printer_factory: Box<dyn NodePrinterFactory<W, AST>>,
}

impl<'a, W: Write, AST> PrettyPrintVisitor<'a, W, AST> {
    /// Create a new visitor.
    ///
    /// # Arguments
    ///
    /// * `stream` - A reference to the [`Write`] object that will receive formatted output.
    /// * `printer` - A reference to the [`PrettyPrinter`] object that will generate formated output.
    pub fn new(
        stream: &'a mut W,
        printer: &'a mut PrettyPrinter,
        factory: Box<dyn NodePrinterFactory<W, AST>>,
    ) -> PrettyPrintVisitor<'a, W, AST> {
        PrettyPrintVisitor {
            stack: vec![],
            pretty_printer: printer,
            out_stream: stream,
            node_printer_factory: factory,
        }
    }
}

impl<'a, W: Write, AST> Visitor<AST> for PrettyPrintVisitor<'a, W, AST> {
    fn on_enter(&mut self, node: &AST) {
        let printer = self.node_printer_factory.printer_for(node);
        self.stack.push(printer);

        if let Some(p) = self.stack.last_mut() {
            p.on_entry(&mut self.out_stream, node, self.pretty_printer);
        }
    }

    fn visit(&mut self, node: &AST) -> bool {
        if let Some(p) = self.stack.last_mut() {
            p.print_node(&mut self.out_stream, node, self.pretty_printer);
        }
        false
    }

    fn visit_children(&mut self, _node: &AST) -> bool {
        if let Some(p) = self.stack.last_mut() {
            return p.visit_children();
        }
        true
    }

    fn on_exit(&mut self, node: &AST) {
        if let Some(p) = self.stack.last_mut() {
            p.on_exit(&mut self.out_stream, node, self.pretty_printer);
        }
        self.stack.pop();
    }
}
