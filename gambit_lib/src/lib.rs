pub mod ast;
pub mod error;
mod json;
pub mod language;
pub mod language_interface;
pub mod mutation;
pub mod mutator;
pub mod pretty_printer;
pub mod recognizer;
pub mod solidity;
pub mod super_ast;
pub mod visitor;
pub mod vyper;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
