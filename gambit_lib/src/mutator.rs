//! The `mutator` module provides traits for objects that perform mutation algorithms.

use crate::mutation::MutationType;
use rand_pcg::*;

/// Trait that describes the functionality that an object that implements a mutation algorithm
/// must provide.
///
/// # Generic Parameter
///
/// * N - The type of the abstract syntax tree on which the [`Mutator<N>`] should operate.
pub trait Mutator<N> {
    /// When given a node from the abstract syntax tree for the program in language `N`, return true
    /// if the mutator can mutate the node.
    fn is_mutable_node(&self, node: &N) -> bool;

    /// When given a node for the program in language `N` and a random generator object, randomly
    /// mutate the node using the algorithm represented by the object that implements the
    /// [`Mutator<N>`] trait.
    fn mutate(&self, node: &mut N, rand: &mut Pcg64);

    /// Returns the enum value identifying the mutation algorithm implemented by the object that
    /// implements the [`Mutator<N>`] trait.
    fn implements(&self) -> MutationType;
}

/// Trait that describes the functionality need from the object that will create mutators on
/// behalf of a specific language.
pub trait MutatorFactory<N> {
    /// If possible return a mutator object that implements mutations for the requested mutation
    /// algorithm.
    ///
    /// # Arguments
    ///
    /// * `mutation_type` - The mutation algorithm
    fn mutator_for(&self, mutation_type: &MutationType) -> Option<Box<dyn Mutator<N>>>;
}
