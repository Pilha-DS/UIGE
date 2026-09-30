//! Tool library: available tools backed by manifests (no duplication per workspace).

use crate::manifest::{bundled_manifests, Manifest, ManifestError};

#[derive(Debug, Clone)]
pub struct Tool {
    pub id: String,
    pub name: String,
    pub manifest: Manifest,
    /// `true` quando a definição vem embutida no app, em vez de importada.
    pub bundled: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ToolLibrary {
    tools: Vec<Tool>,
}

impl ToolLibrary {
    pub fn from_bundled() -> Result<Self, ManifestError> {
        let manifests = bundled_manifests()?;
        let tools = manifests
            .into_iter()
            .map(|manifest| Tool {
                id: manifest.id.clone(),
                name: manifest.name.clone(),
                manifest,
                bundled: true,
            })
            .collect();
        Ok(Self { tools })
    }

    pub fn tools(&self) -> &[Tool] {
        &self.tools
    }

    pub fn get(&self, tool_id: &str) -> Option<&Tool> {
        self.tools.iter().find(|tool| tool.id == tool_id)
    }

    pub fn index_of(&self, tool_id: &str) -> Option<usize> {
        self.tools.iter().position(|tool| tool.id == tool_id)
    }

    /// Insere ou substitui uma definição pelo `id`, devolvendo a anterior.
    ///
    /// O histórico de versões é do `Store`; aqui a biblioteca apenas reflete a
    /// definição mais recente.
    pub fn insert(&mut self, manifest: Manifest, bundled: bool) -> Option<Tool> {
        let tool = Tool {
            id: manifest.id.clone(),
            name: manifest.name.clone(),
            manifest,
            bundled,
        };

        match self.tools.iter_mut().find(|existing| existing.id == tool.id) {
            Some(slot) => Some(std::mem::replace(slot, tool)),
            None => {
                self.tools.push(tool);
                None
            }
        }
    }
}
