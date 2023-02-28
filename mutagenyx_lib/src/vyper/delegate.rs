//! The `delegate` module exposes `get_vyper_delegate()` to get the Vyper JSON language delegate
//! object.

use crate::json_language_delegate::JSONLanguageDelegate;
use crate::vyper::language_interface::VyperLanguageDelegate;

/// Return the Vyper JSON language delegate.
pub fn get_vyper_delegate() -> Box<dyn JSONLanguageDelegate> {
    Box::new(VyperLanguageDelegate::new())
}
