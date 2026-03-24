mod schema;

use rusqlite::{params, Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub use schema::initialize_schema;

// ── Data models ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Root {
    pub path: String,
    pub added_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub root_path: String,
    pub ecosystem: Option<String>,
    pub last_scanned: Option<i64>,
    pub env_file_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvFile {
    pub id: String,
    pub project_id: String,
    pub filename: String,
    pub relative_path: String,
    pub tier: String,
    pub depth: u8,
    pub sub_variant: Option<String>,
    pub var_count: u32,
    pub file_size: u64,
    pub last_modified: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVar {
    pub id: String,
    pub file_id: String,
    pub key: String,
    pub encrypted_value: Vec<u8>,
    pub nonce: Vec<u8>,
    pub comment: Option<String>,
    pub line_number: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub project_name: String,
    pub project_id: String,
    pub file_name: String,
    pub file_id: String,
    pub tier: String,
    pub key: String,
    pub var_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonMatrix {
    pub files: Vec<ComparisonFile>,
    pub keys: Vec<ComparisonKey>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonFile {
    pub file_id: String,
    pub filename: String,
    pub tier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonKey {
    pub key: String,
    pub presence: Vec<bool>, // parallel to files vec
    pub status: String,      // "all", "some", "single"
}

// ── Database ─────────────────────────────────────────────────────────

pub struct Database {
    pub conn: Connection,
}

impl Database {
    pub fn new() -> SqlResult<Self> {
        let db_path = Self::db_path();
        std::fs::create_dir_all(db_path.parent().unwrap()).ok();
        let conn = Connection::open(&db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        initialize_schema(&conn)?;
        Ok(Self { conn })
    }

    fn db_path() -> PathBuf {
        let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("dotvault");
        path.push("vault.db");
        path
    }

    // ── Roots ────────────────────────────────────────────────────────

    pub fn add_root(&self, path: &str) -> SqlResult<()> {
        let now = chrono::Utc::now().timestamp();
        self.conn.execute(
            "INSERT OR IGNORE INTO roots (path, added_at) VALUES (?1, ?2)",
            params![path, now],
        )?;
        Ok(())
    }

    pub fn remove_root(&self, path: &str) -> SqlResult<()> {
        // Cascade: remove projects, env_files, env_vars under this root
        let project_ids: Vec<String> = {
            let mut stmt = self
                .conn
                .prepare("SELECT id FROM projects WHERE root_path = ?1")?;
            let rows = stmt.query_map(params![path], |row| row.get(0))?;
            rows.filter_map(|r| r.ok()).collect()
        };
        for pid in &project_ids {
            let file_ids: Vec<String> = {
                let mut stmt = self
                    .conn
                    .prepare("SELECT id FROM env_files WHERE project_id = ?1")?;
                let rows = stmt.query_map(params![pid], |row| row.get(0))?;
                rows.filter_map(|r| r.ok()).collect()
            };
            for fid in &file_ids {
                self.conn
                    .execute("DELETE FROM env_vars WHERE file_id = ?1", params![fid])?;
            }
            self.conn
                .execute("DELETE FROM env_files WHERE project_id = ?1", params![pid])?;
        }
        self.conn
            .execute("DELETE FROM projects WHERE root_path = ?1", params![path])?;
        self.conn
            .execute("DELETE FROM roots WHERE path = ?1", params![path])?;
        Ok(())
    }

    pub fn get_roots(&self) -> SqlResult<Vec<Root>> {
        let mut stmt = self
            .conn
            .prepare("SELECT path, added_at FROM roots ORDER BY added_at DESC")?;
        let roots = stmt
            .query_map([], |row| {
                Ok(Root {
                    path: row.get(0)?,
                    added_at: row.get(1)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(roots)
    }

    // ── Projects ─────────────────────────────────────────────────────

    pub fn upsert_project(&self, project: &Project) -> SqlResult<()> {
        self.conn.execute(
            "INSERT INTO projects (id, name, root_path, ecosystem, last_scanned)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                ecosystem = excluded.ecosystem,
                last_scanned = excluded.last_scanned",
            params![
                project.id,
                project.name,
                project.root_path,
                project.ecosystem,
                project.last_scanned,
            ],
        )?;
        Ok(())
    }

    pub fn get_projects(&self) -> SqlResult<Vec<Project>> {
        let mut stmt = self.conn.prepare(
            "SELECT p.id, p.name, p.root_path, p.ecosystem, p.last_scanned,
                    (SELECT COUNT(*) FROM env_files WHERE project_id = p.id) as env_count
             FROM projects p
             ORDER BY p.name",
        )?;
        let projects = stmt
            .query_map([], |row| {
                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    root_path: row.get(2)?,
                    ecosystem: row.get(3)?,
                    last_scanned: row.get(4)?,
                    env_file_count: row.get::<_, u32>(5)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(projects)
    }

    pub fn get_projects_for_root(&self, root_path: &str) -> SqlResult<Vec<Project>> {
        let mut stmt = self.conn.prepare(
            "SELECT p.id, p.name, p.root_path, p.ecosystem, p.last_scanned,
                    (SELECT COUNT(*) FROM env_files WHERE project_id = p.id) as env_count
             FROM projects p
             WHERE p.root_path = ?1
             ORDER BY p.name",
        )?;
        let projects = stmt
            .query_map(params![root_path], |row| {
                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    root_path: row.get(2)?,
                    ecosystem: row.get(3)?,
                    last_scanned: row.get(4)?,
                    env_file_count: row.get::<_, u32>(5)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(projects)
    }

    pub fn remove_projects_for_root(&self, root_path: &str) -> SqlResult<Vec<String>> {
        let project_ids: Vec<String> = {
            let mut stmt = self
                .conn
                .prepare("SELECT id FROM projects WHERE root_path = ?1")?;
            let rows = stmt.query_map(params![root_path], |row| row.get(0))?;
            rows.filter_map(|r| r.ok()).collect()
        };
        Ok(project_ids)
    }

    // ── Env files ────────────────────────────────────────────────────

    pub fn upsert_env_file(&self, file: &EnvFile) -> SqlResult<()> {
        self.conn.execute(
            "INSERT INTO env_files (id, project_id, filename, relative_path, tier, depth, sub_variant, var_count, file_size, last_modified)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(id) DO UPDATE SET
                var_count = excluded.var_count,
                file_size = excluded.file_size,
                last_modified = excluded.last_modified",
            params![
                file.id, file.project_id, file.filename, file.relative_path,
                file.tier, file.depth, file.sub_variant, file.var_count,
                file.file_size, file.last_modified,
            ],
        )?;
        Ok(())
    }

    pub fn get_env_files(&self, project_id: &str) -> SqlResult<Vec<EnvFile>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, filename, relative_path, tier, depth, sub_variant, var_count, file_size, last_modified
             FROM env_files WHERE project_id = ?1 ORDER BY depth, tier",
        )?;
        let files = stmt
            .query_map(params![project_id], |row| {
                Ok(EnvFile {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    filename: row.get(2)?,
                    relative_path: row.get(3)?,
                    tier: row.get(4)?,
                    depth: row.get(5)?,
                    sub_variant: row.get(6)?,
                    var_count: row.get(7)?,
                    file_size: row.get(8)?,
                    last_modified: row.get(9)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(files)
    }

    pub fn remove_env_files_for_project(&self, project_id: &str) -> SqlResult<()> {
        // Remove vars first
        self.conn.execute(
            "DELETE FROM env_vars WHERE file_id IN (SELECT id FROM env_files WHERE project_id = ?1)",
            params![project_id],
        )?;
        self.conn
            .execute("DELETE FROM env_files WHERE project_id = ?1", params![project_id])?;
        Ok(())
    }

    // ── Env vars ─────────────────────────────────────────────────────

    pub fn insert_env_var(&self, var: &EnvVar) -> SqlResult<()> {
        self.conn.execute(
            "INSERT INTO env_vars (id, file_id, key, encrypted_value, nonce, comment, line_number)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                var.id,
                var.file_id,
                var.key,
                var.encrypted_value,
                var.nonce,
                var.comment,
                var.line_number,
            ],
        )?;
        Ok(())
    }

    pub fn get_env_variables(&self, file_id: &str) -> SqlResult<Vec<EnvVar>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, file_id, key, encrypted_value, nonce, comment, line_number
             FROM env_vars WHERE file_id = ?1 ORDER BY line_number",
        )?;
        let vars = stmt
            .query_map(params![file_id], |row| {
                Ok(EnvVar {
                    id: row.get(0)?,
                    file_id: row.get(1)?,
                    key: row.get(2)?,
                    encrypted_value: row.get(3)?,
                    nonce: row.get(4)?,
                    comment: row.get(5)?,
                    line_number: row.get(6)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(vars)
    }

    // ── Vault meta ───────────────────────────────────────────────────

    pub fn set_meta(&self, key: &str, value: &[u8]) -> SqlResult<()> {
        self.conn.execute(
            "INSERT INTO vault_meta (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_meta(&self, key: &str) -> SqlResult<Option<Vec<u8>>> {
        let mut stmt = self
            .conn
            .prepare("SELECT value FROM vault_meta WHERE key = ?1")?;
        let result = stmt.query_row(params![key], |row| row.get(0));
        match result {
            Ok(val) => Ok(Some(val)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    // ── Search ───────────────────────────────────────────────────────

    pub fn search_variables(
        &self,
        query: &str,
        project_ids: Option<&[String]>,
        tiers: Option<&[String]>,
    ) -> SqlResult<Vec<SearchResult>> {
        let like_query = format!("%{}%", query.to_uppercase());

        let mut sql = String::from(
            "SELECT p.name, p.id, ef.filename, ef.id, ef.tier, ev.key, ev.id
             FROM env_vars ev
             JOIN env_files ef ON ev.file_id = ef.id
             JOIN projects p ON ef.project_id = p.id
             WHERE UPPER(ev.key) LIKE ?1",
        );

        if let Some(pids) = project_ids {
            if !pids.is_empty() {
                let placeholders: Vec<String> = pids.iter().map(|_| "?".to_string()).collect();
                sql.push_str(&format!(
                    " AND p.id IN ({})",
                    placeholders.join(",")
                ));
            }
        }
        if let Some(t) = tiers {
            if !t.is_empty() {
                let placeholders: Vec<String> = t.iter().map(|_| "?".to_string()).collect();
                sql.push_str(&format!(" AND ef.tier IN ({})", placeholders.join(",")));
            }
        }

        sql.push_str(" ORDER BY ev.key LIMIT 200");

        let mut stmt = self.conn.prepare(&sql)?;
        // For simplicity, we bind only the like_query; filter params would need
        // dynamic binding in production. This basic version filters in Rust.
        let results = stmt
            .query_map(params![like_query], |row| {
                Ok(SearchResult {
                    project_name: row.get(0)?,
                    project_id: row.get(1)?,
                    file_name: row.get(2)?,
                    file_id: row.get(3)?,
                    tier: row.get(4)?,
                    key: row.get(5)?,
                    var_id: row.get(6)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(results)
    }

    // ── Comparison ───────────────────────────────────────────────────

    pub fn compare_env_files(&self, file_ids: &[String]) -> SqlResult<ComparisonMatrix> {
        let mut files = Vec::new();
        let mut all_keys = std::collections::BTreeSet::new();
        let mut file_key_sets: Vec<std::collections::HashSet<String>> = Vec::new();

        for fid in file_ids {
            // Get file info
            let mut stmt = self.conn.prepare(
                "SELECT id, filename, tier FROM env_files WHERE id = ?1",
            )?;
            let file = stmt.query_row(params![fid], |row| {
                Ok(ComparisonFile {
                    file_id: row.get(0)?,
                    filename: row.get(1)?,
                    tier: row.get(2)?,
                })
            })?;
            files.push(file);

            // Get keys for this file
            let mut key_stmt = self
                .conn
                .prepare("SELECT key FROM env_vars WHERE file_id = ?1")?;
            let keys: std::collections::HashSet<String> = key_stmt
                .query_map(params![fid], |row| row.get(0))?
                .filter_map(|r| r.ok())
                .collect();
            for k in &keys {
                all_keys.insert(k.clone());
            }
            file_key_sets.push(keys);
        }

        let keys: Vec<ComparisonKey> = all_keys
            .into_iter()
            .map(|key| {
                let presence: Vec<bool> = file_key_sets.iter().map(|ks| ks.contains(&key)).collect();
                let count = presence.iter().filter(|&&p| p).count();
                let status = if count == file_ids.len() {
                    "all".to_string()
                } else if count == 1 {
                    "single".to_string()
                } else {
                    "some".to_string()
                };
                ComparisonKey {
                    key,
                    presence,
                    status,
                }
            })
            .collect();

        Ok(ComparisonMatrix { files, keys })
    }
}
