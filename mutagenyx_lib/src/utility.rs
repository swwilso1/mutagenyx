//! The `utility` module contains various utility functions such as `shell_execute` used by
//! other parts of the library.

use crate::error::MutagenyxError;
use std::process::{Command, Output};

/// Execute a command using the shell facility on the computer.
///
/// # Arguments
///
/// * `command` - The command to execute.
/// * `arguments` - The array of arguments to the command.
pub fn shell_execute(command: &str, arguments: Vec<String>) -> Result<Output, MutagenyxError> {
    let args: Vec<&str> = arguments.iter().map(|v| v.as_str()).collect();
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .arg("/C")
            .arg(command)
            .args(args.iter().map(|arg| arg.to_string()))
            .output()
    } else {
        Command::new(command)
            .args(args.iter().map(|arg| arg.to_string()))
            .output()
    };

    match output {
        Ok(o) => Ok(o),
        Err(e) => Err(MutagenyxError::from(e)),
    }
}
