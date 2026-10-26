//! Harness support for Legion blackbox tests.

pub mod run_fixture;
pub mod runtime_fixture;

pub fn legion_corpus_root() -> std::path::PathBuf {
    legion_fixtures::root()
}

pub fn legion_corpus_subdir(relative: &str) -> std::path::PathBuf {
    legion_fixtures::root().join(relative)
}
