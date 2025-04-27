use std::{fmt::Display, io, string, sync::mpsc};

use crate::GituEvent;

#[derive(Debug)]
pub enum Error {
    StashList(git2::Error),
    ReadLog(git2::Error),
    OpenRepo(git2::Error),
    FindGitDir(io::Error),
    Term(io::Error),
    EventSendError(mpsc::SendError<GituEvent>),
    EventRecvError(mpsc::RecvError),
    GitDirUtf8(string::FromUtf8Error),
    Config(figment::Error),
    FileWatcherGitignore(ignore::Error),
    FileWatcher(notify::Error),
    ReadRebaseStatusFile(io::Error),
    ReadBranchName(io::Error),
    BranchNameUtf8(Utf8Error),
    GitDiff(io::Error),
    GitDiffUtf8(string::FromUtf8Error),
    GitShow(io::Error),
    GitShowUtf8(string::FromUtf8Error),
    GitShowMeta(git2::Error),
    NotOnBranch,
    GetHead(git2::Error),
    CurrentBranchName(git2::Error),
    GetCurrentBranchUpstream(git2::Error),
    GetCurrentBranchUpstreamUtf8(Utf8Error),
    RemoteNameUtf8(Utf8Error),
    GetRemote(git2::Error),
    ReadGitConfig(git2::Error),
    ReadGitConfigUtf8(Utf8Error),
    DeleteGitConfig(git2::Error),
    SetGitConfig(git2::Error),
    RemoteHasNoName,
    ReadOid(git2::Error),
    ArgMustBePositiveNumber,
    ArgInvalidRegex(regex::Error),
    Clipboard(arboard::Error),
    FindGitRev(git2::Error),
    NoEditorSet,
    GitStatus(git2::Error),
    CmdAlreadyRunning,
    StashWorkTreeEmpty,
    CouldntAwaitCmd(io::Error),
    NoRepoWorkdir,
    SpawnCmd(io::Error),
    CmdBadExit(String, Option<i32>),
    CouldntReadCmdOutput(io::Error),
    ListGitReferences(git2::Error),
    OpenLogFile(io::Error),
    PromptAborted,
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::StashList(e) => f.write_fmt(format_args!("Couldn't list stash: {}", e)),
            Error::ReadLog(e) => f.write_fmt(format_args!("Couldn't read log: {}", e)),
            Error::OpenRepo(e) => match e.code() {
                git2::ErrorCode::NotFound => f.write_str("No .git found in the current directory"),
                _ => f.write_fmt(format_args!("Couldn't open repo: {e:?}")),
            },
            Error::FindGitDir(e) => f.write_fmt(format_args!("Couldn't find git directory: {}", e)),
            Error::Term(e) => f.write_fmt(format_args!("Terminal error: {}", e)),
            Error::EventSendError(e) => {
                f.write_fmt(format_args!("Error when handling events: {}", e))
            }
            Error::EventRecvError(e) => {
                f.write_fmt(format_args!("Error when handling events: {}", e))
            }
            Error::GitDirUtf8(_e) => f.write_str("Git directory not valid UTF-8"),
            Error::Config(e) => f.write_fmt(format_args!("Configuration error: {}", e)),
            Error::FileWatcherGitignore(e) => {
                f.write_fmt(format_args!("File watcher gitignore error: {}", e))
            }
            Error::FileWatcher(e) => f.write_fmt(format_args!("File watcher error: {}", e)),
            Error::ReadRebaseStatusFile(e) => {
                f.write_fmt(format_args!("Couldn't read rebase status file: {}", e))
            }
            Error::ReadBranchName(e) => {
                f.write_fmt(format_args!("Couldn't read branch name: {}", e))
            }
            Error::BranchNameUtf8(_e) => f.write_str("Branch name error"),
            Error::GitDiff(e) => f.write_fmt(format_args!("Git diff error: {}", e)),
            Error::GitDiffUtf8(e) => {
                f.write_fmt(format_args!("Git diff output is not valid UTF-8: {}", e))
            }
            Error::GitShow(e) => f.write_fmt(format_args!("Git show error: {}", e)),
            Error::GitShowUtf8(e) => {
                f.write_fmt(format_args!("Git show output is not valid UTF-8: {}", e))
            }
            Error::GitShowMeta(e) => f.write_fmt(format_args!("Git show metadata error: {}", e)),
            Error::NotOnBranch => f.write_str("Head is not a branch"),
            Error::GetHead(e) => f.write_fmt(format_args!("Couldn't get HEAD: {}", e)),
            Error::CurrentBranchName(e) => {
                f.write_fmt(format_args!("Couldn't get current branch name: {}", e))
            }
            Error::GetCurrentBranchUpstream(e) => {
                f.write_fmt(format_args!("Couldn't get current branch upstream: {}", e))
            }
            Error::GetCurrentBranchUpstreamUtf8(_e) => {
                f.write_str("Current branch upstream is not valid UTF-8")
            }
            Error::RemoteNameUtf8(_e) => f.write_str("Remote name is not valid UTF-8"),
            Error::GetRemote(e) => f.write_fmt(format_args!("Couldn't get remote: {}", e)),
            Error::ReadGitConfig(e) => f.write_fmt(format_args!("Couldn't read git config: {}", e)),
            Error::ReadGitConfigUtf8(_e) => f.write_str("Git config is not valid UTF-8"),
            Error::DeleteGitConfig(e) => {
                f.write_fmt(format_args!("Couldn't delete git config: {}", e))
            }
            Error::SetGitConfig(e) => f.write_fmt(format_args!("Couldn't set git config: {}", e)),
            Error::RemoteHasNoName => f.write_str("Remote has no name"),
            Error::ReadOid(e) => f.write_fmt(format_args!("Couldn't read OID: {}", e)),
            Error::ArgMustBePositiveNumber => f.write_str("Value must be a number greater than 0"),
            Error::ArgInvalidRegex(e) => f.write_fmt(format_args!("Invalid regex: {}", e)),
            Error::Clipboard(e) => f.write_fmt(format_args!("Clipboard error: {}", e)),
            Error::FindGitRev(e) => f.write_fmt(format_args!("Couldn't find git revision: {}", e)),
            Error::NoEditorSet => f.write_fmt(format_args!(
                "No editor environment variable set ({})",
                crate::ops::show::EDITOR_VARS.join(", ")
            )),
            Error::GitStatus(e) => f.write_fmt(format_args!("Git status error: {}", e)),
            Error::CmdAlreadyRunning => f.write_str("A command is already running"),
            Error::StashWorkTreeEmpty => f.write_str("Cannot stash: working tree is empty"),
            Error::CouldntAwaitCmd(e) => f.write_fmt(format_args!("Couldn't await command: {}", e)),
            Error::NoRepoWorkdir => f.write_str("No repository working directory"),
            Error::SpawnCmd(e) => f.write_fmt(format_args!("Failed to spawn command: {}", e)),
            Error::CmdBadExit(args, code) => f.write_fmt(format_args!(
                "'{}' exited with code: {}",
                args,
                code.map(|c| c.to_string())
                    .unwrap_or_else(|| "".to_string())
            )),
            Error::CouldntReadCmdOutput(e) => {
                f.write_fmt(format_args!("Couldn't read command output: {}", e))
            }
            Error::ListGitReferences(e) => {
                f.write_fmt(format_args!("Couldn't list git references: {}", e))
            }
            Error::OpenLogFile(e) => f.write_fmt(format_args!("Couldn't open log file: {}", e)),
            Error::PromptAborted => f.write_str("Aborted"),
        }
    }
}

#[derive(Debug)]
pub enum Utf8Error {
    Str(std::str::Utf8Error),
    String(string::FromUtf8Error),
}

impl std::error::Error for Utf8Error {}

impl Display for Utf8Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("not valid UTF-8")
    }
}
