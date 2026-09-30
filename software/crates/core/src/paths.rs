//! Navegação de diretórios para a seleção de Workdir.
//!
//! Somente leitura: lista o que existe em um caminho. Nada é criado, alterado ou
//! removido aqui — escolher o Workdir é uma preferência, gravada pelo `Store`.

use std::path::{Path, PathBuf};

/// Subdiretórios diretos de `path`, ordenados por nome (sem diferenciar caixa).
///
/// Arquivos ficam de fora: a listagem é para navegação entre pastas. Entradas
/// ilegíveis são ignoradas e caminho inexistente resulta em lista vazia — uma
/// pasta sem permissão não deve interromper a escolha do Workdir.
pub fn list_subdirectories(path: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(path) else {
        return Vec::new();
    };

    let mut directories: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect();

    directories.sort_by_cached_key(|path| {
        path.file_name()
            .map(|name| name.to_string_lossy().to_lowercase())
            .unwrap_or_default()
    });
    directories
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
        let dir = std::env::temp_dir().join(format!("uige-paths-test-{stamp}"));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn lists_only_subdirectories_sorted_by_name() {
        let dir = temp_dir();
        std::fs::create_dir_all(dir.join("zulu")).expect("create dir");
        std::fs::create_dir_all(dir.join("Alfa")).expect("create dir");
        std::fs::create_dir_all(dir.join("meio")).expect("create dir");
        std::fs::write(dir.join("arquivo.txt"), "não é pasta").expect("write file");

        let found = list_subdirectories(&dir);
        let names: Vec<String> = found
            .iter()
            .map(|path| {
                path.file_name()
                    .expect("nome")
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();

        // Ordem sem diferenciar caixa: Alfa antes de meio, que vem antes de zulu.
        // O arquivo não entra.
        assert_eq!(names, vec!["Alfa", "meio", "zulu"]);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_directory_is_not_an_error() {
        let dir = std::env::temp_dir().join("uige-paths-test-nao-existe");

        assert!(list_subdirectories(&dir).is_empty());
    }
}
