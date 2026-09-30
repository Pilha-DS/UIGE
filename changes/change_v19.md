# Mudança

Fase 4, item 16, fatia 3: **UI de montagem e execução de Workflows**. Inclui a definição da sintaxe de abertura por comando (`uige <id>`), que ficou pendente no `change_v18.md`.

# Arquivos

Core:

- `software/crates/core/src/workflow/mod.rs` — `step_label` compartilhado
- `software/crates/core/src/workflow/runner.rs` — `WorkflowStepResult::label`
- `software/crates/core/src/history.rs` — `WorkflowStepRecord::label`

UI:

- `software/crates/ui/ui/components/light-controls.slint` (novo)
- `software/crates/ui/ui/components/create-workflow-dialog.slint` (novo)
- `software/crates/ui/ui/components/create-profile-dialog.slint`
- `software/crates/ui/ui/app.slint`

App:

- `software/crates/app/src/main.rs`

Docs:

- `docs/decisions/0002-abertura-da-interface-por-comando.md`
- `docs/architecture/technology.md`, `docs/product/roadmap.md`, `docs/product/implementation-order.md`
- `docs/product/features.md`, `docs/standards/ui.md`, `README.md`
- `patterns/ui/modal-form.md` (novo), `patterns/ui/README.md`
- `software/README.md`
- `changes/change_v19.md`

# Motivo

O item 16 tinha modelo, motor, persistência e histórico prontos no core, mas nenhuma forma de o usuário montar ou executar um Workflow — só era possível criá-lo por código. Sem a UI, a fatia não entregava valor.

# Impacto

## Sintaxe de abertura por comando

Definida como posicional: `uige` abre a interface principal e `uige <id>` abre a interface focada na ferramenta `<id>` (ex.: `uige git`). Argumentos iniciados por `-` ficam reservados para flags; `id` desconhecido abre a interface principal e informa. A forma `--tool <id>` foi considerada e preterida por verbosidade. **A implementação do argumento continua pendente** — nesta mudança só a decisão foi fechada.

## Interface de Workflows

Nova seção **Workflows** na home:

- combo com os Workflows salvos, botão **Criar workflow** e botão **Executar workflow**;
- **área de etapas** com scroll, no mesmo tratamento visual do output, com título que diz o que está sendo mostrado:
  - **Etapas do workflow selecionado** — a definição do Workflow, para o usuário conferir o que vai rodar;
  - **Etapas da execução** — o resultado (com exit code ou erro) de cada etapa;
- **Execuções de workflow**: combo com o histórico recente; selecionar recarrega as etapas daquela execução.

Sem essa área de definição, um Workflow criado não poderia ser revisado depois — as etapas só apareceriam ao executar.

Criar um Workflow deixa ele já selecionado, para o usuário conferir as etapas gravadas.

O diálogo **Criar workflow** monta a lista de etapas de forma incremental: escolher ferramenta, ação, parâmetro e política de erro, então **Adicionar etapa**. As etapas aparecem numeradas com ferramenta, ação, parâmetro e política, e **Remover última etapa** desfaz. Salvar exige nome e ao menos uma etapa; nome repetido é recusado pelo core (mesma regra dos Profiles).

A ordem das etapas é a única informação que o usuário não consegue inferir da tela depois de salvar, por isso a lista fica sempre visível no diálogo.

## Reuso de componentes

Os controles claros do modal (`LightTextField`, `LightSecondaryButton`, `LightPrimaryButton`) estavam declarados dentro de `create-profile-dialog.slint` e não eram importáveis. Foram extraídos para `components/light-controls.slint`, agora usado pelos dois diálogos — sem mudança de aparência ou comportamento. O padrão correspondente foi registrado em `patterns/ui/modal-form.md`, conforme o fluxo de `docs/standards/ui.md`.

## Core

`step_label` foi centralizado no módulo `workflow` e é usado tanto pelo resultado imediato (`WorkflowStepResult`) quanto pelo histórico gravado (`WorkflowStepRecord`), para os dois não divergirem na formatação. Os rótulos seguem o estilo do `ExecutionRecord::label()` já existente.

## Não incluído

O vínculo `Profile → Workflow` continua pendente. Ele altera uma relação canônica do domínio (hoje um Profile representa uma execução única) e merece decisão própria, em vez de entrar junto da UI.

# Validação

- `cargo build` — workspace compila, incluindo os dois `.slint` novos.
- `cargo test` — 36 testes passam (32 no core, 2 + 2 nas integrações).
- `cargo clippy --all-targets` — sem avisos.
- Execução do binário por ~10s — a UI abre e o processo permanece estável (sem panic na migração, no carregamento de Workflows nem no histórico).
- **Não validado:** a interação visual (montar um Workflow, executar, rolar listas) não foi verificada — exige uso manual da janela.
