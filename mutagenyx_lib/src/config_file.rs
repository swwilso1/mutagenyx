/// The `config_file` module contains code for reading a Mutagenyx configuration file.
/// Mutagenyx uses the configuration files to allow the user to control the tool behavior
/// for each source or AST file with specific settings that apply to just that file.
use crate::json::*;
use crate::language::Language;
use crate::preferences::Preferences;
use crate::{MutagenyxError, MutationType};
use jsonxf;
use serde_json::{from_str, json, Value};
use std::ffi::OsStr;
use std::io::Write;
use std::{path::PathBuf, str::FromStr};

/// The extension used for mutagenyx configuration files.
pub static CONFIG_FILE_EXTENSION: &str = "mgnx";

/// The key in the configuration file for the language.
pub static LANGUAGE_KEY: &str = "language";

/// The key in the configuration file for the file.
pub static FILENAMES_KEY: &str = "filenames";

/// The key in the configuration file for the compiler details.
pub static COMPILER_DETAILS_KEY: &str = "compiler-details";

/// The number-of-mutants key.
pub static NUMBER_OF_MUTANTS_KEY: &str = "num-mutants";

/// The seed key.
pub static SEED_KEY: &str = "seed";

/// The key for the list of mutation algorithms.
pub static MUTATIONS_KEY: &str = "mutations";

/// The key for the boolean value to use all mutation algorithms.
pub static ALL_MUTATIONS_KEY: &str = "all-mutations";

/// The key for the list of functions to mutate.
pub static FUNCTIONS_KEY: &str = "functions";

/// The key to indicate whether the tool should compile the mutants.
pub static VALIDATE_MUTANTS_KEY: &str = "validate-mutants";

/// The key for the string value containing the path to where the tool should place
/// output.
pub static OUTPUT_DIR_KEY: &str = "output-directory";

/// The key for the boolean value indicating whether tool should pretty-print the input
/// source or AST file.
pub static PRINT_ORIGINAL_KEY: &str = "print-original";

/// Configuration details loaded from a .mgnx configuration file.
pub struct ConfigurationFileDetails {
    /// Language specified in configuration file.
    pub language: Option<Language>,

    /// Path to either a source file or AST file to mutate.
    pub filenames: Vec<PathBuf>,

    /// The number of mutations to generate for the file.
    pub number_of_mutants: i64,

    /// The random number generator seed, if present from the configuration file.
    pub seed: Option<u64>,

    /// The list of requested mutations.
    pub mutations: Vec<MutationType>,

    /// True if the tool should use all valid mutation algorithms.
    pub all_mutations: bool,

    /// Details for compiler invocation.
    pub compiler_details: Option<Preferences>,

    /// List of names of functions to mutate.  If the list is empty, all functions can be mutated.
    pub functions: Vec<String>,

    /// True if the tool should compile the mutants for viability.
    pub verify_mutants: bool,

    /// The location where the tool should put generated output files.
    pub output_directory: Option<PathBuf>,

    /// True if the tool should pretty-print the original source or AST file.
    pub print_original: bool,
}

impl ConfigurationFileDetails {
    /// Create a new configuration details object by loading the configuration from a JSON
    /// file.  The file must have the extension ".mgnx".
    ///
    /// # Arguments
    ///
    /// * `config_file` - The path to the configuration file in the file system.
    pub fn new_from_file(config_file: &str) -> Result<ConfigurationFileDetails, MutagenyxError> {
        let config_path = PathBuf::from_str(config_file).unwrap();
        let extension = config_path.extension();

        // The file must have a .mgnx extension.
        if extension.is_none() {
            return Err(MutagenyxError::ConfigFileBadExtension(String::from(
                config_file,
            )));
        }

        if extension.unwrap() != OsStr::new(CONFIG_FILE_EXTENSION) {
            return Err(MutagenyxError::ConfigFileBadExtension(String::from(
                config_file,
            )));
        }

        let mut details = ConfigurationFileDetails {
            language: None,
            filenames: Vec::new(),
            compiler_details: None,
            number_of_mutants: 5,
            // TODO: casting to u64 truncates data here, find something better.
            seed: None,
            mutations: Vec::new(),
            all_mutations: false,
            functions: Vec::new(),
            verify_mutants: false,
            output_directory: None,
            print_original: false,
        };

        if let Ok(json_value) = load_json_from_file_with_name(config_file) {
            let mut missing_keys: Vec<String> = Vec::new();

            if !json_value.contains_key(LANGUAGE_KEY) {
                missing_keys.push(String::from(LANGUAGE_KEY));
            }

            if !json_value.contains_key(FILENAMES_KEY) {
                missing_keys.push(String::from(FILENAMES_KEY));
            }

            if !missing_keys.is_empty() {
                return Err(MutagenyxError::ConfigFileMissingRequiredKey(
                    String::from(config_file),
                    missing_keys,
                ));
            }

            let language_str = json_value.get_str_for_key(LANGUAGE_KEY).unwrap();

            let language = match Language::from_str(language_str) {
                Ok(l) => l,
                Err(_e) => {
                    return Err(MutagenyxError::ConfigFileUnsupportedLanguage(
                        String::from(config_file),
                        String::from(language_str),
                    ))
                }
            };

            details.language = Some(language);

            if let Some(filenames_array) = json_value.get_array_for_key(FILENAMES_KEY) {
                details.filenames = filenames_array
                    .iter()
                    .filter(|v| v.is_string())
                    .map(|v| PathBuf::from_str(v.as_str().unwrap()).unwrap())
                    .collect();
            }

            if let Some(number_of_mutants) = json_value.get_int_for_key(NUMBER_OF_MUTANTS_KEY) {
                details.number_of_mutants = number_of_mutants;
            }

            if let Some(seed) = json_value.get_int_for_key(SEED_KEY) {
                details.seed = Some(seed as u64);
            }

            if let Some(mutations_array) = json_value.get_array_for_key(MUTATIONS_KEY) {
                details.mutations = mutations_array
                    .iter()
                    .filter(|v| v.is_string())
                    .map(|v| v.as_str().unwrap())
                    .filter(|v| matches!(MutationType::from_str(v), Ok(_l)))
                    .map(|v| MutationType::from_str(v).unwrap())
                    .collect();
            }

            if let Some(use_all_algorithms) = json_value.get_bool_for_key(ALL_MUTATIONS_KEY) {
                details.all_mutations = use_all_algorithms;
            }

            if let Some(compiler_details_node) = json_value.get(COMPILER_DETAILS_KEY) {
                match Preferences::try_from(compiler_details_node.clone()) {
                    Ok(p) => details.compiler_details = Some(p),
                    Err(e) => return Err(e),
                }
            }

            if let Some(functions_array) = json_value.get_array_for_key(FUNCTIONS_KEY) {
                details.functions = functions_array
                    .iter()
                    .filter(|v| v.is_string())
                    .map(|v| String::from(v.as_str().unwrap()))
                    .collect();
            }

            if let Some(check) = json_value.get_bool_for_key(VALIDATE_MUTANTS_KEY) {
                details.verify_mutants = check;
            }

            if let Some(output_directory) = json_value.get_str_for_key(OUTPUT_DIR_KEY) {
                details.output_directory = Some(PathBuf::from(output_directory));
            }

            if let Some(print_original) = json_value.get_bool_for_key(PRINT_ORIGINAL_KEY) {
                details.print_original = print_original;
            }
        } else {
            return Err(MutagenyxError::ConfigFileNotSupported(String::from(
                config_file,
            )));
        }

        Ok(details)
    }

    /// Helper function to get the configuration as a JSON object.
    fn convert_to_json(&self) -> Result<Value, MutagenyxError> {
        let json_text = String::from("{}");

        let mut json_value: Value = from_str(&json_text).unwrap();

        if let Some(language) = &self.language {
            json_value.set_str_for_key(LANGUAGE_KEY, &language.to_string());
        }

        let json_filenames_array: Vec<Value> = self
            .filenames
            .iter()
            .map(|p| Value::from(p.to_str().unwrap()))
            .collect();

        json_value.set_node_for_key(FILENAMES_KEY, json![json_filenames_array]);

        json_value.set_node_for_key(NUMBER_OF_MUTANTS_KEY, json![self.number_of_mutants]);

        if let Some(seed) = self.seed {
            json_value.set_node_for_key(SEED_KEY, json![seed]);
        }

        if !self.mutations.is_empty() {
            let mut mutation_algorithm_names_array: Vec<String> = Vec::new();

            for mutation in &self.mutations {
                mutation_algorithm_names_array.push(mutation.to_string());
            }
            json_value.set_node_for_key(MUTATIONS_KEY, json![mutation_algorithm_names_array]);
        }

        if self.all_mutations {
            json_value.set_node_for_key(ALL_MUTATIONS_KEY, json![self.all_mutations]);
        }

        if let Some(compiler_details) = &self.compiler_details {
            let details_value = Value::try_from(compiler_details.clone())?;
            json_value.set_node_for_key(COMPILER_DETAILS_KEY, details_value);
        }

        if !self.functions.is_empty() {
            json_value.set_node_for_key(FUNCTIONS_KEY, json![self.functions]);
        }

        json_value.set_node_for_key(VALIDATE_MUTANTS_KEY, json![self.verify_mutants]);

        if let Some(output_directory) = &self.output_directory {
            json_value.set_node_for_key(OUTPUT_DIR_KEY, json![output_directory.to_str()]);
        }

        json_value.set_node_for_key(PRINT_ORIGINAL_KEY, json![self.print_original]);

        Ok(json_value)
    }

    /// Create a configuration file in JSON form and write it to the path at `config_file`.
    ///
    /// # Arguments
    ///
    /// * `config_file` - The path in the file system to write the configuration file.
    pub fn write_to_file_as_json(&self, config_file: &str) -> Result<(), MutagenyxError> {
        let json_value: Value = self.convert_to_json()?;

        let config_file_path = PathBuf::from_str(config_file).unwrap();

        // Try to create the parent directory if it does not exist.
        std::fs::create_dir_all(config_file_path.parent().unwrap())?;

        let mut f = std::fs::File::create(config_file_path.to_str().unwrap())?;

        // Now pretty print the JSON
        let standard_json = format!("{json_value}");
        let pretty_json = jsonxf::pretty_print(&standard_json).unwrap();

        writeln!(f, "{pretty_json}")?;

        Ok(())
    }

    /// Convert a configuration to JSON and write the JSON to `stream`.
    ///
    /// # Arguments
    ///
    /// * `stream` - The stream that will receive the JSON.
    pub fn write_to_stream_as_json(&self, stream: &mut dyn Write) -> Result<(), MutagenyxError> {
        let json_value = self.convert_to_json()?;

        // Now pretty print the JSON
        let standard_json = format!("{json_value}");
        let pretty_json = jsonxf::pretty_print(&standard_json).unwrap();

        writeln!(stream, "{pretty_json}")?;

        Ok(())
    }
}
