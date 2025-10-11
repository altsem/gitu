use crate::{Res, error::Error, open_repo};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

pub struct FileWatcher {
    pending_updates: Arc<AtomicBool>,
}

impl FileWatcher {
    pub fn new(repo_dir: &Path) -> Res<Self> {
        let pending_updates = Arc::new(AtomicBool::new(false));
        let pending_updates_w = pending_updates.clone();
        let repo_dir_clone = repo_dir.to_path_buf();

        std::thread::spawn(move || {
            if let Err(e) = watch(&repo_dir_clone, pending_updates_w) {
                log::error!("File watcher error: {:?}", e)
            }
        });

        Ok(Self { pending_updates })
    }

    pub fn pending_updates(&self) -> bool {
        self.pending_updates.swap(false, Ordering::Relaxed)
    }
}

fn watch(repo_dir: &Path, pending_updates_w: Arc<AtomicBool>) -> Res<()> {
    let repo = open_repo(repo_dir)?;
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        if let Ok(event) = res {
            if !is_changed(&event) {
                return;
            }

            for path in event.paths {
                if !repo.status_should_ignore(&path).unwrap_or(false) {
                    log::info!("File changed: {:?} ({:?})", path, event.kind);
                    pending_updates_w.store(true, Ordering::Relaxed);
                    break;
                }
            }
        }
    })
    .map_err(Error::FileWatcher)?;

    let path_buf = repo_dir.to_owned();
    watcher
        .watch(&path_buf, RecursiveMode::Recursive)
        .map_err(Error::FileWatcher)?;

    log::info!(
        "File watcher started (kind: {:?})",
        RecommendedWatcher::kind()
    );

    std::mem::forget(watcher);
    Ok(())
}

fn is_changed(event: &Event) -> bool {
    matches!(
        event.kind,
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    )
}
