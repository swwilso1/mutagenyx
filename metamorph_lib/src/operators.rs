//! The `solidity::operators` module provides helper functions to get vectors of various kinds of
//! binary and unary expression operators.

/// Return the [`Vec<&str>`] object containing the arithmetic operators.
pub fn arithmetic_operators() -> Vec<&'static str> {
    vec!["+", "-", "*", "/", "%", "**"]
}

/// Return the [`Vec<&str>`] object containing the logical operators.
pub fn logical_operators() -> Vec<&'static str> {
    vec!["&&", "||"]
}

/// Return the [`Vec<&str>`] object containing the bitwise operators.
pub fn bitwise_operators() -> Vec<&'static str> {
    vec!["&", "|", "^"]
}

pub fn bitshift_operators() -> Vec<&'static str> {
    vec!["<<", ">>"]
}

/// Return the [`Vec<&str>`] object containing the comparison operators.
pub fn comparison_operators() -> Vec<&'static str> {
    vec!["==", "!=", ">", "<", ">=", "<="]
}

// Return the [`Vec<&str`] object containing the non-commutative operators.
pub fn non_commutative_operators() -> Vec<&'static str> {
    vec!["-", "/", "%", "**", ">", "<", "<=", ">=", "<<", ">>"]
}

/// Return the [`Vec<&str>`] object containing the prefix operators.
pub fn prefix_operators() -> Vec<&'static str> {
    vec!["++", "--", "~"]
}

/// Return the [`Vec<&str>`] object containing the postfix operators.
pub fn postfix_operators() -> Vec<&'static str> {
    vec!["++", "--"]
}
