//! # Gambit Lib
//!
//! `gambit_lib` provides an API to use for an application to mutate abstract syntax trees
//! of different programming languages.  The library also contains all the code required to
//! mutate each supported language.
//!
//! ## Gambit Lib Design
//!
//! `gambit_lib` provides generic interfaces that allow the library user to accomplish the
//! the following for supported programming languages:
//! - Recognize the language represented in a source or AST input file.
//! - Select mutation algorithms to use to mutate the input files.
//! - Mutate the program source.
//! - Output a reconstruction of the mutated source AST.

pub use self::error::GambitError;
pub use self::language::Language;
pub use self::language_interface::LanguageInterface;
pub use self::language_interface::MutableLanguage;
pub use self::mutation::MutationType;
pub use self::pretty_printer::PrettyPrinter;
pub use self::recognizer::Recognizer;
pub use self::super_ast::SuperAST;

/// The `ast` module contains the AST trait intended for languages to implement in order to
/// traverse an abstract syntax tree.
mod ast;

/// The `error` module contains `GambitError`, the error enumeration used to communicate
/// library errors.
pub mod error;
mod json;
pub mod language;
pub mod language_interface;
pub mod mutation;
mod mutator;
pub mod pretty_printer;
pub mod recognizer;
mod solidity;
pub mod super_ast;
mod visitor;

// The `vyper` module contains all the Vyper language specific implementation details.
mod vyper;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
