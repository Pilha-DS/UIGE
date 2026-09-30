//! Assemble structured argv from a manifest command and parameter values.

use std::collections::HashMap;

use thiserror::Error;

use crate::manifest::{Command, Manifest, ParameterType};

#[derive(Debug, Error)]
pub enum ArgvError {
    #[error("missing required parameter `{0}`")]
    MissingRequired(String),
    #[error("unknown parameter `{0}`")]
    UnknownParameter(String),
}

/// Builds the argument list (without the executable) for process spawn.
pub fn build_argv(
    _manifest: &Manifest,
    command: &Command,
    values: &HashMap<String, String>,
) -> Result<Vec<String>, ArgvError> {
    for key in values.keys() {
        if !command.parameters.iter().any(|p| p.id == *key) {
            return Err(ArgvError::UnknownParameter(key.clone()));
        }
    }

    let mut argv = command.argv_prefix.clone();

    for param in &command.parameters {
        let raw = values
            .get(&param.id)
            .cloned()
            .or_else(|| param.default.clone());

        match raw {
            None if param.required => {
                return Err(ArgvError::MissingRequired(param.id.clone()));
            }
            None => continue,
            Some(value) => append_parameter(&mut argv, param.flag.as_deref(), &param.param_type, &value),
        }
    }

    Ok(argv)
}

fn append_parameter(
    argv: &mut Vec<String>,
    flag: Option<&str>,
    param_type: &ParameterType,
    value: &str,
) {
    match param_type {
        ParameterType::Bool => {
            let enabled = matches!(value.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on");
            if enabled {
                if let Some(flag) = flag {
                    argv.push(flag.to_string());
                } else {
                    argv.push(value.to_string());
                }
            }
        }
        ParameterType::String | ParameterType::Int | ParameterType::Path => {
            if let Some(flag) = flag {
                argv.push(flag.to_string());
            }
            argv.push(value.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::bundled_demo_manifest;

    #[test]
    fn builds_print_argv_with_default_message() {
        let manifest = bundled_demo_manifest().expect("demo");
        let command = &manifest.commands[0];
        let argv = build_argv(&manifest, command, &HashMap::new()).expect("argv");

        if cfg!(windows) {
            assert_eq!(argv, vec!["/C", "echo", "hello UIGE"]);
        } else {
            assert_eq!(argv, vec!["hello UIGE"]);
        }
    }

    #[test]
    fn builds_print_argv_with_custom_message() {
        let manifest = bundled_demo_manifest().expect("demo");
        let command = &manifest.commands[0];
        let mut values = HashMap::new();
        values.insert("message".to_string(), "phase1".to_string());
        let argv = build_argv(&manifest, command, &values).expect("argv");
        assert!(argv.last().is_some_and(|v| v == "phase1"));
    }
}
