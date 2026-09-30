-- UIGE schema v4 (Phase 4, item 16)
-- Workflows (sequência ordenada de etapas) e histórico de execução de workflow.
--
-- Workflow é referenciado por Profile (`Profile → Execution | Workflow`), não pelo
-- Workspace; por isso não existe tabela de vínculo workspace <-> workflow.

CREATE TABLE IF NOT EXISTS workflows (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS workflow_steps (
    workflow_id TEXT NOT NULL,
    position INTEGER NOT NULL,
    step_id TEXT NOT NULL,
    tool_id TEXT NOT NULL,
    command_id TEXT NOT NULL,
    parameters_json TEXT NOT NULL DEFAULT '{}',
    workdir TEXT,
    on_error TEXT NOT NULL DEFAULT 'Stop',
    PRIMARY KEY (workflow_id, position),
    FOREIGN KEY (workflow_id) REFERENCES workflows(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS workflow_executions (
    id TEXT PRIMARY KEY NOT NULL,
    workspace_id TEXT NOT NULL,
    workflow_id TEXT NOT NULL,
    status TEXT NOT NULL,
    finished_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS workflow_step_results (
    execution_id TEXT NOT NULL,
    position INTEGER NOT NULL,
    step_id TEXT NOT NULL,
    status TEXT NOT NULL,
    exit_code INTEGER,
    stdout TEXT NOT NULL DEFAULT '',
    stderr TEXT NOT NULL DEFAULT '',
    error TEXT,
    PRIMARY KEY (execution_id, position),
    FOREIGN KEY (execution_id) REFERENCES workflow_executions(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_workflow_executions_finished
    ON workflow_executions (finished_at DESC);

INSERT OR IGNORE INTO schema_migrations (version) VALUES (4);
