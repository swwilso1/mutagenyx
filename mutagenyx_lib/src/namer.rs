//! The `namer` module provides the [`Namer`] trait.

/// A trait that grants an implementer the ability to return a string for the name
/// of the trait object.
pub trait Namer<T> {
    /// Return a String representing the name of self.
    fn get_name(&self, thing: &T) -> Option<String>;
}
