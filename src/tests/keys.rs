//! Key constants used in integration tests.
//!
//! These mirror the default bindings in `src/default_config.toml`. When a
//! keybinding changes, update both the default config and the matching
//! constant here.
//!
//! `MOVE_*` bindings are intentionally omitted: single-character move keys
//! (`k`/`j`) are used as building blocks in many sequences, and keeping them
//! inline reads more naturally than introducing constants for `<alt+k>` etc.

pub const HELP: &str = "h";
pub const REFRESH: &str = "gr";
pub const HALF_PAGE_DOWN: &str = "<ctrl+d>";
