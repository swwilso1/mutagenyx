use mutagenyx_lib::compiler_details::*;
use mutagenyx_lib::language::Language;
use mutagenyx_lib::preferences::Preferences;

/// Simple struct to collect references to compiler paths.
pub struct CompilerPaths<'a> {
    /// Path to solidity compiler.
    pub solidity: &'a String,

    /// Path to vyper compiler.
    pub vyper: &'a String,
}

impl<'a> CompilerPaths<'a> {
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
        let mut solidity_compiler_prefs = Preferences::new();
        solidity_compiler_prefs.set_string_for_key(PATH_KEY, self.solidity);

        let mut solidity_prefs = Preferences::new();
        solidity_prefs.set_preferences_for_key(COMPILER_KEY, solidity_compiler_prefs);

        let mut vyper_compiler_prefs = Preferences::new();
        vyper_compiler_prefs.set_string_for_key(PATH_KEY, self.vyper);

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
