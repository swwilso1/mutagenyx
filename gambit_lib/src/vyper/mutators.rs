use crate::mutation::*;
use crate::mutator::*;
use crate::vyper::ast::VyperAST;

/// Implement the [`MutatorFactory<T>`] trait to have an interface for getting mutators for requested
/// mutation algorithms.
pub struct VyperMutatorFactory {}

impl MutatorFactory<VyperAST> for VyperMutatorFactory {
    fn mutator_for(&self, mutation_type: &MutationType) -> Option<Box<dyn Mutator<VyperAST>>> {
        match mutation_type {
            _ => None,
        }
    }
}
