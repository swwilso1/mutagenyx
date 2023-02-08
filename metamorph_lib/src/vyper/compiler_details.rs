/// The `compiler_details` module provides Vyper specific compiler configuration information
/// for invoking the Vyper compiler to generate an AST and possibly to compile a file.
use crate::json::*;
use serde_json::{Value, from_str};
use std::{path::PathBuf, str::FromStr};

static PATH_KEY: &str = "path";
static ROOT_PATH_KEY: &str = "root-path";

/// Details needed to invoking the Vyper compiler for a specific task such as generating an
/// AST, or compiling a file.
#[derive(Debug, Clone)]
pub struct VyperCompilerDetails {
    /// Path to the compiler
    pub path: PathBuf,

    /// Root path for contract imports
    pub root_path: Option<PathBuf>,
}

impl VyperCompilerDetails {
    /// Create a Vyper compiler details object.
    pub fn new() -> VyperCompilerDetails {
        VyperCompilerDetails {
            path: PathBuf::from_str("vyper").unwrap(),
            root_path: None,
        }
    }

    /// Create a Vyper compiler details object from parameters.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the compiler.
    /// * `root_path` - The path in the file system to use as the root path for contract imports.
    pub fn new_from_args(
        path: &str,
        root_path: Option<&str>,
    ) -> VyperCompilerDetails {
        let root_path_option = match root_path {
            Some(p) => Some(PathBuf::from_str(p).unwrap()),
            _ => None
        };

        VyperCompilerDetails {
            path: PathBuf::from(path),
            root_path: root_path_option,
        }
    }

    /// Create a new Vyper compiler details object from JSON.
    ///
    /// # Arguments
    ///
    /// * `value` - The JSON object that should contain the compiler details.
    pub fn new_from_json(value: &Value) -> VyperCompilerDetails {
        let mut details = VyperCompilerDetails::new();

        if let Some(compiler_path) = value.get_str_for_key(PATH_KEY) {
            details.path = PathBuf::from_str(compiler_path).unwrap();
        }

        if let Some(root_path) = value.get_str_for_key(ROOT_PATH_KEY) {
            details.root_path = Some(PathBuf::from_str(root_path).unwrap());
        }

        details
    }

    /// Convert the compiler configuration details object to a JSON representation.
    pub fn to_json(&self) -> Value {
        let json_text = String::from("{}");

        let mut json_node: Value = from_str(&json_text).unwrap();

        json_node.set_str_for_key(PATH_KEY, self.path.to_str().unwrap());
        if let Some(root_path) = &self.root_path {
            json_node.set_str_for_key(ROOT_PATH_KEY, root_path.to_str().unwrap());
        }
        json_node
    }
}
