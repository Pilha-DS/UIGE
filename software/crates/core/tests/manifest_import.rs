//! Fatia vertical da Fase 4: descobrir um manifest em disco, importar,
//! reabrir o store e executar o comando importado.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use uige_core::{discover_manifests, execute_command, ExecutionContext, ExecutionStatus, Store};

/// Manifest de teste que usa um executável realmente disponível (`git`).
const MANIFEST: &str = r#"{
    "schemaVersion": "v1",
    "id": "git-importado",
    "name": "Git importado",
    "executable": "git",
    "commands": [
        {
            "id": "version",
            "name": "Versão",
            "argvPrefix": ["--version"],
            "parameters": []
        }
    ]
}"#;

fn temp_dir(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("uige-{label}-{stamp}"));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

#[tokio::test]
async fn discovers_imports_persists_and_executes_a_manifest() {
    let dir = temp_dir("discovery");
    let db_path = temp_dir("import-db").join("uige.sqlite");
    std::fs::write(dir.join("git-importado.v1.json"), MANIFEST).expect("write manifest");
    std::fs::write(dir.join("lixo.json"), "{ nao e json }").expect("write invalid file");

    // 1. Descoberta: um válido, um inválido, ambos visíveis ao usuário.
    let candidates = discover_manifests(&dir);
    assert_eq!(candidates.len(), 2);
    assert_eq!(candidates[0].file_name(), "git-importado.v1.json");
    assert!(candidates[0].is_valid());
    assert_eq!(candidates[0].id(), Some("git-importado"));
    assert!(!candidates[1].is_valid());
    assert!(candidates[1].error.is_some());

    let valid_path = candidates[0].path.clone();

    // 2. Importação.
    let mut store = Store::open(&db_path, PathBuf::from(".")).expect("open store");
    let outcome = store
        .import_manifest_file(&valid_path)
        .expect("import manifest");
    assert_eq!(outcome.tool_id, "git-importado");
    assert_eq!(outcome.name, "Git importado");
    assert_eq!(outcome.version, 1);
    assert!(outcome.created_version);

    // 3. A ferramenta entra na biblioteca e no workspace padrão.
    let workspace = store.default_workspace().expect("workspace");
    assert!(workspace.tool_ids.contains(&"git-importado".to_string()));

    // 4. Executa o comando importado.
    let tool = store.library().get("git-importado").expect("imported tool");
    let command = tool.manifest.commands.first().expect("command");
    let context = ExecutionContext { workdir: None };
    let result = execute_command(&tool.manifest, command, &Default::default(), &context)
        .await
        .expect("execute imported command");
    assert_eq!(result.status, ExecutionStatus::Success);
    assert!(
        result.stdout.to_lowercase().contains("git version"),
        "saída inesperada: {}",
        result.stdout
    );

    // 5. Reabrir o app preserva a definição importada sem criar nova versão.
    drop(store);
    let store = Store::open(&db_path, PathBuf::from(".")).expect("reopen store");
    let tool = store.library().get("git-importado").expect("tool persisted");
    assert_eq!(tool.name, "Git importado");
    assert!(!tool.bundled, "definição importada, não embutida");
    assert_eq!(store.current_manifest_version("git-importado").unwrap(), 1);
    assert_eq!(
        store
            .list_manifest_versions("git-importado")
            .expect("versions")
            .len(),
        1
    );

    // Ferramentas embutidas continuam intactas.
    assert!(store.library().get("git").expect("bundled git").bundled);
    assert!(store.library().get("echo").is_some());

    drop(store);
    cleanup(&dir);
    cleanup(db_path.parent().expect("db dir"));
}

/// Manifest embutido não pode ser substituído por importação.
#[test]
fn bundled_definition_cannot_be_replaced_by_import() {
    let dir = temp_dir("bundled-conflict");
    let db_path = dir.join("uige.sqlite");

    let mut store = Store::open(&db_path, PathBuf::from(".")).expect("open store");
    let bundled_git = store
        .library()
        .get("git")
        .expect("bundled git")
        .manifest
        .clone();
    let json = serde_json::to_string(&bundled_git).expect("serialize");

    let err = store.import_manifest(&json).expect_err("must refuse");
    assert!(err.to_string().contains("embutida"), "erro inesperado: {err}");

    drop(store);
    cleanup(&dir);
}

fn cleanup(dir: &Path) {
    let _ = std::fs::remove_dir_all(dir);
}
