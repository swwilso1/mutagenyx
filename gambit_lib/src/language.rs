//! The `language` module defines the list of languages supported by the library.

use crate::error::GambitError;
use std::str::FromStr;
use std::string::ToString;

/// The enumerated list of supported languages
#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum Language {
    /// The [Solidity] language for the [Ethereum Virtual Machine]
    ///
    /// [Solidity]: https://soliditylang.org
    Solidity,

    /// The [Vyper] language for the [Ethereum Virtual Machine]
    ///
    /// [Vyper]: https://vyper.readthedocs.io
    /// [Ethereum Virtual Machine]: https://ethereum.org/en/developers/docs/evm
    Vyper,
}

impl FromStr for Language {
    type Err = GambitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Solidity" => Ok(Language::Solidity),
            "Vyper" => Ok(Language::Vyper),
            _ => Err(GambitError::LanguageNotSupported(String::from(s))),
        }
    }
}

impl ToString for Language {
    fn to_string(&self) -> String {
        match self {
            Language::Solidity => String::from("Solidity"),
            Language::Vyper => String::from("Vyper"),
        }
    }
}
