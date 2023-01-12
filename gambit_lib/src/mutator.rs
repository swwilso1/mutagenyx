use crate::mutation::MutationType;
use rand_pcg::*;

pub trait Mutator<N> {
    fn is_mutable_node(&self, node: &N) -> bool;
    fn mutate(&self, node: &mut N, rand: &mut Pcg64);
    fn implements(&self) -> MutationType;
}

pub trait MutatorFactory<N> {
    fn mutator_for(&self, mutation_type: &MutationType) -> Option<Box<dyn Mutator<N>>>;
}
