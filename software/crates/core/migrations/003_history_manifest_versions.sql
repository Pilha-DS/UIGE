-- UIGE schema v3 (Phase 3)
-- Tools persisted, manifest versions (append-only), execution history.

CREATE TABLE IF NOT EXISTS tools (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    current_version INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS manifest_versions (
    tool_id TEXT NOT NULL,
    version INTEGER NOT NULL,
    content_json TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (tool_id, version),
    FOREIGN KEY (tool_id) REFERENCES tools(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS execution_history (
    id TEXT PRIMARY KEY NOT NULL,
    workspace_id TEXT NOT NULL,
    tool_id TEXT NOT NULL,
    command_id TEXT NOT NULL,
    parameters_json TEXT NOT NULL DEFAULT '{}',
    workdir TEXT,
    status TEXT NOT NULL,
    exit_code INTEGER,
    stdout TEXT NOT NULL DEFAULT '',
    stderr TEXT NOT NULL DEFAULT '',
    finished_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_execution_history_finished
    ON execution_history (finished_at DESC);

CREATE INDEX IF NOT EXISTS idx_manifest_versions_tool
    ON manifest_versions (tool_id, version DESC);

INSERT OR IGNORE INTO schema_migrations (version) VALUES (3);
