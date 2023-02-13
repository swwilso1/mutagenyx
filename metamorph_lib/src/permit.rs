/// Query string that means: 'do I have permission to mutate a node?'
pub static MUTATE: &str = "mutate";

/// Query string that means: 'do I have permission to count a node?'
pub static COUNT: &str = "count";

/// Query string that means: 'do I have permission to visit a node?'
pub static VISIT: &str = "visit";

pub trait Permit<AST> {
    fn has_permission_to(&self, query: &str, node: &AST) -> bool;
}
