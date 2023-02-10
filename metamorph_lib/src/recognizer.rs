//! The `recognizer` module contains a generic interface to code that will check a file's
//! contents in order to determine if the file contains support programming language source
//! code, or the language's AST.

use crate::config_file::*;
use crate::json::*;
use crate::language::Language;
use crate::language_interface::*;
use crate::preferences::Preferences;
use crate::MetamorphError;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::str::FromStr;

/// Enumeration of possible file types that a [`Recognizer`] can identify.
#[derive(PartialEq, Eq)]
pub enum FileType {
    Source,
    AST,
    Config,
}

/// Result type to use for results from [`Recognizer`] functions.
#[derive(PartialEq, Eq)]
pub struct RecognizeResult {
    pub language: Language,
    pub file_type: FileType,
}

impl RecognizeResult {
    /// Create a new [`RecognizeResult`].
    ///
    /// # Arguments
    ///
    /// * `language` - The recognized [`Language`].
    /// * `file_type` - The recognized [`FileType`].
    fn new(language: Language, file_type: FileType) -> RecognizeResult {
        RecognizeResult {
            language,
            file_type,
        }
    }
}

/// Provides facilities for identifying the programming language used by a source file or an
/// abstract syntax tree of a program.
pub struct Recognizer<'a> {
    pub preferences: &'a mut Preferences,
}

impl<'a> Recognizer<'a> {
    /// Create a new recognizer object.
    ///
    /// # Arguments
    ///
    /// * `prefs` - A reference to a preferences object.
    pub fn new(prefs: &'a mut Preferences) -> Recognizer<'a> {
        Recognizer { preferences: prefs }
    }

    /// Return the language in use in a source file.
    ///
    /// # Arguments
    ///
    /// * `file_name` - Path to the file in the file system.
    pub fn recognize_source_file(&self, file_name: &str) -> Option<RecognizeResult> {
        let language_objects = match LanguageInterface::get_list_of_all_language_objects() {
            Ok(l) => l,
            Err(_) => return None,
        };

        for language_object in language_objects {
            if language_object.file_is_language_source_file(file_name, self.preferences) {
                return Some(RecognizeResult::new(
                    language_object.implements(),
                    FileType::Source,
                ));
            }
        }

        None
    }

    /// Return the language in use in an abstract syntax tree file.
    ///
    /// # Arguments
    ///
    /// * `file_name` - The path to the file in the file system.
    pub fn recognize_ast_file(&self, file_name: &str) -> Option<RecognizeResult> {
        let language_objects = match LanguageInterface::get_list_of_all_language_objects() {
            Ok(l) => l,
            Err(_) => return None,
        };

        for language_object in language_objects {
            if language_object.file_is_language_ast_file(file_name) {
                return Some(RecognizeResult::new(
                    language_object.implements(),
                    FileType::AST,
                ));
            }
        }

        None
    }

    /// Return the language/file type from a file that could be a configuration file.
    ///
    /// # Arguments
    ///
    /// * `file_name` - The path to the file in the file system.
    pub fn recognize_config_file(&self, file_name: &str) -> Option<RecognizeResult> {
        let file_path = PathBuf::from_str(file_name).unwrap();
        let extension = file_path.extension();
        if extension.is_some() && extension.unwrap() == OsStr::new(CONFIG_FILE_EXTENSION) {
            if let Ok(json_node) = load_json_from_file_with_name(file_name) {
                if let Some(language_str) = json_node.get_str_for_key(LANGUAGE_KEY) {
                    if let Ok(language) = Language::from_str(language_str) {
                        return Some(RecognizeResult::new(language, FileType::Config));
                    }
                }
            }
        }
        None
    }

    /// Wrapper function that checks for either a source file or an AST file and returns the language
    /// in the file.
    ///
    /// # Arguments
    ///
    /// * `file_name` - The path to the file in the file system.
    pub fn recognize_file(&self, file_name: &str) -> Result<RecognizeResult, MetamorphError> {
        // Try to recognize the language of the source file.  The file might be a source code file,
        // an AST file, or a config file.
        let mut recognized_details = self.recognize_config_file(file_name);
        if recognized_details.is_none() {
            recognized_details = self.recognize_source_file(file_name);
            if recognized_details.is_none() {
                recognized_details = self.recognize_ast_file(file_name);
                if recognized_details.is_none() {
                    return Err(MetamorphError::LanguageNotRecognized);
                }
            }
        }

        Ok(recognized_details.unwrap())
    }
}
