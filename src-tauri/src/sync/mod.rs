//! Keeps the active workspace and its folder in step: app changes are written to the
//! folder, and edits made in the folder (for example by `git pull`) are read back.
mod engine;
mod watch;

#[cfg(test)]
mod tests;

pub use engine::SyncEngine;
pub use engine::canonical;
pub use watch::{start_flusher, table_hook, watch};

pub const CHANGED_EVENT: &str = "workspace://changed";
pub const STATUS_EVENT: &str = "workspace://sync";
