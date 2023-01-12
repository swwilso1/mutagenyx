pub trait Visitor<N: ?Sized> {
    fn on_enter(&mut self, node: &N);
    fn visit(&mut self, node: &N) -> bool;
    fn visit_children(&mut self, node: &N) -> bool;
    fn on_exit(&mut self, node: &N);
}

pub trait VisitorMut<N: ?Sized> {
    fn on_enter(&mut self, node: &mut N);
    fn visit_mut(&mut self, node: &mut N) -> bool;
    fn visit_children(&mut self, node: &mut N) -> bool;
    fn on_exit(&mut self, node: &mut N);
}
