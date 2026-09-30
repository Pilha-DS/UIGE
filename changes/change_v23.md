# Mudança

Aba Ferramentas redesenhada. A tela deixou de ser mestre-detalhe (lista + painel
de execução inline) e passou a ser **só a lista**, com cada linha expondo três
ações explícitas: **Executar**, **Abrir** e **Ações**.

A execução se separou em dois caminhos com propósitos distintos: um diálogo de
execução simples, que roda no Workdir global e mostra o resultado em si mesmo, e
a tela da ferramenta, que carrega o Workdir próprio daquela ferramenta e mantém
o histórico.

# Arquivos

UI:

- `software/crates/ui/ui/components/tool-list.slint` (novo)
- `software/crates/ui/ui/components/run-action-dialog.slint` (novo)
- `software/crates/ui/ui/components/tool-actions-dialog.slint` (novo)
- `software/crates/ui/ui/screens/tool-screen.slint` (novo)
- `software/crates/ui/ui/screens/tools.slint` (reescrito — só lista)
- `software/crates/ui/ui/app.slint`

Core:

- `software/crates/core/src/store.rs`

App:

- `software/crates/app/src/main.rs`

Docs:

- `docs/product/interface.md`
- `changes/change_v23.md`

# Motivo

A tela anterior empilhava lista, detalhe e painel de execução na mesma área. Duas
consequências ruins: a lista ficava estreita e a ação de executar exigia primeiro
*selecionar* uma ferramenta, o que transformava um passo em dois.

O pedido foi separar por intenção: um **executar simples** (escolher a ação,
preencher, rodar) e um **abrir em diretório** (trabalhar na ferramenta com um
Workdir próprio). A distinção que faltava era entre *tiro único* e *contexto de
trabalho*.

# Impacto

## Duas superfícies de execução, estado separado

O modal de execução e a tela da ferramenta não compartilham resultado. Se
compartilhassem, executar no modal e depois abrir a ferramenta mostraria o
resultado do modal como se fosse da ferramenta.

Por isso `app.slint` tem dois blocos de estado — `execution-*` (tela da
ferramenta) e `run-*` (modal) — e `main.rs` tem três `Rc<RefCell<Option<String>>>`
para a ferramenta em foco: `detail_tool_id`, `run_tool_id` e `actions_tool_id`.

## Workdir

- O modal executa no **Workdir global**, sem perguntar. O display do Workdir
  dentro do painel de execução deixa claro onde vai rodar.
- A tela da ferramenta **herda o global ao abrir** e permite alterar para aquela
  ferramenta. Alterar ali não toca no global.

**Mudança de comportamento:** carregar um perfil escrevia o Workdir do perfil no
**global**. Agora o Workdir do perfil é aplicado à ferramenta. O global é editado
apenas na barra de contexto, conforme `docs/product/interface.md` — antes havia um
segundo caminho capaz de sobrescrevê-lo.

## Índice deixa de ser seleção

A lista não tem mais item selecionado: as ações informam o índice da linha. O
índice continua sendo a posição na **lista filtrada**, então
`selected_tool_id(ui, visible)` — que lia `get_selected_tool()` — foi substituído
por `tool_id_at(visible, index)`, que recebe o índice do próprio callback.

Saíram junto as propriedades `selected-tool`, `selected-tool-is-favorite` e
`selected-tool-in-workspace`, e os callbacks `selection-changed` e
`toggle-favorite` / `toggle-workspace-membership` (que agiam sobre a seleção
global). As ações agora são `open-run-dialog`, `open-tool-detail` e
`open-tool-actions`, todas com índice.

## Histórico por ferramenta

A tela da ferramenta mostra só o histórico dela. Filtrar no cliente a lista de
execuções recentes do Workspace perderia execuções da ferramenta que ficaram fora
do limite de 20.

`list_recent_executions` e `list_recent_executions_for_tool` agora delegam para um
`list_executions(workspace_id, tool_id: Option<&str>, limit)` privado, com
`AND (?2 IS NULL OR tool_id = ?2)` — uma consulta em vez de duas quase iguais.

## Execução compartilhada

`run_tool_command(store, runtime, tool_id, command_index, param_text, workdir)`
concentra executar + registrar no histórico, devolvendo um `RunResult` com status,
exit code, output e o erro de gravação (se houver). Antes essa lógica vivia
duplicada no handler da execução rápida.

A falha ao gravar o histórico **não** esconde o resultado da execução: aparece na
barra de status, e o output continua visível.

## Decisões de UI

- **Sem tooltip.** O `std-widgets` do Slint 1.18 não exporta `TooltipArea` (o build
  falha com `No exported type called 'TooltipArea'`). Botões só com glifo ficariam
  ambíguos sem rótulo, então as ações usam texto curto: `Executar`, `Abrir`,
  `Ações`. Os três cabem na linha e não dependem de fonte de emoji.
- **A linha não é clicável.** O clique teria que significar uma das três ações, e
  escolher uma implicitamente seria arbitrário. O `TouchArea` da linha existe
  apenas para o realce de hover e fica declarado **antes** do conteúdo, porque em
  Slint quem vem depois é desenhado na frente e capturaria os cliques dos botões.
- **Menu ⋮ é um modal**, não um popup ancorado. Só lista o que funciona hoje
  (favoritar, adicionar/remover do Workspace, criar perfil): item inerte não
  informa. O diálogo permanece aberto após favoritar; **fecha** após alterar
  participação no Workspace, porque o filtro "Do workspace" pode remover a
  ferramenta da lista e o diálogo passaria a agir sobre algo invisível.
- **Tela da ferramenta é sub-nível**, dentro da área da seção — sidebar e barra de
  contexto continuam visíveis, com um "Voltar" para Ferramentas. Profundidade
  `seção → item → diálogo` do ADR 0003 preservada.

# Validação

- `cargo build` — compila.
- `cargo clippy --workspace --all-targets` — sem avisos.
- `cargo test --workspace` — 47 testes passam (43 no core, 2 + 2 nas integrações).
- Execução do binário — abre e permanece estável, sem saída em stderr.

Erros encontrados e corrigidos:

- `TooltipArea` não existe no `std-widgets` da versão em uso; a lista foi reescrita
  com botões de texto;
- `bind_parameter_field` recebia `&Option<String>`, o que forçava `clone` no
  chamador e quebrou o borrow em `open_tool_detail`. Passou a receber
  `Option<&str>`;
- numa edição de `app.slint` a propriedade `manifests-dir-text` foi removida junto
  com uma duplicata de `history-labels` e precisou ser restaurada.

**Não validado:** o resultado visual. O app foi executado, mas o espaçamento, o
alinhamento dos botões de linha e a proporção da tela da ferramenta não foram
conferidos — falta captura e julgamento visual. Os fluxos de clique também não
foram percorridos de ponta a ponta.

**Pendências assumidas nesta mudança:**

- o Workdir é **digitado**, não escolhido por navegação de pastas. Não há seletor
  nativo de diretório; `docs/standards/ui.md` mantém "Seleção de Workdir" como
  pendente. Adicionar um seletor de pasta implica avaliar uma dependência nova;
- `criar atalho`, `editar definição`, `versões`, `informações` e `remover
  ferramenta` seguem não implementados e fora do menu;
- a tela da ferramenta reusa `ExecutionPanel` inteiro, incluindo a lista de ações
  em caixa de 110px. Se a tela mudar de proporção, o componente provavelmente
  precisa de variante.
