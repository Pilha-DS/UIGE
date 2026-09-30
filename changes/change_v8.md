# Mudança

Manifest Git v1 (somente leitura) para testar no Windows, seleção de ferramenta na UI (Echo / Git) e workdir no repositório Git detectado.

# Arquivos

- `patterns/manifests/git.v1.json`
- `patterns/manifests/README.md`
- `software/crates/core/examples/git.v1.json`
- `software/crates/core/src/manifest/mod.rs`
- `software/crates/core/src/lib.rs`
- `software/crates/core/src/execution/runner.rs`
- `software/crates/ui/ui/app.slint`
- `software/crates/app/src/main.rs`
- `changes/change_v8.md`

# Motivo

Permitir exercitar execução real com Git no Windows além do demo Echo.

# Impacto

A UI lista Git com ações `Versão`, `Status`, `Listar branches`, `Log recente` e `Remotes`.
