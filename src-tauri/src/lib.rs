mod commands;
mod crypto;
mod db;
mod parser;
mod scanner;
mod search;
mod watcher;

use db::Database;
use parking_lot::Mutex;
use std::sync::Arc;
use watcher::FileWatcher;

pub struct AppState {
    pub db: Arc<Mutex<Database>>,
    pub watcher: Arc<Mutex<Option<FileWatcher>>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db = Database::new().expect("Failed to initialize database");
    let state = AppState {
        db: Arc::new(Mutex::new(db)),
        watcher: Arc::new(Mutex::new(None)),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            // Project management
            commands::add_root,
            commands::remove_root,
            commands::scan_root,
            commands::scan_all,
            commands::get_roots,
            commands::get_projects,
            // Env file operations
            commands::get_env_files,
            commands::get_env_variables,
            commands::decrypt_value,
            commands::compare_envs,
            // Vault
            commands::setup_vault,
            commands::unlock_vault,
            commands::lock_vault,
            commands::is_vault_setup,
            commands::is_vault_unlocked,
            commands::change_password,
            commands::export_all,
            // Search
            commands::search,
            // Watcher
            commands::start_watcher,
            commands::stop_watcher,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
