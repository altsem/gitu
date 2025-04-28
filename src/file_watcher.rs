use crate::{error::Error, open_repo, Res};
use ignore::gitignore::{gitconfig_excludes_path, GitignoreBuilder, Gitignore};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    path::Path,
    path::PathBuf,
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

        let path_buf = repo_dir.to_owned();
        let path_buf_copy = path_buf.clone();
        let mut gitignore = build_gitignore(repo_dir)?;

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                if !is_changed(&event) {
                    return;
                }

                for path in event.paths {
                    if path
                        .file_name()
                        .map_or(false, |name| name == ".gitignore")
                    {
                        log::info!("Rebuilding gitignore ruleset");
                        if let Ok(new_gitignore) = build_gitignore(&path_buf_copy) {
                            gitignore = new_gitignore;
                        }
                    }

                    if !gitignore
                        .matched_path_or_any_parents(&path, path.is_dir())
                        .is_ignore()
                    {
                        log::info!("File changed: {:?}", path);
                        pending_updates_w.store(true, Ordering::Relaxed);
                        break;
                    }
                }
            }
        })
        .map_err(Error::FileWatcher)?;

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

fn build_gitignore(path: &Path) -> Res<Gitignore> {
    let mut gitignore_builder = GitignoreBuilder::new(path);
    for gitignore_path in repo_gitignore_paths(path)? {
        gitignore_builder.add(gitignore_path);
    }
    gitignore_builder.add_line(None, super::LOG_FILE_NAME).ok();
    gitignore_builder
        .build()
        .map_err(Error::FileWatcherGitignore)
}

fn repo_gitignore_paths(repo_dir: &Path) -> Res<Vec<PathBuf>> {
    let mut gitignore_paths = gitconfig_excludes_path().map_or_else(|| vec![], |path| vec![path]);
    gitignore_paths
        .extend(
            open_repo(repo_dir)?
                .index()
                .map_err(Error::OpenRepo)?
                .iter()
                .filter_map(|entry| {
                    match std::str::from_utf8(&entry.path).map(Path::new) {
                        Ok(path) if path.file_name() == Some(std::ffi::OsStr::new(".gitignore")) => Some(path.to_path_buf()),
                        _ => None
                    }
                })
        );
    Ok(gitignore_paths)
}
