# Mudança

Ajustes de fluxo e de contenção dos modais, em quatro frentes:

## 1. Etapa de Workflow é detalhada em modal

Antes a etapa era montada por combos fixos na própria criação: ferramenta, ação,
política e um campo de parâmetro. O que não coubesse ali (Workdir da etapa) não
tinha lugar — toda etapa herdava o Workdir global.

Agora **Adicionar etapa** abre `WorkflowStepDialog` (`components/workflow-step-dialog.slint`),
onde a etapa é detalhada: ferramenta, ação, parâmetro, **Workdir da etapa** e
política de falha. A ação **adiciona ao confirmar**; fechar sem confirmar descarta
só a etapa em construção.

- o modal abre com ferramenta e ação preservadas da etapa anterior — repetir a
  ferramenta é o caso comum — e só o parâmetro volta ao padrão;
- Workdir vazio herda o global, como no core (`WorkflowStep::workdir = None`);
- o rótulo da etapa passa a mostrar o Workdir próprio quando existe.

## 2. Criar Workflow é tela, não modal

`CreateWorkflowDialog` foi substituído por `WorkflowCreateScreen`
(`screens/workflow-create.slint`), sub-nível da seção Workflows — o mesmo arranjo
de Ferramentas + tela da ferramenta. Motivo: a criação agora abre um modal (o da
etapa), e modal sobre modal empilha dois fundos escurecidos e dois botões de
fechar. Com a criação em tela, a pilha é sempre tela → modal.

A tela concentra nome, lista de etapas (com a ordem visível), remover última etapa,
cancelar e salvar.

## 3. Abrir ferramenta é modal

`ToolScreen` (`screens/tool-screen.slint`) foi substituído por `ToolDialog`
(`components/tool-dialog.slint`): ferramenta + Workdir próprio + painel de execução
+ histórico da ferramenta, agora em modal.

Motivo: a tarefa é pontual e começa e termina no mesmo lugar. Em sub-tela, abrir da
Home exigia trocar de seção para o clique ter efeito; pelo mesmo motivo a criação
de Workflow pela Home também passou a levar para a seção Workflows.

Escolher outro diretório continua sendo o seletor de pastas, que também é modal.
Para não empilhar, o app mantém **um modal por vez**: enquanto o seletor está
aberto, o diálogo da ferramenta sai de cena e volta ao fechar, com o caminho
aplicado.

## 4. Modais contidos na janela

`Modal` limitava o painel pelo conteúdo e o desenhava a partir do topo. Um diálogo
mais alto que a janela era cortado na borda de baixo — sem rolagem, o botão de
confirmar ficava inalcançável.

Agora o painel é limitado à janela e o **corpo** rola, com cabeçalho e ações fixos:

```text
altura = min(preferred-height do conteúdo, altura da janela - 2 * 24px)
largura = min(panel-width, largura da janela - 2 * 24px)
```

O `preferred-height` do corpo vem do conteúdo, e não do `ScrollView`: ele declara
`preferred-height: 100%` do pai, o que amarraria a altura do painel a ela mesma. O
corpo ganhou um `VerticalLayout` interno porque `@children` cai dentro de um
`Flickable`, que não é layout — com filhos soltos, o seletor de pastas (que passa
vários irmãos) empilharia todos na mesma posição.

# Arquivos

- `software/crates/ui/ui/components/modal.slint` — teto de altura, margem, clip e corpo rolável
- `software/crates/ui/ui/components/workflow-step-dialog.slint` (novo) — detalhamento da etapa
- `software/crates/ui/ui/components/tool-dialog.slint` (novo) — ferramenta + Workdir próprio, em modal
- `software/crates/ui/ui/components/create-workflow-dialog.slint` (removido — virou tela)
- `software/crates/ui/ui/screens/tool-screen.slint` (removido — virou modal)
- `software/crates/ui/ui/screens/workflow-create.slint` (novo) — tela de criação
- `software/crates/ui/ui/screens/workflows.slint` — referência à criação em tela
- `software/crates/ui/ui/app.slint` — tela de criação na seção, etapa e ferramenta no overlay, um modal por vez
- `software/crates/app/src/main.rs` — `open`/`close-workflow-step`, Workdir da etapa, reset do estado do rascunho

# Documentação

- `docs/product/interface.md` — Workflows (criação em tela, etapa em modal), Ferramentas (`Abrir` é diálogo), Home
- `docs/standards/ui.md` — modal limitado à janela e um modal por vez
- `patterns/ui/modal.md` — teto de altura, corpo rolável, um modal por vez
- `patterns/ui/modal-form.md` — detalhar um item; construtor de lista vira tela; `Palette` não é mexido no diálogo
- `patterns/ui/workdir-picker.md` — destino em diálogo; substituição da camada de baixo

# Validação

- `cargo build --workspace` — sem erros;
- `cargo test --workspace` — 54 testes, 0 falhas;
- `cargo clippy --workspace --all-targets` — sem avisos;
- verificação visual manual reproduzível: tela de criação de Workflow, modal de
  etapa e modal da ferramenta.

# Impacto

- Etapas de Workflow passam a aceitar Workdir próprio pela UI (o core já suportava).
- A seção Ferramentas deixa de ter sub-nível; a barra de status não mostra mais
  `Ferramentas › <nome>` (a informação está no título do modal).
- Nenhuma alteração de schema, de manifest ou de API pública do core.
