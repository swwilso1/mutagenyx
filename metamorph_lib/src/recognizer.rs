//! The `recognizer` module contains a generic interface to code that will check a file's
//! contents in order to determine if the file contains support programming language source
//! code, or the language's AST.

use crate::language::Language;
use crate::language_interface::*;
use crate::preferences::Preferences;
use crate::MetamorphError;

/// Enumeration of possible file types that a [`Recognizer`] can identify.
#[derive(PartialEq)]
pub enum FileType {
    Source,
    AST,
}

/// Result type to use for results from [`Recognizer`] functions.
#[derive(PartialEq)]
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
    pub preferences: &'a Preferences,
}

impl<'a> Recognizer<'a> {
    /// Create a new recognizer object.
    ///
    /// # Arguments
    ///
    /// * `prefs` - A reference to a preferences object.
    pub fn new(prefs: &'a Preferences) -> Recognizer<'a> {
        Recognizer { preferences: prefs }
    }

    /// Return the language in use in a source file.
    ///
    /// # Arguments
    ///
    /// * `file_name` - Path to the file in the file system.
    pub fn recognize_source_file(&self, file_name: &String) -> Option<RecognizeResult> {
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
    pub fn recognize_ast_file(&self, file_name: &String) -> Option<RecognizeResult> {
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

    /// Wrapper function that checks for either a source file or an AST file and returns the language
    /// in the file.
    ///
    /// # Arguments
    ///
    /// * `file_name` - The path to the file in the file system.
    pub fn recognize_file(&self, file_name: &String) -> Result<RecognizeResult, MetamorphError> {
        // Try to recognize the language of the source file.  The file might be a source code file
        // or perhaps an AST file.
        let mut recognized_language = self.recognize_source_file(file_name);
        if recognized_language == None {
            recognized_language = self.recognize_ast_file(file_name);
            if recognized_language == None {
                return Err(MetamorphError::LanguageNotRecognized);
            }
        }

        Ok(recognized_language.unwrap())
    }
}
