# Mudança

Fase 1 (fatia vertical): modelo Manifest v1 (Serde/JSON), exemplo Echo, execução via Tokio com argumentos estruturados e UI para escolher ação e ver status/exit code/output.

# Arquivos

- `software/crates/core/src/manifest/`
- `software/crates/core/src/execution/`
- `software/crates/core/examples/echo.v1.json`
- `software/crates/core/examples/echo.windows.v1.json`
- `software/crates/core/src/lib.rs`
- `software/crates/ui/ui/app.slint`
- `software/crates/app/src/main.rs`
- `software/crates/app/Cargo.toml`
- `patterns/manifests/echo.v1.json`
- `patterns/manifests/README.md`
- `docs/product/implementation-order.md`
- `software/README.md`
- `changes/change_v7.md`

# Motivo

Entregar o critério de pronto da Fase 1: carregar um manifest, executar um comando real e mostrar o resultado.

# Impacto

O binário `uige` deixa de ser só hello e passa a executar a ferramenta Echo de demonstração. Próximo passo: Fase 2 (Workspace / biblioteca).
