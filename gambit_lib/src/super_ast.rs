//! The `super_ast` module contains the SuperAST enumeration which allows a more abstract,
//! `generic` wrapper of a language specific abstract syntax tree.

use crate::solidity::ast::SolidityAST;
use crate::vyper::ast::VyperAST;
use crate::Language;
use std::cmp::PartialEq;

/// The SuperAST enumeration that encapsulates all the known types of language AST objects.
pub enum SuperAST {
    Solidity(SolidityAST),
    Vyper(VyperAST),
}

impl PartialEq for SuperAST {
    fn eq(&self, other: &Self) -> bool {
        match self {
            SuperAST::Solidity(ast) => match other {
                SuperAST::Solidity(other_ast) => ast == other_ast,
                _ => false,
            },
            SuperAST::Vyper(ast) => match other {
                SuperAST::Vyper(other_ast) => ast == other_ast,
                _ => false,
            },
        }
    }
}

/// Return the Language for a given AST.
///
/// # Arguments
///
/// * `ast` - A reference to an AST.
pub fn language_for_ast(ast: &SuperAST) -> Language {
    match ast {
        SuperAST::Solidity(_) => Language::Solidity,
        SuperAST::Vyper(_) => Language::Vyper,
    }
}
