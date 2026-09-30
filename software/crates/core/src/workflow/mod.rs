//! Workflow: sequência ordenada de etapas de execução.
//!
//! Um Workflow referencia Tools e Commands; ele não duplica a definição do
//! Manifest nem guarda preferências pessoais.

mod runner;

pub use runner::{execute_workflow, WorkflowOutcome, WorkflowStepResult};

use std::collections::HashMap;
use std::path::PathBuf;

use thiserror::Error;

use crate::execution::ExecutionError;
use crate::ids::slug_id;
use crate::manifest::ManifestError;

#[derive(Debug, Error)]
pub enum WorkflowError {
    #[error("workflow must declare at least one step")]
    EmptyWorkflow,
    #[error("tool `{0}` not found")]
    UnknownTool(String),
    /// Command inexistente na Tool da etapa, e demais problemas de manifest.
    #[error(transparent)]
    Manifest(#[from] ManifestError),
    #[error(transparent)]
    Execution(#[from] ExecutionError),
}

/// Comportamento do Workflow quando uma etapa falha.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StepFailurePolicy {
    /// Interrompe o Workflow: as etapas seguintes ficam `Skipped`.
    #[default]
    Stop,
    /// Segue para a etapa seguinte mesmo com a falha.
    Continue,
}

impl StepFailurePolicy {
    /// Forma persistida. Manter estável: já existe histórico gravado.
    pub fn as_str(&self) -> &'static str {
        match self {
            StepFailurePolicy::Stop => "Stop",
            StepFailurePolicy::Continue => "Continue",
        }
    }

    pub fn parse(value: &str) -> StepFailurePolicy {
        match value {
            "Continue" => StepFailurePolicy::Continue,
            _ => StepFailurePolicy::Stop,
        }
    }
}

/// Estado de uma etapa do Workflow.
///
/// Os estados seguem `docs/architecture/execution-model.md`. Esta fatia produz
/// `Success`, `Failed` e `Skipped`; `Pending`, `Running` e `Cancelled` existem
/// para acompanhamento ao vivo, ainda não implementado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepStatus {
    Pending,
    Running,
    Success,
    Failed,
    Skipped,
    Cancelled,
}

impl StepStatus {
    /// Forma persistida. Manter estável: já existe histórico gravado.
    pub fn as_str(&self) -> &'static str {
        match self {
            StepStatus::Pending => "Pending",
            StepStatus::Running => "Running",
            StepStatus::Success => "Success",
            StepStatus::Failed => "Failed",
            StepStatus::Skipped => "Skipped",
            StepStatus::Cancelled => "Cancelled",
        }
    }

    pub fn parse(value: &str) -> StepStatus {
        match value {
            "Pending" => StepStatus::Pending,
            "Running" => StepStatus::Running,
            "Success" => StepStatus::Success,
            "Failed" => StepStatus::Failed,
            "Skipped" => StepStatus::Skipped,
            "Cancelled" => StepStatus::Cancelled,
            _ => StepStatus::Failed,
        }
    }

    /// `true` quando a etapa não pode ser considerada bem-sucedida.
    pub fn is_failure(&self) -> bool {
        matches!(self, StepStatus::Failed)
    }
}

/// Rótulo curto de uma etapa, para exibição.
///
/// Vive no core porque vale para os dois lugares que mostram etapas ao usuário:
/// o resultado imediato da execução e o histórico gravado. Evita duas
/// formatações divergentes para o mesmo dado.
pub fn step_label(
    status: StepStatus,
    step_id: &str,
    exit_code: Option<i32>,
    error: Option<&str>,
) -> String {
    let detail = match (error, exit_code) {
        (Some(error), _) => format!("erro: {error}"),
        (None, Some(code)) => format!("exit {code}"),
        (None, None) => "não executada".to_string(),
    };
    format!("{} · {} · {}", status.as_str(), step_id, detail)
}

/// Uma etapa do Workflow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowStep {
    pub id: String,
    pub tool_id: String,
    pub command_id: String,
    pub parameters: HashMap<String, String>,
    /// Quando `None`, herda o workdir do contexto base da execução.
    pub workdir: Option<PathBuf>,
    pub on_error: StepFailurePolicy,
}

/// Sequência ordenada de etapas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub steps: Vec<WorkflowStep>,
}

impl Workflow {
    pub fn new_id(name: &str) -> String {
        slug_id("workflow", name)
    }

    pub fn validate(&self) -> Result<(), WorkflowError> {
        if self.steps.is_empty() {
            return Err(WorkflowError::EmptyWorkflow);
        }
        Ok(())
    }
}
