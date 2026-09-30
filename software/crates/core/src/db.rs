//! Minimal SQLite bootstrap for local app state.

use std::path::Path;

use rusqlite::{Connection, OptionalExtension};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("failed to open database at {path}: {source}")]
    Open {
        path: String,
        source: rusqlite::Error,
    },
    #[error("failed to apply migration: {0}")]
    Migration(#[from] rusqlite::Error),
}

/// Opens (or creates) the SQLite database and applies pending migrations.
pub fn init_database(path: &Path) -> Result<Connection, DatabaseError> {
    let conn = Connection::open(path).map_err(|source| DatabaseError::Open {
        path: path.display().to_string(),
        source,
    })?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY NOT NULL,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )?;

    let current = schema_version(&conn)?;
    if current < 1 {
        conn.execute_batch(include_str!("../migrations/001_initial.sql"))?;
    }
    if schema_version(&conn)? < 2 {
        conn.execute_batch(include_str!("../migrations/002_workspace_profile.sql"))?;
    }
    if schema_version(&conn)? < 3 {
        conn.execute_batch(include_str!("../migrations/003_history_manifest_versions.sql"))?;
    }
    if schema_version(&conn)? < 4 {
        conn.execute_batch(include_str!("../migrations/004_workflows.sql"))?;
    }
    if schema_version(&conn)? < 5 {
        conn.execute_batch(include_str!("../migrations/005_workspaces.sql"))?;
    }

    Ok(conn)
}

fn schema_version(conn: &Connection) -> Result<i64, DatabaseError> {
    let version = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )
        .optional()?
        .unwrap_or(0);
    Ok(version)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn init_database_applies_schema() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("uige-core-test-{stamp}.sqlite"));

        let conn = init_database(&path).expect("init database");
        let version: i64 = conn
            .query_row(
                "SELECT version FROM schema_migrations ORDER BY version DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("read migration version");
        assert_eq!(version, 5);

        let workspace: String = conn
            .query_row(
                "SELECT name FROM workspaces WHERE id = 'default'",
                [],
                |row| row.get(0),
            )
            .expect("default workspace");
        assert_eq!(workspace, "Padrão");

        let history_table: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='execution_history'",
                [],
                |row| row.get(0),
            )
            .expect("history table");
        assert_eq!(history_table, 1);

        let workflows_table: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='workflow_steps'",
                [],
                |row| row.get(0),
            )
            .expect("workflow_steps table");
        assert_eq!(workflows_table, 1);

        let favorites_table: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='workspace_favorites'",
                [],
                |row| row.get(0),
            )
            .expect("workspace_favorites table");
        assert_eq!(favorites_table, 1);

        // Nome de Workspace é único: o índice precisa existir para a regra valer.
        let unique_index: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name='idx_workspaces_name'",
                [],
                |row| row.get(0),
            )
            .expect("unique index");
        assert_eq!(unique_index, 1);

        let _ = std::fs::remove_file(&path);
    }
}
