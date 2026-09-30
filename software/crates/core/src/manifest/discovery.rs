//! Descoberta de manifests em disco (Fase 4: importação de definições).
//!
//! A descoberta é somente leitura: inspeciona arquivos `*.json` e diz quais
//! podem ser usados. Nada é gravado aqui — a importação é responsabilidade do
//! `Store`.

use std::path::{Path, PathBuf};

use super::{load_manifest, Manifest};

/// Um arquivo inspecionado durante a descoberta.
///
/// Ou o arquivo é um manifest válido (`manifest`), ou traz o motivo da recusa
/// (`error`). Nunca os dois.
#[derive(Debug, Clone)]
pub struct ManifestCandidate {
    pub path: PathBuf,
    pub manifest: Option<Manifest>,
    pub error: Option<String>,
}

impl ManifestCandidate {
    pub fn is_valid(&self) -> bool {
        self.manifest.is_some()
    }

    pub fn id(&self) -> Option<&str> {
        self.manifest.as_ref().map(|manifest| manifest.id.as_str())
    }

    pub fn command_count(&self) -> usize {
        self.manifest
            .as_ref()
            .map(|manifest| manifest.commands.len())
            .unwrap_or(0)
    }

    /// Nome do arquivo, com fallback para o caminho completo.
    pub fn file_name(&self) -> String {
        self.path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| self.path.display().to_string())
    }

    /// Texto curto para listas de seleção.
    pub fn label(&self) -> String {
        match &self.manifest {
            Some(manifest) => format!(
                "{} ({}) · {} comando(s)",
                manifest.name,
                manifest.id,
                manifest.commands.len()
            ),
            None => format!("{} · inválido", self.file_name()),
        }
    }

    /// Descrição do que foi encontrado, sem depender do estado do app.
    pub fn detail(&self) -> String {
        match (&self.manifest, &self.error) {
            (Some(manifest), _) => format!(
                "válido · {} · executável `{}` · {} comando(s)",
                self.file_name(),
                manifest.executable,
                manifest.commands.len()
            ),
            (None, Some(error)) => format!("inválido · {} · {error}", self.file_name()),
            (None, None) => format!("inválido · {}", self.file_name()),
        }
    }
}

/// Lista os arquivos `*.json` de `dir` como candidatos a manifest.
///
/// Pasta inexistente resulta em lista vazia: a descoberta não deve falhar por
/// ausência da pasta padrão. A inspeção não é recursiva.
pub fn discover_manifests(dir: &Path) -> Vec<ManifestCandidate> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };

    let mut candidates: Vec<ManifestCandidate> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && has_json_extension(path))
        .map(|path| inspect(&path))
        .collect();

    candidates.sort_by(|a, b| a.path.cmp(&b.path));
    candidates
}

fn has_json_extension(path: &Path) -> bool {
    path.extension()
        .map(|ext| ext.eq_ignore_ascii_case("json"))
        .unwrap_or(false)
}

fn inspect(path: &Path) -> ManifestCandidate {
    match load_manifest(path) {
        Ok(manifest) => ManifestCandidate {
            path: path.to_path_buf(),
            manifest: Some(manifest),
            error: None,
        },
        Err(error) => ManifestCandidate {
            path: path.to_path_buf(),
            manifest: None,
            error: Some(error.to_string()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir() -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("uige-discovery-test-{stamp}"));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn discovers_valid_and_invalid_json_files_only() {
        let dir = temp_dir();

        std::fs::write(
            dir.join("good.json"),
            r#"{
                "schemaVersion": "v1",
                "id": "demo",
                "name": "Demo",
                "executable": "echo",
                "commands": [{"id": "a", "name": "A", "parameters": []}]
            }"#,
        )
        .expect("write valid manifest");
        std::fs::write(dir.join("broken.json"), "{ not json").expect("write invalid manifest");
        std::fs::write(dir.join("notes.txt"), "ignorado").expect("write non-json file");

        let candidates = discover_manifests(&dir);

        assert_eq!(candidates.len(), 2, "apenas arquivos .json entram na lista");

        let broken = &candidates[0];
        assert_eq!(broken.file_name(), "broken.json");
        assert!(!broken.is_valid());
        assert!(broken.label().contains("inválido"));
        assert!(broken.detail().starts_with("inválido · broken.json"));

        let good = &candidates[1];
        assert!(good.is_valid());
        assert_eq!(good.id(), Some("demo"));
        assert_eq!(good.command_count(), 1);
        assert_eq!(good.label(), "Demo (demo) · 1 comando(s)");
        assert!(good.detail().contains("executável `echo`"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_directory_is_not_an_error() {
        let dir = std::env::temp_dir().join("uige-discovery-missing-does-not-exist");
        let _ = std::fs::remove_dir_all(&dir);

        assert!(discover_manifests(&dir).is_empty());
    }
}
