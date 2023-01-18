/// Convert an array of string slices into a vector of strings
///
/// # Argument
///
/// * `string_list` - The array of string slices to convert to [`String`] objects.
fn str_array_to_string_vector(string_list: Box<[&str]>) -> Vec<String> {
    let mut v: Vec<String> = vec![];
    for s in string_list.iter() {
        v.push(String::from(*s));
    }
    v
}

/// Return the [`Vec<String>`] object containing the arithmetic operators.
pub fn arithmetic_operators() -> Vec<String> {
    str_array_to_string_vector(Box::new(["+", "-", "*", "/", "%", "**"]))
}

/// Return the [`Vec<String>`] object containing the logical operators.
pub fn logical_operators() -> Vec<String> {
    str_array_to_string_vector(Box::new(["&&", "||"]))
}

/// Return the [`Vec<String>`] object containing the bitwise operators.
pub fn bitwise_operators() -> Vec<String> {
    str_array_to_string_vector(Box::new(["&", "|", "<<", ">>"]))
}

/// Return the [`Vec<String>`] object containing the comparison operators.
pub fn comparison_operators() -> Vec<String> {
    str_array_to_string_vector(Box::new(["==", "!=", ">", "<", ">=", "<="]))
}

// Uncomment the lines below here when implementing swap arguments operator mutation algorithm.
///// Return the [`Vec<String`] object containing the non-commutative operators.
// pub fn non_commutative_operators() -> Vec<String> {
//     str_array_to_string_vector(Box::new(["-", "/", "%", "**", ">", "<", "<=", ">=",
//         "<<", ">>"]))
// }

/// Return the [`Vec<String>`] object containing the prefix operators.
pub fn prefix_operators() -> Vec<String> {
    str_array_to_string_vector(Box::new(["++", "--", "~"]))
}

/// Return the [`Vec<String>`] object containing the postfix operators.
pub fn postfix_operators() -> Vec<String> {
    str_array_to_string_vector(Box::new(["++", "--"]))
}
