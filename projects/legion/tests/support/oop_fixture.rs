use legion::{
    CanonicalTarget,
    cmds::{
        build::{BuildArgs, run as run_build},
        run::RunContract as LegionRunContract,
    },
};
use nyar::{RuntimeFixtureResult, assert_or_regenerate_yaml_sidecar, collect_fixture_cases_with_extensions, load_optional_yaml_sidecar};
use nyar_runner::{RuntimeContract, RuntimeFamily};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

use crate::support::{
    SmokeProject, create_script_project_with_manifest, create_smoke_project_with_manifest, runtime_fixture::regenerate_enabled,
};

const DEFAULT_OOP_TARGET_NAMES: [&str; 1] = ["clr"];

/// OOP fixture sidecar: per-target expect map plus optional target declarations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(from = "OopFixtureSpecRaw")]
pub struct OopFixtureSpec {
    #[serde(default)]
    pub targets: Vec<String>,
    #[serde(default)]
    pub expect: BTreeMap<String, RuntimeFixtureResult>,
    /// 标记为已知语言缺口；常规测试跳过，仅在重生成时更新基线。
    #[serde(default)]
    pub known_gap: bool,
}

/// Raw sidecar shape supporting legacy flat clr-only baselines.
#[derive(Debug, Clone, Deserialize, Default)]
struct OopFixtureSpecRaw {
    #[serde(default)]
    targets: Vec<String>,
    #[serde(default)]
    expect: BTreeMap<String, RuntimeFixtureResult>,
    #[serde(default)]
    known_gap: bool,
    #[serde(default)]
    success: bool,
    #[serde(default)]
    stdout: Vec<String>,
    #[serde(default)]
    stderr: Vec<String>,
    #[serde(default)]
    errors: Vec<String>,
    result: Option<i32>,
}

impl From<OopFixtureSpecRaw> for OopFixtureSpec {
    fn from(raw: OopFixtureSpecRaw) -> Self {
        raw.into_normalized()
    }
}

impl OopFixtureSpecRaw {
    fn into_normalized(self) -> OopFixtureSpec {
        let mut expect = self.expect;
        if expect.is_empty() {
            expect.insert(
                "clr".to_string(),
                RuntimeFixtureResult {
                    success: self.success,
                    stdout: self.stdout,
                    stderr: self.stderr,
                    allow_stderr: false,
                    errors: self.errors,
                    result: self.result,
                },
            );
        }
        OopFixtureSpec { targets: self.targets, expect, known_gap: self.known_gap }
    }
}

pub fn load_oop_fixture_spec(fixture_path: &Path) -> Option<OopFixtureSpec> {
    load_optional_yaml_sidecar::<OopFixtureSpec>(fixture_path)
}

pub fn collect_oop_fixture_cases(root: &Path) -> Vec<PathBuf> {
    collect_fixture_cases_with_extensions(root, &["valkyrie"])
}

pub fn can_run_oop_fixtures(fixtures: &[PathBuf]) -> bool {
    let mut required = BTreeSet::new();
    for fixture_path in fixtures {
        for target in resolve_oop_fixture_targets(fixture_path) {
            if let Some((label, command)) = target.required_command() {
                required.insert((label, command));
            }
        }
    }

    for (label, command) in required {
        if !command_exists(command) {
            eprintln!("skip oop fixtures: missing {}", label);
            return false;
        }
    }
    true
}

pub fn verify_oop_fixture(fixture_path: &Path) {
    let existing_spec = load_oop_fixture_spec(fixture_path);
    if let Some(spec) = &existing_spec {
        if spec.known_gap && !regenerate_enabled() {
            eprintln!("skip known-gap oop fixture: {}", fixture_path.display());
            return;
        }
    }

    let source =
        fs::read_to_string(fixture_path).unwrap_or_else(|error| panic!("failed to read oop fixture '{}': {}", fixture_path.display(), error));
    let prepared = prepare_oop_source(&source);
    let project_name = oop_project_name(fixture_path);
    let targets = resolve_oop_fixture_targets(fixture_path);
    let project = create_oop_project(fixture_path, &project_name, &prepared, &targets);
    let previous_known_gap = load_oop_fixture_spec(fixture_path).is_some_and(|spec| spec.known_gap);

    let mut expect = BTreeMap::new();
    for target in &targets {
        let allow_stderr = existing_spec
            .as_ref()
            .and_then(|spec| spec.expect.get(target.as_str()))
            .is_some_and(|result| result.allow_stderr || !result.stderr.is_empty());
        let record_allow_stderr =
            existing_spec.as_ref().and_then(|spec| spec.expect.get(target.as_str())).is_some_and(|result| result.allow_stderr);
        expect.insert(target.as_str().to_string(), execute_oop_target(&project, &project_name, *target, allow_stderr, record_allow_stderr));
    }

    let all_success = expect.values().all(|result| result.success);
    let observed = OopFixtureSpec {
        targets: targets.iter().map(|target| target.as_str().to_string()).collect(),
        expect,
        known_gap: previous_known_gap && !all_success,
    };
    assert_or_regenerate_yaml_sidecar(fixture_path, &observed, regenerate_enabled());
}

fn resolve_oop_fixture_targets(fixture_path: &Path) -> Vec<OopFixtureTarget> {
    let declared = load_oop_fixture_spec(fixture_path).map(|spec| spec.targets).unwrap_or_default();
    let target_names =
        if declared.is_empty() { DEFAULT_OOP_TARGET_NAMES.iter().map(|target| (*target).to_string()).collect() } else { declared };

    target_names
        .into_iter()
        .map(|target| {
            OopFixtureTarget::parse(&target)
                .unwrap_or_else(|| panic!("oop fixture '{}' contains unsupported target '{}'", fixture_path.display(), target))
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum OopFixtureTarget {
    Clr,
    Jvm,
    Native,
    Wasi,
}

impl OopFixtureTarget {
    fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "clr" | "clr-microsoft-unknown-managed" => Some(Self::Clr),
            "jvm" | "jvm-openjdk-unknown-managed" => Some(Self::Jvm),
            "native" | "x86_64-pc-windows-msvc" | "x86_64-unknown-linux-gnu" | "x86_64-apple-darwin" => Some(Self::Native),
            "wasi" => Some(Self::Wasi),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Clr => "clr",
            Self::Jvm => "jvm",
            Self::Native => "native",
            Self::Wasi => "wasi",
        }
    }

    fn canonical_target(self) -> CanonicalTarget {
        match self {
            Self::Clr => CanonicalTarget::clr(),
            Self::Jvm => CanonicalTarget::parse("jvm-openjdk-unknown-managed").unwrap(),
            Self::Wasi => CanonicalTarget::parse("wasi").unwrap(),
            Self::Native => {
                #[cfg(windows)]
                {
                    CanonicalTarget::parse("x86_64-pc-windows-msvc").unwrap()
                }
                #[cfg(all(unix, not(target_os = "macos")))]
                {
                    CanonicalTarget::parse("x86_64-unknown-linux-gnu").unwrap()
                }
                #[cfg(target_os = "macos")]
                {
                    CanonicalTarget::parse("x86_64-apple-darwin").unwrap()
                }
            }
        }
    }

    fn manifest_target(self) -> &'static str {
        match self {
            Self::Clr => "clr",
            Self::Jvm => "jvm-openjdk-unknown-managed",
            Self::Wasi => "wasi",
            Self::Native => {
                #[cfg(windows)]
                {
                    "x86_64-pc-windows-msvc"
                }
                #[cfg(all(unix, not(target_os = "macos")))]
                {
                    "x86_64-unknown-linux-gnu"
                }
                #[cfg(target_os = "macos")]
                {
                    "x86_64-apple-darwin"
                }
            }
        }
    }

    fn runtime_family(self) -> RuntimeFamily {
        match self {
            Self::Clr => RuntimeFamily::Clr,
            Self::Jvm => RuntimeFamily::Jvm,
            Self::Wasi => RuntimeFamily::Wasi,
            Self::Native => RuntimeFamily::Windows,
        }
    }

    fn required_command(self) -> Option<(&'static str, &'static str)> {
        match self {
            Self::Clr => Some(("dotnet", "dotnet")),
            Self::Jvm => Some(("java", "java")),
            Self::Wasi => Some(("wasmtime", "wasmtime")),
            Self::Native => None,
        }
    }
}

fn oop_project_name(fixture_path: &Path) -> String {
    let stem = fixture_path.file_stem().and_then(|value| value.to_str()).unwrap_or("oop_fixture");
    format!("oop_{}", sanitize_identifier(stem))
}

fn sanitize_identifier(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            result.push(ch.to_ascii_lowercase());
        }
        else {
            result.push('_');
        }
    }
    if result.is_empty() { "oop_fixture".to_string() } else { result }
}

fn create_oop_project(fixture_path: &Path, project_name: &str, source: &str, targets: &[OopFixtureTarget]) -> SmokeProject {
    let prefix = format!("legion-oop-{}", sanitize_identifier(fixture_path.file_stem().and_then(|value| value.to_str()).unwrap_or("fixture")));
    let manifest = oop_manifest(project_name, targets);
    if source.contains("micro main") || source.contains("[main]") {
        create_smoke_project_with_manifest(&prefix, &manifest, source)
    }
    else {
        create_script_project_with_manifest(&prefix, &manifest, source)
    }
}

/// Wrap top-level script statements in `[main] micro main()` while preserving type declarations.
pub fn prepare_oop_source(source: &str) -> String {
    if source.contains("[main]") || source.contains("micro main") {
        return source.to_string();
    }

    let (declarations, script) = split_declarations_and_script(source);
    if script.trim().is_empty() {
        if declarations.contains("micro ") {
            return format!("{declarations}\n\n[main]\nmicro main() {{\n}}\n");
        }
        return format!("{declarations}\n\n[main]\nmicro main() {{\n}}\n");
    }

    format!("{declarations}\n\n[main]\nmicro main() {{\n{script}\n}}\n")
}

fn split_declarations_and_script(source: &str) -> (String, String) {
    let mut declarations = String::new();
    let mut script = String::new();
    let mut depth = 0i32;

    for line in source.lines() {
        let trimmed = line.trim();
        let delta = brace_delta(line);
        let in_declaration = depth > 0 || is_top_level_declaration(trimmed);

        if in_declaration {
            declarations.push_str(line);
            declarations.push('\n');
            depth += delta;
            continue;
        }

        if trimmed.is_empty() || trimmed.starts_with("//") {
            if script.is_empty() {
                declarations.push_str(line);
                declarations.push('\n');
            }
            else {
                script.push_str(line);
                script.push('\n');
            }
            continue;
        }

        script.push_str(line);
        script.push('\n');
    }

    (declarations, script)
}

fn is_top_level_declaration(trimmed: &str) -> bool {
    let mut rest = trimmed;
    loop {
        let Some((modifier, suffix)) = declaration_modifier_prefix(rest)
        else {
            break;
        };
        rest = suffix.trim_start();
        if rest.is_empty() {
            return false;
        }
    }

    rest.starts_with("micro ")
        || rest.starts_with("mezzo ")
        || rest.starts_with("macro ")
        || rest.starts_with("class ")
        || rest.starts_with("structure ")
        || rest.starts_with("struct ")
        || rest.starts_with("trait ")
        || rest.starts_with("singleton ")
        || rest.starts_with("widget ")
        || rest.starts_with("neural ")
        || rest.starts_with("imply ")
        || rest.starts_with("namespace ")
        || rest.starts_with("using ")
        || rest.starts_with("union ")
        || rest.starts_with("unite ")
        || rest.starts_with("enums ")
        || rest.starts_with("flags ")
        || rest.starts_with("type ")
        || rest.starts_with("attribute ")
        || rest.starts_with("tests ")
}

fn declaration_modifier_prefix(trimmed: &str) -> Option<(&str, &str)> {
    for modifier in
        ["public", "private", "protected", "internal", "open", "sealed", "abstract", "final", "readonly", "virtual", "override", "static"]
    {
        if let Some(suffix) = trimmed.strip_prefix(modifier) {
            if suffix.is_empty() || suffix.starts_with(char::is_whitespace) {
                return Some((modifier, suffix));
            }
        }
    }
    None
}

fn brace_delta(line: &str) -> i32 {
    line.chars()
        .map(|ch| match ch {
            '{' => 1,
            '}' => -1,
            _ => 0,
        })
        .sum()
}

fn oop_manifest(project_name: &str, targets: &[OopFixtureTarget]) -> String {
    let build = targets
        .iter()
        .map(|target| match target {
            OopFixtureTarget::Clr => r#"        {
            target: "clr",
            msil: true
        }"#
            .to_string(),
            _ => format!(
                r#"        {{
            target: "{}"
        }}"#,
                target.manifest_target()
            ),
        })
        .collect::<Vec<_>>()
        .join(",\n");

    format!(
        r#"{{
    name: "{project_name}",
    version: "0.1.0",
    dependencies: {{
        "std": false,
        "core": false
    }},
    build: [
{build}
    ]
}}
"#
    )
}

fn execute_oop_target(
    project: &SmokeProject,
    project_name: &str,
    target: OopFixtureTarget,
    allow_stderr: bool,
    record_allow_stderr: bool,
) -> RuntimeFixtureResult {
    let output_dir = project.project_dir.join("dist").join(target.as_str());
    let build_status = run_build(&BuildArgs {
        project_dir: project.project_dir.clone(),
        target: target.canonical_target(),
        output_dir: Some(output_dir.clone()),
        workspace: false,
        debug_artifacts: false,
    });

    let Ok(build_status) = build_status
    else {
        let error = build_status.err().unwrap();
        return failed_result(format!("build failed for {}: {}", target.as_str(), error));
    };

    if build_status != ExitCode::SUCCESS {
        return failed_result(format!("build returned non-zero status for {}", target.as_str()));
    }

    let command = match prepare_runtime_command(&output_dir, project_name, target.runtime_family()) {
        Ok(command) => command,
        Err(error) => return failed_result(format!("failed to prepare runtime command for {}: {}", target.as_str(), error)),
    };
    capture_runtime_command(&project.project_dir, &command.command, &command.args, allow_stderr, record_allow_stderr)
}

struct PreparedCommand {
    command: String,
    args: Vec<String>,
}

struct ParsedRunContract {
    logical_entry: String,
    physical_entry: String,
}

fn prepare_runtime_command(output_dir: &Path, project_name: &str, family: RuntimeFamily) -> Result<PreparedCommand, String> {
    let contract = read_run_contract(output_dir)?;
    let artifact = discover_artifact(output_dir, project_name, family, &contract)?;
    let classpath = if artifact.extension().and_then(|value| value.to_str()).is_some_and(|value| value.eq_ignore_ascii_case("jar")) {
        artifact.clone()
    }
    else {
        output_dir.to_path_buf()
    };
    let entry = if contract.logical_entry.is_empty() {
        artifact.file_stem().and_then(|value| value.to_str()).unwrap_or_default().to_string()
    }
    else {
        contract.logical_entry.clone()
    };
    let template = family.default_template(Some(RuntimeContract {
        logical_entry: (!contract.logical_entry.is_empty()).then_some(contract.logical_entry.as_str()),
        physical_entry: Some(contract.physical_entry.as_str()),
        wasi_p3: false,
    }));
    let command = template.prepare_command(&artifact, &classpath, &entry);
    Ok(PreparedCommand { command: command.command, args: command.args })
}

fn read_run_contract(output_dir: &Path) -> Result<ParsedRunContract, String> {
    LegionRunContract::read_from_output_dir(output_dir)
        .map_err(|error| format!("failed to read run-contracts.txt in '{}': {}", output_dir.display(), error))?
        .map(|contract| ParsedRunContract { logical_entry: contract.logical_entry, physical_entry: contract.physical_entry })
        .ok_or_else(|| format!("missing run-contracts.txt in '{}'", output_dir.display()))
}

fn discover_artifact(output_dir: &Path, project_name: &str, family: RuntimeFamily, contract: &ParsedRunContract) -> Result<PathBuf, String> {
    let mut files = fs::read_dir(output_dir)
        .map_err(|error| format!("failed to list '{}': {}", output_dir.display(), error))?
        .filter_map(|entry| entry.ok().map(|item| item.path()))
        .filter(|path| path.is_file())
        .collect::<Vec<_>>();
    files.sort();

    if !contract.physical_entry.is_empty() {
        if let Some(path) = files.iter().find(|path| {
            path.file_name().and_then(|value| value.to_str()).is_some_and(|value| value.eq_ignore_ascii_case(&contract.physical_entry))
        }) {
            return Ok(path.clone());
        }
    }

    let preferred = match family {
        RuntimeFamily::Clr => &["dll", "exe"][..],
        RuntimeFamily::Jvm => &["jar", "class"][..],
        RuntimeFamily::Node => &["mjs", "js", "wasm"][..],
        RuntimeFamily::Windows => &["exe", ""][..],
        RuntimeFamily::Wasi => &["wasm"][..],
        RuntimeFamily::NyarVm => &["nyar", "json"][..],
    };

    for extension in preferred {
        if let Some(path) = files.iter().find(|path| {
            path.extension().and_then(|value| value.to_str()).is_some_and(|value| value.eq_ignore_ascii_case(extension))
                && path.file_stem().and_then(|value| value.to_str()).is_some_and(|value| matches_project_name(value, project_name))
        }) {
            return Ok(path.clone());
        }
    }

    for extension in preferred {
        if let Some(path) = files
            .iter()
            .find(|path| path.extension().and_then(|value| value.to_str()).is_some_and(|value| value.eq_ignore_ascii_case(extension)))
        {
            return Ok(path.clone());
        }
    }

    Err(format!("no runnable artifact found in '{}'", output_dir.display()))
}

fn matches_project_name(file_stem: &str, project_name: &str) -> bool {
    file_stem.eq_ignore_ascii_case(project_name)
        || file_stem.replace('_', ".").eq_ignore_ascii_case(project_name)
        || file_stem.replace('.', "_").eq_ignore_ascii_case(project_name)
}

fn capture_runtime_command(cwd: &Path, program: &str, args: &[String], allow_stderr: bool, record_allow_stderr: bool) -> RuntimeFixtureResult {
    match Command::new(program).args(args).current_dir(cwd).output() {
        Ok(output) => {
            let success = process_success_with_stderr_policy(output.status.success(), &output.stderr, allow_stderr);
            let stdout = normalize_output_lines(&output.stdout);
            let stderr = normalize_output_lines(&output.stderr);
            let result = output.status.code();
            let errors = if success { Vec::new() } else { vec![format!("process exited with code {}", result.unwrap_or(1))] };
            RuntimeFixtureResult { success, stdout, stderr, allow_stderr: record_allow_stderr, errors, result }
        }
        Err(error) => failed_result(format!("failed to start '{}': {}", program, error)),
    }
}

fn normalize_output_lines(bytes: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(bytes).replace("\r\n", "\n").lines().map(|line| line.to_string()).collect()
}

fn process_success_with_stderr_policy(exit_success: bool, stderr: &[u8], allow_stderr: bool) -> bool {
    if !exit_success {
        return false;
    }
    if allow_stderr {
        return true;
    }
    String::from_utf8_lossy(stderr).trim().is_empty()
}

fn failed_result(message: String) -> RuntimeFixtureResult {
    RuntimeFixtureResult { success: false, stdout: Vec::new(), stderr: Vec::new(), allow_stderr: false, errors: vec![message], result: None }
}

fn command_exists(command: &str) -> bool {
    let Some(path) = env::var_os("PATH")
    else {
        return false;
    };

    for dir in env::split_paths(&path) {
        for candidate in candidate_command_paths(&dir, command) {
            if candidate.is_file() {
                return true;
            }
        }
    }
    false
}

fn candidate_command_paths(dir: &Path, command: &str) -> Vec<PathBuf> {
    let base = dir.join(command);
    if Path::new(command).extension().is_some() {
        return vec![base];
    }

    let mut candidates = vec![base.clone()];
    if cfg!(windows) {
        let extensions = env::var("PATHEXT")
            .ok()
            .map(|value| value.split(';').filter(|item| !item.is_empty()).map(|item| item.to_ascii_lowercase()).collect::<Vec<_>>())
            .unwrap_or_else(|| vec![".exe".to_string(), ".cmd".to_string(), ".bat".to_string(), ".com".to_string()]);
        for ext in extensions {
            candidates.push(dir.join(format!("{command}{ext}")));
        }
    }
    candidates
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_legacy_flat_sidecar_to_expect_clr() {
        let raw = OopFixtureSpecRaw {
            targets: Vec::new(),
            expect: BTreeMap::new(),
            known_gap: false,
            success: true,
            stdout: vec!["hello".to_string()],
            stderr: Vec::new(),
            errors: Vec::new(),
            result: Some(0),
        };
        let spec = raw.into_normalized();
        assert_eq!(spec.expect["clr"].stdout, vec!["hello".to_string()]);
        assert!(spec.expect["clr"].success);
    }
}
