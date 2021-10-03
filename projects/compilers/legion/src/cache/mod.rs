//! Legion compilation-cache conventions on top of `legion-workspace`.
//!
//! The workspace crate provides bucket storage; this module defines stage
//! type tags, key composition, and when build uses the IR/artifact bucket.

mod artifact;
mod compilation;
mod pipeline;
mod resolve;

pub use artifact::{
    BuildBundle, CachedBuildBundle, collect_build_bundle, compute_artifact_hash, load_cached_build, materialize_build_bundle,
    store_cached_build, toolchain_fingerprint, try_restore_cached_build,
};
pub use compilation::{CompilationCache, EntrySliceCacheEntry, IrCacheEntry, SemanticCacheEntry, StageCacheEntry, TokenCacheEntry};
pub use pipeline::{CachedFrontendCompile, compile_frontend_with_cache, compile_semantic_source_groups};
pub use resolve::{cache_root_for, resolve_cache_root, resolve_workspace_root};
