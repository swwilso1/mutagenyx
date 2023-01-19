use crate::pretty_printer::PrettyPrinter;
use crate::visitor::Visitor;
use crate::vyper::ast::VyperAST;
use std::io::Write;

/// [`Visitor<T>`] conforming object that will emit the source code version of a Vyper program.
pub struct VyperPrettyPrintVisitor {}

impl VyperPrettyPrintVisitor {
    pub fn new<W: Write>(_stream: &mut W, _printer: &mut PrettyPrinter) -> VyperPrettyPrintVisitor {
        VyperPrettyPrintVisitor {}
    }
}

impl Visitor<VyperAST> for VyperPrettyPrintVisitor {
    fn on_enter(&mut self, _node: &VyperAST) {
        return;
    }

    fn visit(&mut self, _node: &VyperAST) -> bool {
        false
    }

    fn visit_children(&mut self, _node: &VyperAST) -> bool {
        true
    }

    fn on_exit(&mut self, _node: &VyperAST) {
        return;
    }
}
