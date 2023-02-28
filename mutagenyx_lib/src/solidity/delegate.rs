//! The `delegate` module exposes `get_solidity_delegate()` to get the Solidity JSON language delegate
//! object.

use crate::json_language_delegate::JSONLanguageDelegate;
use crate::solidity::language_interface::SolidityLanguageSubDelegate;

/// Return the Solidity JSON language delegate.
pub fn get_solidity_delegate() -> Box<dyn JSONLanguageDelegate> {
    Box::new(SolidityLanguageSubDelegate::new())
}
