//! The `error` module contains `MetamorphError`, the error enumeration used to communicate
//! library errors.

use std::convert::From;
use thiserror::Error;

/// The list of errors that the library can generate.
#[derive(Error, Debug)] // , PartialEq)]
pub enum MetamorphError {
    #[error("IO error: {0}")]
    IO(std::io::Error),

    /// An error indicating that JSON parsing failed.
    #[error("JSON error occurred: {0}")]
    JSON(serde_json::Error),

    /// An error indicating that the AST does not have any nodes suitable for
    /// mutating the AST according to the specified mutation algorithms.
    #[error("AST does not contain any mutable node for requested mutations")]
    NoMutableNode,

    /// An error indicating that an API call attempted to load facilities for
    /// a language not supported by metamorph_lib.
    #[error("Language {0} not supported")]
    LanguageNotSupported(String),

    /// An error indicating a function tried to operate on an AST associated with
    /// a language that the function does not recognize.
    #[error("Unable to determine language from input file")]
    LanguageNotRecognized,

    /// An error indicating that a function tried to access a low level AST not supported
    /// by the current language module.
    #[error("Language does not support this AST type")]
    ASTTypeNotSupported,

    /// An error indicating that a language does not implement a mutation algorithm.
    #[error("Language does not support mutation algorithm: {0}")]
    MutationAlgorithmNotSupported(String),
}

impl From<std::io::Error> for MetamorphError {
    fn from(e: std::io::Error) -> Self {
        MetamorphError::IO(e)
    }
}

impl From<serde_json::Error> for MetamorphError {
    fn from(e: serde_json::Error) -> Self {
        MetamorphError::JSON(e)
    }
}
