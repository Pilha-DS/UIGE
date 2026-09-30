//! Executa as etapas de um Workflow em ordem, com argumentos estruturados.

use crate::execution::{execute_command, ExecutionContext, ExecutionOutcome, ExecutionStatus};
use crate::manifest::{Command, Manifest};

use super::{StepFailurePolicy, StepStatus, Workflow, WorkflowError, WorkflowStep};

/// Resultado de uma etapa.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowStepResult {
    pub step_id: String,
    pub status: StepStatus,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    /// Motivo quando a etapa não pôde ser executada ou falhou.
    pub error: Option<String>,
}

impl WorkflowStepResult {
    fn from_execution(step: &WorkflowStep, outcome: &ExecutionOutcome) -> Self {
        Self {
            step_id: step.id.clone(),
            status: match outcome.status {
                ExecutionStatus::Success => StepStatus::Success,
                ExecutionStatus::Failed => StepStatus::Failed,
            },
            exit_code: outcome.exit_code,
            stdout: outcome.stdout.clone(),
            stderr: outcome.stderr.clone(),
            error: None,
        }
    }

    fn failed(step: &WorkflowStep, error: String) -> Self {
        Self {
            step_id: step.id.clone(),
            status: StepStatus::Failed,
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            error: Some(error),
        }
    }

    fn skipped(step: &WorkflowStep) -> Self {
        Self {
            step_id: step.id.clone(),
            status: StepStatus::Skipped,
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            error: None,
        }
    }

    /// Rótulo curto para exibição.
    pub fn label(&self) -> String {
        super::step_label(
            self.status,
            &self.step_id,
            self.exit_code,
            self.error.as_deref(),
        )
    }
}

/// Resultado completo de um Workflow, com uma entrada por etapa declarada.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowOutcome {
    pub workflow_id: String,
    pub steps: Vec<WorkflowStepResult>,
}

impl WorkflowOutcome {
    /// `true` quando nenhuma etapa falhou.
    pub fn success(&self) -> bool {
        self.steps
            .iter()
            .all(|step| step.status != StepStatus::Failed)
    }

    pub fn failed_steps(&self) -> usize {
        self.steps
            .iter()
            .filter(|step| step.status == StepStatus::Failed)
            .count()
    }
}

/// Executa as etapas na ordem declarada.
///
/// `resolve` traduz a etapa em definição executável (Tool + Command). Falha ao
/// resolver é tratada como falha da etapa, não do Workflow inteiro: o usuário vê
/// qual etapa quebrou. Retorna `Err` apenas quando o Workflow é inválido.
pub async fn execute_workflow<F>(
    workflow: &Workflow,
    base_context: &ExecutionContext,
    resolve: F,
) -> Result<WorkflowOutcome, WorkflowError>
where
    F: Fn(&WorkflowStep) -> Result<(Manifest, Command), WorkflowError>,
{
    workflow.validate()?;

    let mut steps = Vec::with_capacity(workflow.steps.len());
    let mut stopped = false;

    for step in &workflow.steps {
        if stopped {
            steps.push(WorkflowStepResult::skipped(step));
            continue;
        }

        let result = match resolve(step) {
            Err(error) => WorkflowStepResult::failed(step, error.to_string()),
            Ok((manifest, command)) => {
                let context = ExecutionContext {
                    workdir: step
                        .workdir
                        .clone()
                        .or_else(|| base_context.workdir.clone()),
                };

                match execute_command(&manifest, &command, &step.parameters, &context).await {
                    Ok(outcome) => WorkflowStepResult::from_execution(step, &outcome),
                    Err(error) => WorkflowStepResult::failed(step, error.to_string()),
                }
            }
        };

        if result.status == StepStatus::Failed && step.on_error == StepFailurePolicy::Stop {
            stopped = true;
        }
        steps.push(result);
    }

    Ok(WorkflowOutcome {
        workflow_id: workflow.id.clone(),
        steps,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{bundled_demo_manifest, bundled_git_manifest};
    use std::path::PathBuf;

    fn echo_step(id: &str, message: &str, on_error: StepFailurePolicy) -> WorkflowStep {
        let mut parameters = std::collections::HashMap::new();
        parameters.insert("message".to_string(), message.to_string());

        WorkflowStep {
            id: id.to_string(),
            tool_id: "echo".to_string(),
            command_id: "print".to_string(),
            parameters,
            workdir: None,
            on_error,
        }
    }

    /// Manifest cujo executável não existe: falha de spawn determinística.
    fn missing_binary_manifest() -> Manifest {
        Manifest {
            schema_version: "v1".to_string(),
            id: "ausente".to_string(),
            name: "Ausente".to_string(),
            description: None,
            executable: "uige-binary-that-does-not-exist".to_string(),
            commands: vec![Command {
                id: "run".to_string(),
                name: "Rodar".to_string(),
                description: None,
                argv_prefix: Vec::new(),
                parameters: Vec::new(),
            }],
        }
    }

    fn failing_step(id: &str, on_error: StepFailurePolicy) -> WorkflowStep {
        WorkflowStep {
            id: id.to_string(),
            tool_id: "ausente".to_string(),
            command_id: "run".to_string(),
            parameters: std::collections::HashMap::new(),
            workdir: None,
            on_error,
        }
    }

    fn resolver_for(
        manifests: &[Manifest],
    ) -> impl Fn(&WorkflowStep) -> Result<(Manifest, Command), WorkflowError> + '_ {
        move |step: &WorkflowStep| {
            let manifest = manifests
                .iter()
                .find(|manifest| manifest.id == step.tool_id)
                .ok_or_else(|| WorkflowError::UnknownTool(step.tool_id.clone()))?;
            let command = crate::manifest::find_command(manifest, &step.command_id)?;
            Ok((manifest.clone(), command.clone()))
        }
    }

    fn workflow_with(steps: Vec<WorkflowStep>) -> Workflow {
        Workflow {
            id: "workflow-test".to_string(),
            name: "Teste".to_string(),
            description: None,
            steps,
        }
    }

    #[tokio::test]
    async fn runs_steps_in_declared_order() {
        let manifests = vec![bundled_demo_manifest().expect("echo")];
        let workflow = workflow_with(vec![
            echo_step("passo-1", "primeiro", StepFailurePolicy::Stop),
            echo_step("passo-2", "segundo", StepFailurePolicy::Stop),
        ]);

        let outcome = execute_workflow(
            &workflow,
            &ExecutionContext::default(),
            resolver_for(&manifests),
        )
        .await
        .expect("run workflow");

        assert_eq!(outcome.workflow_id, "workflow-test");
        assert_eq!(outcome.steps.len(), 2);
        assert!(outcome.success());
        assert_eq!(outcome.failed_steps(), 0);
        assert_eq!(outcome.steps[0].step_id, "passo-1");
        assert_eq!(outcome.steps[0].status, StepStatus::Success);
        assert!(outcome.steps[0].stdout.contains("primeiro"));
        assert_eq!(outcome.steps[1].status, StepStatus::Success);
        assert!(outcome.steps[1].stdout.contains("segundo"));
    }

    #[tokio::test]
    async fn stops_and_skips_remaining_steps_after_failure() {
        let manifests = vec![bundled_demo_manifest().expect("echo"), missing_binary_manifest()];
        let workflow = workflow_with(vec![
            failing_step("passo-1", StepFailurePolicy::Stop),
            echo_step("passo-2", "nao deve rodar", StepFailurePolicy::Stop),
        ]);

        let outcome = execute_workflow(
            &workflow,
            &ExecutionContext::default(),
            resolver_for(&manifests),
        )
        .await
        .expect("run workflow");

        assert!(!outcome.success());
        assert_eq!(outcome.failed_steps(), 1);
        assert_eq!(outcome.steps[0].status, StepStatus::Failed);
        assert!(outcome.steps[0].error.is_some());
        assert_eq!(outcome.steps[1].status, StepStatus::Skipped);
        assert!(outcome.steps[1].stdout.is_empty(), "etapa pulada não executa");
    }

    #[tokio::test]
    async fn continue_policy_keeps_running_after_failure() {
        let manifests = vec![bundled_demo_manifest().expect("echo"), missing_binary_manifest()];
        let workflow = workflow_with(vec![
            failing_step("passo-1", StepFailurePolicy::Continue),
            echo_step("passo-2", "segue", StepFailurePolicy::Continue),
        ]);

        let outcome = execute_workflow(
            &workflow,
            &ExecutionContext::default(),
            resolver_for(&manifests),
        )
        .await
        .expect("run workflow");

        assert_eq!(outcome.steps[0].status, StepStatus::Failed);
        assert_eq!(outcome.steps[1].status, StepStatus::Success);
        assert!(outcome.steps[1].stdout.contains("segue"));
        assert!(!outcome.success());
    }

    #[tokio::test]
    async fn unknown_tool_fails_the_step_and_not_the_workflow() {
        let manifests = vec![bundled_demo_manifest().expect("echo")];
        let mut step = echo_step("passo-1", "x", StepFailurePolicy::Stop);
        step.tool_id = "nao-existe".to_string();

        let workflow = workflow_with(vec![step, echo_step("passo-2", "x", StepFailurePolicy::Stop)]);

        let outcome = execute_workflow(
            &workflow,
            &ExecutionContext::default(),
            resolver_for(&manifests),
        )
        .await
        .expect("workflow em si é válido");

        assert_eq!(outcome.steps[0].status, StepStatus::Failed);
        assert!(outcome.steps[0]
            .error
            .as_deref()
            .is_some_and(|error| error.contains("nao-existe")));
        assert_eq!(outcome.steps[1].status, StepStatus::Skipped);
    }

    #[tokio::test]
    async fn unknown_command_fails_the_step() {
        let manifests = vec![bundled_demo_manifest().expect("echo")];
        let mut step = echo_step("passo-1", "x", StepFailurePolicy::Stop);
        step.command_id = "nao-existe".to_string();

        let outcome = execute_workflow(
            &workflow_with(vec![step]),
            &ExecutionContext::default(),
            resolver_for(&manifests),
        )
        .await
        .expect("workflow válido");

        assert_eq!(outcome.steps[0].status, StepStatus::Failed);
        assert!(outcome.steps[0]
            .error
            .as_deref()
            .is_some_and(|error| error.contains("nao-existe")));
    }

    #[tokio::test]
    async fn empty_workflow_is_rejected() {
        let manifests = vec![bundled_demo_manifest().expect("echo")];
        let err = execute_workflow(
            &workflow_with(Vec::new()),
            &ExecutionContext::default(),
            resolver_for(&manifests),
        )
        .await
        .expect_err("workflow sem etapas");

        assert!(matches!(err, WorkflowError::EmptyWorkflow));
    }

    #[tokio::test]
    async fn step_workdir_overrides_the_base_context() {
        let manifests = vec![bundled_git_manifest().expect("git")];
        let missing_dir = std::env::temp_dir().join("uige-workflow-dir-que-nao-existe");
        let _ = std::fs::remove_dir_all(&missing_dir);

        let step = WorkflowStep {
            id: "passo-1".to_string(),
            tool_id: "git".to_string(),
            command_id: "version".to_string(),
            parameters: std::collections::HashMap::new(),
            workdir: Some(PathBuf::from(&missing_dir)),
            on_error: StepFailurePolicy::Stop,
        };

        // O workdir da etapa vira `current_dir`; um diretório ausente falha o spawn.
        let outcome = execute_workflow(
            &workflow_with(vec![step]),
            &ExecutionContext::default(),
            resolver_for(&manifests),
        )
        .await
        .expect("workflow válido");

        assert_eq!(outcome.steps[0].status, StepStatus::Failed);
    }

    #[tokio::test]
    async fn inherits_base_workdir_when_step_has_none() {
        let manifests = vec![bundled_git_manifest().expect("git")];

        // O repositório do próprio UIGE é um workdir válido para `git --version`.
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .expect("repo root")
            .to_path_buf();

        let step = WorkflowStep {
            id: "passo-1".to_string(),
            tool_id: "git".to_string(),
            command_id: "version".to_string(),
            parameters: std::collections::HashMap::new(),
            workdir: None,
            on_error: StepFailurePolicy::Stop,
        };

        let outcome = execute_workflow(
            &workflow_with(vec![step]),
            &ExecutionContext {
                workdir: Some(repo_root),
            },
            resolver_for(&manifests),
        )
        .await
        .expect("workflow válido");

        assert_eq!(outcome.steps[0].status, StepStatus::Success);
    }
}
