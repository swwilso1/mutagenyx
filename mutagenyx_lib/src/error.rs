//! The `error` module contains `MutagenyxError`, the error enumeration used to communicate
//! library errors.

use std::convert::From;
use std::time::SystemTimeError;
use thiserror::Error;

/// The list of errors that the library can generate.
#[derive(Error, Debug)] // , PartialEq)]
pub enum MutagenyxError {
    #[error("IO error: {0}")]
    IO(std::io::Error),

    /// An error indicating that JSON parsing failed.
    #[error("JSON error occurred: {0}")]
    JSON(serde_json::Error),

    /// An error indicating the tool found an unrecognized JSON element.
    #[error("Unrecognized JSON element: {0}")]
    UnrecognizedJSON(String),

    /// An error indicating that the AST does not have any nodes suitable for
    /// mutating the AST according to the specified mutation algorithms.
    #[error("AST does not contain any mutable node for requested mutations")]
    NoMutableNode,

    /// An error indicating that an API call attempted to load facilities for
    /// a language not supported by mutagenyx_lib.
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

    /// An error indicating that source file did not compile.
    #[error("Source file {0} would not compile")]
    SourceDoesNotCompile(String),

    /// An error indicating the tool could not determine the compiler version.
    #[error("Compiler {0} does not report its version number")]
    CompilerNoVersion(String),

    /// An error indicating that the tool received a config file that it does not support or
    /// cannot support in the current function.
    #[error("Configuration file {0} not supported")]
    ConfigFileNotSupported(String),

    /// An error indicating that the tool received a configuration file that does not have the
    /// correct file extension.
    #[error("Configuration file {0} does not have the correct extension")]
    ConfigFileBadExtension(String),

    /// An error indicating that configuration file keys are missing.
    #[error("Configuration file {0} does not have keys: {1:?}")]
    ConfigFileMissingRequiredKey(String, Vec<String>),

    /// An error indicating the configuration file contains an unsupported value for the language
    /// key.
    #[error("Configuration file {0} contains an invalid value for the language key: {1}")]
    ConfigFileUnsupportedLanguage(String, String),

    /// An error indicating an attempt to work with system time failed.
    #[error("Request for or operation on system time failed")]
    SystemTime,
}

impl From<std::io::Error> for MutagenyxError {
    fn from(e: std::io::Error) -> Self {
        MutagenyxError::IO(e)
    }
}

impl From<serde_json::Error> for MutagenyxError {
    fn from(e: serde_json::Error) -> Self {
        MutagenyxError::JSON(e)
    }
}

impl From<SystemTimeError> for MutagenyxError {
    fn from(_: SystemTimeError) -> Self {
        MutagenyxError::SystemTime
    }
}
