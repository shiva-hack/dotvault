use crate::crypto::Encryptor;
use crate::db::{Database, EnvFile, EnvVar, Project};
use crate::parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Directories to skip during scanning
const IGNORED_DIRS: &[&str] = &[
    "node_modules",
    ".git",
    "vendor",
    "target",
    "__pycache__",
    "dist",
    "build",
    ".next",
    ".nuxt",
    ".svelte-kit",
    "venv",
    ".venv",
    "env",
];

/// Project marker files
const PROJECT_MARKERS: &[&str] = &[
    "package.json",
    "Cargo.toml",
    "pyproject.toml",
    "setup.py",
    "go.mod",
    "Gemfile",
    "composer.json",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub root_path: String,
    pub projects_found: u32,
    pub env_files_found: u32,
    pub projects: Vec<ScanProject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProject {
    pub id: String,
    pub name: String,
    pub path: String,
    pub ecosystem: Option<String>,
    pub env_file_count: u32,
}

/// Scan a root directory, discover projects and their .env files,
/// parse variables, encrypt values, and store everything in the database.
pub fn scan_root(
    root_path: &str,
    db: &Database,
    encryptor: Option<&Encryptor>,
) -> Result<ScanResult, String> {
    let root = Path::new(root_path);
    if !root.exists() || !root.is_dir() {
        return Err(format!(
            "Root path does not exist or is not a directory: {}",
            root_path
        ));
    }

    // Step 1: Discover projects
    let projects = discover_projects(root);

    let mut scan_projects = Vec::new();
    let mut total_env_files = 0u32;

    for (project_path, _marker) in &projects {
        let project_name = parser::extract_project_name(project_path);
        let ecosystem = parser::detect_ecosystem(project_path);

        // Generate a stable ID from path
        let project_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();

        // Check if project already exists by path
        let existing_projects = db.get_projects().unwrap_or_default();
        let actual_id = existing_projects
            .iter()
            .find(|p| p.root_path == root_path && p.name == project_name)
            .map(|p| p.id.clone())
            .unwrap_or_else(|| project_id.clone());

        let project = Project {
            id: actual_id.clone(),
            name: project_name.clone(),
            root_path: root_path.to_string(),
            ecosystem: ecosystem.clone(),
            last_scanned: Some(now),
            env_file_count: 0,
        };

        db.upsert_project(&project).map_err(|e| e.to_string())?;

        // Clear old env files for re-scan
        db.remove_env_files_for_project(&actual_id)
            .map_err(|e| e.to_string())?;

        // Step 2: Find .env files in this project
        let env_files = find_env_files(project_path);
        let env_count = env_files.len() as u32;
        total_env_files += env_count;

        for env_path in &env_files {
            let filename = env_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            let relative_path = env_path
                .strip_prefix(project_path)
                .unwrap_or(env_path)
                .to_string_lossy()
                .to_string();

            let tier_info = parser::parse_tier(&filename);
            let metadata = std::fs::metadata(env_path).ok();
            let file_size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
            let last_modified = metadata
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);

            // Parse the .env file
            let contents = std::fs::read_to_string(env_path).unwrap_or_default();
            let parsed_vars = parser::parse_env_contents(&contents);

            let file_id = uuid::Uuid::new_v4().to_string();
            let env_file = EnvFile {
                id: file_id.clone(),
                project_id: actual_id.clone(),
                filename: filename.clone(),
                relative_path,
                tier: tier_info.tier,
                depth: tier_info.depth,
                sub_variant: tier_info.sub_variant,
                var_count: parsed_vars.len() as u32,
                file_size,
                last_modified,
            };

            db.upsert_env_file(&env_file).map_err(|e| e.to_string())?;

            // Step 3: Store variables (encrypted if vault is unlocked)
            for pv in &parsed_vars {
                let (encrypted_value, nonce) = if let Some(enc) = encryptor {
                    enc.encrypt(pv.value.as_bytes())
                        .map_err(|e| e.to_string())?
                } else {
                    // If vault not unlocked, store plaintext as-is
                    // (will be re-encrypted when vault is set up)
                    (pv.value.as_bytes().to_vec(), vec![0u8; 12])
                };

                let var = EnvVar {
                    id: uuid::Uuid::new_v4().to_string(),
                    file_id: file_id.clone(),
                    key: pv.key.clone(),
                    encrypted_value,
                    nonce,
                    comment: pv.comment.clone(),
                    line_number: pv.line_number,
                };

                db.insert_env_var(&var).map_err(|e| e.to_string())?;
            }
        }

        scan_projects.push(ScanProject {
            id: actual_id,
            name: project_name,
            path: project_path.to_string_lossy().to_string(),
            ecosystem,
            env_file_count: env_count,
        });
    }

    Ok(ScanResult {
        root_path: root_path.to_string(),
        projects_found: scan_projects.len() as u32,
        env_files_found: total_env_files,
        projects: scan_projects,
    })
}

/// Discover projects under a root directory by looking for marker files.
/// Handles nested projects (monorepos) by finding the deepest marker.
fn discover_projects(root: &Path) -> Vec<(PathBuf, String)> {
    let mut projects: HashMap<PathBuf, String> = HashMap::new();

    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !IGNORED_DIRS.contains(&name.as_ref())
        })
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }

        let filename = entry.file_name().to_string_lossy().to_string();
        let is_marker = PROJECT_MARKERS.contains(&filename.as_str());
        let is_dotnet = filename.ends_with(".sln") || filename.ends_with(".csproj");

        if is_marker || is_dotnet {
            if let Some(parent) = entry.path().parent() {
                projects.entry(parent.to_path_buf()).or_insert(filename);
            }
        }
    }

    // If no project markers found, treat root itself as a project if it has .env files
    if projects.is_empty() && has_env_files(root) {
        projects.insert(root.to_path_buf(), ".git".to_string());
    }

    let mut result: Vec<(PathBuf, String)> = projects.into_iter().collect();
    result.sort_by(|a, b| a.0.cmp(&b.0));
    result
}

/// Find all .env* files in a project directory (non-recursive, same level)
fn find_env_files(project_path: &Path) -> Vec<PathBuf> {
    let mut env_files = Vec::new();

    if let Ok(entries) = std::fs::read_dir(project_path) {
        for entry in entries.filter_map(|e| e.ok()) {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(".env") && entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                // Skip .env.example files
                if name.ends_with(".example")
                    || name.ends_with(".sample")
                    || name.ends_with(".template")
                {
                    continue;
                }
                env_files.push(entry.path());
            }
        }
    }

    env_files.sort();
    env_files
}

fn has_env_files(dir: &Path) -> bool {
    std::fs::read_dir(dir)
        .ok()
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .any(|e| e.file_name().to_string_lossy().starts_with(".env"))
        })
        .unwrap_or(false)
}
