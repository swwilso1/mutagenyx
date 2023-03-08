//! The `mutation_visitor` module provides one [`Visitor`] type for traversing an AST and counting
//! mutable nodes and one [`VisitorMut`] type for traversing an AST and mutating nodes.

use crate::id::Id;
use crate::mutation::MutationType;
use crate::mutator::*;
use crate::mutator_result::MutatorResult;
use crate::namer::Namer;
use crate::permissions::*;
use crate::permit::*;
use crate::visitor::*;
use crate::MutagenyxError;
use rand_pcg::*;
use std::collections::HashMap;

/// Helper function that returns true if the permitter allows the traverser permission to visit `node`.
///
/// # Arguments
///
/// * `namer` - A [`Namer<AST>`] trait object that can convert an AST node to a name string.
/// * `permitter` - A [`Permit<AST>`] trait object that can grant permission for a requested action.
/// * `node` - An AST object.
fn has_permission_to_visit<AST>(
    namer: &dyn Namer<AST>,
    permitter: &dyn Permit<AST>,
    node: &AST,
) -> bool {
    if let Some(node_name) = namer.get_name(node) {
        permitter.has_permission_to(PermissionAction::Visit(PermissionScope::Name(
            PermissionObject::Value(node_name, Permission::Allow),
        )))
    } else {
        permitter.has_permission_to(PermissionAction::Visit(PermissionScope::Any(
            Permission::Allow,
        )))
    }
}

/// Helper function that returns true if the permitter allows the traverser permission to mutate `node`.
///
/// # Arguments
///
/// * `namer` - A [`Namer<AST>`] trait object that can convert an AST node to a name string.
/// * `permitter` - A [`Permit<AST>`] trait object that can grant permission for a requested action.
/// * `node` - An AST object.
fn has_permission_to_mutate<AST>(
    namer: &dyn Namer<AST>,
    permitter: &dyn Permit<AST>,
    node: &AST,
) -> bool {
    if let Some(node_name) = namer.get_name(node) {
        permitter.has_permission_to(PermissionAction::Mutate(PermissionScope::Name(
            PermissionObject::Value(node_name, Permission::Allow),
        )))
    } else {
        permitter.has_permission_to(PermissionAction::Mutate(PermissionScope::Name(
            PermissionObject::Any(Permission::Allow),
        )))
    }
}

/// Helper function that returns true if the permitter allows the traverser permission to mutate
/// the children of `node`.
///
/// # Arguments
///
/// * `namer` - A [`Namer<AST>`] trait object that can convert an AST node to a name string.
/// * `permitter` - A [`Permit<AST>`] trait object that can grant permission for a requested action.
/// * `node` - An AST object.
fn has_permission_to_mutate_children<AST>(
    namer: &dyn Namer<AST>,
    permitter: &dyn Permit<AST>,
    node: &AST,
) -> bool {
    if let Some(node_name) = namer.get_name(node) {
        permitter.has_permission_to(PermissionAction::Mutate(PermissionScope::Children(
            PermissionObject::Value(node_name, Permission::Allow),
        )))
    } else {
        permitter.has_permission_to(PermissionAction::Mutate(PermissionScope::Children(
            PermissionObject::Any(Permission::Allow),
        )))
    }
}

pub type NodePath = Vec<u64>;
pub type NodePathMap = HashMap<u64, NodePath>;

/// Visitor for creating a path analysis of the AST tree.
pub struct PathVisitor<'a, AST> {
    /// A [`Permit`] trait object that responds to permission queries.
    permitter: Box<dyn Permit<AST> + 'a>,

    /// A [`Id<AST>`] object that can generate unique ids for each node in the AST.
    id_maker: Box<dyn Id<AST> + 'a>,

    /// The map of paths to each node in the AST.
    pub path_map: NodePathMap,

    /// The current path vector updated as the visitor moves through the AST.
    current_path: NodePath,

    /// A [`Namer<AST>`] trait object that can convert an AST node to a name string.
    namer: Box<dyn Namer<AST> + 'a>,
}

impl<'a, AST> PathVisitor<'a, AST> {
    /// Create a new path visitor object.
    ///
    /// # Arguments
    ///
    /// * `permitter` - A [`Permit<AST>`] object that responds to permission queries.
    /// * `id_maker` - A [`Id<AST>`] object that can uniquely identify each node in the AST.
    /// * `namer` - A [`Namer<AST>`] object that can generate a name from an AST node.
    pub fn new(
        permitter: Box<dyn Permit<AST> + 'a>,
        id_maker: Box<dyn Id<AST> + 'a>,
        namer: Box<dyn Namer<AST> + 'a>,
    ) -> PathVisitor<'a, AST> {
        PathVisitor {
            permitter,
            id_maker,
            path_map: NodePathMap::new(),
            current_path: NodePath::new(),
            namer,
        }
    }
}

impl<'a, AST> Visitor<AST> for PathVisitor<'a, AST> {
    fn have_permission_to_visit(&self, node: &AST) -> bool {
        has_permission_to_visit(self.namer.as_ref(), self.permitter.as_ref(), node)
    }

    fn visit(&mut self, node: &AST) -> bool {
        if let Some(node_id) = self.id_maker.get_id(node) {
            let mut current_path_copy = self.current_path.clone();
            current_path_copy.push(node_id);
            self.path_map.insert(node_id, current_path_copy);
        }
        false
    }

    fn on_start_visit_children(&mut self, node: &AST) {
        if let Some(node_id) = self.id_maker.get_id(node) {
            self.current_path.push(node_id);
        }
    }

    fn on_end_visit_children(&mut self, node: &AST) {
        if self.id_maker.get_id(node).is_some() {
            self.current_path.pop();
        }
    }
}

/// The struct for traversing an AST and counting mutable nodes in the AST.  The type conforms/adopts
/// the [`Visitor`] trait.
pub struct MutableNodesCounter<'a, AST> {
    /// Table for counting the number of mutable nodes by [`MutationType`].
    pub counter_table: HashMap<MutationType, u64>,

    /// A reference to the map of mutators by [`MutationType`].
    mutator_map: &'a mut HashMap<MutationType, Box<dyn Mutator<AST>>>,

    /// A [`Permit`] trait object that responds to permission queries.
    permitter: Box<dyn Permit<AST> + 'a>,

    /// A reference to the random number given to the mutator to use for mutating the node.
    rng: &'a mut Pcg64,

    /// A [`Namer<AST>`] trait object that can convert an AST node to a name string.
    namer: Box<dyn Namer<AST> + 'a>,

    /// True if the visitor is traversing the children of a mutable node and does not need to
    /// check the permission for each child.
    skip_mutation_permission_check: bool,
}

impl<'a, AST> MutableNodesCounter<'a, AST> {
    /// Create the new node counter.
    ///
    /// # Arguments
    ///
    /// * `mutator_map` - A reference to an external map of mutators by [`MutationType`].
    /// * `permitter` - A [`Permit`] trait object that answers permission questions.
    /// * `namer` - A [`Namer<AST>`] object that can generate a name from an AST node.
    pub fn new(
        mutator_map: &'a mut HashMap<MutationType, Box<dyn Mutator<AST>>>,
        permitter: Box<dyn Permit<AST> + 'a>,
        rng: &'a mut Pcg64,
        namer: Box<dyn Namer<AST> + 'a>,
    ) -> MutableNodesCounter<'a, AST> {
        MutableNodesCounter {
            counter_table: HashMap::new(),
            mutator_map,
            permitter,
            rng,
            namer,
            skip_mutation_permission_check: false,
        }
    }
}

impl<'a, AST> Visitor<AST> for MutableNodesCounter<'a, AST> {
    fn on_enter(&mut self, _node: &AST) {}

    fn have_permission_to_visit(&self, node: &AST) -> bool {
        has_permission_to_visit(self.namer.as_ref(), self.permitter.as_ref(), node)
    }

    fn visit(&mut self, node: &AST) -> bool {
        if self.skip_mutation_permission_check
            || has_permission_to_mutate(self.namer.as_ref(), self.permitter.as_ref(), node)
        {
            for (key, mutator) in self.mutator_map.iter_mut() {
                if mutator.is_mutable_node(node, self.rng) {
                    if self.counter_table.contains_key(key) {
                        let size = self.counter_table.get_mut(key).unwrap();
                        *size += 1;
                    } else {
                        self.counter_table.insert(*key, 1);
                    }
                }
            }
        }
        false
    }

    fn visit_children(&mut self, _node: &AST) -> bool {
        true
    }

    fn on_start_visit_children(&mut self, node: &AST) {
        if has_permission_to_mutate_children(self.namer.as_ref(), self.permitter.as_ref(), node) {
            self.skip_mutation_permission_check = true;
        }
    }

    fn on_end_visit_children(&mut self, node: &AST) {
        if has_permission_to_mutate_children(self.namer.as_ref(), self.permitter.as_ref(), node) {
            self.skip_mutation_permission_check = false;
        }
    }

    fn on_exit(&mut self, _node: &AST) {}
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
    mutator: &'a mut dyn Mutator<AST>,

    /// A reference to the random number given to the mutator to use for mutating the node.
    rng: &'a mut Pcg64,

    /// The index of node to mutate in the AST.
    index: usize,

    /// The current count of mutable nodes in the AST.
    current_index: usize,

    /// A [`Permit`] trait object that responds to permission queries.
    permitter: Box<dyn Permit<AST> + 'a>,

    /// A [`Id<AST>`] trait object that can produce an id value for a node.
    id_maker: Box<dyn Id<AST> + 'a>,

    /// The node id of the node that this visitor mutated.
    pub mutated_node_id: u64,

    /// A comment string retrieved from the mutator after it mutates a node.
    pub mutator_comment: Option<AST>,

    /// A [`Namer<AST>`] trait object that can convert an AST node to a name string.
    namer: Box<dyn Namer<AST> + 'a>,

    /// True if the visitor is mutating child nodes and can skip the node mutation permission check.
    skip_mutation_permission_check: bool,

    /// The cached results from the mutation.
    pub mutation_results: MutatorResult,

    /// A cached error from the mutator.
    pub error: Option<MutagenyxError>,
}

impl<'a, AST> MutationMaker<'a, AST> {
    /// Create the new mutation maker.
    ///
    /// # Arguments
    ///
    /// * `mutator` - The reference to the mutator.
    /// * `rng` - The reference to the random number generator.
    /// * `index` - The index of the node in the AST to mutate.
    /// * `permitter` - A [`Permit`] trait object that answers permission questions.
    /// * `namer` - A [`Namer<AST>`] object that can generate a name from an AST node.
    pub fn new(
        mutator: &'a mut dyn Mutator<AST>,
        rng: &'a mut Pcg64,
        index: usize,
        permitter: Box<dyn Permit<AST> + 'a>,
        id_maker: Box<dyn Id<AST> + 'a>,
        namer: Box<dyn Namer<AST> + 'a>,
    ) -> MutationMaker<'a, AST> {
        MutationMaker {
            mutator,
            rng,
            index,
            current_index: 0,
            permitter,
            id_maker,
            mutated_node_id: 0_u64,
            mutator_comment: None,
            namer,
            skip_mutation_permission_check: false,
            mutation_results: MutatorResult::new(),
            error: None,
        }
    }
}

impl<'a, AST> VisitorMut<AST> for MutationMaker<'a, AST> {
    fn on_enter(&mut self, _node: &mut AST) {}

    fn have_permission_to_visit(&self, node: &AST) -> bool {
        has_permission_to_visit(self.namer.as_ref(), self.permitter.as_ref(), node)
    }

    fn visit_mut(&mut self, node: &mut AST) -> bool {
        if (self.skip_mutation_permission_check
            || has_permission_to_mutate(self.namer.as_ref(), self.permitter.as_ref(), node))
            && self.mutator.is_mutable_node(node, self.rng)
        {
            if self.current_index == self.index {
                match self.mutator.mutate(node, self.rng) {
                    Ok(result) => {
                        self.mutation_results = result.clone();
                        if let Some(node_id) = result.mutated_node_id {
                            self.mutated_node_id = node_id;
                        } else if let Some(other_id) = self.id_maker.get_id(node) {
                            self.mutated_node_id = other_id;
                        }
                    }
                    Err(e) => {
                        self.error = Some(e);
                        return true;
                    }
                }
                self.mutator_comment = self.mutator.get_comment_node();
                return true;
            }
            self.current_index += 1;
        }
        false
    }

    fn visit_children(&mut self, _node: &mut AST) -> bool {
        true
    }

    fn on_start_visit_children(&mut self, node: &AST) {
        if has_permission_to_mutate_children(self.namer.as_ref(), self.permitter.as_ref(), node) {
            self.skip_mutation_permission_check = true;
        }
    }

    fn on_end_visit_children(&mut self, node: &AST) {
        if has_permission_to_mutate_children(self.namer.as_ref(), self.permitter.as_ref(), node) {
            self.skip_mutation_permission_check = false;
        }
    }

    fn on_exit(&mut self, _node: &mut AST) {}
}
