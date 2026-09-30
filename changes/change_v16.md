# Mudança

Fase 4, item 16 (fatia 2 de 3): persistência de Workflows (schema v4) e histórico de execução de workflow com resultado por etapa.

# Arquivos

- `software/crates/core/migrations/004_workflows.sql` (novo)
- `software/crates/core/src/db.rs`
- `software/crates/core/src/history.rs`
- `software/crates/core/src/execution/runner.rs`
- `software/crates/core/src/workflow/mod.rs`
- `software/crates/core/src/workflow/runner.rs`
- `software/crates/core/src/store.rs`
- `software/crates/core/src/lib.rs`
- `software/crates/core/tests/workflow_run.rs` (novo)
- `docs/architecture/domain-model.md`
- `docs/architecture/execution-model.md`
- `docs/product/features.md`
- `docs/product/implementation-order.md`
- `changes/change_v16.md`

# Motivo

A fatia 1 entregou modelo e motor, mas nada sobrevivia ao reinício e não havia registro do que rodou. Sem isso o Workflow não é utilizável de ponta a ponta.

# Impacto

Schema SQLite sobe para **v4**. Tabelas novas:

- `workflows` (id, name, description, created_at);
- `workflow_steps` (workflow_id, position, step_id, tool_id, command_id, parameters_json, workdir, on_error);
- `workflow_executions` (id, workspace_id, workflow_id, status, finished_at);
- `workflow_step_results` (execution_id, position, step_id, status, exit_code, stdout, stderr, error).

Store:

- `save_workflow` / `get_workflow` / `list_workflows` / `delete_workflow`. Salvar exige Workflow válido (ao menos uma etapa) e nome único sem diferenciar caixa; as etapas são **substituídas** a cada salvamento, o que cobre altas, baixas e reordenações de forma previsível. Remover workflow apaga as etapas em cascata.
- `record_workflow_execution` / `list_recent_workflow_executions`, com uma linha por etapa — inclusive as `Skipped`, para o usuário ver o que não chegou a rodar.
- `WorkflowNotFound`, `WorkflowNameTaken`, `EmptyWorkflowName` e `Workflow(WorkflowError)` em `StoreError`.

Decisões:

- **Não existe tabela de vínculo Workspace ↔ Workflow.** Em `domain-model.md` o Workspace referencia Profiles, e é o Profile que configura `Execution | Workflow`. Criar `workspace_workflows` inverteria essa relação. O histórico de workflow é ligado ao Workspace ativo (como o histórico de execução já era), porque o registro é do uso, não da definição.
- `WorkflowError::UnknownCommand` foi removido em favor de `#[from] ManifestError`: o erro de command inexistente já existia em `ManifestError::CommandNotFound` e o resolver pode usar `find_command` direto.
- `ExecutionStatus`, `StepStatus` e `StepFailurePolicy` passaram a expor `as_str`/`parse`. Antes o mesmo `match` de status aparecia em `store.rs` e `history.rs`; agora a forma persistida tem um único lugar.

Pendente para a fatia 3: UI de montagem/execução e o vínculo `Profile → Workflow` (hoje o Profile aponta para um único comando).
