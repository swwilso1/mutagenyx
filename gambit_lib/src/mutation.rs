use std::str::FromStr;
use std::string::ToString;

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
    type Err = ();

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

            _ => {
                // TODO: Return a meaningful error
                println!("Unrecognized mutation algorithm");
                Err(())
            }
        }
    }
}

impl ToString for MutationType {
    fn to_string(&self) -> String {
        match self {
            MutationType::Generic(g) => match g {
                GenericMutation::ArithmeticBinaryOp => String::from("ArithmeticBinaryOp"),
                GenericMutation::LogicalBinaryOp => String::from("LogicalBinaryOp"),
                GenericMutation::PredicateBinaryOp => String::from("PredicateBinaryOp"),
                GenericMutation::Assignment => String::from("Assignment"),
                GenericMutation::DeleteExpression => String::from("DeleteExpression"),
                GenericMutation::FunctionCall => String::from("FunctionCall"),
                GenericMutation::IfStatement => String::from("IfStatement"),
                GenericMutation::Integer => String::from("Integer"),
                GenericMutation::FunctionSwapArguments => String::from("FunctionSwapArguments"),
                GenericMutation::OperatorSwapArguments => String::from("OperatorSwapArguments"),
                GenericMutation::LinesSwap => String::from("LinesSwap"),
                GenericMutation::UnaryOp => String::from("UnaryOp"),
            },
            MutationType::Solidity(s) => match s {
                SolidityMutation::Require => String::from("Require"),
                SolidityMutation::UncheckedBlock => String::from("UncheckedBlock"),
            },
        }
    }
}
