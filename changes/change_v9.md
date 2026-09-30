# Mudança

Fase 2: Workspace padrão com Workdir global, biblioteca de ferramentas, execução rápida e Profile simples persistido no SQLite.

# Arquivos

- `software/crates/core/migrations/002_workspace_profile.sql`
- `software/crates/core/src/db.rs`
- `software/crates/core/src/store.rs`
- `software/crates/core/src/tool/`
- `software/crates/core/src/workspace/`
- `software/crates/core/src/profile/`
- `software/crates/core/src/lib.rs`
- `software/crates/ui/ui/app.slint`
- `software/crates/app/src/main.rs`
- `docs/product/implementation-order.md`
- `software/README.md`
- `changes/change_v9.md`

# Motivo

Entregar a base do produto: organizar um Workspace, listar/abrir ferramentas, executar ações no workdir global e reutilizar um Profile.

# Impacto

A app persiste workdir e perfis no SQLite (schema v2). Fase 3 ainda pode expandir histórico e versionamento de manifests.
