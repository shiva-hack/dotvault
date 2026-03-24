use rusqlite::{Connection, Result as SqlResult};

pub fn initialize_schema(conn: &Connection) -> SqlResult<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS roots (
            path TEXT PRIMARY KEY,
            added_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS projects (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            root_path TEXT NOT NULL,
            ecosystem TEXT,
            last_scanned INTEGER,
            FOREIGN KEY (root_path) REFERENCES roots(path)
        );

        CREATE TABLE IF NOT EXISTS env_files (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            filename TEXT NOT NULL,
            relative_path TEXT NOT NULL,
            tier TEXT NOT NULL DEFAULT 'base',
            depth INTEGER NOT NULL DEFAULT 0,
            sub_variant TEXT,
            var_count INTEGER DEFAULT 0,
            file_size INTEGER,
            last_modified INTEGER,
            FOREIGN KEY (project_id) REFERENCES projects(id)
        );

        CREATE TABLE IF NOT EXISTS env_vars (
            id TEXT PRIMARY KEY,
            file_id TEXT NOT NULL,
            key TEXT NOT NULL,
            encrypted_value BLOB NOT NULL,
            nonce BLOB NOT NULL,
            comment TEXT,
            line_number INTEGER,
            FOREIGN KEY (file_id) REFERENCES env_files(id)
        );

        CREATE TABLE IF NOT EXISTS vault_meta (
            key TEXT PRIMARY KEY,
            value BLOB NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_vars_key ON env_vars(key);
        CREATE INDEX IF NOT EXISTS idx_files_project ON env_files(project_id);
        CREATE INDEX IF NOT EXISTS idx_files_tier ON env_files(tier);
        CREATE INDEX IF NOT EXISTS idx_projects_root ON projects(root_path);
        ",
    )?;
    Ok(())
}
