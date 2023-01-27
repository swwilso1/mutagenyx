use crate::error::MetamorphError;
use std::process::{Command, Output};

/// Execute a command using the shell facility on the computer.
///
/// # Arguments
///
/// * `command` - The command to execute.
/// * `arguments` - The array of arguments to the command.
pub fn shell_execute(command: &str, arguments: Vec<&str>) -> Result<Output, MetamorphError> {
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .arg("/C")
            .arg(command)
            .args(arguments.iter().map(|arg| arg.to_string()))
            .output()
    } else {
        Command::new(command)
            .args(arguments.iter().map(|arg| arg.to_string()))
            .output()
    };

    match output {
        Ok(o) => Ok(o),
        Err(e) => Err(MetamorphError::from(e)),
    }
}
