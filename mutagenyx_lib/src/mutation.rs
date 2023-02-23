//! The `mutation` module contains the enumeration of the mutation algorithms supported by the
//! library.

use crate::error::MutagenyxError;
use crate::operators::{
    arithmetic_operators, bitshift_operators, bitwise_operators, comparison_operators,
    logical_operators, non_commutative_operators, prefix_operators,
};
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;

/// Each item in the enumeration represents a generic mutation strategy deployable across all
/// programming languages.
#[derive(Hash, Eq, PartialEq, PartialOrd, Ord, Clone, Copy, Debug)]
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
#[derive(Hash, Eq, PartialEq, PartialOrd, Ord, Clone, Copy, Debug)]
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

    /// For function calls of the form object.delegatecall(...), replace delegatecall() with call().
    ///
    /// # Examples
    ///
    /// `let a := foo.delegatecall()` would become `let a := foo.call()`
    ElimDelegateCall,
}

/// This enumeration collects all variations of mutation algorithms into a single enumeration.
#[derive(Hash, Eq, PartialEq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum MutationType {
    /// A generic mutation algorithm usable by all language instances
    Generic(GenericMutation),

    /// Mutation algorithms specific to Solidity
    Solidity(SolidityMutation),
}

/// Provide the ability to convert a string value into a [`MutationType`] enum.
impl FromStr for MutationType {
    type Err = MutagenyxError;

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
            "ElimDelegateCall" => Ok(MutationType::Solidity(SolidityMutation::ElimDelegateCall)),

            _last => Err(MutagenyxError::MutationAlgorithmNotSupported(String::from(
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
                SolidityMutation::ElimDelegateCall => "ElimDelegateCall",
            },
        };

        write!(f, "{}", text)
    }
}

/// Return a list of all the available mutation algorithms.
pub fn get_all_mutation_algorithms() -> Vec<MutationType> {
    let map = all_algorithm_descriptions();

    let mut algorithms: Vec<MutationType> = vec![];

    for (key, _) in map {
        algorithms.push(key);
    }

    algorithms.sort_unstable_by(|a, b| {
        let a_str = a.to_string();
        let b_str = b.to_string();
        a_str.cmp(&b_str)
    });

    algorithms
}

/// Type that holds the description of a mutation algorithm.
pub struct MutationAlgorithmDescription {
    /// The basic summary of the algorithm.
    pub summary: &'static str,
    /// Extra useful details about the algorithm.
    pub extra_details: &'static str,
    /// A list of operators the algorithm may affect.
    pub operators: Vec<&'static str>,
    /// Examples of algorithm behavior.
    pub examples: &'static str,
}

/// Return a map of mutation types to the mutation algorithm descriptions.
pub fn all_algorithm_descriptions() -> HashMap<MutationType, MutationAlgorithmDescription> {
    let mut algorithm_map: HashMap<MutationType, MutationAlgorithmDescription> = HashMap::new();

    algorithm_map.insert(
        MutationType::Generic(GenericMutation::ArithmeticBinaryOp),
        MutationAlgorithmDescription {
            summary: "Randomly replace the arithmetic operator in a binary expression.",
            extra_details: "This algorithm will replace the operators in the operator list \
                with another operator from the same list.",
            operators: arithmetic_operators(),
            examples: "a + b might become a - b, a * by might become a / b",
        },
    );

    algorithm_map.insert(
        MutationType::Generic(GenericMutation::LogicalBinaryOp),
        MutationAlgorithmDescription {
            summary: "Randomly replace the logical operator in a binary expression.",
            extra_details: "This algorithm will replace the operators in the operator list with \
            another operator from the same list.",
            operators: logical_operators(),
            examples: "a || b might become a && b",
        },
    );

    algorithm_map.insert(
        MutationType::Generic(GenericMutation::BitwiseBinaryOp),
        MutationAlgorithmDescription {
            summary: "Randomly replaces a bitwise operator in a binary expression.",
            extra_details: "Find bitwise binary operation expressions in the program and \
            replace the operator in the expression with another operator from the list of bitwise \
            operators.",
            operators: bitwise_operators(),
            examples: "a & b might become a | b",
        },
    );

    algorithm_map.insert(
        MutationType::Generic(GenericMutation::BitshiftBinaryOp),
        MutationAlgorithmDescription {
            summary: "Randomly replaces a bitshift operator in a binary expression.",
            extra_details: "Find bitshift binary operator expressions in the program and \
            replace the operator in the expression with another operator from the list of \
            bitshift operators.",
            operators: bitshift_operators(),
            examples: "a << b might become a >> b",
        },
    );

    algorithm_map.insert(
        MutationType::Generic(GenericMutation::ComparisonBinaryOp),
        MutationAlgorithmDescription {
            summary: "Randomly replace the comparison operator in a binary expression",
            extra_details: "Find logical binary operator expressions in the program and replace \
            the operator in the expression with another operator from the list of logical \
            operators.",
            operators: comparison_operators(),
            examples: "a < b might become a > b, a == b might become a <= b",
        },
    );

    algorithm_map.insert(
        MutationType::Generic(GenericMutation::Assignment),
        MutationAlgorithmDescription {
            summary: "Replace right hand side of assignment expressions with type \
            appropriate random alternative values.",
            extra_details: "Find assignment expressions in the program and evaluate the left \
            hand side for type. After finding the type, attempt to replace the right hand size of \
            the expression with a randomly generated type appropriate value. This algorithm \
            currently only operates on expressions that have integer, floating-point, or boolean \
            types.",
            operators: vec![],
            examples: "a = b + 10; where a is of type uint, might become a = 29494243244;",
        },
    );

    algorithm_map.insert(
        MutationType::Generic(GenericMutation::DeleteStatement),
        MutationAlgorithmDescription {
            summary: "Randomly select a statement in a program block and delete the \
            statement.",
            extra_details: "For languages that have variable declarations and return statements \
            the algorithm will not delete declarations or return statements in order to minimize \
            compilation issues caused by the mutation.",
            operators: vec![],
            examples: "",
        },
    );

    algorithm_map.insert(
        MutationType::Generic(GenericMutation::FunctionCall),
        MutationAlgorithmDescription {
            summary: "Replace function calls with one of the randomly selected arguments \
            to the function call.",
            extra_details: "For function calls that have one or more arguments, randomly \
            select an argument from the argument list and replace the entire function call in the \
            expression with the selected argument. This mutation algorithm will attempt to select \
            arguments of the correct type (the return type of the function call) to minimize \
            compilation issues caused by the mutation.",
            operators: vec![],
            examples: "a = foo(b, c); might become a = c;",
        },
    );

    algorithm_map.insert(
        MutationType::Generic(GenericMutation::IfStatement),
        MutationAlgorithmDescription {
            summary: "Replace the condition expression in an if(c) statement with true, \
            false, or the logical negation of the condition if(!(c)).",
            extra_details: "",
            operators: vec![],
            examples: "if(a > b) might become if(true), if(c == 10) might become if(! (c == 1))",
        },
    );

    algorithm_map.insert(
        MutationType::Generic(GenericMutation::Integer),
        MutationAlgorithmDescription {
            summary: "Randomly replace integer constants with random values.",
            extra_details: "The mutation algorithm chooses between three possible behaviors \
            when mutating the constant: add one to the existing value, subtract one from the \
            existing value, or select a random integer value between 0:max(type of the constant).",
            operators: vec![],
            examples: "a = 10; might become a = 11;, a = 10 might become a = 9;, a = 10 might \
            become a = 2932;",
        },
    );

    algorithm_map.insert(
        MutationType::Generic(GenericMutation::FunctionSwapArguments),
        MutationAlgorithmDescription {
            summary: "Randomly swap two arguments in a function call.",
            extra_details: "Find function calls in the program with two or more arguments, \
            randomly select two arguments, and swap them.  When possible, the mutation algorithm \
            will select arguments with the same type to avoid compilation issues.",
            operators: vec![],
            examples: "a = foo(bar, bat, bug); might become a = foo(but, bat, bar);",
        },
    );

    algorithm_map.insert(
        MutationType::Generic(GenericMutation::OperatorSwapArguments),
        MutationAlgorithmDescription {
            summary: "Swap left and right hand sides of binary expressions with \
            non-commutative operators.",
            extra_details: "Select a random binary expression that has a non-commutative \
            operator and swap the left and right hand sides of the expression.",
            operators: non_commutative_operators(),
            examples: "thing = a - b; might become thing = b - a;, this = that << 5; might become \
            this = 5 << that;",
        },
    );

    algorithm_map.insert(
        MutationType::Generic(GenericMutation::LinesSwap),
        MutationAlgorithmDescription {
            summary: "Randomly select two statements in a block and swap the two \
            statements.",
            extra_details: "The mutation algorithm will attempt to identify expression \
            statements and to avoid return statements when selecting statements to swap.",
            operators: vec![],
            examples: "a = foo - bar(); ... foo += 8; might become foo += 8; ... a = foo - bar();",
        },
    );

    algorithm_map.insert(
        MutationType::Generic(GenericMutation::UnaryOp),
        MutationAlgorithmDescription {
            summary: "Random replace unary operators for both prefix and postfix \
            expressions with operators from the unary operator list.",
            extra_details: "This mutation algorithm will not convert a prefix unary expression \
            into a postfix unary expression.",
            operators: prefix_operators(),
            examples: "++a; might become ~a;, a-- might become a++.",
        },
    );

    algorithm_map.insert(
        MutationType::Solidity(SolidityMutation::Require),
        MutationAlgorithmDescription {
            summary: "Randomly select a use of the Solidity require() function and \
            replace the argument with the logical negation of the argument expression.",
            extra_details: "This mutation algorithm only works for Solidity programs.",
            operators: vec![],
            examples: "require(b > 10); would become require(!(b > 0);",
        },
    );

    algorithm_map.insert(
        MutationType::Solidity(SolidityMutation::UncheckedBlock),
        MutationAlgorithmDescription {
            summary: "Randomly select an expression statement in a block of statements \
            and decorate the statement with the Solidity unchecked{} block.",
            extra_details: "This mutation algorithm only works for Solidity programs.",
            operators: vec![],
            examples: "a = b + c; would become unchecked{ a = b + c; }",
        },
    );

    algorithm_map.insert(
        MutationType::Solidity(SolidityMutation::ElimDelegateCall),
        MutationAlgorithmDescription {
            summary: "Replace delegatecall() functions with call().",
            extra_details: "This mutation algorithm only works for Solidity programs.",
            operators: vec![],
            examples: "let a := foo.delegatecall() would become let a := foo.call()",
        },
    );

    algorithm_map
}
