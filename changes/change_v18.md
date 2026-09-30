# Mudança

Revisão da decisão sobre linha de comando: o UIGE continua sendo **apenas interface gráfica**. O que entra é abertura da interface por comando (janela principal ou uma ferramenta específica), sem modo headless.

Esta mudança **substitui** `changes/change_v17.md`, que registrava uma CLI headless em binário único. O v17 foi preservado como histórico.

# Arquivos

- `docs/decisions/0002-abertura-da-interface-por-comando.md` (substitui `0002-cli-e-binario-unico.md`, removido)
- `docs/decisions/README.md`
- `docs/architecture/technology.md`
- `docs/product/vision.md`
- `docs/product/roadmap.md`
- `docs/product/implementation-order.md`
- `docs/domain/glossary.md`
- `docs/README.md`
- `README.md`
- `changes/change_v18.md`

# Motivo

O mantenedor definiu que o produto permanece somente com interface gráfica. A linha de comando não é uma segunda interface: serve para escolher o que a interface abre — a janela principal ou a interface de uma ferramenta. Isso reduz muito o escopo e elimina a necessidade de um contrato de saída em stdout/stderr.

# Impacto

Decisão revisada (ADR 0002):

- sem argumentos, `uige` abre a interface principal;
- com argumento (`uige --tool <id>`), abre a interface já focada na ferramenta indicada, identificada pelo `id` do Manifest;
- **não existe modo headless**: nenhuma ferramenta é executada sem GUI;
- exit code refere-se apenas à abertura, não ao resultado de uma ferramenta;
- sem servidor gráfico o produto não é utilizável — a falha deve ser explícita.

O que a revisão **remove**:

- a segunda camada de apresentação (CLI) e o contrato de saída em stdout/stderr;
- o pré-requisito de mover a orquestração de casos de uso de `software/crates/app/src/main.rs` para o core. Como a GUI continua sendo a única camada de apresentação, a orquestração permanece onde está. O item passa a não ter pré-requisitos.

O que permanece válido da versão anterior:

- a fronteira de que formatação e recorte de output (ex.: "últimas 5 linhas") pertencem ao front, não ao core — regra que já valia antes desta discussão;
- a ligação com Launcher e com o item 17 (launchers `.desktop`), que passam a poder apontar para uma ferramenta específica.

Correções de documentação incluídas: `README.md` voltou a descrever o produto como interface gráfica e teve a stack sem a linha de CLI; `docs/README.md` voltou a descrever o código como Rust + Slint; `vision.md` reafirma a interface gráfica; o termo `CLI` saiu do glossário (era detalhe de interface, não conceito de domínio).

Nenhum código foi alterado.
