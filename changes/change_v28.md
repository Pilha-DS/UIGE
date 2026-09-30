# Mudança

As listas do UIGE exibiam tudo o que existisse. Com duas ferramentas isso não
aparece; com quarenta, a lista empurra o resto da tela para fora e a pessoa perde o
que estava vendo. Nesta mudança toda lista cujo tamanho **depende do uso** passou a
ser **paginada em 10 itens**, e a Home passou a mostrar no **máximo 10 favoritas**.

Quem sofre o recorte é o app, não a UI: o Slint não consegue fatiar um modelo. A UI
recebe a página pronta e devolve só a intenção de andar.

# Arquivos

UI:

- `software/crates/ui/ui/components/pager.slint` (novo) — `PageInfo` e `Pager`
- `software/crates/ui/ui/components/select-list.slint` — paginação embutida
- `software/crates/ui/ui/components/tool-list.slint` — paginação embutida
- `software/crates/ui/ui/app.slint` — uma propriedade de página por lista, com os
  callbacks de pedido ligados às telas
- `software/crates/ui/ui/screens/{tools,tool-screen,profiles,workflows,settings}.slint`
- `software/crates/ui/ui/screens/home.slint` — nota das favoritas excedentes

App:

- `software/crates/app/src/main.rs` — recorte de página, teto da Home e os
  handlers de pedido de página

Docs:

- `docs/standards/ui.md` — seção **Listas longas**
- `patterns/ui/pagination.md` (novo) e `patterns/ui/README.md`
- `changes/change_v28.md`

# Motivo

Relato: *"coloque paginação nas ferramentas, workflow e etc, e na home limite para 10
ferramentas máximas na home como favoritas"*.

São dois pedidos com a mesma origem: **nenhuma lista sem limite**. A lista de
Ferramentas cresce com a biblioteca; as execuções crescem com o uso; os candidatos
crescem com o diretório de manifests. A Home é o caso extremo do mesmo problema —
uma lista que cresce sozinha empurrando as Execuções recentes para fora da primeira
tela, num lugar que existe para ser ponto de partida.

# Impacto

## Listas paginadas

| Lista | Onde | Estado |
|---|---|---|
| Ferramentas | seção Ferramentas | `tool-page-info` |
| Perfis | seção Perfis | `profile-page-info` |
| Execuções de uma ferramenta | tela da ferramenta | `history-page-info` |
| Candidatos de manifest | Configurações | `candidate-page-info` |
| Workflows | seção Workflows | `workflow-page-info` |
| Execuções de um workflow | seção Workflows | `workflow-history-page-info` |

Não paginam: as **ações de uma ferramenta** no painel de execução (conjunto curto e
fixo, vindo da definição) e o **seletor de Workspace** na barra de contexto (é
dropdown de contexto, não lista de conteúdo).

Na seção Workflows o `ComboBox` de `std-widgets` saiu: escolher workflow é escolher
em lista de conteúdo, e a lista é a que cresce com o uso — um dropdown não pagina.
O `ComboBox` continua nos diálogos (fundo branco), onde não briga com superfície
nenhuma.

## Protocolo de página

```text
PageInfo { page, page-count, total, offset }
```

- `total` é o tamanho da lista **inteira** (depois do filtro). É o que a pessoa quer
  saber antes de avançar; a página sozinha não diz o tamanho do conjunto.
- `offset` é a posição do primeiro item da página na lista inteira. A lista soma
  `offset` ao traduzir o índice da linha, então **nenhum consumidor de seleção
  precisa saber que existe paginação** — os callbacks de ▶ / 📁 / ⋮ continuam
  recebendo a posição na lista cheia, como antes.

O estado da página vive na propriedade que já existia por lista, e não em estado
paralelo no app: a UI pede uma página *escrevendo* nela, e o `refresh` seguinte
recorta a partir do que está publicado. Um número, dois pontos do mesmo ciclo.

O recorte é sempre **ajustado antes de exibir**:

| Situação | Página |
|---|---|
| A lista encolheu (busca, filtro, remoção) | última página válida |
| Busca ou filtro novos | primeira página |
| Histórico de uma ferramenta aberta | primeira página (o mais recente) |
| Workflow recém-criado | página que contém o workflow |
| Execução de workflow recém-gravada | primeira página (a mais recente) |

Os dois últimos são o caso que motivou o `reveal`: uma lista que o app acabou de
produzir **abre onde está o item novo**. Criá-lo e não vê-lo seria o mesmo que não
tê-lo criado.

## Ponto de Slint

- **`if`, não `visible`.** Em Slint um item com `visible: false` continua ocupando
  lugar no layout: o paginador escondido de uma lista curta reservaria `30px` de vão
  em toda lista de uma página. Por isso ele é instanciado dentro de
  `if page-count > 1`.
- **`for item[idx] in items: if ...` não existe.** Filtrar a página no Slint foi a
  primeira tentativa e não passa pelo parser; fatiar no app resolveu e ainda tirou da
  UI a responsabilidade de saber quantos itens existem.

## Teto de favoritas na Home

| | Antes | Agora |
|---|---|---|
| Favoritas exibidas | todas | `10` (`MAX_HOME_FAVORITES`) |
| Favoritas escondidas | — | nota com a contagem e o caminho |

Cortar em silêncio esconderia favoritos sem que a pessoa soubesse por quê. A nota
(*"Mostrando 10 de 14 favoritas · veja todas em Ferramentas."*) aparece só quando há
excedente e diz para onde ir.

## Regra registrada

- `docs/standards/ui.md` — seção **Listas longas**, com o que pagina e o que não
  pagina, o teto da Home e as armadilhas de Slint;
- `patterns/ui/pagination.md` — padrão novo, entrando no catálogo como **definido**.

# Validação

- `cargo build --workspace` — compila.
- `cargo clippy --workspace --all-targets -- -D warnings` — sem avisos.
- `cargo test --workspace` — 54 testes passam (45 no core, 5 de recorte de página no
  app, 2 em `manifest_import`, 2 em `workflow_run`).

Os cinco testes do app são novos e cobrem a aritmética do recorte, não a aparência:

| Teste | Cobre |
|---|---|
| `a_short_list_has_a_single_page` | lista curta = uma página, sem paginador |
| `pages_cover_the_list_without_overlap` | 25 itens (10/10/5) cobrem a lista, sem sobreposição |
| `a_page_past_the_end_is_clamped_to_the_last_one` | página 9 com 12 itens cai na página 1 |
| `revealing_an_index_moves_to_the_page_that_contains_it` | índice 21 abre na página 2 |
| `an_empty_list_has_no_pages` | lista vazia tem zero páginas |

O recorte é o único ponto com lógica nova: o resto é ligação de propriedade e
callback, que o compilador do Slint já confere — uma propriedade inexistente ou um
callback sem assinatura não compila.

O app foi aberto (`uige.exe`, janela real) e permaneceu rodando: as propriedades de
página novas não quebram a inicialização nem o primeiro desenho das listas.

## Não validado

- **Nada foi medido na tela nesta mudança.** O paginador só aparece a partir de 11
  itens, e a base local não tem nenhuma lista desse tamanho, então ele não foi visto
  renderizado — nem a ausência de vão em lista curta foi medida por pixel. O que
  sustenta essa ausência é a escolha de `if` no lugar de `visible`, não uma captura.
- A troca do `ComboBox` por `SelectList` na seção Workflows muda a interação (escolha
  em lista com scroll, em vez de dropdown suspenso) e não foi conferida aberta.
- O rodapé do paginador não foi conferido na janela mínima. Pela conta, ele cabe:
  em `620px` (o mínimo) a sidebar já é trilho de `52px`, a coluna fica em `520px` e
  o conteúdo do paginador (rótulo da faixa e dois botões) não passa de `~400px`. É
  estimativa a partir das medidas declaradas, não medição.
