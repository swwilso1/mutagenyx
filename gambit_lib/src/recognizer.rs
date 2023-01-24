//! The `recognizer` module contains a generic interface to code that will check a file's
//! contents in order to determine if the file contains support programming language source
//! code, or the language's AST.

use crate::language::Language;
use crate::language_interface::*;
use crate::GambitError;

/// Provides facilities for identifying the programming language used by a source file or an
/// abstract syntax tree of a program.
pub struct Recognizer {}

impl Recognizer {
    /// Return the language in use in a source file.
    ///
    /// # Arguments
    ///
    /// * `file_name` - Path to the file in the file system.
    pub fn recognize_source_file(file_name: &String) -> Option<Language> {
        let language_objects = match LanguageInterface::get_list_of_all_language_objects() {
            Ok(l) => l,
            Err(_) => return None,
        };

        for language_object in language_objects {
            if language_object.file_is_language_source_file(file_name) {
                return Some(language_object.implements());
            }
        }

        None
    }

    /// Return the language in use in an abstract syntax tree file.
    ///
    /// # Arguments
    ///
    /// * `file_name` - The path to the file in the file system.
    pub fn recognize_ast_file(file_name: &String) -> Option<Language> {
        let language_objects = match LanguageInterface::get_list_of_all_language_objects() {
            Ok(l) => l,
            Err(_) => return None,
        };

        for language_object in language_objects {
            if language_object.file_is_language_ast_file(file_name) {
                return Some(language_object.implements());
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
    pub fn recognize_file(file_name: &String) -> Result<Language, GambitError> {
        // Try to recognize the language of the source file.  The file might be a source code file
        // or perhaps an AST file.
        let mut recognized_language = Recognizer::recognize_source_file(file_name);
        if recognized_language == None {
            recognized_language = Recognizer::recognize_ast_file(file_name);
            if recognized_language == None {
                return Err(GambitError::LanguageNotRecognized);
            }
        }

        Ok(recognized_language.unwrap())
    }
}
