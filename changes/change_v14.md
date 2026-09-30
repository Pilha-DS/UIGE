# Mudança

Fase 4, item 18: descoberta e importação de manifests. A UI lista os `*.json` de uma pasta, mostra quais são válidos e importa o escolhido para a biblioteca de ferramentas.

# Arquivos

- `software/crates/core/src/manifest/discovery.rs` (novo)
- `software/crates/core/src/manifest/mod.rs`
- `software/crates/core/src/tool/mod.rs`
- `software/crates/core/src/store.rs`
- `software/crates/core/src/lib.rs`
- `software/crates/core/tests/manifest_import.rs` (novo)
- `software/crates/ui/ui/app.slint`
- `software/crates/app/src/main.rs`
- `software/README.md`
- `docs/product/features.md`
- `docs/product/implementation-order.md`
- `patterns/manifests/README.md`
- `changes/change_v14.md`

# Motivo

Até aqui a biblioteca de ferramentas vinha apenas de manifests embutidos no código. A Fase 4 abre a evolução: o usuário passa a trazer definições de fora sem editar o app.

# Impacto

Core:

- `discover_manifests(dir)` inspeciona `*.json` (não recursivo) e devolve `ManifestCandidate` com `label()`/`detail()`; pasta ausente devolve lista vazia em vez de erro. A descoberta não grava nada.
- `Tool` ganhou `bundled: bool` e `ToolLibrary::insert` permite registrar/substituir uma definição.
- `Store::import_manifest` / `import_manifest_file` validam, registram a versão (append-only), adicionam ao Workspace padrão e devolvem `ImportOutcome` (`version`, `created_version`).
- `Store::open` passa a recarregar definições importadas do banco, então a importação sobrevive ao reinício. Entradas ilegíveis no banco são ignoradas para não impedir o app de abrir.
- Importação com `id` já embutido no app é recusada (`StoreError::BundledToolConflict`): evita que o conteúdo do banco divirja do código a cada abertura.
- `sync_bundled_manifest_versions` → `sync_manifest_versions` (agora cobre também as importadas).

UI/app:

- Nova seção **Manifests**: pasta, botão “Procurar”, lista de candidatos com o motivo quando inválidos, detalhe do selecionado e “Importar selecionado”.
- A home passou a ficar dentro de um `ScrollView`, para a seção nova não esticar a janela além do monitor.
- Pasta padrão: `manifests/` dentro da raiz de dados do app (mesma do banco). O campo é editável.
- `bind_tools` virou `refresh_tools`, que preserva ou define a seleção — após importar, a ferramenta nova fica selecionada.
