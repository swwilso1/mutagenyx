fn str_array_to_string_vector(string_list: Box<[&str]>) -> Vec<String> {
    let mut v: Vec<String> = vec![];
    for s in string_list.iter() {
        v.push(String::from(*s));
    }
    v
}

pub fn arithmetic_operators() -> Vec<String> {
    str_array_to_string_vector(Box::new(["+", "-", "*", "/", "%", "**"]))
}

pub fn logical_operators() -> Vec<String> {
    str_array_to_string_vector(Box::new(["&&", "||", "&", "|", "^", "<<", ">>"]))
}

pub fn predicate_operators() -> Vec<String> {
    str_array_to_string_vector(Box::new(["==", "!=", ">", "<", ">=", "<="]))
}

// pub fn non_commutative_operators() -> Vec<String> {
//     str_array_to_string_vector(Box::new(["-", "/", "%", "**", ">", "<", "<=", ">=",
//         "<<", ">>"]))
// }

pub fn prefix_operators() -> Vec<String> {
    str_array_to_string_vector(Box::new(["++", "--", "~"]))
}

pub fn postfix_operators() -> Vec<String> {
    str_array_to_string_vector(Box::new(["++", "--"]))
}
