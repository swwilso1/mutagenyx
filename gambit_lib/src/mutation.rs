use crate::error::GambitError;
use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum GenericMutation {
    ArithmeticBinaryOp,
    LogicalBinaryOp,
    PredicateBinaryOp,
    Assignment,
    DeleteExpression,
    FunctionCall,
    IfStatement,
    Integer,
    FunctionSwapArguments,
    OperatorSwapArguments,
    LinesSwap,
    UnaryOp,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum SolidityMutation {
    Require,
    UncheckedBlock,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum MutationType {
    Generic(GenericMutation),
    Solidity(SolidityMutation),
}

impl FromStr for MutationType {
    type Err = GambitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ArithmeticBinaryOp" => Ok(MutationType::Generic(GenericMutation::ArithmeticBinaryOp)),
            "LogicalBinaryOp" => Ok(MutationType::Generic(GenericMutation::LogicalBinaryOp)),
            "PredicateBinaryOp" => Ok(MutationType::Generic(GenericMutation::PredicateBinaryOp)),
            "Assignment" => Ok(MutationType::Generic(GenericMutation::Assignment)),
            "DeleteExpression" => Ok(MutationType::Generic(GenericMutation::DeleteExpression)),
            "FunctionCall" => Ok(MutationType::Generic(GenericMutation::FunctionCall)),
            "IfStatement" => Ok(MutationType::Generic(GenericMutation::IfStatement)),
            "Integer" => Ok(MutationType::Generic(GenericMutation::Integer)),
            "FunctionSwapArguments" => Ok(MutationType::Generic(
                GenericMutation::FunctionSwapArguments,
            )),
            "OperatorSwapArguments" => Ok(MutationType::Generic(
                GenericMutation::OperatorSwapArguments,
            )),
            "LinesSwap" => Ok(MutationType::Generic(GenericMutation::LinesSwap)),
            "UnaryOp" => Ok(MutationType::Generic(GenericMutation::UnaryOp)),

            "Require" => Ok(MutationType::Solidity(SolidityMutation::Require)),
            "UncheckedBlock" => Ok(MutationType::Solidity(SolidityMutation::UncheckedBlock)),

            _last => Err(GambitError::MutationAlgorithmNotSupported(String::from(
                _last,
            ))),
        }
    }
}

impl fmt::Display for MutationType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let text = match self {
            MutationType::Generic(g) => match g {
                GenericMutation::ArithmeticBinaryOp => "ArithmeticBinaryOp",
                GenericMutation::LogicalBinaryOp => "LogicalBinaryOp",
                GenericMutation::PredicateBinaryOp => "PredicateBinaryOp",
                GenericMutation::Assignment => "Assignment",
                GenericMutation::DeleteExpression => "DeleteExpression",
                GenericMutation::FunctionCall => "FunctionCall",
                GenericMutation::IfStatement => "IfStatement",
                GenericMutation::Integer => "Integer",
                GenericMutation::FunctionSwapArguments => "FunctionSwapArguments",
                GenericMutation::OperatorSwapArguments => "OperatorSwapArguments",
                GenericMutation::LinesSwap => "LinesSwap",
                GenericMutation::UnaryOp => "UnaryOp",
            },
            MutationType::Solidity(s) => match s {
                SolidityMutation::Require => "Require",
                SolidityMutation::UncheckedBlock => "UncheckedBlock",
            },
        };

        write!(f, "{}", text)
    }
}
