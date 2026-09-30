# Mudança

Fase 4, item 16 (fatia 1 de 3): modelo e motor de Workflows no core. Ainda sem persistência e sem UI.

# Arquivos

- `software/crates/core/src/workflow/mod.rs` (novo)
- `software/crates/core/src/workflow/runner.rs` (novo)
- `software/crates/core/src/ids.rs` (novo)
- `software/crates/core/src/profile/mod.rs`
- `software/crates/core/src/lib.rs`
- `docs/architecture/execution-model.md`
- `docs/product/features.md`
- `docs/product/implementation-order.md`
- `changes/change_v15.md`

# Motivo

`overview.md` e `execution-model.md` já definem `Workflow` e `WorkflowStep`, mas nada existia em código: só execução única. Esta fatia fecha o modelo e o comportamento de execução antes de tocar em banco e UI.

# Impacto

Core:

- `Workflow` (`id`, `name`, `description`, `steps[]`) e `WorkflowStep` (`id`, `tool_id`, `command_id`, `parameters`, `workdir`, `on_error`). A etapa referencia Tool/Command; não duplica o Manifest.
- `StepFailurePolicy` (`Stop` padrão, `Continue`) define o comportamento em caso de erro; `StepStatus` segue os estados de `execution-model.md`.
- `execute_workflow(workflow, base_context, resolve)` roda as etapas em ordem. O `resolve` traduz etapa → (Manifest, Command), então o motor não conhece `Store` nem banco. Workdir da etapa sobrepõe o do contexto base; sem workdir próprio, herda o base.
- Falha ao resolver (Tool/Command inexistente) e falha de spawn/argv são falha **da etapa**, não do Workflow — a UI consegue apontar qual etapa quebrou. `Err` só para Workflow inválido (sem etapas).
- `WorkflowId`/`Profile::new_id` passam a compartilhar `ids::slug_id`, que gera `{prefix}-{timestamp}-{slug}`. O formato do id de Profile não mudou.

Decisões:

- Execução sequencial, sem paralelismo: a ordem declarada é a semântica.
- Compartilhar output entre etapas continua fora do escopo (roadmap, seção Futuro).
- `Pending`, `Running` e `Cancelled` estão declarados por seguirem o modelo canônico, mas esta fatia produz apenas `Success`, `Failed` e `Skipped` — acompanhamento ao vivo ainda não existe.

Pendente para as próximas fatias: persistência (migração v4) e UI de montagem/execução. O `Profile` ainda aponta para um único comando; passar a referenciar Workflow é decisão das próximas fatias.
