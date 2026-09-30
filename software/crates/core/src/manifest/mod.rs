//! Load and validate manifests from JSON.

mod discovery;
mod model;

pub use discovery::{discover_manifests, ManifestCandidate};
pub use model::{Command, Manifest, Parameter, ParameterType};

use std::path::Path;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("failed to read manifest at {path}: {source}")]
    Io {
        path: String,
        source: std::io::Error,
    },
    #[error("invalid manifest JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unsupported schemaVersion `{0}` (expected v1)")]
    UnsupportedSchema(String),
    #[error("manifest must declare a non-empty executable")]
    EmptyExecutable,
    #[error("manifest must declare at least one command")]
    NoCommands,
    #[error("command `{0}` not found in manifest")]
    CommandNotFound(String),
}

/// Parse a manifest from a JSON string.
pub fn parse_manifest(json: &str) -> Result<Manifest, ManifestError> {
    let manifest: Manifest = serde_json::from_str(json)?;
    validate_manifest(&manifest)?;
    Ok(manifest)
}

/// Load a manifest from a file path.
pub fn load_manifest(path: &Path) -> Result<Manifest, ManifestError> {
    let json = std::fs::read_to_string(path).map_err(|source| ManifestError::Io {
        path: path.display().to_string(),
        source,
    })?;
    parse_manifest(&json)
}

/// Bundled Phase 1 demo (platform-appropriate echo).
pub fn bundled_demo_manifest() -> Result<Manifest, ManifestError> {
    let json = if cfg!(windows) {
        include_str!("../../examples/echo.windows.v1.json")
    } else {
        include_str!("../../examples/echo.v1.json")
    };
    parse_manifest(json)
}

/// Bundled Git manifest (works on Windows and Linux when `git` is on PATH).
pub fn bundled_git_manifest() -> Result<Manifest, ManifestError> {
    parse_manifest(include_str!("../../examples/git.v1.json"))
}

/// All bundled demo manifests available in the Phase 1 UI.
pub fn bundled_manifests() -> Result<Vec<Manifest>, ManifestError> {
    Ok(vec![bundled_demo_manifest()?, bundled_git_manifest()?])
}

fn validate_manifest(manifest: &Manifest) -> Result<(), ManifestError> {
    if manifest.schema_version != "v1" {
        return Err(ManifestError::UnsupportedSchema(
            manifest.schema_version.clone(),
        ));
    }
    if manifest.executable.trim().is_empty() {
        return Err(ManifestError::EmptyExecutable);
    }
    if manifest.commands.is_empty() {
        return Err(ManifestError::NoCommands);
    }
    Ok(())
}

pub fn find_command<'a>(
    manifest: &'a Manifest,
    command_id: &str,
) -> Result<&'a Command, ManifestError> {
    manifest
        .commands
        .iter()
        .find(|c| c.id == command_id)
        .ok_or_else(|| ManifestError::CommandNotFound(command_id.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bundled_demo() {
        let manifest = bundled_demo_manifest().expect("bundled demo");
        assert_eq!(manifest.id, "echo");
        assert_eq!(manifest.schema_version, "v1");
        assert!(!manifest.commands.is_empty());
    }

    #[test]
    fn parses_bundled_git() {
        let manifest = bundled_git_manifest().expect("bundled git");
        assert_eq!(manifest.id, "git");
        assert_eq!(manifest.executable, "git");
        assert!(manifest.commands.iter().any(|c| c.id == "version"));
    }

    #[test]
    fn bundled_manifests_include_echo_and_git() {
        let manifests = bundled_manifests().expect("bundled manifests");
        let ids: Vec<_> = manifests.iter().map(|m| m.id.as_str()).collect();
        assert!(ids.contains(&"echo"));
        assert!(ids.contains(&"git"));
    }

    #[test]
    fn rejects_unknown_schema() {
        let json = r#"{
            "schemaVersion": "v99",
            "id": "x",
            "name": "X",
            "executable": "echo",
            "commands": [{"id": "a", "name": "A", "parameters": []}]
        }"#;
        let err = parse_manifest(json).unwrap_err();
        assert!(matches!(err, ManifestError::UnsupportedSchema(_)));
    }
}
