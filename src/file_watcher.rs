use crate::Res;
use ignore::gitignore::GitignoreBuilder;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
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
    pub fn new(path: &Path) -> Res<Self> {
        let pending_updates = Arc::new(AtomicBool::new(false));
        let pending_updates_w = pending_updates.clone();

        let gitignore = GitignoreBuilder::new(path).build().unwrap();

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                for path in event.paths {
                    if !gitignore.matched(&path, path.is_dir()).is_ignore() {
                        pending_updates_w.store(true, Ordering::Relaxed);
                        break;
                    }
                }
            }
        })?;

        watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

        Ok(Self {
            _watcher: watcher,
            pending_updates,
        })
    }

    pub fn pending_updates(&self) -> bool {
        self.pending_updates.swap(false, Ordering::Relaxed)
    }
}
