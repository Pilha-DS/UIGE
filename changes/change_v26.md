# Mudança

O interior de uma seção ocupava a área inteira: em janela larga, uma linha de
ferramenta atravessava a tela de ponta a ponta, com o nome de um lado e os botões
do outro. Nesta mudança o corpo virou uma **coluna de leitura**: largura limitada a
`640px` e centralizada, com recuo mínimo de `24px` quando a janela é estreita.

# Arquivos

UI:

- `software/crates/ui/ui/components/page-body.slint` (novo)
- `software/crates/ui/ui/screens/home.slint`
- `software/crates/ui/ui/screens/tools.slint`
- `software/crates/ui/ui/screens/tool-screen.slint`
- `software/crates/ui/ui/screens/profiles.slint`
- `software/crates/ui/ui/screens/workflows.slint`
- `software/crates/ui/ui/screens/settings.slint`

Docs:

- `docs/standards/ui.md`
- `patterns/ui/page-body.md` (novo)
- `patterns/ui/README.md`
- `changes/change_v26.md`

# Motivo

Relato: "por body digo o interior de uma sessão, como aba ferramentas, a parte
interna, aba home e etc, ele deve ser mais fino horizontalmente, o body da aba".

O `change_v25.md` tratou de recuo e densidade, mas o recuo era um número fixo: em
uma janela maximizada o conteúdo continuava esticando até as bordas, só que com
`20px` de folga. O que faltava não era mais recuo — era um **teto de largura**.

# Impacto

O recuo lateral deixou de ser declarado nas telas e passou a ser consequência da
coluna:

```text
largura = min(área da seção - 2 * 24px, 640px)
```

- **janela larga** — manda o teto: a coluna tem `640px` e centraliza; o recuo cresce
  junto com a janela;
- **janela estreita** — manda o recuo mínimo de `24px`: a coluna encolhe com a área,
  sem apertar o conteúdo contra a borda.

Consequências:

| | Antes (`change_v25`) | Agora |
|---|---|---|
| Largura do corpo em `900x600` | `692px` | `640px` |
| Recuo da sidebar em `900x600` | `20px` | `47px` |
| Largura do corpo maximizado em `1920` | ~`1720px` | `640px` |

As telas perderam `padding-left`/`padding-right`: agora elas recebem a coluna já
dimensionada, e as superfícies de grupo (`#f7f6f5`, `#f7f6f4`) vão até a borda da
coluna, delimitando-a. O recuo vertical (`16px` no topo e na base) continua na tela.

## Ponto de Slint encontrado

Duas restrições apareceram ao escrever o componente, e o código ficou ajustado a
elas:

- `parent` **não** é acessível no corpo do componente, só dentro de um elemento
  filho. Por isso a coluna é um `Rectangle` interno, e o cálculo de `x`/`width` vive
  nele;
- `@children` funciona em elemento interno (não só no raiz) — é o que permite as
  children entrarem já dentro da coluna;
- `length / número` não retorna `length` em Slint (`Cannot convert float to length`).
  A centralização usa `* 0.5`, não `/ 2`.

## Regra registrada

- `docs/standards/ui.md` — a seção **Margens e densidade** passou a descrever a
  coluna, com a fórmula e os valores canônicos; `PageBody` entrou na lista de
  componentes que dimensionam no corpo;
- `patterns/ui/page-body.md` — novo padrão, no catálogo como **definido**.

# Validação

- `cargo build --workspace` — compila.
- `cargo clippy --workspace --all-targets -- -D warnings` — sem avisos.
- `cargo test --workspace` — 49 testes passam (45 no core, 2 em `manifest_import`,
  2 em `workflow_run`).

Verificação visual com o app em execução, medindo por pixel na captura (DPI 125%).
A coluna foi medida em três larguras de janela:

| Janela | Área da seção | Coluna medida | Recuo da sidebar |
|---|---|---|---|
| `900x600` (padrão) | `732px` | `214,4..852,8` → `639,2px` | `47,2px` |
| `1920x1080` (maximizada) | `1368px` | `524,8..1164` → `640px` | `357,6px` |
| `640x520` (estreita) | `588px` | `76..615,2` → `540px` | `24,8px` |

Em janela estreita a sidebar vira trilho de `52px` e a coluna passa a ser
dimensionada pelo recuo mínimo, como projetado.

As cinco seções e a tela da ferramenta foram capturadas com clique real na sidebar;
a seção ativa foi confirmada pelo realce detectado na captura, não por suposição. O
título da seção começa em `x ≈ 216` em todas elas — coerente com a coluna começando
em `214,4`.

**Não validado:** nenhum diálogo foi aberto depois da mudança (eles têm largura
própria e não usam `PageBody`); a lista de Ferramentas foi verificada com 2
ferramentas, então uma linha com o nome muito longo não foi exercitada — ela elide,
como antes.
