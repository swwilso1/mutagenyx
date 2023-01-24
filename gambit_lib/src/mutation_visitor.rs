//! The `mutation_visitor` module provides one [`Visitor`] type for traversing an AST and counting
//! mutable nodes and one [`VisitorMut`] type for traversing an AST and mutating nodes.

use crate::mutation::MutationType;
use crate::mutator::*;
use crate::visitor::*;
use rand_pcg::*;
use std::collections::HashMap;

/// The struct for traversing an AST and counting mutable nodes in the AST.  The type conforms/adopts
/// the [`Visitor`] trait.
pub struct MutableNodesCounter<'a, AST> {
    /// Table for counting the number of mutable nodes by [`MutationType`].
    pub counter_table: HashMap<MutationType, u64>,

    /// A reference to the map of mutators by [`MutationType`].
    mutator_map: &'a HashMap<MutationType, Box<dyn Mutator<AST>>>,
}

impl<'a, AST> MutableNodesCounter<'a, AST> {
    /// Create the new node counter.
    ///
    /// # Arguments
    ///
    /// * `mutator_map` - A reference to an external map of mutators by [`MutationType`].
    pub fn new(
        mutator_map: &'a HashMap<MutationType, Box<dyn Mutator<AST>>>,
    ) -> MutableNodesCounter<'a, AST> {
        MutableNodesCounter {
            counter_table: HashMap::new(),
            mutator_map,
        }
    }
}

impl<'a, AST> Visitor<AST> for MutableNodesCounter<'a, AST> {
    fn on_enter(&mut self, _node: &AST) {
        return;
    }

    fn visit(&mut self, node: &AST) -> bool {
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

    fn visit_children(&mut self, _node: &AST) -> bool {
        true
    }

    fn on_exit(&mut self, _node: &AST) {
        return;
    }
}

/// The struct for traversing an AST and mutating nodes. The type conforms/adopts the [`VisitorMut`]
/// trait.
///
/// The mutation generation algorithm currently only generates one mutation per syntax tree pass.
///
/// The mutation maker passes through the AST trying to find the ith instance of a mutable node
/// and mutates that node.
pub struct MutationMaker<'a, AST> {
    /// A reference to a [`Mutator<T>`] that can mutate a node in the AST.
    mutator: &'a Box<dyn Mutator<AST>>,

    /// A reference to the random number given to the mutator to use for mutating the node.
    rng: &'a mut Pcg64,

    /// The index of node to mutate in the AST.
    index: usize,

    /// The current count of mutable nodes in the AST.
    current_index: usize,
}

impl<'a, AST> MutationMaker<'a, AST> {
    /// Create the new mutation maker.
    ///
    /// # Arguments
    ///
    /// * `mutator` - The reference to the mutator.
    /// * `rng` - The reference to the random number generator.
    /// * `index` - The index of the node in the AST to mutate.
    pub fn new(
        mutator: &'a Box<dyn Mutator<AST>>,
        rng: &'a mut Pcg64,
        index: usize,
    ) -> MutationMaker<'a, AST> {
        MutationMaker {
            mutator,
            rng,
            index,
            current_index: 0,
        }
    }
}

impl<'a, AST> VisitorMut<AST> for MutationMaker<'a, AST> {
    fn on_enter(&mut self, _node: &mut AST) {
        return;
    }

    fn visit_mut(&mut self, node: &mut AST) -> bool {
        if self.mutator.is_mutable_node(node) {
            if self.current_index == self.index {
                self.mutator.mutate(node, self.rng);
                return true;
            }
            self.current_index += 1;
        }
        false
    }

    fn visit_children(&mut self, _node: &mut AST) -> bool {
        true
    }

    fn on_exit(&mut self, _node: &mut AST) {
        return;
    }
}