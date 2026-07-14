

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Mutex as StdMutex;

use notify::{Event, EventKind, RecursiveMode, Watcher};
use tokio::sync::mpsc;
use tomodachi_shared::CreatureEvent;
use tracing::{debug, info, warn};

static WATCHED_REPOS: StdMutex<Option<HashSet<PathBuf>>> = StdMutex::new(None);

pub async fn run_watcher(event_tx: mpsc::UnboundedSender<CreatureEvent>) -> anyhow::Result<()> {
    info!("starting filesystem watcher");

    {
        let mut repos = WATCHED_REPOS.lock().unwrap();
        *repos = Some(HashSet::new());
    }

    let tx = event_tx.clone();

    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        match res {
            Ok(event) => {
                handle_fs_event(&event, &tx);
            }
            Err(e) => {
                warn!("watcher error: {}", e);
            }
        }
    })?;

    let home = dirs_home();
    if let Some(home) = home {
        let common_dirs = vec![
            home.join("projects"),
            home.join("repos"),
            home.join("code"),
            home.join("dev"),
            home.join("src"),
        ];

        for dir in common_dirs {
            if dir.exists() {
                info!(path = %dir.display(), "scanning for git repos");
                discover_and_watch_repos(&dir, &mut watcher);
            }
        }
    }

    if let Ok(cwd) = std::env::current_dir() {
        discover_and_watch_repos(&cwd, &mut watcher);
    }

    info!("filesystem watcher running");

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    }
}

#[allow(dead_code)]
pub fn register_cwd(cwd: &str) {
    let path = PathBuf::from(cwd);

    let mut current = Some(path.as_path());
    while let Some(dir) = current {
        let git_dir = dir.join(".git");
        if git_dir.exists() {
            let mut repos = WATCHED_REPOS.lock().unwrap();
            if let Some(ref mut set) = *repos {
                if set.insert(dir.to_path_buf()) {
                    info!(repo = %dir.display(), "discovered new git repo from CWD");
                    
                    }
            }
            break;
        }
        current = dir.parent();
    }
}

fn handle_fs_event(event: &Event, tx: &mpsc::UnboundedSender<CreatureEvent>) {
    for path in &event.paths {
        let path_str = path.to_string_lossy();

        if path_str.contains(".git")
            && (path_str.ends_with("logs/HEAD") || path_str.ends_with("logs\\HEAD"))
        {
            match event.kind {
                EventKind::Modify(_) | EventKind::Create(_) => {
                    debug!(path = %path_str, "git commit detected");
                    let _ = tx.send(CreatureEvent::GitCommit);
                }
                _ => {}
            }
        }

        if path_str.contains(".git")
            && (path_str.ends_with("index") || path_str.ends_with("index.lock"))
        {
            match event.kind {
                EventKind::Modify(_) => {
                    debug!(path = %path_str, "git index changed");
                    let _ = tx.send(CreatureEvent::UnstagedChanges);
                }
                _ => {}
            }
        }
    }
}

fn discover_and_watch_repos(base: &std::path::Path, watcher: &mut impl Watcher) {
    
    discover_repos_recursive(base, watcher, 0, 3);
}

fn discover_repos_recursive(
    dir: &std::path::Path,
    watcher: &mut impl Watcher,
    depth: usize,
    max_depth: usize,
) {
    if depth > max_depth {
        return;
    }

    let git_dir = dir.join(".git");
    if git_dir.exists() && git_dir.is_dir() {
        let logs_dir = git_dir.join("logs");
        if logs_dir.exists() {
            match watcher.watch(&logs_dir, RecursiveMode::NonRecursive) {
                Ok(()) => {
                    info!(repo = %dir.display(), "watching git repo");
                    let mut repos = WATCHED_REPOS.lock().unwrap();
                    if let Some(ref mut set) = *repos {
                        set.insert(dir.to_path_buf());
                    }
                }
                Err(e) => {
                    warn!(repo = %dir.display(), error = %e, "failed to watch repo");
                }
            }
        }
        return; 
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            
            if !name_str.starts_with('.')
                && name_str != "node_modules"
                && name_str != "target"
                && name_str != "__pycache__"
                && name_str != ".git"
            {
                discover_repos_recursive(&path, watcher, depth + 1, max_depth);
            }
        }
    }
}

fn dirs_home() -> Option<PathBuf> {
    directories::BaseDirs::new().map(|d| d.home_dir().to_path_buf())
}
