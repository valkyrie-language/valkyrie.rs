use std::{fs, path::{Component, Path, PathBuf}};

pub(crate) fn select_artifact<'entry>(
    output_dir: &Path,
    entries: impl Iterator<Item = &'entry str>,
    artifact_override: Option<&Path>,
) -> Result<(PathBuf, usize), String> {
    let entries = entries.collect::<Vec<_>>();
    if entries.is_empty() {
        return Err("execution manifest has no run contracts".into());
    }
    if artifact_override.is_none() && entries.len() != 1 {
        return Err("execution manifest has multiple run contracts; select one with --artifact".into());
    }
    let root = fs::canonicalize(output_dir).map_err(|error| error.to_string())?;
    let requested = artifact_override.map(fs::canonicalize).transpose().map_err(|error| error.to_string())?;
    let mut selected = None;
    for (index, entry) in entries.into_iter().enumerate() {
        let relative = Path::new(entry);
        if relative.as_os_str().is_empty()
            || relative.components().any(|component| !matches!(component, Component::Normal(_)))
        {
            return Err(format!("execution manifest has an invalid artifact path: '{entry}'"));
        }
        let artifact = fs::canonicalize(root.join(relative)).map_err(|error| error.to_string())?;
        if !artifact.starts_with(&root) || !artifact.is_file() {
            return Err("execution manifest artifact is not a file inside the output directory".into());
        }
        if requested.as_ref().is_none_or(|path| path == &artifact) {
            if selected.is_some() {
                return Err("artifact has multiple run contracts".into());
            }
            selected = Some((artifact, index));
        }
    }
    selected.ok_or_else(|| "explicit artifact is not listed by the current execution manifest".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_DIRECTORY: AtomicUsize = AtomicUsize::new(0);

    struct Fixture {
        root: PathBuf,
    }

    impl Fixture {
        fn new() -> Self {
            let index = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!("legion-artifact-selection-{}-{index}", std::process::id()));
            fs::create_dir(&root).expect("创建独立测试目录");
            for name in ["first.mjs", "second.mjs", "old.mjs"] {
                fs::write(root.join(name), "artifact").expect("创建产物夹具");
            }
            Self { root }
        }

        fn select(&self, entries: &[&str], requested: Option<&Path>) -> Result<(PathBuf, usize), String> {
            select_artifact(&self.root, entries.iter().copied(), requested)
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            for name in ["first.mjs", "second.mjs", "old.mjs"] {
                fs::remove_file(self.root.join(name)).expect("移除测试创建的文件");
            }
            fs::remove_dir(&self.root).expect("移除空测试目录");
        }
    }

    #[test]
    fn selects_only_the_declared_artifact() {
        let fixture = Fixture::new();
        let (artifact, index) = fixture.select(&["second.mjs"], None).expect("精确合同应成功");
        assert_eq!(artifact, fs::canonicalize(fixture.root.join("second.mjs")).unwrap());
        assert_eq!(index, 0);
    }

    #[test]
    fn refuses_empty_and_ambiguous_contracts() {
        let fixture = Fixture::new();
        assert!(fixture.select(&[], None).is_err());
        assert!(fixture.select(&["first.mjs", "second.mjs"], None).is_err());
    }

    #[test]
    fn selects_explicit_contract_without_reordering() {
        let fixture = Fixture::new();
        let requested = fixture.root.join("second.mjs");
        let (artifact, index) = fixture.select(&["first.mjs", "second.mjs"], Some(&requested)).expect("显式合同应成功");
        assert_eq!(artifact, fs::canonicalize(requested).unwrap());
        assert_eq!(index, 1);
    }

    #[test]
    fn refuses_unlisted_and_duplicate_artifacts() {
        let fixture = Fixture::new();
        assert!(fixture.select(&["first.mjs"], Some(&fixture.root.join("old.mjs"))).is_err());
        assert!(fixture.select(&["first.mjs", "first.mjs"], Some(&fixture.root.join("first.mjs"))).is_err());
    }

    #[test]
    fn refuses_missing_entry_even_when_other_files_exist() {
        let fixture = Fixture::new();
        assert!(fixture.select(&["missing.mjs"], None).is_err());
        assert!(fixture.select(&["first"], None).is_err());
    }

    #[test]
    fn refuses_invalid_paths_and_directories() {
        let fixture = Fixture::new();
        for entry in ["", "../first.mjs", "./first.mjs", "."] {
            assert!(fixture.select(&[entry], None).is_err(), "{entry}");
        }
        let absolute = fixture.root.join("first.mjs");
        assert!(fixture.select(&[absolute.to_str().unwrap()], None).is_err());
        fs::create_dir(fixture.root.join("directory")).unwrap();
        assert!(fixture.select(&["directory"], None).is_err());
        fs::remove_dir(fixture.root.join("directory")).unwrap();
    }
}
