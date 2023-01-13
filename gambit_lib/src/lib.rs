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

/// The `language_interface` module provides abstractions for interacting with a generic form
/// of a language AST.
pub mod language_interface;

/// The `mutation` module contains the enumeration of the mutation algorithms supported by the
/// library.
pub mod mutation;

/// The `mutator` module provides traits for objects that perform mutation algorithms.
mod mutator;

/// The `pretty_printer` module contains a low-level stream 'token' emitter to use when
/// reconstructing source code from an AST.
pub mod pretty_printer;

/// The `recognizer` module contains a generic interface to code that will check a file's
/// contents in order to determine if the file contains support programming language source
/// code, or the language's AST.
pub mod recognizer;

/// The `solidity` module contains all the Solidity language specific implementation details.
mod solidity;

/// The `super_ast` module contains the SuperAST enumeration which allows a more abstract,
/// `generic` wrapper of a language specific abstract syntax tree.
pub mod super_ast;

/// The `visitor` module provides the traits that describe the functionality an object must have
/// to traverse an abstract syntax tree.
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
