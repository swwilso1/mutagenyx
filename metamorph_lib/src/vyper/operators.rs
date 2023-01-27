use std::collections::HashMap;

pub fn get_python_operator_map() -> HashMap<&'static str, &'static str> {
    let mut operator_map: HashMap<&'static str, &'static str> = HashMap::new();

    operator_map.insert("Add", "+");
    operator_map.insert("Sub", "-");
    operator_map.insert("Mult", "*");
    operator_map.insert("Div", "/");
    operator_map.insert("Mod", "%");
    operator_map.insert("Pow", "**");
    operator_map.insert("And", "&&");
    operator_map.insert("Or", "||");
    operator_map.insert("BitAnd", "&");
    operator_map.insert("BitOr", "|");
    operator_map.insert("BitXor", "^");
    operator_map.insert("Eq", "==");
    operator_map.insert("NotEq", "!=");
    operator_map.insert("Gt", ">");
    operator_map.insert("GtE", ">=");
    operator_map.insert("Lt", "<");
    operator_map.insert("LtE", "<=");
    operator_map.insert("Not", "!");
    operator_map.insert("Invert", "~");

    operator_map
}
