-- UIGE schema v2 (Phase 2)
-- Workspace padrão, referências a tools e profiles simples.

CREATE TABLE IF NOT EXISTS workspaces (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    global_workdir TEXT NOT NULL DEFAULT '',
    is_default INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS workspace_tools (
    workspace_id TEXT NOT NULL,
    tool_id TEXT NOT NULL,
    PRIMARY KEY (workspace_id, tool_id),
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS profiles (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    tool_id TEXT NOT NULL,
    command_id TEXT NOT NULL,
    parameters_json TEXT NOT NULL DEFAULT '{}',
    workdir TEXT
);

CREATE TABLE IF NOT EXISTS workspace_profiles (
    workspace_id TEXT NOT NULL,
    profile_id TEXT NOT NULL,
    PRIMARY KEY (workspace_id, profile_id),
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE,
    FOREIGN KEY (profile_id) REFERENCES profiles(id) ON DELETE CASCADE
);

INSERT OR IGNORE INTO workspaces (id, name, global_workdir, is_default)
VALUES ('default', 'Padrão', '', 1);

INSERT OR IGNORE INTO schema_migrations (version) VALUES (2);
