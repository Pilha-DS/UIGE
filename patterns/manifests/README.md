# Manifest Patterns

Exemplos canônicos de manifests e recursos do schema.

## Schema v1 (Fase 1)

- [`echo.v1.json`](echo.v1.json) — demonstração mínima (`echo` + comando `print` + parâmetro `message`).
- [`git.v1.json`](git.v1.json) — Git somente leitura (`version`, `status`, `branch`, `log`, `remote`).
- Variante Windows do Echo: `software/crates/core/examples/echo.windows.v1.json`.

Campos mínimos do schema `v1`:

```text
schemaVersion
id
name
executable
commands[]
  id, name, argvPrefix?, parameters[]
    id, name, type, flag?, required?, default?
```

## Descoberta e importação

Qualquer arquivo nesse formato pode ser descoberto e importado pelo app; as regras estão em `docs/product/features.md`.

Para testar: copie um exemplo para a pasta padrão de manifests (`~/.local/share/uige/manifests` no Linux) e use “Procurar” na seção **Manifests** do app.
