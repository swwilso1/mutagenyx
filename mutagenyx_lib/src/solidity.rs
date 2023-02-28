//! The `solidity` module contains all the Solidity language specific implementation details.

pub mod ast;
mod commenter;
pub mod compiler_details;
pub mod delegate;
mod language_interface;
mod mutators;
mod node_finder;
mod pretty_printer;
