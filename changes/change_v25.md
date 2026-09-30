# Mudança

O conteúdo das telas estava encostando na borda: o recuo lateral era de `16px` e a
mesma medida valia para a moldura, então os grupos de superfície (`Ferramentas
favoritas`, `Execuções recentes`) chegavam a poucos pixels da barra da janela.

Nesta mudança o corpo foi para dentro e ficou mais compacto ao mesmo tempo: cresceu
o **recuo até a borda**, diminuiu o **espaçamento entre os blocos** e as linhas de
lista ficaram mais baixas.

# Arquivos

UI:

- `software/crates/ui/ui/app.slint`
- `software/crates/ui/ui/components/sidebar.slint`
- `software/crates/ui/ui/components/tool-list.slint`
- `software/crates/ui/ui/components/select-list.slint`
- `software/crates/ui/ui/components/execution-panel.slint`
- `software/crates/ui/ui/screens/home.slint`
- `software/crates/ui/ui/screens/tools.slint`
- `software/crates/ui/ui/screens/tool-screen.slint`
- `software/crates/ui/ui/screens/profiles.slint`
- `software/crates/ui/ui/screens/workflows.slint`
- `software/crates/ui/ui/screens/settings.slint`

Docs:

- `docs/standards/ui.md`
- `changes/change_v25.md`

# Motivo

Relato: "o conteúdo está muito perto da borda, deixe os mais para dentro, o body
mais compacto".

As duas coisas parecem opostas — para dentro costuma significar maior — mas não
são: eram dois eixos diferentes medidos com o mesmo número. O recuo até a borda
precisava crescer; o espaçamento interno, diminuir. Antes, `padding: 16px` nas
telas e `padding-left: 12px` na barra de contexto davam a impressão de conteúdo
espremido contra a moldura, com o miolo frouxo.

# Impacto

## Recuos

| Área | Antes | Agora |
|---|---|---|
| Barra de contexto (laterais) | `12px` | `16px` |
| Barra de status (laterais) | `12px` | `16px` |
| Corpo da seção (laterais) | `16px` | `20px` |
| Corpo da seção (topo e base) | `16px` | `16px` (inalterado) |
| Item da sidebar | `8px` da borda | `10px` da borda |

O recuo vertical do corpo ficou como estava de propósito: aumentar as laterais
resolve "perto da borda" sem empurrar o conteúdo para fora da janela, que já é
compacta por decisão anterior (`preferred-width: 900px` e `preferred-height: 600px`
em `app.slint`).

## Densidade

| Elemento | Antes | Agora |
|---|---|---|
| Espaçamento entre blocos das telas | `10px` | `8px` |
| Espaçamento entre blocos da Home | `12px` | `10px` |
| Linha de `SelectList` | `34px` | `30px` |
| Linha de `ToolRow` | `32px` | `30px` |
| Lista de ações do painel de execução | `108px` | `100px` |
| Console do painel de execução | `118px` | `110px` |
| Histórico da tela da ferramenta | `132px` | `124px` |
| Caixa de etapas do workflow | `148px` | `136px` |
| Lista de candidatos (Configurações) | `132px` | `124px` |
| Item da sidebar | `32px` | `30px` |

## Regra registrada

`docs/standards/ui.md` ganhou a seção **Margens e densidade**, com a tabela de
valores em uso e a regra que evita redescobrir a confusão: densidade se ajusta no
espaçamento entre blocos, não no recuo até a borda.

# Validação

- `cargo build --workspace` — compila.
- `cargo clippy --workspace --all-targets -- -D warnings` — sem avisos.
- `cargo test --workspace` — 49 testes passam (45 no core, 2 em `manifest_import`,
  2 em `workflow_run`).

Verificação visual, com o app em execução (janela `900x600` lógicos, DPI 125%) e
captura da janela na resolução física. Os recuos foram medidos por pixel na
captura, não por inspeção do código.

| Medida | Antes | Depois |
|---|---|---|
| Tinta do título da Home | `x = 185,6` | `x = 189,6` |
| Superfície `#f7f6f4` da Home | `x 184..883` | `x 188..879,2` |
| Recuo lateral esquerdo do grupo | `16px` | `20px` |
| Recuo lateral direito do grupo | `17px` | `20px` |
| Primeira tinta da barra de contexto | — | `x = 17,6` |
| Primeira tinta da barra de status | — | `x = 16,8` |
| Altura do item da sidebar | — | `29,6px` |

Os valores "antes" da Home foram medidos na captura anterior à mudança; os das
barras só foram medidos depois (`x = 17,6` e `x = 16,8` são `16px` de recuo mais o
desenho do glifo).

O corpo começa em `x = 188` lógicos nas cinco seções verificadas (Home,
Ferramentas, Perfis, Workflows, Configurações) e na tela da ferramenta — ou seja,
`20px` dentro da área da seção, que começa em `x = 168`.

A troca de seção foi feita por clique real na sidebar, e a seção ativa foi
confirmada pelo realce detectado na captura (índices `0 a 4` conferidos um a um);
não pela suposição de qual tela tinha sido aberta.

O modo responsivo foi reconferido depois da mudança de altura do item: com a janela
em `700px` lógicos a sidebar vira trilho de `52px` (ícones sem rótulo) e os botões da
barra de contexto ficam só ícone — nada fica cortado.

**Não validado:** a renderização de listas longas que precisem rolar depois das
novas alturas (o gutter do `ScrollView`, que reserva espaço para a barra, não foi
alterado); e nenhum diálogo foi reaberto nesta mudança — os recuos do `Modal` e dos
diálogos ficaram como estavam.
