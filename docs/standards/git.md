# Git e mudanças

## Commits

Commits devem ser pequenos o suficiente para explicar uma mudança coerente.

## Mudanças documentadas

Toda mudança relevante deve criar um novo arquivo em `/changes/`.

Formato sugerido:

```text
change_v1.md
change_v2.md
change_v3.md
```

Nunca apagar um registro anterior apenas porque uma nova mudança o substituiu.

## Conteúdo mínimo

Cada change deve conter:

- `# Mudança` — resumo do que mudou;
- `# Arquivos` — arquivos afetados;
- motivação quando não for óbvia;
- impacto em compatibilidade/migração quando existir.
