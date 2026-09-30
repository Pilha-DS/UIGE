# Mudança

Implementação do shell de navegação definido pela mudança anterior: barra de contexto, sidebar por seções, área da seção e barra de status. A página única foi substituída e o conteúdo migrado para as telas. Tema claro fixo.

# Arquivos

UI — novos:

- `software/crates/ui/ui/components/sidebar.slint`
- `software/crates/ui/ui/components/select-list.slint`
- `software/crates/ui/ui/components/execution-panel.slint`
- `software/crates/ui/ui/screens/home.slint`
- `software/crates/ui/ui/screens/tools.slint`
- `software/crates/ui/ui/screens/profiles.slint`
- `software/crates/ui/ui/screens/workflows.slint`
- `software/crates/ui/ui/screens/settings.slint`

UI — alterados:

- `software/crates/ui/ui/app.slint` (reescrito: deixou de ser a página única)
- `software/crates/ui/ui/components/create-profile-dialog.slint`
- `software/crates/ui/ui/components/create-workflow-dialog.slint`

App:

- `software/crates/app/src/main.rs`

Docs:

- `docs/standards/ui.md`
- `docs/product/interface.md`
- `changes/change_v21.md`

# Motivo

A `change_v20.md` definiu a estrutura da interface no papel. Faltava implementá-la: a página única acumulava nove blocos num scroll, misturava modos de uso, partia o fluxo principal em dois e deixava o resultado longe da ação.

# Impacto

## Shell

- **Barra de contexto** no topo: nome do Workspace e Workdir global com Salvar. Estava no meio da página.
- **Sidebar** fixa com Home, Ferramentas, Perfis, Workflows e Configurações. A seção ativa é um índice no `AppWindow` (`current-section`), ligado por vínculo de duas vias — a sidebar só desenha.
- **Barra de status** na base para mensagens do aplicativo.
- `preferred-size` passou de 640×720 para 960×680, por causa da sidebar.

## Telas

- **Home** — execuções recentes e atalhos de criação (perfil, workflow, ferramenta). Não contém execução nem importação.
- **Ferramentas** — lista à esquerda, detalhe à direita. O detalhe é *inline* (mestre-detalhe) em vez de tela própria: mantém a escolha da ferramenta e a execução no mesmo lugar e evita um quarto nível de navegação.
- **Painel de execução** (componente) — ação, parâmetro, botão Executar, status/exit code e output. É onde o resultado passou a viver.
- **Perfis** — lista e criação.
- **Workflows** — lista, criação, execução, etapas e execuções anteriores.
- **Configurações** — curadoria de definições (pasta, descobrir, importar).

## Mudanças de semântica no app

Estas vão além de mover widgets:

- As props `status-text`, `exit-code-text` e `output-text` foram separadas. `status-text` ficou só com mensagens do aplicativo; o resultado de execução virou `execution-status`, `execution-exit-code` e `execution-output`. Sem isso, o status da barra seria sobrescrito pelo resultado de cada execução.
- `on_tool_selection_changed` foi adicionado. Antes havia só `selection_changed`, que recarregava a ação; agora a troca de ferramenta na lista precisa recarregar as ações e o campo de parâmetro.
- `refresh_tool_selection` passou a expor `selected-tool-name` e a limpar o nome quando não há ferramenta.

## Tema claro fixo

`Palette.color-scheme = ColorScheme.light` é definido uma vez no `init` do `AppWindow`. Os dois diálogos deixaram de alternar o `Palette` ao abrir e fechar — a restauração para `unknown` agora seria errada e reintroduziria o bug de texto invisível. Regra registrada em `docs/standards/ui.md`.

## Consequências

- A estrutura passa a existir; a página única deixou de ser o que existe.
- O seletor de Workspace ainda não existe, porque há só o Workspace padrão e o app usa o id fixo `"default"` em 10 pontos. A barra de contexto mostra o nome.
- Favoritos, busca de ferramentas, menu de ações por ferramenta, modos de abertura de perfil, edição de definição e versões continuam pendentes.
- Adaptação do layout feita com posicionamento absoluto (`x`/`y` a partir de `root.width`/`root.height`) em vez de grades aninhadas. Foi mais simples de acertar, mas é sensível a mudanças de `preferred-size`.

# Validação

- `cargo build` — compila.
- `cargo test` — 36 testes passam (32 no core, 2 + 2 nas integrações).
- `cargo clippy --all-targets` — sem avisos.
- Execução do binário — abre e permanece estável.

Erros encontrados e corrigidos durante a implementação:

- `VerticalLayout`/`HorizontalLayout` são nativos do Slint, não exports de `std-widgets`;
- `index` do `for` no Slint precisa ser declarado (`for item[idx] in ...`);
- `manifests-dir-text` é vinculado com `<=>` no `SettingsScreen`, logo precisa ser `in-out`, não `in`.

**Não validado:** o resultado visual. A captura de tela da janela para conferir o layout foi interrompida por queda de conexão e não foi refeita. O layout foi verificado apenas por compilação e execução, o que **não** confirma espaçamento, alinhamento ou proporção das áreas. Enquanto isso não for conferido, o ajuste fino do layout permanece em aberto.
