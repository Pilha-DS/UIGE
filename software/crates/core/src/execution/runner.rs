//! Run an external process via Tokio using structured arguments.

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;

use thiserror::Error;
use tokio::process::Command as TokioCommand;

use crate::execution::argv::{build_argv, ArgvError};
use crate::manifest::{Command, Manifest};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionStatus {
    Success,
    Failed,
}

impl ExecutionStatus {
    /// Forma persistida. Manter estável: já existe histórico gravado.
    pub fn as_str(&self) -> &'static str {
        match self {
            ExecutionStatus::Success => "Success",
            ExecutionStatus::Failed => "Failed",
        }
    }

    pub fn parse(value: &str) -> ExecutionStatus {
        match value {
            "Success" => ExecutionStatus::Success,
            _ => ExecutionStatus::Failed,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionOutcome {
    pub status: ExecutionStatus,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Error)]
pub enum ExecutionError {
    #[error(transparent)]
    Argv(#[from] ArgvError),
    #[error("failed to spawn `{program}`: {source}")]
    Spawn {
        program: String,
        source: std::io::Error,
    },
    #[error("failed to wait for process: {0}")]
    Wait(std::io::Error),
}

/// Optional context for a single execution (Phase 1: workdir only).
#[derive(Debug, Clone, Default)]
pub struct ExecutionContext {
    pub workdir: Option<PathBuf>,
}

/// Builds argv and runs the process asynchronously with structured arguments.
pub async fn execute_command(
    manifest: &Manifest,
    command: &Command,
    values: &HashMap<String, String>,
    context: &ExecutionContext,
) -> Result<ExecutionOutcome, ExecutionError> {
    let args = build_argv(manifest, command, values)?;
    let program = manifest.executable.clone();

    let mut child = TokioCommand::new(&program);
    child.args(&args);
    child.stdin(Stdio::null());
    child.stdout(Stdio::piped());
    child.stderr(Stdio::piped());
    if let Some(workdir) = &context.workdir {
        child.current_dir(workdir);
    }

    // Avoid flashing a console window on Windows for short demo commands.
    #[cfg(windows)]
    {
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        child.creation_flags(CREATE_NO_WINDOW);
    }

    let output = child
        .output()
        .await
        .map_err(|source| ExecutionError::Spawn {
            program: program.clone(),
            source,
        })?;

    let exit_code = output.status.code();
    let success = output.status.success();

    Ok(ExecutionOutcome {
        status: if success {
            ExecutionStatus::Success
        } else {
            ExecutionStatus::Failed
        },
        exit_code,
        stdout: String::from_utf8_lossy(&output.stdout).trim_end().to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).trim_end().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{bundled_demo_manifest, bundled_git_manifest, find_command};

    #[tokio::test]
    async fn executes_bundled_echo_print() {
        let manifest = bundled_demo_manifest().expect("demo");
        let command = &manifest.commands[0];
        let mut values = HashMap::new();
        values.insert("message".to_string(), "uige-phase1".to_string());

        let outcome = execute_command(&manifest, command, &values, &ExecutionContext::default())
            .await
            .expect("execute");

        assert_eq!(outcome.status, ExecutionStatus::Success);
        assert_eq!(outcome.exit_code, Some(0));
        assert!(
            outcome.stdout.contains("uige-phase1"),
            "stdout was: {:?}",
            outcome.stdout
        );
    }

    #[tokio::test]
    async fn executes_git_version() {
        let manifest = bundled_git_manifest().expect("git");
        let command = find_command(&manifest, "version").expect("version command");

        let outcome = execute_command(
            &manifest,
            command,
            &HashMap::new(),
            &ExecutionContext::default(),
        )
        .await
        .expect("execute git --version");

        assert_eq!(outcome.status, ExecutionStatus::Success);
        assert!(
            outcome.stdout.to_ascii_lowercase().contains("git"),
            "stdout was: {:?}",
            outcome.stdout
        );
    }
}
