//! The `string` module provides extra functionality for [`String`] objects.

/// Trait that adds the ability to remove all the occurrences of a substring.
pub trait MetamorphString {
    /// Remove all instances of `s` in `self`
    ///
    /// # Arguments
    ///
    /// * `s` - The reference to the string slice containing the text to remove.
    fn remove_all(&mut self, s: &str);
}

impl MetamorphString for String {
    fn remove_all(&mut self, s: &str) {
        let mut find_result = self.find(s);
        while find_result != None {
            let index = match find_result {
                Some(i) => i,
                None => return,
            };

            self.remove(index);

            find_result = self.find(s);
        }
    }
}
