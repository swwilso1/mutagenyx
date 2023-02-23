//! The `pretty_print_visitor` module provides a common pretty-print [`Visitor`] object that can
//! traverse an AST. In order to provide language specific behavior, the user must supply an object
//! that can conform to [`NodePrinterFactory<AST>`] and that will produce [`NodePrinter<AST>`]
//! trait objects that can handle instances of AST nodes.

use crate::node_printer::*;
use crate::pretty_printer::PrettyPrinter;
use crate::visitor::Visitor;
use std::io::Write;

/// [`Visitor<T>`] conforming object that will emit the source code version of a program.
pub struct PrettyPrintVisitor<'a, AST> {
    /// Stack of [`NodePrinter<AST>`] objects.
    stack: Vec<Box<dyn NodePrinter<AST>>>,
    /// A reference to [`PrettyPrinter`] object that will write formatted output to a [`Write`] object.
    pretty_printer: &'a mut PrettyPrinter,
    /// A reference to a [`Write`] object that will receive formatted output.
    out_stream: &'a mut dyn Write,
    /// A [`NodePrinterFactory<AST>`] object that will produce [`NodePrinter<AST>`] trait
    /// objects
    node_printer_factory: &'a dyn NodePrinterFactory<AST>,
}

impl<'a, AST> PrettyPrintVisitor<'a, AST> {
    /// Create a new visitor.
    ///
    /// # Arguments
    ///
    /// * `stream` - A reference to the [`Write`] object that will receive formatted output.
    /// * `printer` - A reference to the [`PrettyPrinter`] object that will generate formatted output.
    /// * `factory` - A boxed trait reference object for [`NodePrinterFactory<AST>`] that generates
    /// [`NodePrinter<AST>`] objects for nodes in the AST.
    pub fn new(
        stream: &'a mut dyn Write,
        printer: &'a mut PrettyPrinter,
        factory: &'a dyn NodePrinterFactory<AST>,
    ) -> PrettyPrintVisitor<'a, AST> {
        PrettyPrintVisitor {
            stack: vec![],
            pretty_printer: printer,
            out_stream: stream,
            node_printer_factory: factory,
        }
    }
}

impl<'a, AST> Visitor<AST> for PrettyPrintVisitor<'a, AST> {
    fn on_enter(&mut self, node: &AST) {
        let printer = self.node_printer_factory.printer_for(node);
        self.stack.push(printer);

        if let Some(p) = self.stack.last_mut() {
            p.on_entry(
                self.pretty_printer,
                self.out_stream,
                self.node_printer_factory,
                node,
            );
        }
    }

    fn visit(&mut self, node: &AST) -> bool {
        if let Some(p) = self.stack.last_mut() {
            p.print_node(
                self.pretty_printer,
                self.out_stream,
                self.node_printer_factory,
                node,
            );
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
            p.on_exit(
                self.pretty_printer,
                self.out_stream,
                self.node_printer_factory,
                node,
            );
        }
        self.stack.pop();
    }
}
