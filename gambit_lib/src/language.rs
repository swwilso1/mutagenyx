use crate::error::GambitError;
use std::str::FromStr;
use std::string::ToString;

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum Language {
    Solidity,
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
