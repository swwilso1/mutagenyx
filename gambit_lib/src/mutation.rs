//! The `mutation` module contains the enumeration of the mutation algorithms supported by the
//! library.

use crate::error::GambitError;
use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;

/// Each item in the enumeration represents a generic mutation strategy deployable across all
/// programming languages.
#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum GenericMutation {
    /// Randomly replace the arithmetic operator in a binary expression.
    ///
    /// # Example
    ///
    /// `a + b` might become `a - b`.
    ///
    /// `a * b` might become `a / b`
    ///
    /// # Operators
    ///
    /// This algorithm will replace the operators in the list (+, -, *, /, %, **) with another
    /// operator from the same list.
    ArithmeticBinaryOp,

    /// Randomly replace the logical operator in a binary expression.
    ///
    /// # Examples
    ///
    /// `a || b` might become `a && b`
    ///
    /// `a & b` might become `a | b`
    ///
    /// # Operators
    ///
    /// This algorithm will replace the operators in the list (&&, ||) with
    /// another operator from the same list.
    LogicalBinaryOp,

    /// Randomly replaces a bitwise operator in a binary expression.
    ///
    /// # Examples
    ///
    /// `a & b` might become `a | b`
    ///
    /// `a << b` might become `a >> b`
    ///
    /// # Operators
    ///
    /// This algorithm will replace the operators in the list (&, |, ^) with another operator
    /// from the same list.
    BitwiseBinaryOp,

    /// Randomly replaces a bitshift operator in a binary expression.
    ///
    /// # Examples
    ///
    /// `a << b` might become `a >> b`
    ///
    /// # Operators
    ///
    /// This algorithm will replace the operators in the list (<<, >>) with another operator
    /// from the same list.
    BitshiftBinaryOp,

    /// Randomly replace the comparison operator in a binary expression.
    ///
    /// # Examples
    ///
    /// `a < b` might become `a >= b`
    ///
    /// `a == b` might become `a != b`
    ///
    /// # Operators
    ///
    /// This algorithm will replace the operators in the list (==, !=, <, >, <=, >=) with another
    /// operator from the list
    ComparisonBinaryOp,

    /// In an assignment expression statement, replace the expression on the right hand side of the
    /// assignment with a type appropriate random alternate value.
    ///
    /// # Examples
    ///
    /// ```solidity
    /// uint a = 0;
    /// a = b + 10;
    /// ```
    ///
    /// Might become
    ///
    /// ```solidity
    /// uint a = 10;
    /// a = 293949291329;
    /// ```
    Assignment,

    /// Randomly select a statement in the program and delete it.
    DeleteStatement,

    /// In an expression that contains a function call, randomly select one of the arguments
    /// of the function call, and replace the function call itself with the argument.
    ///
    /// # Examples
    ///
    /// `a = foo(b);` might become `a = b;`
    FunctionCall,

    /// This algorithm either randomly replace the expression inside an if() test with a direct
    /// `true` or `false` value or replaces the expression (here denoted as `c`) with
    /// `if(!(c))`
    ///
    /// # Examples
    ///
    /// `if(a > b) {...}` might become `if(true) {...}`
    ///
    /// `if(c == 10) {...}` might become `if(!(c == 10)) {...}`
    IfStatement,

    /// Replaces integer constants with a random selection from this list of behaviors:
    /// * add one to the constant.
    /// * subtract one from the constant.
    /// * a random value between (0, max(integer type))
    ///
    /// # Examples
    ///
    /// `a = b + 10;` might become `a = b + 11;`
    ///
    /// `a = b + 10;` might become `a = b - 9;`
    ///
    /// `a = b + 10;` might become `a = 2932;`
    Integer,

    /// Randomly swaps two arguments to a function call.  The arguments must have the same type
    /// in order to generate a valid mutation.
    ///
    /// # Examples
    ///
    /// `a = foo(bar, bat);` might become `a = foo(bat, bar);`
    ///
    /// `b = bip(boop, 10, bop);` might become `b = bip(bop, 10, boop);`
    FunctionSwapArguments,

    /// Swaps left and right hand sides of expressions that contain non-commutative operators.
    ///
    /// # Examples
    ///
    /// `thing = a - b;` might become `thing = b - a;`
    ///
    /// `this = that << 5;` might become `this = 5 << that;`
    ///
    /// # Operators
    ///
    /// This algorithm will swap left and right expressions for binary expressions using operators
    /// from this list; (-, /, %, **, >, <, >=, <=, <<, >>)
    OperatorSwapArguments,

    /// Randomly picks two statements in a block and swaps the two statements.
    ///
    /// # Examples
    ///
    /// ```solidity
    /// a = foo - bar();
    /// ...
    /// foo += 9;
    /// ```
    ///
    /// might become
    ///
    /// ```solidity
    /// foo += 9;
    /// ...
    /// a = foo - bar();
    /// ```
    LinesSwap,

    /// If the unary expression contains a prefix operator from (++, --, ~), the algorithm swaps
    /// the operator with another operator from the list.  If the unary expression contains a
    /// postfix operator from (++, --), the algorithm will replace the operator with the other
    /// operator from the list.
    ///
    /// # Examples
    ///
    /// `++a;` might become `~a;`
    ///
    /// `a--` would become `a++;`
    UnaryOp,
}

/// The items in this enumeration represent Solidity language specific mutation algorithms.
#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum SolidityMutation {
    /// Replaces the argument expression to the Solidity function `require` with the negation
    /// of the expression.
    ///
    /// # Examples
    ///
    /// `require(b > 10);` would become `require(! (b > 0);`
    Require,

    /// Picks and expression statement node and decorates the statement with the `unchecked{ }`
    /// decorator.
    ///
    /// # Examples
    ///
    /// `a = b + c;` would become `unchecked{ a = b + c; }`
    UncheckedBlock,
}

/// This enumeration collects all variations of mutation algorithms into a single enumeration.
#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum MutationType {
    /// A generic mutation algorithm usable by all language instances
    Generic(GenericMutation),

    /// Mutation algorithms specific to Solidity
    Solidity(SolidityMutation),
}

/// Provide the ability to convert a string value into a [`MutationType`] enum.
impl FromStr for MutationType {
    type Err = GambitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ArithmeticBinaryOp" => Ok(MutationType::Generic(GenericMutation::ArithmeticBinaryOp)),
            "LogicalBinaryOp" => Ok(MutationType::Generic(GenericMutation::LogicalBinaryOp)),
            "BitwiseBinaryOp" => Ok(MutationType::Generic(GenericMutation::BitwiseBinaryOp)),
            "BitshiftBinaryOp" => Ok(MutationType::Generic(GenericMutation::BitshiftBinaryOp)),
            "ComparisonBinaryOp" => Ok(MutationType::Generic(GenericMutation::ComparisonBinaryOp)),
            "Assignment" => Ok(MutationType::Generic(GenericMutation::Assignment)),
            "DeleteStatement" => Ok(MutationType::Generic(GenericMutation::DeleteStatement)),
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
                GenericMutation::BitwiseBinaryOp => "BitwiseBinaryOp",
                GenericMutation::BitshiftBinaryOp => "BitshiftBinaryOp",
                GenericMutation::ComparisonBinaryOp => "ComparisonBinaryOp",
                GenericMutation::Assignment => "Assignment",
                GenericMutation::DeleteStatement => "DeleteStatement",
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

/// Return a list of all the available mutation algorithms.
pub fn all_mutation_algorithms() -> Vec<MutationType> {
    let algorithms = vec![
        MutationType::Generic(GenericMutation::ArithmeticBinaryOp),
        MutationType::Generic(GenericMutation::LogicalBinaryOp),
        MutationType::Generic(GenericMutation::BitwiseBinaryOp),
        MutationType::Generic(GenericMutation::BitshiftBinaryOp),
        MutationType::Generic(GenericMutation::ComparisonBinaryOp),
        MutationType::Generic(GenericMutation::Assignment),
        MutationType::Generic(GenericMutation::DeleteStatement),
        MutationType::Generic(GenericMutation::FunctionCall),
        MutationType::Generic(GenericMutation::IfStatement),
        MutationType::Generic(GenericMutation::Integer),
        MutationType::Generic(GenericMutation::FunctionSwapArguments),
        MutationType::Generic(GenericMutation::OperatorSwapArguments),
        MutationType::Generic(GenericMutation::LinesSwap),
        MutationType::Generic(GenericMutation::UnaryOp),
        MutationType::Solidity(SolidityMutation::Require),
        MutationType::Solidity(SolidityMutation::UncheckedBlock),
    ];

    algorithms
}
