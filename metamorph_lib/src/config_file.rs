/// The `config_file` module contains code for reading a Metamorph configuration file.
/// Metamorph uses the configuration files to allow the user to control the tool behavior
/// for each source or AST file with specific settings that apply to just that file.
use crate::json::*;
use crate::language::Language;
use crate::preferences::Preferences;
use crate::{MetamorphError, MutationType};
use jsonxf;
use serde_json::{from_str, json, Value};
use std::ffi::OsStr;
use std::io::Write;
use std::{path::PathBuf, str::FromStr};

/// The extension used for metamorph configuration files.
pub static CONFIG_FILE_EXTENSION: &str = "morph";

/// The key in the configuration file for the language.
pub static LANGUAGE_KEY: &str = "language";

/// The key in the configuration file for the file.
pub static FILENAME_KEY: &str = "filename";

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

/// Configuration details loaded from a .morph configuration file.
pub struct ConfigurationFileDetails {
    /// Language specified in configuration file.
    pub language: Option<Language>,

    /// Path to either a source file or AST file to mutate.
    pub filename: PathBuf,

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
}

impl ConfigurationFileDetails {
    /// Create a new configuration details object
    ///
    /// # Arguments
    ///
    /// * `language` - The language for the file.
    /// * `filename` - The path to the file to mutate.
    /// * `number_of_mutants` - The number of mutants to attempt to generate.
    /// * `seed` - The random number generator seed.
    /// * `mutations` - The array of mutation algorithms.
    /// * `all_mutations` - True if the tool should use all mutation algorithms to generate mutants.
    /// * `compiler_details` - The information needed to properly invoke the compiler.
    pub fn new(
        language: Option<Language>,
        filename: String,
        number_of_mutants: i64,
        seed: Option<u64>,
        mutations: &[MutationType],
        all_mutations: bool,
        compiler_details: Option<Preferences>,
    ) -> ConfigurationFileDetails {
        let filename_pathbuf = PathBuf::from(filename);
        ConfigurationFileDetails {
            language,
            filename: filename_pathbuf,
            number_of_mutants,
            seed,
            mutations: mutations.to_owned(),
            all_mutations,
            compiler_details,
        }
    }

    /// Create a new configuration details object by loading the configuration from a JSON
    /// file.  The file must have the extension ".morph".
    ///
    /// # Arguments
    ///
    /// * `config_file` - The path to the configuration file in the file system.
    pub fn new_from_file(config_file: &str) -> Result<ConfigurationFileDetails, MetamorphError> {
        let config_path = PathBuf::from_str(config_file).unwrap();
        let extension = config_path.extension();

        // The file must have a .morph extension.
        if extension.is_none() {
            return Err(MetamorphError::ConfigFileBadExtension(String::from(
                config_file,
            )));
        }

        if extension.unwrap() != OsStr::new(CONFIG_FILE_EXTENSION) {
            return Err(MetamorphError::ConfigFileBadExtension(String::from(
                config_file,
            )));
        }

        let mut details = ConfigurationFileDetails {
            language: None,
            filename: PathBuf::new(),
            compiler_details: None,
            number_of_mutants: 5,
            // TODO: casting to u64 truncates data here, find something better.
            seed: None,
            mutations: Vec::new(),
            all_mutations: false,
        };

        if let Ok(json_value) = load_json_from_file_with_name(config_file) {
            let mut missing_keys: Vec<String> = Vec::new();

            if !json_value.contains_key(LANGUAGE_KEY) {
                missing_keys.push(String::from(LANGUAGE_KEY));
            }

            if !json_value.contains_key(FILENAME_KEY) {
                missing_keys.push(String::from(FILENAME_KEY));
            }

            if !missing_keys.is_empty() {
                return Err(MetamorphError::ConfigFileMissingRequiredKey(
                    String::from(config_file),
                    missing_keys,
                ));
            }

            let language_str = json_value.get_str_for_key(LANGUAGE_KEY).unwrap();

            let language = match Language::from_str(language_str) {
                Ok(l) => l,
                Err(_e) => {
                    return Err(MetamorphError::ConfigFileUnsupportedLanguage(
                        String::from(config_file),
                        String::from(language_str),
                    ))
                }
            };

            details.language = Some(language);

            let filename_str = json_value.get_str_for_key(FILENAME_KEY).unwrap();

            details.filename = PathBuf::from_str(filename_str).unwrap();

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
        } else {
            return Err(MetamorphError::ConfigFileNotSupported(String::from(
                config_file,
            )));
        }

        Ok(details)
    }

    /// Create a configuration file in JSON form and write it to the path at `config_file`.
    ///
    /// # Arguments
    ///
    /// * `config_file` - The path in the file system to write the configuration file.
    pub fn write_to_file_as_json(&self, config_file: &str) -> Result<(), MetamorphError> {
        let json_text = String::from("{}");

        let mut json_value: Value = from_str(&json_text).unwrap();

        if let Some(language) = &self.language {
            json_value.set_str_for_key(LANGUAGE_KEY, &language.to_string());
        }

        json_value.set_str_for_key(FILENAME_KEY, self.filename.to_str().unwrap());

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
}
