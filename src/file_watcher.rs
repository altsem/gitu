use crate::{error::Error, open_repo, Res};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

pub struct FileWatcher {
    _watcher: RecommendedWatcher,
    pending_updates: Arc<AtomicBool>,
}

impl FileWatcher {
    pub fn new(repo_dir: &Path) -> Res<Self> {
        let pending_updates = Arc::new(AtomicBool::new(false));
        let pending_updates_w = pending_updates.clone();

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

        Ok(Self {
            _watcher: watcher,
            pending_updates,
        })
    }

    pub fn pending_updates(&self) -> bool {
        self.pending_updates.swap(false, Ordering::Relaxed)
    }
}

fn is_changed(event: &Event) -> bool {
    matches!(
        event.kind,
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    )
}
