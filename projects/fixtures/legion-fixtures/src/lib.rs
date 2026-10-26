#![warn(missing_docs)]

//! Legion corpora root for consumers (e.g. `rusty-valkyrie` integration tests).
//!
//! Fixture files live next to this crate manifest; use [`root`] and join subpaths.

use std::path::PathBuf;

/// Directory containing `cmds_run/`, `bootstrap_node/`, `runtime_smoke/`, etc.
pub fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
