//! The `super_ast` module contains the SuperAST enumeration which allows a more abstract,
//! `generic` wrapper of a language specific abstract syntax tree.

use crate::solidity::ast::SolidityAST;
use crate::vyper::ast::VyperAST;
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
