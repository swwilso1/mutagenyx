//! # Mutagenyx Lib
//!
//! `mutagenyx_lib` provides an API to use for an application to mutate abstract syntax trees
//! of different programming languages.  The library also contains all the code required to
//! mutate each supported language.
//!
//! ## Mutagenyx Lib Design
//!
//! `mutagenyx_lib` provides generic interfaces that allow the library user to accomplish the
//! the following for supported programming languages:
//! - Recognize the language represented in a source or AST input file.
//! - Select mutation algorithms to use to mutate the input files.
//! - Mutate the program source.
//! - Output a reconstruction of the mutated source AST.

pub use self::error::MutagenyxError;
pub use self::language::Language;
pub use self::language_interface::LanguageInterface;
pub use self::language_interface::MutableLanguage;
pub use self::mutation::MutationType;
pub use self::pretty_printer::PrettyPrinter;
pub use self::recognizer::Recognizer;
pub use self::super_ast::SuperAST;

mod ast;
mod commenter;
pub mod compiler_details;
pub mod config_file;
pub mod error;
mod id;
mod json;
mod json_ast;
mod json_ast_id_maker;
mod json_ast_language_interface;
mod json_ast_permitter;
mod json_comment_helpers;
mod json_comment_inserter;
mod json_finder_helpers;
mod json_language_delegate;
pub mod language;
pub mod language_interface;
pub mod mutation;
mod mutation_visitor;
mod mutator;
mod node_finder;
mod node_printer;
mod node_printer_helpers;
mod operators;
pub mod permissions;
mod permit;
pub mod preferences;
mod pretty_print_visitor;
pub mod pretty_printer;
pub mod recognizer;
mod solidity;
pub mod string;
pub mod super_ast;
mod utility;
mod visitor;
mod vyper;
