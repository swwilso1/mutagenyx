use crate::language::Language;
use crate::language_interface::*;

pub struct Recognizer {}

impl Recognizer {
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
}
