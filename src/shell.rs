use std::process::Command;
use crate::errors::McpcError;

pub struct ShellCommandBuilder {
    program: String,
    args: Vec<String>,
}

impl ShellCommandBuilder {
    pub fn new(program: &str) -> Self {
        Self {
            program: program.to_string(),
            args: Vec::new(),
        }
    }

    pub fn arg(&mut self, arg: &str) -> &mut Self {
        self.args.push(arg.to_string());
        self
    }

    pub fn execute(&self) -> Result<std::process::Output, McpcError> {
        let mut cmd = Command::new(&self.program);
        
        for arg in &self.args {
            // Aggressive sanitization check:
            // Check for malicious characters that could trigger command injection if passed to a shell
            let forbidden = [';', '|', '&', '$', '`', '\\', '<', '>', '\n', '\r'];
            for &c in &forbidden {
                if arg.contains(c) {
                    return Err(McpcError::Build(format!(
                        "Security Violation: Command injection vector detected in argument '{}'",
                        arg
                    )));
                }
            }
            cmd.arg(arg);
        }

        cmd.output().map_err(|e| McpcError::Build(format!("Failed to execute command: {}", e)))
    }
}

/// Helper function to sanitize a string argument specifically for shell safety.
pub fn sanitize_shell_arg(arg: &str) -> Result<String, McpcError> {
    let forbidden = [';', '|', '&', '$', '`', '\\', '<', '>', '\n', '\r'];
    for &c in &forbidden {
        if arg.contains(c) {
            return Err(McpcError::Build(format!(
                "Security Violation: Malicious shell character '{}' detected in argument",
                c
            )));
        }
    }
    
    // Perform standard cross-shell escaping wrapping
    let mut escaped = String::new();
    #[cfg(unix)]
    {
        escaped.push('\'');
        escaped.push_str(&arg.replace('\'', "'\\''"));
        escaped.push('\'');
    }
    #[cfg(windows)]
    {
        escaped.push('"');
        escaped.push_str(&arg.replace('"', "\\\""));
        escaped.push('"');
    }
    Ok(escaped)
}
