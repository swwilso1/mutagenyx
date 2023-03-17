//! The `compiler_settings` module provides the [`CompilerSettings`] type that can convert paths to
//! various compilers into a [`Preferences`] object usable by the mutation algorithms.

use mutagenyx_lib::compiler_details::*;
use mutagenyx_lib::language::Language;
use mutagenyx_lib::preferences::{PreferenceValue, Preferences};
use mutagenyx_lib::solidity::compiler_details::*;
use mutagenyx_lib::vyper::compiler_details::*;

pub struct SolidityCompilerSettings {
    /// Solidity compiler path
    pub solidity_compiler: String,

    /// Solidity base path (equivalent to --base-path)
    pub solidity_base_path: Option<String>,

    /// Solidity include paths (equivalent to --include-path)
    pub solidity_include_path: Vec<String>,

    /// Solidity allow paths (equivalent to --allow-paths)
    pub solidity_allow_paths: Vec<String>,

    /// Solidity re-mapping (context:prefix=path)
    pub solidity_remappings: Vec<String>,

    /// Solidity '--stop-after MODULE' flag.
    pub solidity_stop_after: Option<String>,
}

impl SolidityCompilerSettings {
    pub fn to_preferences(&self) -> Preferences {
        let mut preferences = Preferences::new();
        preferences.set_string_for_key(PATH_KEY, &self.solidity_compiler);

        if let Some(base_path) = &self.solidity_base_path {
            preferences.set_string_for_key(BASE_PATH_KEY, base_path.as_str());
        }

        if !self.solidity_include_path.is_empty() {
            let mut include_path_array: Vec<PreferenceValue> = Vec::new();
            for path in &self.solidity_include_path {
                include_path_array.push(PreferenceValue::String(path.clone()));
            }
            preferences.set_array_for_key(INCLUDE_PATHS_KEY, include_path_array);
        }

        if !self.solidity_allow_paths.is_empty() {
            let mut allow_path_array: Vec<PreferenceValue> = Vec::new();
            for path in &self.solidity_allow_paths {
                allow_path_array.push(PreferenceValue::String(path.clone()));
            }
            preferences.set_array_for_key(ALLOW_PATHS_KEY, allow_path_array);
        }

        if !self.solidity_remappings.is_empty() {
            let mut remappings_array: Vec<PreferenceValue> = Vec::new();
            for remapping in &self.solidity_remappings {
                remappings_array.push(PreferenceValue::String(remapping.clone()));
            }
            preferences.set_array_for_key(REMAPPINGS_KEY, remappings_array);
        }

        if let Some(module) = &self.solidity_stop_after {
            preferences.set_string_for_key(STOP_AFTER_KEY, module.as_str());
        }

        preferences
    }
}

pub struct VyperCompilerSettings {
    /// Vyper compiler path
    pub vyper_compiler: String,

    /// Vyper root path (equivalent to -p)
    pub vyper_root_path: Option<String>,
}

impl VyperCompilerSettings {
    pub fn to_preferences(&self) -> Preferences {
        let mut preferences = Preferences::new();
        preferences.set_string_for_key(PATH_KEY, &self.vyper_compiler);

        if let Some(root_path) = &self.vyper_root_path {
            preferences.set_string_for_key(ROOT_PATH_KEY, root_path.as_str());
        }

        preferences
    }
}

/// Simple struct to collect references to compiler paths.
pub struct CompilerSettings {
    /// Solidity settings
    pub solidity: SolidityCompilerSettings,

    /// Vyper settings
    pub vyper: VyperCompilerSettings,
}

impl CompilerSettings {
    /// Generate a basic language Preferences layout starting from compiler paths.
    ///
    /// The Preferences object is not JSON, but in JSON, the returned Preferences has the following
    /// form.
    ///
    /// {
    ///     "Solidity:" {
    ///         "compiler": {
    ///             "path": "..."
    ///         }
    ///     },
    ///     "Vyper": {
    ///         "compiler": {
    ///             "path": "..."
    ///         }
    ///     }
    /// }
    pub fn to_preferences(&self) -> Preferences {
        let solidity_compiler_prefs = self.solidity.to_preferences();

        let mut solidity_prefs = Preferences::new();
        solidity_prefs.set_preferences_for_key(COMPILER_KEY, solidity_compiler_prefs);

        let vyper_compiler_prefs = self.vyper.to_preferences();

        let mut vyper_prefs = Preferences::new();
        vyper_prefs.set_preferences_for_key(COMPILER_KEY, vyper_compiler_prefs);

        let mut preferences = Preferences::new();
        let solidity_key = format!("{}", Language::Solidity);
        let vyper_key = format!("{}", Language::Vyper);
        preferences.set_preferences_for_key(&solidity_key, solidity_prefs);
        preferences.set_preferences_for_key(&vyper_key, vyper_prefs);

        preferences
    }
}
