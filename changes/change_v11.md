# Mudança

Fase 3: persistência ampliada no SQLite — tools + versionamento append-only de manifests e histórico básico de execuções na UI.

# Arquivos

- `software/crates/core/migrations/003_history_manifest_versions.sql`
- `software/crates/core/src/db.rs`
- `software/crates/core/src/history.rs`
- `software/crates/core/src/store.rs`
- `software/crates/core/src/lib.rs`
- `software/crates/ui/ui/app.slint`
- `software/crates/app/src/main.rs`
- `docs/product/implementation-order.md`
- `software/README.md`
- `changes/change_v11.md`

# Motivo

Fechar a base local-first: estado (workspace/perfis/tools) sobrevive ao reinício, manifests têm histórico de versões e execuções ficam consultáveis.

# Impacto

Schema SQLite sobe para v3. Nova versão de manifest só é criada quando o conteúdo muda; versões antigas não são sobrescritas.
