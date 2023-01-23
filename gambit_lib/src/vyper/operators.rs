use std::collections::HashMap;

pub fn get_python_operator_map() -> HashMap<String, String> {
    let mut operator_map: HashMap<String, String> = HashMap::new();

    operator_map.insert(String::from("Add"), String::from("+"));
    operator_map.insert(String::from("Sub"), String::from("-"));
    operator_map.insert(String::from("Mult"), String::from("*"));
    operator_map.insert(String::from("Div"), String::from("/"));
    operator_map.insert(String::from("Mod"), String::from("%"));
    operator_map.insert(String::from("Pow"), String::from("**"));
    operator_map.insert(String::from("And"), String::from("&&"));
    operator_map.insert(String::from("Or"), String::from("||"));
    operator_map.insert(String::from("BitAnd"), String::from("&"));
    operator_map.insert(String::from("BitOr"), String::from("|"));
    operator_map.insert(String::from("BitXor"), String::from("^"));
    operator_map.insert(String::from("Eq"), String::from("=="));
    operator_map.insert(String::from("NotEq"), String::from("!="));
    operator_map.insert(String::from("Gt"), String::from(">"));
    operator_map.insert(String::from("GtE"), String::from(">="));
    operator_map.insert(String::from("Lt"), String::from("<"));
    operator_map.insert(String::from("LtE"), String::from("<="));
    operator_map.insert(String::from("Not"), String::from("!"));

    operator_map
}
