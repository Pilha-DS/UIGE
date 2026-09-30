-- UIGE schema v5 (Phase 4)
-- Workspaces administráveis: nome único, favoritos de ferramenta e workspace ativo.

-- Nome de Workspace é único (sem diferenciar maiúsculas/minúsculas), como Profile.
-- Se existirem duplicatas de dados antigos, o índice falha e a migração deve ser
-- revista em vez de ignorar o conflito silenciosamente.
CREATE UNIQUE INDEX IF NOT EXISTS idx_workspaces_name
    ON workspaces(name COLLATE NOCASE);

-- Favoritos referenciam Tools; são preferência do usuário, então vivem aqui e
-- não no Manifest.
CREATE TABLE IF NOT EXISTS workspace_favorites (
    workspace_id TzzzEXT NOT NULL,
    tool_id TEXT NOT NULL,
    PRIMARY KEY (workspace_id, tool_id),
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE
);

INSERT OR IGNORE INTO app_meta (key, value) VALUES ('active_workspace', 'default');

INSERT OR IGNORE INTO schema_migrations (version) VALUES (5);
