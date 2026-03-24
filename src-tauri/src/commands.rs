use crate::crypto::{Encryptor, VaultState};
use crate::db::{ComparisonMatrix, EnvFile, EnvVar, Project, Root, SearchResult};
use crate::scanner::{self, ScanResult};
use crate::search::SearchFilters;
use crate::watcher::FileWatcher;
use crate::AppState;
use parking_lot::Mutex;
use tauri::State;

// We store the vault state in a lazy static since it contains the encryption key
use std::sync::OnceLock;
static VAULT_STATE: OnceLock<Mutex<VaultState>> = OnceLock::new();

fn vault_state() -> &'static Mutex<VaultState> {
    VAULT_STATE.get_or_init(|| Mutex::new(VaultState::new()))
}

// ── Root management ──────────────────────────────────────────────────

#[tauri::command]
pub fn add_root(state: State<'_, AppState>, path: String) -> Result<ScanResult, String> {
    let db = state.db.lock();
    db.add_root(&path).map_err(|e| e.to_string())?;

    let vs = vault_state().lock();
    let encryptor = vs.get_encryptor();
    scanner::scan_root(&path, &db, encryptor)
}

#[tauri::command]
pub fn remove_root(state: State<'_, AppState>, path: String) -> Result<(), String> {
    let db = state.db.lock();
    db.remove_root(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn scan_root(state: State<'_, AppState>, path: String) -> Result<ScanResult, String> {
    let db = state.db.lock();
    let vs = vault_state().lock();
    let encryptor = vs.get_encryptor();
    scanner::scan_root(&path, &db, encryptor)
}

#[tauri::command]
pub fn scan_all(state: State<'_, AppState>) -> Result<Vec<ScanResult>, String> {
    let db = state.db.lock();
    let roots = db.get_roots().map_err(|e| e.to_string())?;
    let vs = vault_state().lock();
    let encryptor = vs.get_encryptor();

    let mut results = Vec::new();
    for root in &roots {
        match scanner::scan_root(&root.path, &db, encryptor) {
            Ok(result) => results.push(result),
            Err(e) => eprintln!("Error scanning {}: {}", root.path, e),
        }
    }
    Ok(results)
}

#[tauri::command]
pub fn get_roots(state: State<'_, AppState>) -> Result<Vec<Root>, String> {
    let db = state.db.lock();
    db.get_roots().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_projects(state: State<'_, AppState>) -> Result<Vec<Project>, String> {
    let db = state.db.lock();
    db.get_projects().map_err(|e| e.to_string())
}

// ── Env file operations ──────────────────────────────────────────────

#[tauri::command]
pub fn get_env_files(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<Vec<EnvFile>, String> {
    let db = state.db.lock();
    db.get_env_files(&project_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_env_variables(
    state: State<'_, AppState>,
    file_id: String,
) -> Result<Vec<EnvVar>, String> {
    let db = state.db.lock();
    db.get_env_variables(&file_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn decrypt_value(
    _state: State<'_, AppState>,
    encrypted_value: Vec<u8>,
    nonce: Vec<u8>,
) -> Result<String, String> {
    let vs = vault_state().lock();
    let encryptor = vs.get_encryptor().ok_or("Vault is locked")?;

    let plaintext = encryptor
        .decrypt(&encrypted_value, &nonce)
        .map_err(|e| e.to_string())?;

    String::from_utf8(plaintext).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn compare_envs(
    state: State<'_, AppState>,
    file_ids: Vec<String>,
) -> Result<ComparisonMatrix, String> {
    let db = state.db.lock();
    db.compare_env_files(&file_ids).map_err(|e| e.to_string())
}

// ── Vault management ─────────────────────────────────────────────────

#[tauri::command]
pub fn setup_vault(state: State<'_, AppState>, master_pw: String) -> Result<(), String> {
    let mut vs = vault_state().lock();
    let (salt, password_hash) = vs.setup(&master_pw).map_err(|e| e.to_string())?;

    let db = state.db.lock();
    db.set_meta("argon2_salt", &salt)
        .map_err(|e| e.to_string())?;
    db.set_meta("password_verification_hash", password_hash.as_bytes())
        .map_err(|e| e.to_string())?;
    db.set_meta("lock_timeout_minutes", &15u32.to_le_bytes())
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn unlock_vault(state: State<'_, AppState>, master_pw: String) -> Result<bool, String> {
    let db = state.db.lock();

    let salt = db
        .get_meta("argon2_salt")
        .map_err(|e| e.to_string())?
        .ok_or("Vault not set up")?;

    let hash_bytes = db
        .get_meta("password_verification_hash")
        .map_err(|e| e.to_string())?
        .ok_or("Vault not set up")?;

    let hash = String::from_utf8(hash_bytes).map_err(|e| e.to_string())?;

    let mut vs = vault_state().lock();
    vs.unlock(&master_pw, &salt, &hash)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn lock_vault() -> Result<(), String> {
    let mut vs = vault_state().lock();
    vs.lock();
    Ok(())
}

#[tauri::command]
pub fn is_vault_setup(state: State<'_, AppState>) -> Result<bool, String> {
    let db = state.db.lock();
    let salt = db.get_meta("argon2_salt").map_err(|e| e.to_string())?;
    Ok(salt.is_some())
}

#[tauri::command]
pub fn is_vault_unlocked() -> Result<bool, String> {
    let vs = vault_state().lock();
    Ok(vs.is_unlocked())
}

#[tauri::command]
pub fn change_password(
    state: State<'_, AppState>,
    old_pw: String,
    new_pw: String,
) -> Result<(), String> {
    // Verify old password
    let db = state.db.lock();

    let hash_bytes = db
        .get_meta("password_verification_hash")
        .map_err(|e| e.to_string())?
        .ok_or("Vault not set up")?;
    let old_hash = String::from_utf8(hash_bytes).map_err(|e| e.to_string())?;

    let valid = Encryptor::verify_password(&old_pw, &old_hash).map_err(|e| e.to_string())?;
    if !valid {
        return Err("Current password is incorrect".to_string());
    }

    // Set up with new password
    let mut vs = vault_state().lock();
    let (new_salt, new_hash) = vs.setup(&new_pw).map_err(|e| e.to_string())?;

    db.set_meta("argon2_salt", &new_salt)
        .map_err(|e| e.to_string())?;
    db.set_meta("password_verification_hash", new_hash.as_bytes())
        .map_err(|e| e.to_string())?;

    // TODO: Re-encrypt all stored values with new key
    // This would require decrypting with old key and re-encrypting with new key

    Ok(())
}

#[tauri::command]
pub fn export_all(state: State<'_, AppState>, target_dir: String) -> Result<u32, String> {
    let vs = vault_state().lock();
    let encryptor = vs.get_encryptor().ok_or("Vault is locked")?;
    let db = state.db.lock();

    let projects = db.get_projects().map_err(|e| e.to_string())?;
    let mut exported = 0u32;

    for project in &projects {
        let files = db.get_env_files(&project.id).map_err(|e| e.to_string())?;

        for file in &files {
            let vars = db.get_env_variables(&file.id).map_err(|e| e.to_string())?;

            let mut content = String::new();
            for var in &vars {
                if let Some(comment) = &var.comment {
                    content.push_str(comment);
                    content.push('\n');
                }
                let value = encryptor
                    .decrypt(&var.encrypted_value, &var.nonce)
                    .map_err(|e| e.to_string())?;
                let value_str = String::from_utf8(value).map_err(|e| e.to_string())?;
                content.push_str(&format!("{}={}\n", var.key, value_str));
            }

            let out_path = std::path::Path::new(&target_dir)
                .join(&project.name)
                .join(&file.filename);
            std::fs::create_dir_all(out_path.parent().unwrap()).map_err(|e| e.to_string())?;
            std::fs::write(&out_path, &content).map_err(|e| e.to_string())?;
            exported += 1;
        }
    }

    Ok(exported)
}

// ── Search ───────────────────────────────────────────────────────────

#[tauri::command]
pub fn search(
    state: State<'_, AppState>,
    query: String,
    filters: Option<SearchFilters>,
) -> Result<Vec<SearchResult>, String> {
    let db = state.db.lock();

    let project_ids = filters.as_ref().and_then(|f| f.project_ids.as_ref());
    let tiers = filters.as_ref().and_then(|f| f.tiers.as_ref());

    db.search_variables(
        &query,
        project_ids.map(|v| v.as_slice()),
        tiers.map(|v| v.as_slice()),
    )
    .map_err(|e| e.to_string())
}

// ── File watcher ─────────────────────────────────────────────────────

#[tauri::command]
pub fn start_watcher(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock();
    let roots = db.get_roots().map_err(|e| e.to_string())?;
    let root_paths: Vec<String> = roots.iter().map(|r| r.path.clone()).collect();
    drop(db); // Release the lock

    if root_paths.is_empty() {
        return Ok(());
    }

    let app_handle = app.clone();
    let watcher = FileWatcher::new(root_paths, move |changed_path| {
        // Emit event to frontend
        use tauri::Emitter;
        let _ = app_handle.emit("env-file-changed", changed_path);
    })?;

    let mut w = state.watcher.lock();
    *w = Some(watcher);
    Ok(())
}

#[tauri::command]
pub fn stop_watcher(state: State<'_, AppState>) -> Result<(), String> {
    let mut w = state.watcher.lock();
    *w = None;
    Ok(())
}
