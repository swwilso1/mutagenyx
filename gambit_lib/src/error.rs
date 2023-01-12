use std::convert::From;
use std::io::ErrorKind;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum GambitError {
    #[error("Unable to convert {source_type} type to {destination_type} type")]
    Conversion {
        source_type: String,
        destination_type: String,
    },

    #[error("IO error: {kind}")]
    IO { kind: ErrorKind },

    #[error("JSON error occurred at {line}:{column}")]
    JSON {
        line: usize,
        column: usize,
        io_error: bool,
        syntax_error: bool,
        data_error: bool,
        eof: bool,
    },

    #[error("AST node {0} not found")]
    NodeNotFound(String),

    #[error("AST node {0} does not contain a value at index {1}")]
    NodeAtIndexNotFound(String, usize),

    #[error("Mutation failed: {0}")]
    MutationFailed(String),

    #[error("AST does not contain any mutable node for requested mutations")]
    NoMutableNode,

    #[error("Language {0} not supported")]
    LanguageNotSupported(String),

    #[error("Unable to determine language from input file")]
    LanguageNotRecognized,

    #[error("Language does not support this AST type")]
    ASTTypeNotSupported,
}

impl From<std::io::Error> for GambitError {
    fn from(e: std::io::Error) -> Self {
        GambitError::IO { kind: e.kind() }
    }
}

impl From<serde_json::Error> for GambitError {
    fn from(e: serde_json::Error) -> Self {
        GambitError::JSON {
            line: e.line(),
            column: e.column(),
            io_error: e.is_io(),
            syntax_error: e.is_syntax(),
            data_error: e.is_data(),
            eof: e.is_eof(),
        }
    }
}
