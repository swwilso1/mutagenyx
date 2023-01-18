//! The `solidity::visitor` module contains visitors for counting mutable nodes in a Solidity
//! AST and visitors for mutating nodes in the AST.

use crate::mutation::MutationType;
use crate::mutator::*;
use crate::solidity::ast::SolidityAST;
use crate::visitor::*;
use rand_pcg::*;
use std::collections::HashMap;

/// [`Visitor`] conforming object that exists to count the number of nodes in the AST that match
/// various mutation algorithms.
pub struct SolidityMutationNodeCounter<'a> {
    /// The table that holds the number of mutable nodes for each mutation type.
    pub counter_table: HashMap<MutationType, u64>,

    /// A reference to the map that maps mutation algorithm types to the mutators that implement
    /// the algorithms.
    mutator_map: &'a HashMap<MutationType, Box<dyn Mutator<SolidityAST>>>,
}

impl<'a> SolidityMutationNodeCounter<'a> {
    /// Create a new visitor.
    ///
    /// # Arguments
    /// * `mutator_map` - A reference to the map of mutators.
    pub fn new(
        mutator_map: &'a HashMap<MutationType, Box<dyn Mutator<SolidityAST>>>,
    ) -> SolidityMutationNodeCounter<'a> {
        SolidityMutationNodeCounter {
            counter_table: HashMap::new(),
            mutator_map,
        }
    }
}

impl<'a> Visitor<SolidityAST> for SolidityMutationNodeCounter<'a> {
    fn on_enter(&mut self, _node: &SolidityAST) {
        return;
    }

    fn visit(&mut self, node: &SolidityAST) -> bool {
        for (key, value) in self.mutator_map {
            if value.is_mutable_node(node) {
                if self.counter_table.contains_key(key) {
                    let size = self.counter_table.get_mut(key).unwrap();
                    *size += 1;
                } else {
                    self.counter_table.insert(*key, 1);
                }
            }
        }
        false
    }

    fn visit_children(&mut self, _node: &SolidityAST) -> bool {
        true
    }

    fn on_exit(&mut self, _node: &SolidityAST) {
        return;
    }
}

/// [`VisitorMut`] conforming object that exists to mutate a node of the Solidity AST.
pub struct SolidityMutationMaker<'a> {
    /// A reference to the mutator that will mutate the node.
    mutator: &'a Box<dyn Mutator<SolidityAST>>,

    /// A reference to a random number generator that the mutator can use to perform the mutation.
    rng: &'a mut Pcg64,

    /// The index of the node in the AST to mutate.
    index: usize,

    /// The number of mutable nodes (mutable by the mutator) already found in the AST.
    current_index: usize,
}

impl<'a> SolidityMutationMaker<'a> {
    /// Create the new visitor.
    ///
    /// # Arguments
    ///
    /// * `mutator` - The object that will mutate the node.
    /// * `rng` - The random number generator object for use by the `mutator`.
    /// * `index` - The index of the node to mutate.
    pub fn new(
        mutator: &'a Box<dyn Mutator<SolidityAST>>,
        rng: &'a mut Pcg64,
        index: usize,
    ) -> SolidityMutationMaker<'a> {
        SolidityMutationMaker {
            mutator,
            rng,
            index,
            current_index: 0,
        }
    }
}

impl<'a> VisitorMut<SolidityAST> for SolidityMutationMaker<'a> {
    fn on_enter(&mut self, _node: &mut SolidityAST) {
        return;
    }

    fn visit_mut(&mut self, node: &mut SolidityAST) -> bool {
        if self.mutator.is_mutable_node(node) {
            if self.current_index == self.index {
                self.mutator.mutate(node, self.rng);
                return true;
            }
            self.current_index += 1;
        }
        false
    }

    fn visit_children(&mut self, _node: &mut SolidityAST) -> bool {
        true
    }

    fn on_exit(&mut self, _node: &mut SolidityAST) {
        return;
    }
}
