# Mudança

Scaffold da Fase 0: workspace Cargo em `software/` com crates `uige-core`, `uige-ui` e binário `uige` (hello Slint ↔ core + SQLite mínimo).

# Arquivos

- `software/Cargo.toml`
- `software/README.md`
- `software/crates/core/`
- `software/crates/ui/`
- `software/crates/app/`
- `docs/architecture/technology.md`
- `docs/product/implementation-order.md`
- `README.md`
- `.gitignore`
- `changes/change_v6.md`

# Motivo

Iniciar a implementação conforme a ordem da Fase 0, isolando o código da aplicação em `software/`.

# Impacto

É possível compilar e rodar o hello no Linux. Próximo passo: validar no alvo Linux e avançar para Fase 1 (manifest + execução).
