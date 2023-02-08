//! The `language` module defines the list of languages supported by the library.

use crate::error::MetamorphError;
use std::fmt;
use std::str::FromStr;

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
    type Err = MetamorphError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Solidity" => Ok(Language::Solidity),
            "Vyper" => Ok(Language::Vyper),
            _ => Err(MetamorphError::LanguageNotSupported(String::from(s))),
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Language::Solidity => "Solidity",
            Language::Vyper => "Vyper",
        };

        write!(f, "{}", text)
    }
}
