# UI Patterns

Exemplos canônicos de componentes da interface.

Os padrões nascem **durante o desenvolvimento**, na primeira necessidade real. Até lá, o catálogo e as regras ficam em `docs/standards/ui.md`.

Cada padrão deve indicar:

- quando usar;
- quando não usar;
- estrutura / comportamento;
- variantes permitidas;
- exemplo canônico.

Sugestão de nomes:

```text
button.md
modal.md
form.md
feedback-states.md
destructive-confirm.md
execution-output.md
workdir-picker.md
```

## Definidos

- [`modal.md`](modal.md) — overlay para tarefas pontuais (ex.: criar perfil); painel limitado à janela, corpo rolável e um modal por vez.
- [`modal-form.md`](modal-form.md) — formulário dentro do modal, inclusive a variante de detalhar um item e o construtor de lista que vira tela.
- [`page-body.md`](page-body.md) — coluna de leitura do interior de uma seção, com largura limitada e centralizada.
- [`pagination.md`](pagination.md) — recorte de lista longa em páginas, com o índice da linha continuando a ser global.
- [`workdir-picker.md`](workdir-picker.md) — diálogo de seleção de pasta, compartilhado pelo Workdir global e pelo de uma ferramenta.
