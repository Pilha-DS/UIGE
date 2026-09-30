//! Local app store: SQLite persistence for workspace, profiles, manifests and history.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, OptionalExtension};
use thiserror::Error;

use crate::db::{init_database, DatabaseError};
use crate::execution::{ExecutionOutcome, ExecutionStatus};
use crate::history::{ExecutionRecord, ManifestVersionInfo, WorkflowExecutionRecord, WorkflowStepRecord};
use crate::manifest::{load_manifest, parse_manifest, Manifest, ManifestError};
use crate::profile::Profile;
use crate::tool::{Tool, ToolLibrary};
use crate::workflow::{StepFailurePolicy, StepStatus, Workflow, WorkflowError, WorkflowOutcome};
use crate::workspace::{Workspace, DEFAULT_WORKSPACE_ID};

#[derive(Debug, Error)]
pub enum StoreError {
    #[error(transparent)]
    Database(#[from] DatabaseError),
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Manifest(#[from] ManifestError),
    #[error("workspace `{0}` not found")]
    WorkspaceNotFound(String),
    #[error("informe um nome para o workspace")]
    EmptyWorkspaceName,
    #[error("já existe um workspace chamado `{0}`")]
    WorkspaceNameTaken(String),
    #[error("o workspace padrão não pode ser removido")]
    DefaultWorkspaceProtected,
    #[error("profile `{0}` not found")]
    ProfileNotFound(String),
    #[error("já existe um perfil chamado `{0}` no workspace")]
    ProfileNameTaken(String),
    #[error("informe um nome para o perfil")]
    EmptyProfileName,
    #[error("tool `{0}` not found in library")]
    ToolNotFound(String),
    #[error("`{0}` é uma definição embutida no app e não pode ser substituída por importação")]
    BundledToolConflict(String),
    #[error("workflow `{0}` not found")]
    WorkflowNotFound(String),
    #[error("já existe um workflow chamado `{0}`")]
    WorkflowNameTaken(String),
    #[error("informe um nome para o workflow")]
    EmptyWorkflowName,
    #[error(transparent)]
    Workflow(#[from] WorkflowError),
    #[error("execution `{0}` not found")]
    ExecutionNotFound(String),
    #[error("invalid JSON: {0}")]
    Json(#[from] serde_json::Error),
}

/// Resultado de uma importação de manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportOutcome {
    pub tool_id: String,
    pub name: String,
    /// Versão vigente da definição após a importação.
    pub version: i64,
    /// `false` quando o conteúdo importado já era o registrado (nada mudou).
    pub created_version: bool,
}

pub struct Store {
    conn: Connection,
    library: ToolLibrary,
}

impl Store {
    pub fn open(path: &Path, default_workdir: PathBuf) -> Result<Self, StoreError> {
        let conn = init_database(path)?;
        let mut library = ToolLibrary::from_bundled()?;
        for manifest in load_persisted_manifests(&conn)? {
            // Definições embutidas têm precedência; importação não as substitui.
            if library.get(&manifest.id).is_none() {
                library.insert(manifest, false);
            }
        }

        let store = Self { conn, library };
        store.ensure_default_workspace(default_workdir)?;
        store.sync_manifest_versions()?;
        store.sync_workspace_tools()?;
        Ok(store)
    }

    pub fn library(&self) -> &ToolLibrary {
        &self.library
    }

    pub fn tools(&self) -> &[Tool] {
        self.library.tools()
    }

    /// Workspace padrão, criado na primeira abertura e não removível.
    pub fn default_workspace(&self) -> Result<Workspace, StoreError> {
        self.load_workspace(DEFAULT_WORKSPACE_ID)
    }

    /// Workspaces existentes: o padrão primeiro, os demais por nome.
    pub fn list_workspaces(&self) -> Result<Vec<Workspace>, StoreError> {
        let ids: Vec<String> = {
            let mut stmt = self.conn.prepare(
                "SELECT id FROM workspaces ORDER BY is_default DESC, name COLLATE NOCASE",
            )?;
            let rows = stmt.query_map([], |row| row.get(0))?;
            // Coletar antes do fim do bloco: `stmt` é liberado antes dos
            // temporários da expressão final, e as linhas o emprestam.
            rows.collect::<Result<Vec<String>, _>>()?
        };

        ids.iter().map(|id| self.load_workspace(id)).collect()
    }

    /// Workspace ativo: o último selecionado pelo usuário.
    ///
    /// Cai no padrão quando a chave não existe ou aponta para um Workspace que
    /// já não está no banco — um registro órfão não deve impedir o app de abrir.
    pub fn active_workspace(&self) -> Result<Workspace, StoreError> {
        let id: Option<String> = self
            .conn
            .query_row(
                "SELECT value FROM app_meta WHERE key = 'active_workspace'",
                [],
                |row| row.get(0),
            )
            .optional()?;

        match id {
            Some(id) if self.workspace_exists(&id)? => self.load_workspace(&id),
            _ => self.default_workspace(),
        }
    }

    pub fn set_active_workspace(&self, workspace_id: &str) -> Result<(), StoreError> {
        if !self.workspace_exists(workspace_id)? {
            return Err(StoreError::WorkspaceNotFound(workspace_id.to_string()));
        }
        self.conn.execute(
            "INSERT INTO app_meta (key, value) VALUES ('active_workspace', ?1)
             ON CONFLICT(key) DO UPDATE SET value = ?1",
            params![workspace_id],
        )?;
        Ok(())
    }

    /// Cria ou atualiza um Workspace.
    ///
    /// O nome é único sem diferenciar maiúsculas/minúsculas, como o de Profile:
    /// recusar é melhor que gerar duas entradas visualmente iguais.
    pub fn save_workspace(&self, workspace: &Workspace) -> Result<(), StoreError> {
        let name = workspace.name.trim();
        if name.is_empty() {
            return Err(StoreError::EmptyWorkspaceName);
        }

        let taken = self.workspace_name_conflict(name)?;
        if let Some(existing_id) = taken {
            if existing_id != workspace.id {
                return Err(StoreError::WorkspaceNameTaken(name.to_string()));
            }
        }

        let exists = self.workspace_exists(&workspace.id)?;
        if exists {
            self.conn.execute(
                "UPDATE workspaces SET name = ?1, global_workdir = ?2 WHERE id = ?3",
                params![
                    name,
                    workspace.global_workdir.display().to_string(),
                    workspace.id
                ],
            )?;
            return Ok(());
        }

        self.conn.execute(
            "INSERT INTO workspaces (id, name, global_workdir, is_default) VALUES (?1, ?2, ?3, 0)",
            params![
                workspace.id,
                name,
                workspace.global_workdir.display().to_string()
            ],
        )?;

        // Um Workspace novo nasce com as ferramentas disponíveis, para ser útil
        // de imediato; a curadoria (remover) é feita depois, em Ferramentas.
        for tool in self.library.tools() {
            self.add_tool_to_workspace(&workspace.id, &tool.id)?;
        }

        Ok(())
    }

    /// Remove um Workspace. O padrão é protegido.
    pub fn delete_workspace(&self, workspace_id: &str) -> Result<(), StoreError> {
        if workspace_id == DEFAULT_WORKSPACE_ID {
            return Err(StoreError::DefaultWorkspaceProtected);
        }
        if !self.workspace_exists(workspace_id)? {
            return Err(StoreError::WorkspaceNotFound(workspace_id.to_string()));
        }

        self.conn
            .execute("DELETE FROM workspaces WHERE id = ?1", params![workspace_id])?;

        // Se o removido era o ativo, voltar para o padrão.
        let active: Option<String> = self
            .conn
            .query_row(
                "SELECT value FROM app_meta WHERE key = 'active_workspace'",
                [],
                |row| row.get(0),
            )
            .optional()?;
        if active.as_deref() == Some(workspace_id) {
            self.set_active_workspace(DEFAULT_WORKSPACE_ID)?;
        }

        Ok(())
    }

    /// Adiciona a ferramenta às referências do Workspace (sem duplicar).
    pub fn add_tool_to_workspace(
        &self,
        workspace_id: &str,
        tool_id: &str,
    ) -> Result<(), StoreError> {
        if !self.workspace_exists(workspace_id)? {
            return Err(StoreError::WorkspaceNotFound(workspace_id.to_string()));
        }
        if self.library.get(tool_id).is_none() {
            return Err(StoreError::ToolNotFound(tool_id.to_string()));
        }

        self.conn.execute(
            "INSERT OR IGNORE INTO workspace_tools (workspace_id, tool_id) VALUES (?1, ?2)",
            params![workspace_id, tool_id],
        )?;
        Ok(())
    }

    /// Remove a ferramenta deste Workspace. A ferramenta continua disponível.
    pub fn remove_tool_from_workspace(
        &self,
        workspace_id: &str,
        tool_id: &str,
    ) -> Result<(), StoreError> {
        self.conn.execute(
            "DELETE FROM workspace_tools WHERE workspace_id = ?1 AND tool_id = ?2",
            params![workspace_id, tool_id],
        )?;
        // Favorito de ferramenta que saiu do Workspace não faz sentido manter.
        self.set_tool_favorite(workspace_id, tool_id, false)?;
        Ok(())
    }

    /// Marca ou desmarca a ferramenta como favorita no Workspace.
    pub fn set_tool_favorite(
        &self,
        workspace_id: &str,
        tool_id: &str,
        favorite: bool,
    ) -> Result<(), StoreError> {
        if favorite {
            self.conn.execute(
                "INSERT OR IGNORE INTO workspace_favorites (workspace_id, tool_id) VALUES (?1, ?2)",
                params![workspace_id, tool_id],
            )?;
        } else {
            self.conn.execute(
                "DELETE FROM workspace_favorites WHERE workspace_id = ?1 AND tool_id = ?2",
                params![workspace_id, tool_id],
            )?;
        }
        Ok(())
    }

    fn workspace_exists(&self, workspace_id: &str) -> Result<bool, StoreError> {
        let found: Option<String> = self
            .conn
            .query_row(
                "SELECT id FROM workspaces WHERE id = ?1",
                params![workspace_id],
                |row| row.get(0),
            )
            .optional()?;
        Ok(found.is_some())
    }

    /// Id do Workspace que já usa este nome, se houver.
    fn workspace_name_conflict(&self, name: &str) -> Result<Option<String>, StoreError> {
        let mut stmt = self.conn.prepare("SELECT id, name FROM workspaces")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        for row in rows {
            let (id, existing) = row?;
            if names_match(&existing, name) {
                return Ok(Some(id));
            }
        }
        Ok(None)
    }

    /// Id do perfil que já usa este nome no Workspace, se houver.
    fn profile_name_conflict(
        &self,
        workspace_id: &str,
        name: &str,
    ) -> Result<Option<String>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT p.id, p.name
             FROM profiles p
             INNER JOIN workspace_profiles wp ON wp.profile_id = p.id
             WHERE wp.workspace_id = ?1",
        )?;
        let rows = stmt.query_map(params![workspace_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        for row in rows {
            let (id, existing) = row?;
            if names_match(&existing, name) {
                return Ok(Some(id));
            }
        }
        Ok(None)
    }

    pub fn set_global_workdir(&self, workspace_id: &str, workdir: &Path) -> Result<(), StoreError> {
        let updated = self.conn.execute(
            "UPDATE workspaces SET global_workdir = ?1 WHERE id = ?2",
            params![workdir.display().to_string(), workspace_id],
        )?;
        if updated == 0 {
            return Err(StoreError::WorkspaceNotFound(workspace_id.to_string()));
        }
        Ok(())
    }

    pub fn current_manifest_version(&self, tool_id: &str) -> Result<i64, StoreError> {
        let version: Option<i64> = self
            .conn
            .query_row(
                "SELECT current_version FROM tools WHERE id = ?1",
                params![tool_id],
                |row| row.get(0),
            )
            .optional()?;
        Ok(version.unwrap_or(0))
    }

    pub fn list_manifest_versions(
        &self,
        tool_id: &str,
    ) -> Result<Vec<ManifestVersionInfo>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT tool_id, version, content_hash, created_at
             FROM manifest_versions
             WHERE tool_id = ?1
             ORDER BY version DESC",
        )?;
        let rows = stmt.query_map(params![tool_id], |row| {
            Ok(ManifestVersionInfo {
                tool_id: row.get(0)?,
                version: row.get(1)?,
                content_hash: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(StoreError::from)
    }

    pub fn record_execution(
        &self,
        workspace_id: &str,
        tool_id: &str,
        command_id: &str,
        parameters: &HashMap<String, String>,
        workdir: Option<&Path>,
        outcome: &ExecutionOutcome,
    ) -> Result<ExecutionRecord, StoreError> {
        let record = ExecutionRecord {
            id: ExecutionRecord::new_id(),
            workspace_id: workspace_id.to_string(),
            tool_id: tool_id.to_string(),
            command_id: command_id.to_string(),
            parameters: parameters.clone(),
            workdir: workdir.map(Path::to_path_buf),
            status: outcome.status,
            exit_code: outcome.exit_code,
            stdout: outcome.stdout.clone(),
            stderr: outcome.stderr.clone(),
            finished_at: String::new(),
        };

        let parameters_json = serde_json::to_string(&record.parameters)?;
        let status = record.status.as_str();
        let workdir = record
            .workdir
            .as_ref()
            .map(|p| p.display().to_string());

        self.conn.execute(
            "INSERT INTO execution_history (
                id, workspace_id, tool_id, command_id, parameters_json, workdir,
                status, exit_code, stdout, stderr, finished_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, datetime('now'))",
            params![
                record.id,
                record.workspace_id,
                record.tool_id,
                record.command_id,
                parameters_json,
                workdir,
                status,
                record.exit_code,
                record.stdout,
                record.stderr
            ],
        )?;

        let finished_at: String = self.conn.query_row(
            "SELECT finished_at FROM execution_history WHERE id = ?1",
            params![record.id],
            |row| row.get(0),
        )?;

        Ok(ExecutionRecord {
            finished_at,
            ..record
        })
    }

    pub fn list_recent_executions(
        &self,
        workspace_id: &str,
        limit: usize,
    ) -> Result<Vec<ExecutionRecord>, StoreError> {
        self.list_executions(workspace_id, None, limit)
    }

    /// Execuções recentes de uma ferramenta específica.
    ///
    /// A tela da ferramenta mostra só o histórico dela; listar tudo e filtrar no
    /// cliente perderia execuções da ferramenta que ficaram fora do limite.
    pub fn list_recent_executions_for_tool(
        &self,
        workspace_id: &str,
        tool_id: &str,
        limit: usize,
    ) -> Result<Vec<ExecutionRecord>, StoreError> {
        self.list_executions(workspace_id, Some(tool_id), limit)
    }

    fn list_executions(
        &self,
        workspace_id: &str,
        tool_id: Option<&str>,
        limit: usize,
    ) -> Result<Vec<ExecutionRecord>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, workspace_id, tool_id, command_id, parameters_json, workdir,
                    status, exit_code, stdout, stderr, finished_at
             FROM execution_history
             WHERE workspace_id = ?1 AND (?2 IS NULL OR tool_id = ?2)
             ORDER BY finished_at DESC, id DESC
             LIMIT ?3",
        )?;

        let rows = stmt.query_map(params![workspace_id, tool_id, limit as i64], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, Option<i32>>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, String>(9)?,
                row.get::<_, String>(10)?,
            ))
        })?;

        let mut records = Vec::new();
        for row in rows {
            let (
                id,
                workspace_id,
                tool_id,
                command_id,
                parameters_json,
                workdir,
                status,
                exit_code,
                stdout,
                stderr,
                finished_at,
            ) = row?;
            records.push(ExecutionRecord {
                id,
                workspace_id,
                tool_id,
                command_id,
                parameters: serde_json::from_str(&parameters_json)?,
                workdir: workdir.filter(|s| !s.is_empty()).map(PathBuf::from),
                status: parse_status(&status),
                exit_code,
                stdout,
                stderr,
                finished_at,
            });
        }
        Ok(records)
    }

    pub fn get_execution(&self, execution_id: &str) -> Result<ExecutionRecord, StoreError> {
        let row = self
            .conn
            .query_row(
                "SELECT id, workspace_id, tool_id, command_id, parameters_json, workdir,
                        status, exit_code, stdout, stderr, finished_at
                 FROM execution_history WHERE id = ?1",
                params![execution_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, Option<String>>(5)?,
                        row.get::<_, String>(6)?,
                        row.get::<_, Option<i32>>(7)?,
                        row.get::<_, String>(8)?,
                        row.get::<_, String>(9)?,
                        row.get::<_, String>(10)?,
                    ))
                },
            )
            .optional()?;

        let Some((
            id,
            workspace_id,
            tool_id,
            command_id,
            parameters_json,
            workdir,
            status,
            exit_code,
            stdout,
            stderr,
            finished_at,
        )) = row
        else {
            return Err(StoreError::ExecutionNotFound(execution_id.to_string()));
        };

        Ok(ExecutionRecord {
            id,
            workspace_id,
            tool_id,
            command_id,
            parameters: serde_json::from_str(&parameters_json)?,
            workdir: workdir.filter(|s| !s.is_empty()).map(PathBuf::from),
            status: parse_status(&status),
            exit_code,
            stdout,
            stderr,
            finished_at,
        })
    }

    pub fn list_profiles(&self, workspace_id: &str) -> Result<Vec<Profile>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT p.id, p.name, p.tool_id, p.command_id, p.parameters_json, p.workdir
             FROM profiles p
             INNER JOIN workspace_profiles wp ON wp.profile_id = p.id
             WHERE wp.workspace_id = ?1
             ORDER BY p.name COLLATE NOCASE",
        )?;

        let rows = stmt.query_map(params![workspace_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Option<String>>(5)?,
            ))
        })?;

        let mut profiles = Vec::new();
        for row in rows {
            let (id, name, tool_id, command_id, parameters_json, workdir) = row?;
            profiles.push(Profile {
                id,
                name,
                tool_id,
                command_id,
                parameters: serde_json::from_str(&parameters_json)?,
                workdir: workdir.filter(|s| !s.is_empty()).map(PathBuf::from),
            });
        }
        Ok(profiles)
    }

    pub fn save_profile(&self, workspace_id: &str, profile: &Profile) -> Result<(), StoreError> {
        if self.library.get(&profile.tool_id).is_none() {
            return Err(StoreError::ToolNotFound(profile.tool_id.clone()));
        }

        // Nomes de perfil são únicos por workspace, exceto ao atualizar o próprio.
        let name = profile.name.trim();
        if name.is_empty() {
            return Err(StoreError::EmptyProfileName);
        }
        if let Some(existing_id) = self.profile_name_conflict(workspace_id, name)? {
            if existing_id != profile.id {
                return Err(StoreError::ProfileNameTaken(name.to_string()));
            }
        }

        let parameters_json = serde_json::to_string(&profile.parameters)?;
        let workdir = profile
            .workdir
            .as_ref()
            .map(|p| p.display().to_string());

        self.conn.execute(
            "INSERT INTO profiles (id, name, tool_id, command_id, parameters_json, workdir)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                tool_id = excluded.tool_id,
                command_id = excluded.command_id,
                parameters_json = excluded.parameters_json,
                workdir = excluded.workdir",
            params![
                profile.id,
                profile.name,
                profile.tool_id,
                profile.command_id,
                parameters_json,
                workdir
            ],
        )?;

        self.conn.execute(
            "INSERT OR IGNORE INTO workspace_profiles (workspace_id, profile_id) VALUES (?1, ?2)",
            params![workspace_id, profile.id],
        )?;

        Ok(())
    }

    pub fn get_profile(&self, profile_id: &str) -> Result<Profile, StoreError> {
        let row = self
            .conn
            .query_row(
                "SELECT id, name, tool_id, command_id, parameters_json, workdir
                 FROM profiles WHERE id = ?1",
                params![profile_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, Option<String>>(5)?,
                    ))
                },
            )
            .optional()?;

        let Some((id, name, tool_id, command_id, parameters_json, workdir)) = row else {
            return Err(StoreError::ProfileNotFound(profile_id.to_string()));
        };

        Ok(Profile {
            id,
            name,
            tool_id,
            command_id,
            parameters: serde_json::from_str(&parameters_json)?,
            workdir: workdir.filter(|s| !s.is_empty()).map(PathBuf::from),
        })
    }

    /// Importa um manifest a partir de JSON já carregado.
    pub fn import_manifest(&mut self, json: &str) -> Result<ImportOutcome, StoreError> {
        let manifest = parse_manifest(json)?;
        self.register_imported_manifest(manifest)
    }

    /// Importa um manifest a partir de um arquivo.
    pub fn import_manifest_file(&mut self, path: &Path) -> Result<ImportOutcome, StoreError> {
        let manifest = load_manifest(path)?;
        self.register_imported_manifest(manifest)
    }

    fn register_imported_manifest(
        &mut self,
        manifest: Manifest,
    ) -> Result<ImportOutcome, StoreError> {
        let tool_id = manifest.id.clone();

        if self
            .library
            .get(&tool_id)
            .map(|existing| existing.bundled)
            .unwrap_or(false)
        {
            return Err(StoreError::BundledToolConflict(tool_id));
        }

        let previous_version = self.current_manifest_version(&tool_id)?;

        self.library.insert(manifest, false);
        let tool = self
            .library
            .get(&tool_id)
            .expect("tool inserted immediately above");
        self.upsert_manifest_version(tool)?;

        self.conn.execute(
            "INSERT OR IGNORE INTO workspace_tools (workspace_id, tool_id) VALUES (?1, ?2)",
            params![DEFAULT_WORKSPACE_ID, tool_id],
        )?;

        let version = self.current_manifest_version(&tool_id)?;
        Ok(ImportOutcome {
            tool_id,
            name: tool.name.clone(),
            version,
            created_version: version > previous_version,
        })
    }

    pub fn save_workflow(&self, workflow: &Workflow) -> Result<(), StoreError> {
        workflow.validate()?;

        let name = workflow.name.trim();
        if name.is_empty() {
            return Err(StoreError::EmptyWorkflowName);
        }

        // Nome único (sem diferenciar caixa), exceto ao atualizar o próprio workflow.
        let taken_by: Option<String> = self
            .conn
            .query_row(
                "SELECT id FROM workflows WHERE name = ?1 COLLATE NOCASE LIMIT 1",
                params![name],
                |row| row.get(0),
            )
            .optional()?;
        if let Some(existing_id) = taken_by {
            if existing_id != workflow.id {
                return Err(StoreError::WorkflowNameTaken(name.to_string()));
            }
        }

        self.conn.execute(
            "INSERT INTO workflows (id, name, description) VALUES (?1, ?2, ?3)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                description = excluded.description",
            params![workflow.id, name, workflow.description],
        )?;

        // As etapas são a definição do workflow: substituir é mais simples (e mais
        // previsível) que tentar diferenciar altas, baixas e reordenações.
        self.conn.execute(
            "DELETE FROM workflow_steps WHERE workflow_id = ?1",
            params![workflow.id],
        )?;

        for (position, step) in workflow.steps.iter().enumerate() {
            let parameters_json = serde_json::to_string(&step.parameters)?;
            let workdir = step.workdir.as_ref().map(|path| path.display().to_string());
            self.conn.execute(
                "INSERT INTO workflow_steps (
                    workflow_id, position, step_id, tool_id, command_id,
                    parameters_json, workdir, on_error
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    workflow.id,
                    position as i64,
                    step.id,
                    step.tool_id,
                    step.command_id,
                    parameters_json,
                    workdir,
                    step.on_error.as_str()
                ],
            )?;
        }

        Ok(())
    }

    pub fn get_workflow(&self, workflow_id: &str) -> Result<Workflow, StoreError> {
        let row = self
            .conn
            .query_row(
                "SELECT id, name, description FROM workflows WHERE id = ?1",
                params![workflow_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<String>>(2)?,
                    ))
                },
            )
            .optional()?;

        let Some((id, name, description)) = row else {
            return Err(StoreError::WorkflowNotFound(workflow_id.to_string()));
        };

        Ok(Workflow {
            id,
            name,
            description,
            steps: self.load_workflow_steps(workflow_id)?,
        })
    }

    pub fn list_workflows(&self) -> Result<Vec<Workflow>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, description FROM workflows ORDER BY name COLLATE NOCASE")?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
            ))
        })?;

        let mut workflows = Vec::new();
        for row in rows {
            let (id, name, description) = row?;
            let steps = self.load_workflow_steps(&id)?;
            workflows.push(Workflow {
                id,
                name,
                description,
                steps,
            });
        }
        Ok(workflows)
    }

    pub fn delete_workflow(&self, workflow_id: &str) -> Result<(), StoreError> {
        let deleted = self.conn.execute(
            "DELETE FROM workflows WHERE id = ?1",
            params![workflow_id],
        )?;
        if deleted == 0 {
            return Err(StoreError::WorkflowNotFound(workflow_id.to_string()));
        }
        Ok(())
    }

    fn load_workflow_steps(
        &self,
        workflow_id: &str,
    ) -> Result<Vec<crate::workflow::WorkflowStep>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT step_id, tool_id, command_id, parameters_json, workdir, on_error
             FROM workflow_steps
             WHERE workflow_id = ?1
             ORDER BY position",
        )?;

        let rows = stmt.query_map(params![workflow_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, String>(5)?,
            ))
        })?;

        let mut steps = Vec::new();
        for row in rows {
            let (id, tool_id, command_id, parameters_json, workdir, on_error) = row?;
            steps.push(crate::workflow::WorkflowStep {
                id,
                tool_id,
                command_id,
                parameters: serde_json::from_str(&parameters_json)?,
                workdir: workdir.filter(|value| !value.is_empty()).map(PathBuf::from),
                on_error: StepFailurePolicy::parse(&on_error),
            });
        }
        Ok(steps)
    }

    /// Registra o resultado de um Workflow, com uma linha por etapa.
    pub fn record_workflow_execution(
        &self,
        workspace_id: &str,
        outcome: &WorkflowOutcome,
    ) -> Result<WorkflowExecutionRecord, StoreError> {
        let record = WorkflowExecutionRecord {
            id: WorkflowExecutionRecord::new_id(),
            workspace_id: workspace_id.to_string(),
            workflow_id: outcome.workflow_id.clone(),
            status: if outcome.success() {
                ExecutionStatus::Success
            } else {
                ExecutionStatus::Failed
            },
            steps: outcome
                .steps
                .iter()
                .enumerate()
                .map(|(position, step)| WorkflowStepRecord {
                    position: position as i64,
                    step_id: step.step_id.clone(),
                    status: step.status,
                    exit_code: step.exit_code,
                    stdout: step.stdout.clone(),
                    stderr: step.stderr.clone(),
                    error: step.error.clone(),
                })
                .collect(),
            finished_at: String::new(),
        };

        self.conn.execute(
            "INSERT INTO workflow_executions (id, workspace_id, workflow_id, status)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                record.id,
                record.workspace_id,
                record.workflow_id,
                record.status.as_str()
            ],
        )?;

        for step in &record.steps {
            self.conn.execute(
                "INSERT INTO workflow_step_results (
                    execution_id, position, step_id, status, exit_code, stdout, stderr, error
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    record.id,
                    step.position,
                    step.step_id,
                    step.status.as_str(),
                    step.exit_code,
                    step.stdout,
                    step.stderr,
                    step.error
                ],
            )?;
        }

        let finished_at: String = self.conn.query_row(
            "SELECT finished_at FROM workflow_executions WHERE id = ?1",
            params![record.id],
            |row| row.get(0),
        )?;

        Ok(WorkflowExecutionRecord {
            finished_at,
            ..record
        })
    }

    pub fn list_recent_workflow_executions(
        &self,
        workspace_id: &str,
        limit: usize,
    ) -> Result<Vec<WorkflowExecutionRecord>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, workflow_id, status, finished_at
             FROM workflow_executions
             WHERE workspace_id = ?1
             ORDER BY finished_at DESC, id DESC
             LIMIT ?2",
        )?;

        let rows = stmt.query_map(params![workspace_id, limit as i64], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })?;

        let mut records = Vec::new();
        for row in rows {
            let (id, workflow_id, status, finished_at) = row?;
            records.push(WorkflowExecutionRecord {
                steps: self.load_workflow_step_results(&id)?,
                id,
                workspace_id: workspace_id.to_string(),
                workflow_id,
                status: ExecutionStatus::parse(&status),
                finished_at,
            });
        }
        Ok(records)
    }

    fn load_workflow_step_results(
        &self,
        execution_id: &str,
    ) -> Result<Vec<WorkflowStepRecord>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT position, step_id, status, exit_code, stdout, stderr, error
             FROM workflow_step_results
             WHERE execution_id = ?1
             ORDER BY position",
        )?;

        let rows = stmt.query_map(params![execution_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<i32>>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, Option<String>>(6)?,
            ))
        })?;

        let mut steps = Vec::new();
        for row in rows {
            let (position, step_id, status, exit_code, stdout, stderr, error) = row?;
            steps.push(WorkflowStepRecord {
                position,
                step_id,
                status: StepStatus::parse(&status),
                exit_code,
                stdout,
                stderr,
                error,
            });
        }
        Ok(steps)
    }

    fn sync_manifest_versions(&self) -> Result<(), StoreError> {
        for tool in self.library.tools() {
            self.upsert_manifest_version(tool)?;
        }
        Ok(())
    }

    fn upsert_manifest_version(&self, tool: &Tool) -> Result<(), StoreError> {
        let content_json = serde_json::to_string(&tool.manifest)?;
        let content_hash = hash_content(&content_json);

        self.conn.execute(
            "INSERT INTO tools (id, name, current_version) VALUES (?1, ?2, 1)
             ON CONFLICT(id) DO UPDATE SET name = excluded.name",
            params![tool.id, tool.name],
        )?;

        let latest: Option<(i64, String)> = self
            .conn
            .query_row(
                "SELECT version, content_hash FROM manifest_versions
                 WHERE tool_id = ?1
                 ORDER BY version DESC LIMIT 1",
                params![tool.id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;

        let next_version = match latest {
            None => 1,
            Some((_, hash)) if hash == content_hash => {
                return Ok(());
            }
            Some((version, _)) => version + 1,
        };

        self.conn.execute(
            "INSERT INTO manifest_versions (tool_id, version, content_json, content_hash)
             VALUES (?1, ?2, ?3, ?4)",
            params![tool.id, next_version, content_json, content_hash],
        )?;

        self.conn.execute(
            "UPDATE tools SET current_version = ?1 WHERE id = ?2",
            params![next_version, tool.id],
        )?;

        Ok(())
    }

    fn ensure_default_workspace(&self, default_workdir: PathBuf) -> Result<(), StoreError> {
        let exists: Option<String> = self
            .conn
            .query_row(
                "SELECT id FROM workspaces WHERE id = 'default'",
                [],
                |row| row.get(0),
            )
            .optional()?;

        if exists.is_none() {
            self.conn.execute(
                "INSERT INTO workspaces (id, name, global_workdir, is_default) VALUES (?1, ?2, ?3, 1)",
                params!["default", "Padrão", default_workdir.display().to_string()],
            )?;
            return Ok(());
        }

        let current: String = self.conn.query_row(
            "SELECT global_workdir FROM workspaces WHERE id = 'default'",
            [],
            |row| row.get(0),
        )?;
        if current.trim().is_empty() {
            self.set_global_workdir("default", &default_workdir)?;
        }
        Ok(())
    }

    /// Garante que as ferramentas disponíveis estejam referenciadas no Workspace
    /// padrão. Os demais Workspaces têm curadoria própria e não são tocados.
    fn sync_workspace_tools(&self) -> Result<(), StoreError> {
        for tool in self.library.tools() {
            self.conn.execute(
                "INSERT OR IGNORE INTO workspace_tools (workspace_id, tool_id) VALUES (?1, ?2)",
                params![DEFAULT_WORKSPACE_ID, tool.id],
            )?;
        }
        Ok(())
    }

    fn load_workspace(&self, workspace_id: &str) -> Result<Workspace, StoreError> {
        let (id, name, global_workdir): (String, String, String) = self
            .conn
            .query_row(
                "SELECT id, name, global_workdir FROM workspaces WHERE id = ?1",
                params![workspace_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|_| StoreError::WorkspaceNotFound(workspace_id.to_string()))?;

        let mut tool_stmt = self.conn.prepare(
            "SELECT tool_id FROM workspace_tools WHERE workspace_id = ?1 ORDER BY tool_id",
        )?;
        let tool_ids = tool_stmt
            .query_map(params![workspace_id], |row| row.get(0))?
            .collect::<Result<Vec<String>, _>>()?;

        let mut profile_stmt = self.conn.prepare(
            "SELECT profile_id FROM workspace_profiles WHERE workspace_id = ?1 ORDER BY profile_id",
        )?;
        let profile_ids = profile_stmt
            .query_map(params![workspace_id], |row| row.get(0))?
            .collect::<Result<Vec<String>, _>>()?;

        let mut favorite_stmt = self.conn.prepare(
            "SELECT tool_id FROM workspace_favorites WHERE workspace_id = ?1 ORDER BY tool_id",
        )?;
        let favorite_tool_ids = favorite_stmt
            .query_map(params![workspace_id], |row| row.get(0))?
            .collect::<Result<Vec<String>, _>>()?;

        Ok(Workspace {
            id,
            name,
            global_workdir: PathBuf::from(global_workdir),
            tool_ids,
            profile_ids,
            favorite_tool_ids,
        })
    }
}

/// Dois nomes colidem quando diferem apenas em maiúsculas/minúsculas.
///
/// A comparação é feita em Rust, e não com `COLLATE NOCASE`, porque o SQLite só
/// dobra ASCII: `Ação` e `AÇÃO` seriam aceitos como nomes distintos. A dobra via
/// `to_lowercase` é sensível a Unicode, que é o que um produto em português
/// precisa.
fn names_match(existing: &str, candidate: &str) -> bool {
    existing.trim().to_lowercase() == candidate.trim().to_lowercase()
}

/// Definições importadas registradas no banco (a versão vigente de cada tool).
///
/// Entradas ilegíveis são ignoradas: um registro corrompido não deve impedir o
/// app de abrir.
fn load_persisted_manifests(conn: &Connection) -> Result<Vec<Manifest>, StoreError> {
    let mut stmt = conn.prepare(
        "SELECT mv.content_json
         FROM tools t
         INNER JOIN manifest_versions mv
            ON mv.tool_id = t.id AND mv.version = t.current_version",
    )?;

    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;

    let mut manifests = Vec::new();
    for row in rows {
        if let Ok(manifest) = parse_manifest(&row?) {
            manifests.push(manifest);
        }
    }
    Ok(manifests)
}

fn hash_content(content: &str) -> String {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn parse_status(value: &str) -> ExecutionStatus {
    ExecutionStatus::parse(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::{StepStatus, WorkflowStepResult};
    use std::time::{SystemTime, UNIX_EPOCH};

    const IMPORTED_MANIFEST: &str = r#"{
        "schemaVersion": "v1",
        "id": "demo-imported",
        "name": "Demo importada",
        "executable": "echo",
        "commands": [{"id": "print", "name": "Imprimir", "argvPrefix": ["hello"]}]
    }"#;

    fn temp_db() -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!("uige-store-test-{stamp}.sqlite"))
    }

    fn sample_workflow(name: &str, steps: Vec<(&str, &str, &str, StepFailurePolicy)>) -> Workflow {
        let steps = steps
            .into_iter()
            .map(|(step_id, tool_id, command_id, on_error)| {
                let mut parameters = HashMap::new();
                parameters.insert("message".to_string(), "do workflow".to_string());
                crate::workflow::WorkflowStep {
                    id: step_id.to_string(),
                    tool_id: tool_id.to_string(),
                    command_id: command_id.to_string(),
                    parameters,
                    workdir: None,
                    on_error,
                }
            })
            .collect();

        Workflow {
            id: "workflow-teste".to_string(),
            name: name.to_string(),
            description: Some("workflow de teste".to_string()),
            steps,
        }
    }

    #[test]
    fn opens_with_default_workspace_and_bundled_tools() {
        let path = temp_db();
        let workdir = PathBuf::from("/tmp/uige-demo");
        let store = Store::open(&path, workdir.clone()).expect("open store");
        let workspace = store.default_workspace().expect("workspace");

        assert_eq!(workspace.id, "default");
        assert_eq!(workspace.name, "Padrão");
        assert_eq!(workspace.global_workdir, workdir);
        assert!(workspace.tool_ids.contains(&"echo".to_string()));
        assert!(workspace.tool_ids.contains(&"git".to_string()));
        assert!(store.library().get("git").is_some());
        assert!(store.current_manifest_version("git").expect("version") >= 1);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn saves_and_lists_profile() {
        let path = temp_db();
        let store = Store::open(&path, PathBuf::from(".")).expect("open store");

        let mut parameters = HashMap::new();
        parameters.insert("message".to_string(), "from-profile".to_string());

        let profile = Profile {
            id: "profile-test-1".to_string(),
            name: "Echo hello".to_string(),
            tool_id: "echo".to_string(),
            command_id: "print".to_string(),
            parameters,
            workdir: None,
        };

        store
            .save_profile("default", &profile)
            .expect("save profile");

        let listed = store.list_profiles("default").expect("list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].name, "Echo hello");
        assert_eq!(
            listed[0].parameters.get("message").map(String::as_str),
            Some("from-profile")
        );

        let loaded = store.get_profile("profile-test-1").expect("get");
        assert_eq!(loaded.command_id, "print");

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn rejects_duplicate_profile_name_in_workspace() {
        let path = temp_db();
        let store = Store::open(&path, PathBuf::from(".")).expect("open store");

        let build = |id: &str, name: &str| Profile {
            id: id.to_string(),
            name: name.to_string(),
            tool_id: "echo".to_string(),
            command_id: "print".to_string(),
            parameters: HashMap::new(),
            workdir: None,
        };

        store
            .save_profile("default", &build("profile-a", "Deploy"))
            .expect("first save");

        // Mesmo nome, ainda que com caixa diferente, é recusado.
        let err = store
            .save_profile("default", &build("profile-b", "deploy"))
            .expect_err("duplicate name");
        assert!(matches!(err, StoreError::ProfileNameTaken(name) if name == "deploy"));
        assert_eq!(store.list_profiles("default").expect("list").len(), 1);

        // Salvar de novo o próprio perfil não é bloqueado.
        store
            .save_profile("default", &build("profile-a", "Deploy"))
            .expect("update own profile");

        // Nome vazio é recusado.
        let err = store
            .save_profile("default", &build("profile-c", "   "))
            .expect_err("empty name");
        assert!(matches!(err, StoreError::EmptyProfileName));

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn profile_name_collision_ignores_case_beyond_ascii() {
        let path = temp_db();
        let store = Store::open(&path, PathBuf::from(".")).expect("open store");

        let build = |id: &str, name: &str| Profile {
            id: id.to_string(),
            name: name.to_string(),
            tool_id: "echo".to_string(),
            command_id: "print".to_string(),
            parameters: HashMap::new(),
            workdir: None,
        };

        store
            .save_profile("default", &build("profile-a", "Ação"))
            .expect("first save");

        // `COLLATE NOCASE` do SQLite só dobra ASCII: sem a comparação em Rust,
        // este nome passaria como distinto.
        let err = store
            .save_profile("default", &build("profile-b", "AÇÃO"))
            .expect_err("duplicata com acento");
        assert!(matches!(err, StoreError::ProfileNameTaken(_)));

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn imports_manifest_and_keeps_it_across_reopen() {
        let path = temp_db();
        let mut store = Store::open(&path, PathBuf::from(".")).expect("open store");

        let outcome = store
            .import_manifest(IMPORTED_MANIFEST)
            .expect("import manifest");
        assert_eq!(outcome.tool_id, "demo-imported");
        assert_eq!(outcome.version, 1);
        assert!(outcome.created_version);
        assert!(store.library().get("demo-imported").is_some());
        assert!(!store.library().get("demo-imported").expect("tool").bundled);
        assert!(store
            .default_workspace()
            .expect("workspace")
            .tool_ids
            .contains(&"demo-imported".to_string()));

        drop(store);
        let store = Store::open(&path, PathBuf::from(".")).expect("reopen");
        let tool = store.library().get("demo-imported").expect("imported tool");
        assert_eq!(tool.name, "Demo importada");
        assert_eq!(tool.manifest.executable, "echo");
        assert_eq!(store.current_manifest_version("demo-imported").unwrap(), 1);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn importing_same_content_does_not_create_a_new_version() {
        let path = temp_db();
        let mut store = Store::open(&path, PathBuf::from(".")).expect("open store");

        store
            .import_manifest(IMPORTED_MANIFEST)
            .expect("first import");
        let again = store
            .import_manifest(IMPORTED_MANIFEST)
            .expect("second import");

        assert_eq!(again.version, 1);
        assert!(!again.created_version, "conteúdo idêntico não versiona");

        let changed = IMPORTED_MANIFEST.replace("Demo importada", "Demo importada v2");
        let updated = store.import_manifest(&changed).expect("changed import");
        assert_eq!(updated.version, 2);
        assert!(updated.created_version);

        let versions = store.list_manifest_versions("demo-imported").expect("versions");
        assert_eq!(versions.len(), 2);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn refuses_to_import_over_a_bundled_definition() {
        let path = temp_db();
        let mut store = Store::open(&path, PathBuf::from(".")).expect("open store");

        let git = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/examples/git.v1.json"
        ))
        .expect("read bundled git example");

        let err = store
            .import_manifest(&git)
            .expect_err("bundled id must be refused");
        assert!(matches!(err, StoreError::BundledToolConflict(id) if id == "git"));
        assert!(store.library().get("git").expect("git tool").bundled);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn refuses_to_import_an_invalid_manifest() {
        let path = temp_db();
        let mut store = Store::open(&path, PathBuf::from(".")).expect("open store");

        let err = store
            .import_manifest(r#"{"schemaVersion": "v1", "id": "x", "name": "X", "executable": " "}"#)
            .expect_err("invalid manifest");
        assert!(matches!(err, StoreError::Manifest(_)));

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn persists_workflow_with_ordered_steps_across_reopen() {
        let path = temp_db();
        let store = Store::open(&path, PathBuf::from(".")).expect("open store");

        let workflow = sample_workflow("Deploy", vec![
            ("passo-1", "echo", "print", StepFailurePolicy::Stop),
            ("passo-2", "git", "status", StepFailurePolicy::Continue),
        ]);

        store.save_workflow(&workflow).expect("save workflow");

        let listed = store.list_workflows().expect("list workflows");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].name, "Deploy");
        assert_eq!(listed[0].steps.len(), 2);
        // A ordem é a do vetor declarado, não a alfabética.
        assert_eq!(listed[0].steps[0].id, "passo-1");
        assert_eq!(listed[0].steps[0].tool_id, "echo");
        assert_eq!(listed[0].steps[1].id, "passo-2");
        assert_eq!(listed[0].steps[1].on_error, StepFailurePolicy::Continue);
        assert_eq!(
            listed[0].steps[0].parameters.get("message").map(String::as_str),
            Some("do workflow")
        );

        drop(store);
        let store = Store::open(&path, PathBuf::from(".")).expect("reopen");
        let loaded = store.get_workflow("workflow-teste").expect("get workflow");
        assert_eq!(loaded.name, "Deploy");
        assert_eq!(loaded.steps.len(), 2);
        assert_eq!(loaded.steps[1].command_id, "status");

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn saving_workflow_replaces_steps_and_keeps_name_unique() {
        let path = temp_db();
        let store = Store::open(&path, PathBuf::from(".")).expect("open store");

        store
            .save_workflow(&sample_workflow("Deploy", vec![
                ("passo-1", "echo", "print", StepFailurePolicy::Stop),
                ("passo-2", "git", "status", StepFailurePolicy::Stop),
            ]))
            .expect("save");

        // Reordenar e remover: a definição de etapas é substituída, não acumulada.
        store
            .save_workflow(&sample_workflow("Deploy", vec![
                ("passo-2", "git", "status", StepFailurePolicy::Stop),
            ]))
            .expect("update");

        let loaded = store.get_workflow("workflow-teste").expect("get");
        assert_eq!(loaded.steps.len(), 1);
        assert_eq!(loaded.steps[0].id, "passo-2");

        // Outro workflow com o mesmo nome (caixa diferente) é recusado.
        let mut other = sample_workflow("deploy", vec![
            ("passo-1", "echo", "print", StepFailurePolicy::Stop),
        ]);
        other.id = "workflow-outro".to_string();
        let err = store.save_workflow(&other).expect_err("duplicate name");
        assert!(matches!(err, StoreError::WorkflowNameTaken(name) if name == "deploy"));

        // Workflow sem etapas não é persistido.
        let vazio = sample_workflow("Vazio", Vec::new());
        let err = store.save_workflow(&vazio).expect_err("empty workflow");
        assert!(matches!(err, StoreError::Workflow(WorkflowError::EmptyWorkflow)));

        // Nome vazio é recusado.
        let sem_nome = sample_workflow("   ", vec![
            ("passo-1", "echo", "print", StepFailurePolicy::Stop),
        ]);
        let err = store.save_workflow(&sem_nome).expect_err("empty name");
        assert!(matches!(err, StoreError::EmptyWorkflowName));

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn deleting_workflow_removes_its_steps() {
        let path = temp_db();
        let store = Store::open(&path, PathBuf::from(".")).expect("open store");

        store
            .save_workflow(&sample_workflow("Deploy", vec![
                ("passo-1", "echo", "print", StepFailurePolicy::Stop),
            ]))
            .expect("save");

        store.delete_workflow("workflow-teste").expect("delete");

        assert!(store.list_workflows().expect("list").is_empty());
        assert!(matches!(
            store.get_workflow("workflow-teste").expect_err("gone"),
            StoreError::WorkflowNotFound(_)
        ));
        let leftover: i64 = store
            .conn
            .query_row("SELECT COUNT(*) FROM workflow_steps", [], |row| row.get(0))
            .expect("count steps");
        assert_eq!(leftover, 0, "etapas devem sair junto (ON DELETE CASCADE)");

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn records_workflow_execution_with_per_step_results() {
        let path = temp_db();
        let store = Store::open(&path, PathBuf::from(".")).expect("open store");

        let outcome = WorkflowOutcome {
            workflow_id: "workflow-teste".to_string(),
            steps: vec![
                WorkflowStepResult {
                    step_id: "passo-1".to_string(),
                    status: StepStatus::Success,
                    exit_code: Some(0),
                    stdout: "primeiro".to_string(),
                    stderr: String::new(),
                    error: None,
                },
                WorkflowStepResult {
                    step_id: "passo-2".to_string(),
                    status: StepStatus::Failed,
                    exit_code: None,
                    stdout: String::new(),
                    stderr: String::new(),
                    error: Some("binário ausente".to_string()),
                },
                WorkflowStepResult {
                    step_id: "passo-3".to_string(),
                    status: StepStatus::Skipped,
                    exit_code: None,
                    stdout: String::new(),
                    stderr: String::new(),
                    error: None,
                },
            ],
        };

        let record = store
            .record_workflow_execution("default", &outcome)
            .expect("record");
        assert_eq!(record.status, ExecutionStatus::Failed);
        assert_eq!(record.failed_steps(), 1);
        assert!(!record.finished_at.is_empty());
        assert!(record.label().contains("1 falha(s)"));

        let history = store
            .list_recent_workflow_executions("default", 10)
            .expect("history");
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].steps.len(), 3);
        assert_eq!(history[0].steps[0].status, StepStatus::Success);
        assert_eq!(history[0].steps[0].stdout, "primeiro");
        assert_eq!(history[0].steps[1].status, StepStatus::Failed);
        assert_eq!(history[0].steps[1].error.as_deref(), Some("binário ausente"));
        assert_eq!(history[0].steps[2].status, StepStatus::Skipped);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn records_execution_history_and_manifest_versions_are_append_only() {
        let path = temp_db();
        let store = Store::open(&path, PathBuf::from(".")).expect("open store");

        let versions = store.list_manifest_versions("echo").expect("versions");
        assert!(!versions.is_empty());
        let first = versions[0].version;

        drop(store);
        let store = Store::open(&path, PathBuf::from(".")).expect("reopen");
        let versions_again = store.list_manifest_versions("echo").expect("versions");
        assert_eq!(versions_again[0].version, first);

        let outcome = ExecutionOutcome {
            status: ExecutionStatus::Success,
            exit_code: Some(0),
            stdout: "hello".into(),
            stderr: String::new(),
        };
        let mut params = HashMap::new();
        params.insert("message".into(), "hello".into());
        store
            .record_execution("default", "echo", "print", &params, None, &outcome)
            .expect("record");

        let history = store
            .list_recent_executions("default", 10)
            .expect("history");
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].stdout, "hello");
        assert_eq!(history[0].status, ExecutionStatus::Success);

        let _ = std::fs::remove_file(&path);
    }

    fn workspace_named(name: &str) -> Workspace {
        Workspace {
            id: Workspace::new_id(name),
            name: name.to_string(),
            global_workdir: PathBuf::from("."),
            tool_ids: Vec::new(),
            profile_ids: Vec::new(),
            favorite_tool_ids: Vec::new(),
        }
    }

    #[test]
    fn creates_lists_and_renames_workspaces() {
        let path = temp_db();
        let store = Store::open(&path, PathBuf::from(".")).expect("open store");

        let created = workspace_named("Segurança");
        store.save_workspace(&created).expect("create");

        let all = store.list_workspaces().expect("list");
        // O padrão vem primeiro, independente da ordem de criação.
        assert_eq!(all.len(), 2);
        assert!(all[0].is_default());
        assert_eq!(all[1].name, "Segurança");

        let mut renamed = created.clone();
        renamed.name = "Pentest".to_string();
        store.save_workspace(&renamed).expect("rename");

        let all = store.list_workspaces().expect("list");
        assert_eq!(all.len(), 2, "renomear não deve criar outro workspace");
        assert!(all.iter().any(|workspace| workspace.name == "Pentest"));

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn rejects_duplicate_workspace_name_ignoring_case() {
        let path = temp_db();
        let store = Store::open(&path, PathBuf::from(".")).expect("open store");

        store
            .save_workspace(&workspace_named("Segurança"))
            .expect("create");

        let mut duplicate = workspace_named("SEGURANÇA");
        duplicate.id = "workspace-outro".to_string();
        let err = store
            .save_workspace(&duplicate)
            .expect_err("nome repetido");
        assert!(matches!(err, StoreError::WorkspaceNameTaken(_)));

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn rejects_empty_workspace_name() {
        let path = temp_db();
        let store = Store::open(&path, PathBuf::from(".")).expect("open store");

        let err = store
            .save_workspace(&workspace_named("   "))
            .expect_err("nome vazio");
        assert!(matches!(err, StoreError::EmptyWorkspaceName));

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn new_workspace_starts_with_the_available_tools() {
        let path = temp_db();
        let store = Store::open(&path, PathBuf::from(".")).expect("open store");

        let created = workspace_named("DevOps");
        store.save_workspace(&created).expect("create");

        let loaded = store
            .list_workspaces()
            .expect("list")
            .into_iter()
            .find(|workspace| workspace.id == created.id)
            .expect("workspace criado");
        assert!(loaded.tool_ids.contains(&"echo".to_string()));
        assert!(loaded.tool_ids.contains(&"git".to_string()));

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn switching_the_active_workspace_persists() {
        let path = temp_db();
        let store = Store::open(&path, PathBuf::from(".")).expect("open store");

        assert!(store.active_workspace().expect("ativo").is_default());

        let created = workspace_named("Segurança");
        store.save_workspace(&created).expect("create");
        store
            .set_active_workspace(&created.id)
            .expect("ativar");

        drop(store);
        let store = Store::open(&path, PathBuf::from(".")).expect("reopen");
        assert_eq!(store.active_workspace().expect("ativo").id, created.id);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn active_workspace_falls_back_to_default_when_the_record_is_gone() {
        let path = temp_db();
        let store = Store::open(&path, PathBuf::from(".")).expect("open store");

        // Chave órfã: aponta para um workspace que não existe.
        store
            .conn
            .execute(
                "INSERT INTO app_meta (key, value) VALUES ('active_workspace', 'fantasma')
                 ON CONFLICT(key) DO UPDATE SET value = 'fantasma'",
                [],
            )
            .expect("gravar chave órfã");

        assert!(store.active_workspace().expect("ativo").is_default());

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn removing_a_tool_drops_it_from_the_workspace_and_from_favorites() {
        let path = temp_db();
        let store = Store::open(&path, PathBuf::from(".")).expect("open store");

        store
            .set_tool_favorite("default", "git", true)
            .expect("favoritar");
        store
            .remove_tool_from_workspace("default", "git")
            .expect("remover");

        let workspace = store.default_workspace().expect("workspace");
        assert!(!workspace.tool_ids.contains(&"git".to_string()));
        assert!(
            !workspace.favorite_tool_ids.contains(&"git".to_string()),
            "favorito não deve sobreviver à saída do workspace"
        );

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn protects_the_default_workspace_from_removal() {
        let path = temp_db();
        let store = Store::open(&path, PathBuf::from(".")).expect("open store");

        let err = store
            .delete_workspace("default")
            .expect_err("padrão protegido");
        assert!(matches!(err, StoreError::DefaultWorkspaceProtected));

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn deleting_the_active_workspace_returns_to_default() {
        let path = temp_db();
        let store = Store::open(&path, PathBuf::from(".")).expect("open store");

        let created = workspace_named("Temporário");
        store.save_workspace(&created).expect("create");
        store
            .set_active_workspace(&created.id)
            .expect("ativar");
        store
            .delete_workspace(&created.id)
            .expect("remover");

        assert!(store.active_workspace().expect("ativo").is_default());
        assert_eq!(store.list_workspaces().expect("list").len(), 1);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn adding_an_unknown_tool_to_a_workspace_is_refused() {
        let path = temp_db();
        let store = Store::open(&path, PathBuf::from(".")).expect("open store");

        let err = store
            .add_tool_to_workspace("default", "nao-existe")
            .expect_err("ferramenta desconhecida");
        assert!(matches!(err, StoreError::ToolNotFound(_)));

        let _ = std::fs::remove_file(&path);
    }
}
