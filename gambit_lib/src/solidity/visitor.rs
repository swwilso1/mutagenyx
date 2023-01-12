use crate::mutation::MutationType;
use crate::mutator::*;
use crate::solidity::ast::SolidityAST;
use crate::visitor::*;
use rand_pcg::*;
use std::collections::HashMap;

pub struct SolidityMutationNodeCounter<'a> {
    pub counter_table: HashMap<MutationType, u64>,
    mutator_map: &'a HashMap<MutationType, Box<dyn Mutator<SolidityAST>>>,
}

impl<'a> SolidityMutationNodeCounter<'a> {
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

pub struct SolidityMutationMaker<'a> {
    mutator: &'a Box<dyn Mutator<SolidityAST>>,
    rng: &'a mut Pcg64,
    index: usize,
    current_index: usize,
}

impl<'a> SolidityMutationMaker<'a> {
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
