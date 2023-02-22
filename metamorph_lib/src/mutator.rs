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
    ///
    /// # Arguments
    ///
    /// * `node` - The node to check for mutability.
    /// * `rand` - The random number generator.
    fn is_mutable_node(&mut self, node: &N, rand: &mut Pcg64) -> bool;

    /// When given a node for the program in language `N` and a random generator object, randomly
    /// mutate the node using the algorithm represented by the object that implements the
    /// [`Mutator<N>`] trait.  Return the node id of the mutated node or none if mutated node
    /// did not contain an id.
    ///
    /// We return the id of the mutated node because `node` might be an encapsulating node and
    /// the actual mutated node might be a child of `node`.  This situation can occur when mutating
    /// a member of a block such as if(test){ ... } where the {  } might be a block in of itself,
    /// but the mutator mutates a statement in the block.
    ///
    /// # Arguments
    ///
    /// * `node` - The node to mutate
    /// * `rand` - The random number generator.
    fn mutate(&mut self, node: &mut N, rand: &mut Pcg64) -> Option<u64>;

    /// Returns the enum value identifying the mutation algorithm implemented by the object that
    /// implements the [`Mutator<N>`] trait.
    fn implements(&self) -> MutationType;

    /// Return a comment from the mutator that describes the mutation made by the mutator.
    fn get_comment_node(&self) -> Option<N> {
        None
    }
}

/// Trait that describes the functionality need from the object that will create mutators on
/// behalf of a specific language.
pub trait MutatorFactory<N> {
    /// If possible, return a mutator object that implements mutations for the requested mutation
    /// algorithm.
    ///
    /// # Arguments
    ///
    /// * `mutation_type` - The mutation algorithm
    fn mutator_for(&self, mutation_type: &MutationType) -> Option<Box<dyn Mutator<N>>>;
}
