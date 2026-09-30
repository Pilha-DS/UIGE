# Mudança

Três ajustes de acabamento na moldura e nos controles, a partir de leitura visual
direta da interface:

1. **corpo mais compacto** — a coluna de leitura da seção caiu de `640px` para
   `560px`;
2. **barra de contexto** — a barra passou de `52px` para `44px` e o seletor de
   Workspace deixou de ser o `ComboBox` de `std-widgets` (fundo branco e borda
   próprios, fora da superfície do UIGE) para um controle claro do próprio projeto;
3. **ícone sem rótulo** — quando não há palavra ao lado, o ícone centraliza na
   caixa; com rótulo ele continua à esquerda do texto.

# Arquivos

   20|UI:

- `software/crates/ui/ui/components/light-controls.slint` — novo `LightSelect` e
  correção de tamanho do `LightSecondaryButton` em `icon-only`
- `software/crates/ui/ui/components/icons.slint` — `IconKind.chevron` /
  `IconChevron` e centralização do `Icon` dentro de layouts
- `software/crates/ui/ui/app.slint` — `bar-height: 44px`, `LightSelect` no lugar do
  `ComboBox` e deslocamentos da moldura presos a `bar-height`
- `software/crates/ui/ui/components/page-body.slint` — `max-content-width: 560px`

   30|Docs:

- `docs/standards/ui.md`
- `patterns/ui/page-body.md`
- `changes/change_v27.md`

# Motivo

Relato: *"1 - deixe mais compacto ainda / 2 - arrume esse header como da para ver na
imagem / 3 - quando tiver espaco para a palavra o icone fica como esta, mas quando
nao tem o icone deve ficar centralizado"*.

   40|Os três têm a mesma origem: a moldura estava desenhada com medidas que não
conversavam entre si.

- O `ComboBox` de `std-widgets` não segue a superfície do UIGE: numa barra `#f7f6f5`
  ele virava uma laje branca, com altura e raio diferentes dos botões ao lado. Dois
  controles de alturas diferentes na mesma linha nunca ficam alinhados.
- Barra de `52px` com controles de `30px` deixava `11px` de folga em cima e embaixo:
  espaço pago sem uso, num app cuja direção é densa e sem bordas.
- `LightSecondaryButton` prometia um quadrado de `30px` em `icon-only`, mas o ícone
  aparecia colado à esquerda — e a caixa vazava sobre o campo de texto seguinte.

# Impacto

## 1. Corpo

| | Antes | Agora |
|---|---|---|
| `max-content-width` | `640px` | `560px` |

O teto é o mesmo para todas as telas: quem muda de largura entre abas faz o conteúdo
pular ao trocar de seção. Reduzir a coluna é o que deixa o corpo "mais fino
horizontalmente" sem mexer no recuo até a borda.

## 2. Barra de contexto

| | Antes | Agora |
|---|---|---|
| Altura da barra | `52px` | `44px` |
| Seletor de Workspace | `ComboBox` (`std-widgets`) | `LightSelect` |
| Altura dos controles | `30px` (`ComboBox` divergia) | `30px`, iguais |
| Raio | `4px` (`ComboBox`) | `6px`, igual aos botões |

O `LightSelect` repete a superfície dos outros controles claros: fundo `#f2f1ef`,
raio `6px`, sombra leve, sem borda. A largura é decidida por quem usa (`156px`, ou
`132px` em janela estreita), para a barra não mexer quando o nome do Workspace muda
de tamanho. A lista abre em `PopupWindow`, com altura vinda do modelo.

`LightSelect` existe no lugar do `ComboBox` **onde o controle divide a barra com os
controles claros**. Dentro de diálogo (fundo branco) o `ComboBox` continua em uso:
lá ele não briga com superfície nenhuma.

## 3. Ícone

| Forma | Ícone | Caixa |
|---|---|---|
| Com rótulo | à esquerda do texto, junto dele | tamanho do conteúdo |
| Só ícone (`icon-only`) | centralizado | quadrada de `30px` |

Duas causas, não uma:

- o `Icon` tinha `height: size`; em Slint, item de layout com altura **explícita**
  alinha ao **início** do eixo cruzado, então o desenho subia para o topo da linha
  (efeito visível no `+` do botão e no trilho da sidebar). Trocado por
  `min-height: size` + `vertical-stretch: 1`, com um retângulo interno que centraliza
  o desenho dentro da caixa;
- o quadrado de `30px` estava declarado como `width` no `Rectangle` interno. O layout
  pai dimensiona a **raiz** do componente, então a instância encolhia para os `14px`
  do ícone: o desenho ficava à esquerda de uma caixa maior e o `Rectangle` vazava
  `16px` sobre o campo de texto seguinte. O `min-width` foi para o corpo do
  componente, e o layout interno recebeu `width: 100%` para ter folga e centralizar.

# Validação

Medição por pixel na captura da janela real (`PrintWindow`, DPI 125%), com o app em
execução — não por leitura do código.

**Barra de contexto, janela `900x600`** (larguras em px lógicos):

| Controle | Largura | Centro vertical |
|---|---|---|
| `LightSelect` | `157,6` | `22,8` |
| `+` Novo | `76,8` | `22,8` |
| Campo do Workdir | `267,2` | `22,8` |
| Escolher | `96,8` | `22,8` |
| Salvar | `83,2` | `22,8` |

Centro da barra: `22`. Todos os controles com a mesma altura e o mesmo centro — a
barra não tem mais controle fora de linha.

**Botão só ícone, janela estreita** (`770x530` lógicos), posição da tinta na captura:

| | Antes | Depois |
|---|---|---|
| Ícone do `+` | `159,2..165,6` | `167,2..173,6` |
| Início do campo | `188,8` | `204,8` |

O `+` andou `8px` — exatamente `(30px − 14px) * 0,5`, o deslocamento de centralização.
O campo andou `16px` porque o botão passou a ocupar os `30px` que ele já declarava,
em vez de vazar sobre o campo.

**Trilho da sidebar** (`52px`), desvio do ícone em relação ao centro da linha:

| Linha | Desvio vertical | Desvio horizontal |
|---|---|---|
| 1 | `−0,2` | `−0,8` |
| 2 | `−0,6` | `−0,4` |
| 3 | `−0,6` | `−0,4` |
| 4 | `−0,2` | `−0,4` |

Abaixo de `1px` nos dois eixos. A linha 0 não entra na conta: ela é a seção ativa e a
marca de `3px` na borda esquerda puxa a caixa da tinta para a esquerda.

**Com rótulo** o ícone continua à esquerda do texto: medido no botão `Novo` em janela
larga, o `+` começa a `12px` da borda do controle e o texto a `33,6px`.

- `cargo build --workspace` — compila.
- `cargo clippy --workspace --all-targets -- -D warnings` — sem avisos.
- `cargo test --workspace` — 49 testes passam (45 no core, 2 em `manifest_import`,
  2 em `workflow_run`).

## Não validado

- a lista do `LightSelect` foi conferida aberta durante o desenvolvimento, mas a
  captura final é do estado fechado — o `PopupWindow` é janela própria e não aparece
  no `PrintWindow` da janela principal;
- os diálogos não foram reabertos: eles usam `ComboBox` e não foram tocados.
