# Padrões de UI/UX

## Princípio

A interface deve esconder complexidade incidental, não capacidade.

A direção visual e de interação inspira-se no GNOME HIG. Detalhes de stack: `docs/architecture/technology.md`.

A **estrutura** da interface (navegação, telas e o que cada uma concentra) está em [`../product/interface.md`](../product/interface.md). Este documento trata de componentes, interação e consistência visual — não de onde cada tela mora.

## Modos

Quando necessário, preferir:

- modo simples como padrão;
- modo avançado para relações, riscos, permissões e edição técnica.

## Tema

O UIGE usa **tema claro fixo**. Não segue o tema do sistema.

Motivo: `std-widgets` do Slint seguem o esquema de cores do sistema, e partes da interface usam cores explícitas. Misturar os dois já produziu texto ilegível (controle claro sobre painel escuro). Com tema fixo essa classe de bug deixa de existir.

Regras:

- `Palette.color-scheme = ColorScheme.light` é definido **uma vez**, no `init` do `AppWindow`;
- componentes novos não mexem no `Palette`; não restaurar para `unknown` ao fechar diálogos;
- quando um controle precisar de cor fora do esquema, declarar a cor explicitamente, em vez de depender do tema.

Tema alternativo (escuro) ou acompanhar o sistema exige decisão própria.

## Dimensionamento de componente

Um componente que precisa **preencher o pai** declara o tamanho no **corpo do componente** — antes do elemento raiz — e não apenas no raiz:

```slint
export component Tela {
    width: 100%;
    height: 100%;

    Rectangle {
        width: 100%;
        height: 100%;
```

Motivo: em Slint, `100%` declarado no **elemento raiz** resolve contra a própria instância, e a instância sem tamanho definido fica com o tamanho do conteúdo. O componente encolhe e aparece centralizado em vez de ocupar a área. Declarado no corpo, `100%` resolve contra o **pai** — que é o que se espera.

Regras:

- vale para telas na área da seção (`app.slint`) e para listas colocadas dentro de uma caixa: `HomeScreen`, `ToolsScreen`, `WorkflowCreateScreen`, `ProfilesScreen`, `WorkflowsScreen`, `SettingsScreen`, `ToolList`, `SelectList`, `PageBody`, diálogos e `Modal`;
- **não** vale para componente usado dentro de um layout: ali o layout dimensiona a instância, e declarar tamanho fixo atrapalha o `horizontal-stretch`;
- `Sidebar` declara `width: 168px` no corpo pelo mesmo motivo, e dialogs já seguem o padrão.

Este é o exemplo canônico do par no corpo: [`patterns/ui/modal.md`](../../patterns/ui/modal.md).

### Modal dentro da janela

O `Modal` limita o painel à janela e mantém o **corpo** rolável, com cabeçalho e ações fixos:

```text
altura = min(preferred-height do conteúdo, altura da janela - 2 * 24px)
```

Motivo: sem o teto, um diálogo mais alto que a janela era cortado na borda de baixo — sem scroll, o botão de confirmar ficava inalcançável. Vale para todo modal sem exceção, porque qualquer um pode receber conteúdo longo (`error-text` de várias linhas, output de execução, lista de etapas). Detalhes e o porquê da implementação: [`patterns/ui/modal.md`](../../patterns/ui/modal.md).

E o app mantém **um modal por vez**: quando o conteúdo de um modal abriria outro, ou a camada de baixo sai de cena e volta depois, ou o que era modal vira tela.

### Ícone com e sem rótulo

Um controle que aceita ícone tem duas formas, e a diferença é a presença do rótulo:

| Forma | Posição do ícone | Caixa |
|---|---|---|
| Com rótulo | à esquerda do texto, junto dele | do tamanho do conteúdo |
| Só ícone (`icon-only`) | centralizado na caixa | quadrada de `30px` |

Motivo: com rótulo o ícone é reforço do texto e fica colado a ele; sem rótulo o desenho é o controle inteiro, e um ícone encostado à esquerda de uma caixa maior parece desalinhado — além de deixar o alvo de clique do tamanho do desenho (`14px`). Vale para a barra de contexto e para o trilho da `Sidebar`.

### Tamanho dentro de um layout

Quando o componente é **item de um layout**, o tamanho que o layout respeita é o da **raiz do componente**:

- `min-width` / `min-height` no corpo do componente — é o que sustenta o quadrado do `icon-only` em `LightSecondaryButton`;
- `width` no `Rectangle` interno **não** propaga para o tamanho preferido da instância: a instância encolhe e o `Rectangle` vaza sobre o vizinho.

E um `HorizontalLayout` dentro de um `Rectangle` **não** assume a largura do pai (assume a largura do conteúdo; a altura, sim). Para o layout ter folga e conseguir centralizar, declarar `width: 100%` e `height: 100%` nele.

## Margens e densidade

O recuo externo é maior que o espaçamento interno. O corpo fica **mais para dentro** sem ficar mais espaçado: o que cresce é a distância até a borda da janela e até a sidebar, não a distância entre os elementos.

O interior de uma seção tem **largura limitada** e é **centralizado** — é uma coluna, não a área inteira. Quem faz isso é `PageBody` (`software/crates/ui/ui/components/page-body.slint`), usado por todas as telas; o padrão está em [`patterns/ui/page-body.md`](../../patterns/ui/page-body.md).

```text
largura da coluna = min(área - 2 * 24px, 560px)
```

Em janela larga manda o teto de `560px`; em janela estreita manda o recuo mínimo de `24px`. As telas não declaram recuo lateral próprio nem passam largura: o recuo lateral é do corpo, e uma aba mais larga que outra faria o conteúdo pular ao trocar de seção.

Valores em uso:

| Área | Recuo |
|---|---|
| Barra de contexto e barra de status | `16px` nas laterais |
| Corpo da seção (telas) | coluna limitada; `16px` no topo e na base |
| Sidebar | `10px` entre o item e a borda (ícone a `20px`) |
| Espaçamento vertical entre blocos | `8px` (a Home usa `10px`) |
| Linha de lista (`ToolRow`, `SelectList`) | `30px` de altura |
| Botão (barra e rodapés de diálogo) | `30px` de altura |
| Barra de contexto | `44px` de altura, com os controles centralizados |

Regras:

- recuos laterais do corpo nunca menores que os da moldura — o conteúdo não pode encostar na borda;
- aumentar a densidade é reduzir o espaçamento **entre** blocos, não o recuo até a borda;
- listas com superfície (`#f7f6f5`, `#f7f6f4`) acompanham os mesmos recuos laterais do resto do corpo;
- um caminho ou texto longo elide em vez de empurrar o layout.

## Listas longas

Uma lista cujo tamanho **depende do uso** é paginada, com `10` itens por página. Lista de tamanho fixo definido pela definição da ferramenta (as ações de uma ferramenta no painel de execução) e dropdown de contexto (seletor de Workspace) **não** paginam.

O paginador é o componente `Pager` (`software/crates/ui/ui/components/pager.slint`), embutido em `ToolList` e `SelectList`. As telas não montam o seu.

Regras:

- o recorte é feito no app (Rust): a UI não fatiar um modelo. A lista recebe a página pronta em `items` e o estado em `PageInfo`;
- o índice que a UI devolve (`selected-index`) é a posição na **lista inteira** — a lista soma `page-info.offset` ao traduzir para o índice da linha. Nenhum consumidor de seleção precisa saber que existe paginação;
- a página é ajustada antes de exibir; busca ou filtro novos voltam para a primeira página; uma lista recém-criada pelo app (execução recém-gravada, workflow recém-criado) abre na página do item novo;
- o paginador aparece só quando há mais de uma página, e é instanciado com `if`: em Slint um item com `visible: false` **ainda ocupa espaço no layout**, e um paginador escondido deixaria um vão em toda lista curta.

Padrão: [`patterns/ui/pagination.md`](../../patterns/ui/pagination.md).

### Teto de favoritas na Home

A Home mostra no máximo **10** ferramentas favoritas. A Home é ponto de partida, não a lista de Ferramentas; passando do teto, a nota abaixo da lista diz quantas ficaram de fora e onde encontrá-las — favorito escondido sem aviso seria defeito.

## Linguagem

Preferir termos orientados à tarefa. Ex.: mostrar “Nova ferramenta” em vez de `ManifestBuilder`.

## Como os padrões de componente nascem

Padrões concretos de componente **não são inventados de antemão**.

Eles são definidos **durante o desenvolvimento**, na primeira vez em que o componente for necessário de forma reutilizável.

Fluxo:

1. surge o uso real na UI;
2. extrai-se o padrão canônico;
3. documenta-se em `/patterns/ui/`;
4. usos seguintes reutilizam esse padrão.

Até existir o arquivo do padrão, não há implementação “oficial” daquele componente.

## Catálogo (a preencher)

Cada item abaixo deve ganhar um padrão em `/patterns/ui/` quando for usado de verdade. Enquanto estiver sem padrão, o status é **pendente**.

| Componente | Padrão | Status |
|---|---|---|
| Button | — | pendente |
| Modal | [`patterns/ui/modal.md`](../../patterns/ui/modal.md) | definido |
| Formulário em modal | [`patterns/ui/modal-form.md`](../../patterns/ui/modal-form.md) | definido |
| Corpo de seção | [`patterns/ui/page-body.md`](../../patterns/ui/page-body.md) | definido |
| Paginação de lista | [`patterns/ui/pagination.md`](../../patterns/ui/pagination.md) | definido |
| Loading / erro / sucesso | — | pendente |
| Confirmação destrutiva | — | pendente |
| Output de execução | — | pendente |
| Seleção de Workdir | [`patterns/ui/workdir-picker.md`](../../patterns/ui/workdir-picker.md) | definido |

Novos componentes recorrentes entram nesta tabela no mesmo momento em que o padrão for criado.

## O que o padrão deve registrar

Quando um padrão for definido, o arquivo em `/patterns/ui/` deve indicar ao menos:

- quando usar;
- quando não usar;
- estrutura / comportamento esperado;
- variantes permitidas (se houver);
- exemplo canônico.

Não duplicar a especificação completa do GNOME HIG aqui: linkar e adaptar ao UIGE.
