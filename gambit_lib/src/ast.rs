use crate::visitor::*;

pub trait AST<N> {
    fn traverse<V: Visitor<N> + ?Sized>(&self, visitor: &mut V) -> bool;
    fn traverse_mut<V: VisitorMut<N> + ?Sized>(&mut self, visitor: &mut V) -> bool;
}
