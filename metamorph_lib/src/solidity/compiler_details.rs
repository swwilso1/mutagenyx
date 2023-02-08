/// The `compiler_details` module provides Solidity specific compiler configuration information
/// for invoking the Solidity compiler to generate an AST and possibly to compile a file.
use crate::json::*;
use serde_json::{from_str, json, Value};
use std::{path::PathBuf, str::FromStr};

static PATH_KEY: &str = "path";
static BASE_PATH_KEY: &str = "base-path";
static INCLUDE_PATHS_KEY: &str = "include-paths";
static REMAPPINGS_KEY: &str = "remappings";

/// Details needed to invoking the Solidity compiler for a specific task such as generating an
/// AST, or compiling a file.
#[derive(Debug, Clone)]
pub struct SolidityCompilerDetails {
    /// Path to the compiler
    pub path: PathBuf,

    /// Base path value for Solidity --base-path compiler command-line flag.
    /// This value is optional.
    pub base_path: Option<PathBuf>,

    /// Include path values for Solidity --include-path compiler command-line flag.
    /// The `base_path` struct member must not be `None` for these path settings to apply.
    pub include_paths: Vec<PathBuf>,

    /// Solidity compiler remap paths of the form context:prefix=path (example:
    /// dapp-bin=/usr/local/lib/dapp-bin).
    pub remappings: Vec<String>,
}

impl SolidityCompilerDetails {
    /// Create a new Solidity compiler details object.
    pub fn new() -> SolidityCompilerDetails {
        SolidityCompilerDetails {
            path: PathBuf::from_str("solc").unwrap(),
            base_path: None,
            include_paths: Vec::new(),
            remappings: Vec::new(),
        }
    }

    /// Create a new solidity compiler details object from arguments
    ///
    /// # Arguments
    ///
    /// * `path` - The location of the compiler in the file system.
    /// * `base_path` - The optional path to use for the compiler --base-path flag.
    /// * `include_paths` - The array of paths to use with the --include-path compiler flag.
    /// * `remappings` - The array of strings containing context:prefix=path mappings.
    pub fn new_from_args(
        path: &str,
        base_path: Option<&str>,
        include_paths: Option<&Vec<&str>>,
        remappings: Option<&Vec<&str>>,
    ) -> SolidityCompilerDetails {
        let base_path_buf = match base_path {
            Some(p) => Some(PathBuf::from_str(p).unwrap()),
            None => None,
        };
        let mut my_include_paths: Vec<PathBuf> = Vec::new();

        if let Some(paths) = &include_paths {
            for path in *paths {
                my_include_paths.push(PathBuf::from_str(*path).unwrap());
            }
        }

        let mut my_remappings: Vec<String> = Vec::new();

        if let Some(mappings) = &remappings {
            for mapping in *mappings {
                my_remappings.push(String::from(*mapping));
            }
        }

        SolidityCompilerDetails {
            path: PathBuf::from_str(path).unwrap(),
            base_path: base_path_buf,
            include_paths: my_include_paths,
            remappings: my_remappings,
        }
    }

    /// Create a new Solidity compiler details object from JSON.
    ///
    /// # Arguments
    ///
    /// * `value` - The JSON object that should contain the compiler details.
    pub fn new_from_json(value: &Value) -> SolidityCompilerDetails {
        let mut details = SolidityCompilerDetails::new();

        if let Some(compiler_path) = value.get_str_for_key(PATH_KEY) {
            details.path = PathBuf::from_str(compiler_path).unwrap();
        }

        if let Some(base_path) = value.get_str_for_key(BASE_PATH_KEY) {
            details.base_path = Some(PathBuf::from_str(base_path).unwrap());
        }

        if let Some(include_path_array) = value.get_array_for_key(INCLUDE_PATHS_KEY) {
            details.include_paths = include_path_array
                .iter()
                .filter(|v| v.is_string())
                .map(|v| PathBuf::from_str(v.as_str().unwrap()).unwrap())
                .collect();
        }

        if let Some(remappings_array) = value.get_array_for_key(REMAPPINGS_KEY) {
            details.remappings = remappings_array
                .iter()
                .filter(|v| v.is_string())
                .map(|v| String::from(v.as_str().unwrap()))
                .collect();
        }

        details
    }

    /// Convert a compiler details object to a JSON representation.
    pub fn to_json(&self) -> Value {
        let json_text = String::from("{}");

        let mut json_value: Value = from_str(&json_text).unwrap();

        json_value.set_str_for_key(PATH_KEY, self.path.to_str().unwrap());

        if let Some(base_path) = &self.base_path {
            json_value.set_str_for_key(BASE_PATH_KEY, base_path.to_str().unwrap());
        }

        if self.include_paths.len() > 0 {
            let mut include_path_array: Vec<String> = Vec::new();
            for include_path in &self.include_paths {
                include_path_array.push(String::from(include_path.to_str().unwrap()));
            }
            json_value.set_node_for_key(INCLUDE_PATHS_KEY, json![include_path_array]);
        }

        if self.remappings.len() > 0 {
            json_value.set_node_for_key(REMAPPINGS_KEY, json![self.remappings]);
        }

        json_value
    }
}
