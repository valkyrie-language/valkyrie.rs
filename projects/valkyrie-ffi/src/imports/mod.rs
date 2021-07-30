use std::path::Path;
use wit_parser::{Resolve, SourceMap, UnresolvedPackage};

fn load_all() {
    let mut resolved = Resolve::new();
    resolved.push_dir("tests");
}

pub struct ValkyrieFFI {
    cache: Resolve,
}

impl ValkyrieFFI {
    pub fn new<P: AsRef<Path>>(directory: P) -> anyhow::Result<Self> {
        let mut resolved = Resolve::new();
        resolved.push_dir(directory.as_ref())?;
        Ok(Self { cache: resolved })
    }
}
