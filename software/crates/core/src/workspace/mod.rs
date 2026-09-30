//! Workspace domain model (references tools/profiles; does not copy manifests).

use std::path::PathBuf;

use crate::ids::slug_id;

/// Id do Workspace padrão. É criado na primeira abertura e não pode ser removido.
pub const DEFAULT_WORKSPACE_ID: &str = "default";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub global_workdir: PathBuf,
    pub tool_ids: Vec<String>,
    pub profile_ids: Vec<String>,
    /// Ferramentas marcadas como favoritas neste Workspace.
    pub favorite_tool_ids: Vec<String>,
}

impl Workspace {
    /// Workspace padrão, criado automaticamente com o diretório informado.
    pub fn default_new(global_workdir: PathBuf) -> Self {
        Self {
            id: DEFAULT_WORKSPACE_ID.to_string(),
            name: "Padrão".to_string(),
            global_workdir,
            tool_ids: Vec::new(),
            profile_ids: Vec::new(),
            favorite_tool_ids: Vec::new(),
        }
    }

    pub fn new_id(name: &str) -> String {
        slug_id("workspace", name)
    }

    pub fn is_default(&self) -> bool {
        self.id == DEFAULT_WORKSPACE_ID
    }
}
