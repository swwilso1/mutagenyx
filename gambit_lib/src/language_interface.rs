use crate::error::GambitError;
use crate::language::Language;
use crate::mutation::MutationType;
use crate::pretty_printer::PrettyPrinter;
use crate::solidity::language_interface::SolidityLanguageInterface;
use crate::super_ast::SuperAST;
use rand_pcg::*;
use std::collections::HashMap;

pub trait MutableLanguage {
    fn load_ast_from_file(&mut self, file_name: &str) -> Result<SuperAST, GambitError>;
    fn select_mutators_for_mutation_types(
        &mut self,
        mutation_types: &Vec<MutationType>,
    ) -> Result<(), GambitError>;
    fn count_mutable_nodes(
        &mut self,
        ast: &SuperAST,
    ) -> Result<HashMap<MutationType, usize>, GambitError>;
    fn mutate_ast(
        &mut self,
        ast: &SuperAST,
        mutation_type: &MutationType,
        index: usize,
        rng: &mut Pcg64,
    ) -> Result<SuperAST, GambitError>;
    fn write_mutated_ast_to_file(
        &mut self,
        ast: &SuperAST,
        file_name: &str,
        pretty_printer: &mut PrettyPrinter,
    ) -> Result<(), GambitError>;
    fn get_extension_for_output_file(&self) -> &str;
    fn file_is_language_source_file(&self, file_name: &str) -> bool;
    fn file_is_language_ast_file(&self, file_name: &str) -> bool;
    fn implements(&self) -> Language;
}

pub struct LanguageInterface {}

impl LanguageInterface {
    pub fn get_language_object_for_language(
        language: &Language,
    ) -> Result<Box<dyn MutableLanguage>, GambitError> {
        match language {
            Language::Solidity => Ok(Box::new(SolidityLanguageInterface::new())),
            Language::Vyper => Err(GambitError::LanguageNotSupported(String::from("Vyper"))),
        }
    }

    pub fn get_list_of_all_language_objects() -> Result<Vec<Box<dyn MutableLanguage>>, GambitError>
    {
        let mut language_list: Vec<Box<dyn MutableLanguage>> = vec![];
        language_list.push(Box::new(SolidityLanguageInterface::new()));
        Ok(language_list)
    }
}
