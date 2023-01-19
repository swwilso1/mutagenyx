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

mod ast;
pub mod error;
mod json;
pub mod language;
pub mod language_interface;
pub mod mutation;
mod mutation_visitor;
mod mutator;
pub mod operators;
pub mod pretty_printer;
pub mod recognizer;
mod solidity;
pub mod super_ast;
mod visitor;
mod vyper;
