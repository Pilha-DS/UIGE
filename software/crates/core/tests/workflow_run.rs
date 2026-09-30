//! Fatia 2 do item 16: Workflow persistido, executado e registrado no histórico.
//!
//! Reproduz a cadeia que a UI vai montar na fatia 3: salvar o Workflow, resolver
//! cada etapa pela biblioteca de ferramentas e registrar o resultado.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use uige_core::{
    execute_workflow, find_command, ExecutionContext, StepFailurePolicy, StepStatus, Store,
    Workflow, WorkflowError, WorkflowStep,
};

fn temp_dir(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("uige-{label}-{stamp}"));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn step(id: &str, tool_id: &str, command_id: &str, on_error: StepFailurePolicy) -> WorkflowStep {
    WorkflowStep {
        id: id.to_string(),
        tool_id: tool_id.to_string(),
        command_id: command_id.to_string(),
        parameters: HashMap::new(),
        workdir: None,
        on_error,
    }
}

/// Resolve etapas contra a biblioteca do Store, como o app fará.
fn resolver(
    store: &Store,
) -> impl Fn(&WorkflowStep) -> Result<(uige_core::Manifest, uige_core::Command), WorkflowError> + '_ {
    move |step: &WorkflowStep| {
        let tool = store
            .library()
            .get(&step.tool_id)
            .ok_or_else(|| WorkflowError::UnknownTool(step.tool_id.clone()))?;
        let command = find_command(&tool.manifest, &step.command_id)?;
        Ok((tool.manifest.clone(), command.clone()))
    }
}

#[tokio::test]
async fn persists_runs_and_records_a_workflow() {
    let dir = temp_dir("workflow");
    let db_path = dir.join("uige.sqlite");
    let store = Store::open(&db_path, PathBuf::from(".")).expect("open store");

    // Workflow de duas etapas bem-sucedidas usando tools embutidas.
    let workflow = Workflow {
        id: Workflow::new_id("Versão e status"),
        name: "Versão e status".to_string(),
        description: Some("Demonstra execução multi-etapa".to_string()),
        steps: vec![
            step("passo-1", "git", "version", StepFailurePolicy::Stop),
            step("passo-2", "git", "status", StepFailurePolicy::Continue),
        ],
    };

    store.save_workflow(&workflow).expect("save workflow");
    let reloaded = store.get_workflow(&workflow.id).expect("get workflow");
    assert_eq!(reloaded.steps.len(), 2);

    let outcome = execute_workflow(
        &reloaded,
        &ExecutionContext { workdir: None },
        resolver(&store),
    )
    .await
    .expect("run workflow");

    assert_eq!(outcome.steps.len(), 2);
    assert_eq!(outcome.steps[0].status, StepStatus::Success);
    assert!(outcome.steps[0].stdout.to_lowercase().contains("git version"));

    let record = store
        .record_workflow_execution("default", &outcome)
        .expect("record execution");

    // O histórico guarda uma linha por etapa, inclusive as que não falharam.
    let history = store
        .list_recent_workflow_executions("default", 10)
        .expect("history");
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].id, record.id);
    assert_eq!(history[0].workflow_id, workflow.id);
    assert_eq!(history[0].steps.len(), 2);
    assert_eq!(history[0].steps[0].step_id, "passo-1");
    assert_eq!(history[0].steps[1].step_id, "passo-2");

    drop(store);
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn records_skipped_steps_when_a_step_fails() {
    let dir = temp_dir("workflow-falha");
    let db_path = dir.join("uige.sqlite");
    let store = Store::open(&db_path, PathBuf::from(".")).expect("open store");

    // Tool inexistente na primeira etapa: as seguintes ficam `Skipped`.
    let workflow = Workflow {
        id: Workflow::new_id("Falha no meio"),
        name: "Falha no meio".to_string(),
        description: None,
        steps: vec![
            step("passo-1", "tool-inexistente", "run", StepFailurePolicy::Stop),
            step("passo-2", "git", "version", StepFailurePolicy::Stop),
        ],
    };
    store.save_workflow(&workflow).expect("save workflow");

    let outcome = execute_workflow(
        &workflow,
        &ExecutionContext { workdir: None },
        resolver(&store),
    )
    .await
    .expect("workflow é válido; a etapa é que falha");

    assert!(!outcome.success());
    assert_eq!(outcome.steps[0].status, StepStatus::Failed);
    assert_eq!(outcome.steps[1].status, StepStatus::Skipped);

    let record = store
        .record_workflow_execution("default", &outcome)
        .expect("record");
    assert_eq!(record.failed_steps(), 1);
    assert_eq!(record.steps[1].status, StepStatus::Skipped);

    drop(store);
    let _ = std::fs::remove_dir_all(&dir);
}
