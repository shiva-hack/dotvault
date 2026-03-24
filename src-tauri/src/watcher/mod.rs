use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc;
use std::thread;

pub struct FileWatcher {
    _watcher: RecommendedWatcher,
    _handle: thread::JoinHandle<()>,
}

impl FileWatcher {
    /// Start watching a set of root directories for .env file changes.
    /// Calls the callback on the main thread via Tauri's event system.
    pub fn new<F>(roots: Vec<String>, on_change: F) -> Result<Self, String>
    where
        F: Fn(String) + Send + 'static,
    {
        let (tx, rx) = mpsc::channel::<Event>();

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
                }
            },
            Config::default(),
        )
        .map_err(|e| format!("Failed to create watcher: {}", e))?;

        for root in &roots {
            watcher
                .watch(Path::new(root), RecursiveMode::Recursive)
                .map_err(|e| format!("Failed to watch {}: {}", root, e))?;
        }

        let handle = thread::spawn(move || {
            while let Ok(event) = rx.recv() {
                // Only care about .env file changes
                let env_paths: Vec<String> = event
                    .paths
                    .iter()
                    .filter(|p| {
                        p.file_name()
                            .map(|n| n.to_string_lossy().starts_with(".env"))
                            .unwrap_or(false)
                    })
                    .map(|p| p.to_string_lossy().to_string())
                    .collect();

                if env_paths.is_empty() {
                    continue;
                }

                match event.kind {
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                        for path in env_paths {
                            on_change(path);
                        }
                    }
                    _ => {}
                }
            }
        });

        Ok(Self {
            _watcher: watcher,
            _handle: handle,
        })
    }
}
