use crate::{error::Error, Res};
use ignore::gitignore::GitignoreBuilder;
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};

pub struct FileWatcher {
    pending_updates: Arc<AtomicBool>,
}

impl FileWatcher {
    pub fn new(path: &Path) -> Res<Self> {
        let pending_updates = Arc::new(AtomicBool::new(false));
        let pending_updates_w = pending_updates.clone();

        let gitignore = GitignoreBuilder::new(path)
            .add_line(None, super::LOG_FILE_NAME)
            .map_err(Error::FileWatcherGitignore)?
            .build()
            .unwrap();

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                if !is_changed(&event) {
                    return;
                }

                for path in event.paths {
                    if !gitignore.matched(&path, path.is_dir()).is_ignore() {
                        log::info!("File changed: {:?}", path);
                        pending_updates_w.store(true, Ordering::Relaxed);
                        break;
                    }
                }
            }
        })
        .map_err(Error::FileWatcher)?;

        let path_buf = path.to_owned();
        thread::spawn(move || {
            if let Err(err) = watcher
                .watch(path_buf.as_ref(), RecursiveMode::Recursive)
                .map_err(Error::FileWatcher)
            {
                log::error!("Couldn't start file-watcher due to: {}", err);
            }
        });

        Ok(Self { pending_updates })
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
