# Mudança

As telas voltaram a ocupar a área da seção. Um bug de dimensionamento em Slint
fazia `HomeScreen`, `ToolsScreen`, `ToolScreen`, `ProfilesScreen`,
`WorkflowsScreen` e `SettingsScreen` renderizarem no **tamanho do conteúdo**, em
vez de preencherem a área: a Home ficava em branco e a aba Ferramentas mostrava
título, dica e busca — mas nenhuma ferramenta.

Também corrigido o alinhamento das linhas da lista de Ferramentas: o rótulo e os
botões ficavam centralizados na linha em vez de rótulo à esquerda e botões à
direita.

# Arquivos

UI:

- `software/crates/ui/ui/screens/home.slint`
- `software/crates/ui/ui/screens/tools.slint`
- `software/crates/ui/ui/screens/tool-screen.slint`
- `software/crates/ui/ui/screens/profiles.slint`
- `software/crates/ui/ui/screens/workflows.slint`
- `software/crates/ui/ui/screens/settings.slint`
- `software/crates/ui/ui/components/tool-list.slint`
- `software/crates/ui/ui/components/select-list.slint`

Docs:

- `docs/standards/ui.md`
- `changes/change_v24.md`

# Motivo

O relato foi "não tem nenhuma ferramenta aparecendo" na aba Ferramentas. As
ferramentas existiam: o banco tinha `echo` e `git`, o Workspace padrão referenciava
as duas e o modelo da lista era preenchido com 2 linhas. O problema era de
renderização.

Causa: em Slint, `width: 100%` declarado no **elemento raiz** de um componente
resolve contra a própria instância. Sem tamanho definido na instância, ela fica
com o tamanho do conteúdo. Os diálogos e `Modal` já declaram `width: 100%;
height: 100%;` no **corpo** do componente — que resolve contra o pai — então
funcionavam. As telas declaram só no elemento raiz, então encolhiam.

Consequência por tela: Home com tamanho zero (área em branco), Ferramentas com o
tamanho do conteúdo (título, dica e busca visíveis, lista sem altura), e as
demais seções com o mesmo defeito.

# Impacto

## Correção

`width: 100%; height: 100%;` passou para o corpo de cada tela, de `ToolList` e de
`SelectList`. As listas precisavam do par porque são colocadas dentro de uma caixa
(`Rectangle`), que não é layout.

Componentes usados **dentro de um layout** não foram tocados: ali o layout
dimensiona a instância, e um tamanho declarado no corpo atrapalharia o
`horizontal-stretch` (`ExecutionPanel`, `LightTextField`, `LightPrimaryButton`,
`LightSecondaryButton`, `LightToggleButton`).

## Alinhamento das linhas

O `HorizontalLayout` da linha em `tool-list.slint` tinha `alignment: center`,
herdado do padrão da sidebar (onde ícone e rótulo centralizados fazem sentido). O
efeito era o conjunto rótulo + três botões centralizado na linha. Removido: com o
alinhamento padrão, o `horizontal-stretch: 1` do rótulo absorve o espaço livre e
empurra os botões para a direita.

## Regra registrada

`docs/standards/ui.md` ganhou a seção **Dimensionamento de componente**, para que a
distinção "corpo do componente" × "elemento raiz" não precise ser redescoberta —
foi o que faltou em `change_v23.md`.

# Validação

- `cargo build` — compila.
- `cargo clippy --workspace --all-targets -- -D warnings` — sem avisos.
- `cargo test --workspace` — 47 testes passam (43 no core, 2 + 2 nas integrações).

Verificação visual — o app foi executado, trazido para primeiro plano e as cinco
seções foram capturadas, trocando de seção por clique real na sidebar:

| Seção | Antes | Depois |
|---|---|---|
| Home | área vazia | título, seções com divisores e atalhos |
| Ferramentas | título, dica e busca; lista sem altura | 2 linhas (`Echo`, `Git`), rótulo à esquerda e `Executar`/`Abrir`/`Ações` à direita |
| Perfis | — | título, dica, lista vazia e botão |
| Workflows | — | título, ComboBox, caixa de etapas e histórico |
| Configurações | — | renderiza na área inteira |

A seção correta era confirmada pelo realce da sidebar em cada captura. O conteúdo
foi medido por pixels (proporção de pixels escuros e caixa delimitadora na área
principal), não só por inspeção do código.

**Não validado:** os fluxos de clique dentro das telas (executar pelo modal, abrir
tela da ferramenta, criar perfil) não foram percorridos de ponta a ponta nesta
mudança; a verificação cobriu a renderização das telas. O alinhamento vertical da
mensagem de lista vazia ficou como está (centralizado), por ser um efeito herdado
de `Text` e não ter sido pedido.
