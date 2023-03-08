//! The `mutator_result` module contains the code for [`MutatorResult`].
//!

use crate::mutation::MutationType;

/// Result type returned from the mutation operation.
#[derive(Debug, Clone, PartialEq)]
pub struct MutatorResult {
    /// The mutation algorithm used.
    pub mutation_type: Option<MutationType>,

    /// The index of the mutated node.
    pub index: Option<usize>,

    /// The id of the mutated node or None if the mutated node did not have an id.
    pub mutated_node_id: Option<u64>,

    /// The hash value of the random mutation
    pub random_behavior_hash: Option<String>,
}

impl MutatorResult {
    /// Create a new, empty, mutation result.
    pub fn new() -> MutatorResult {
        MutatorResult {
            mutation_type: None,
            index: None,
            mutated_node_id: None,
            random_behavior_hash: None,
        }
    }
}

impl Default for MutatorResult {
    fn default() -> Self {
        Self::new()
    }
}
