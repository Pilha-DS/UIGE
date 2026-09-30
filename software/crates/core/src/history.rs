//! Execution history records (persisted diagnostics).

use std::collections::HashMap;
use std::path::PathBuf;

use crate::execution::ExecutionStatus;
use crate::workflow::StepStatus;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionRecord {
    pub id: String,
    pub workspace_id: String,
    pub tool_id: String,
    pub command_id: String,
    pub parameters: HashMap<String, String>,
    pub workdir: Option<PathBuf>,
    pub status: ExecutionStatus,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub finished_at: String,
}

impl ExecutionRecord {
    pub fn new_id() -> String {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        format!("exec-{stamp}")
    }

    pub fn label(&self) -> String {
        format!(
            "{} · {} · {} · {}",
            self.status.as_str(),
            self.tool_id,
            self.command_id,
            self.finished_at
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestVersionInfo {
    pub tool_id: String,
    pub version: i64,
    pub content_hash: String,
    pub created_at: String,
}

/// Resultado de uma etapa registrado no histórico de um Workflow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowStepRecord {
    pub position: i64,
    pub step_id: String,
    pub status: StepStatus,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub error: Option<String>,
}

impl WorkflowStepRecord {
    /// Rótulo curto para exibição.
    pub fn label(&self) -> String {
        crate::workflow::step_label(
            self.status,
            &self.step_id,
            self.exit_code,
            self.error.as_deref(),
        )
    }
}

/// Execução de um Workflow registrada para diagnóstico.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowExecutionRecord {
    pub id: String,
    pub workspace_id: String,
    pub workflow_id: String,
    pub status: ExecutionStatus,
    pub steps: Vec<WorkflowStepRecord>,
    pub finished_at: String,
}

impl WorkflowExecutionRecord {
    pub fn new_id() -> String {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        format!("wrun-{stamp}")
    }

    /// Quantidade de etapas que falharam (etapas puladas não contam).
    pub fn failed_steps(&self) -> usize {
        self.steps
            .iter()
            .filter(|step| step.status.is_failure())
            .count()
    }

    pub fn label(&self) -> String {
        let failed = self.failed_steps();
        let detail = if failed == 0 {
            format!("{} etapa(s)", self.steps.len())
        } else {
            format!("{failed} falha(s) em {} etapa(s)", self.steps.len())
        };
        format!(
            "{} · {} · {} · {}",
            self.status.as_str(),
            self.workflow_id,
            detail,
            self.finished_at
        )
    }
}
