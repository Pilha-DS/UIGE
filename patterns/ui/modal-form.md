# Formulário em modal

## Quando usar

- coletar dados de uma entidade dentro de um modal ([`modal.md`](modal.md)), com validação e erro;
- formulários com poucos campos, montados a partir de dados que o core já conhece (combo de ferramenta, combo de ação, campo de parâmetro);
- detalhar **um** item de uma lista cuja montagem acontece em outro lugar (ex.: uma etapa de workflow).

## Quando não usar

- formulários longos, com muitos campos independentes ou seções (preferir tela dedicada);
- fluxos com múltiplos passos navegáveis (preferir wizard/tela dedicada);
- quando o próprio formulário é o que monta uma lista longa — nesse caso a montagem
  vai para uma tela e o formulário de cada item continua modal (ver *Variantes*);
- edição de campos que exijam contexto externo (diff, pré-visualização de arquivo).

## Estrutura / comportamento

1. Um arquivo `.slint` por diálogo, em `components/`, montado com `if open:` na home.
2. O diálogo encapsula o [`Modal`](modal.md) e expõe `open`, `busy`, os valores do formulário, `error-text` e callbacks (`selection-changed`, `confirm-requested`, `close-requested`).
3. **Cores explícitas.** O painel do modal é claro mesmo sob tema escuro. Campos e botões vêm de `components/light-controls.slint` (`LightTextField`, `LightSecondaryButton`, `LightPrimaryButton`) — não usar `LineEdit`/`Button` do `std-widgets` dentro do painel.
4. **`ComboBox` é a exceção.** Continuamos usando o do `std-widgets`. Ele segue o `Palette`, que é fixado em `ColorScheme.light` **uma vez**, no `init` do `AppWindow` (`docs/standards/ui.md`) — o diálogo **não** mexe no `Palette`.
5. **Validação no Rust, não no Slint.** O diálogo só exibe `error-text`; quem decide se os dados são válidos é o core (ou o app ao chamar o core). Mensagem de erro em português, orientada à correção.
6. **`busy` desabilita tudo.** Enquanto o salvamento ocorre, campos, combos e botões ficam desabilitados e “Cancelar” não fecha.
7. **Layout vertical** com rótulo acima do controle (`Text` + campo), `spacing` de 12px, e separador de 1px quando o formulário tiver blocos distintos.
8. **Confirmar adiciona, cancelar descarta.** O item em construção pertence ao rascunho, não ao formulário: fechar sem confirmar não pode alterar o que já está montado.

## Variantes

- **Formulário simples** — campos + Cancelar/Salvar (ex.: criar perfil).
- **Detalhar um item** — o formulário de **uma** etapa: ferramenta, ação, parâmetro e o que fazer se falhar (ex.: `components/workflow-step-dialog.slint`). Quem monta a **sequência** não é o modal, e sim uma tela.
- **Construtor de lista (em tela)** — quando o formulário precisa montar uma lista
  longa, o que era modal vira **tela**: a lista fica visível o tempo todo e o modal
  passa a ser o detalhe de cada item. É o que evita modal sobre modal
  ([`modal.md`](modal.md), *Um modal por vez*). Exemplo:
  `screens/workflow-create.slint` (tela) + `components/workflow-step-dialog.slint`
  (modal).
- Futuro: variante com bloqueio de campos dependentes entre si (ver item 19 da Fase 4).

## Exemplo canônico

- Controles compartilhados: `software/crates/ui/ui/components/light-controls.slint`
- Formulário simples: `software/crates/ui/ui/components/create-profile-dialog.slint`
- Detalhar um item: `software/crates/ui/ui/components/workflow-step-dialog.slint`
- Construtor de lista em tela: `software/crates/ui/ui/screens/workflow-create.slint`
