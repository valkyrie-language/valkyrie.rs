use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::{Display, Formatter},
    fs,
    path::{Path, PathBuf},
};

use miette::{Diagnostic, Severity};
use serde::{Deserialize, Serialize};
use valkyrie_compiler::CanonicalTarget;

use crate::manifest::{BuildTargetSpec, DependencySpec, ManifestError, ProjectManifest, WorkspaceManifest};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildRequest {
    pub project_dir: PathBuf,
    pub target: CanonicalTarget,
    pub output_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannedDependency {
    pub name: String,
    pub manifest_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannedHostContract {
    pub id: String,
    pub source_file: PathBuf,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannedHostProvider {
    pub contract: String,
    pub symbol: String,
    pub source_file: PathBuf,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannedProject {
    pub name: String,
    pub manifest_dir: PathBuf,
    pub manifest_path: PathBuf,
    pub source_files: Vec<PathBuf>,
    pub host_contracts: Vec<PlannedHostContract>,
    pub host_provider_candidates: Vec<PlannedHostProvider>,
    pub selected_host_providers: Vec<PlannedHostProvider>,
    pub build_target: BuildTargetSpec,
    pub dependencies: Vec<PlannedDependency>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildPlan {
    pub workspace_root: PathBuf,
    pub project: PlannedProject,
    pub output_dir: PathBuf,
}

#[derive(Debug)]
pub struct LegionWorkspace {
    pub root_dir: PathBuf,
    pub workspace_manifest: Option<WorkspaceManifest>,
    projects: BTreeMap<PathBuf, ProjectManifest>,
    projects_by_name: BTreeMap<String, PathBuf>,
}

#[derive(Debug)]
pub enum PlannerError {
    Io(std::io::Error),
    Manifest(ManifestError),
    MissingWorkspace(PathBuf),
    MissingProjectManifest(PathBuf),
    MissingBuildTarget { project: String, target: CanonicalTarget },
    MissingDependency { project: String, dependency: String },
    UnknownHostProviderContract { provider: String, contract: String, source_file: PathBuf, line: usize },
    ConflictingHostProviders { contract: String, providers: Vec<String> },
}

impl Display for PlannerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => Display::fmt(error, f),
            Self::Manifest(error) => Display::fmt(error, f),
            Self::MissingWorkspace(path) => write!(f, "cannot locate `legions.von` from {}", path.display()),
            Self::MissingProjectManifest(path) => write!(f, "cannot locate `legion.von` in {}", path.display()),
            Self::MissingBuildTarget { project, target } => {
                write!(f, "project '{}' does not declare build target '{}'", project, target)
            }
            Self::MissingDependency { project, dependency } => {
                write!(f, "project '{}' cannot resolve dependency '{}'", project, dependency)
            }
            Self::UnknownHostProviderContract { provider, contract, source_file, line } => {
                write!(f, "host provider '{}' references unknown host contract '{}' at {}:{}", provider, contract, source_file.display(), line)
            }
            Self::ConflictingHostProviders { contract, providers } => {
                write!(f, "host contract '{}' has multiple visible providers: {}", contract, providers.join(", "))
            }
        }
    }
}

impl std::error::Error for PlannerError {}

impl Diagnostic for PlannerError {
    fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        Some(Box::new(match self {
            PlannerError::Io(_) => "legion::planner::io",
            PlannerError::Manifest(_) => "legion::planner::manifest",
            PlannerError::MissingWorkspace(_) => "legion::planner::missing_workspace",
            PlannerError::MissingProjectManifest(_) => "legion::planner::missing_project_manifest",
            PlannerError::MissingBuildTarget { .. } => "legion::planner::missing_build_target",
            PlannerError::MissingDependency { .. } => "legion::planner::missing_dependency",
            PlannerError::UnknownHostProviderContract { .. } => "legion::planner::unknown_host_provider_contract",
            PlannerError::ConflictingHostProviders { .. } => "legion::planner::conflicting_host_providers",
        }))
    }

    fn severity(&self) -> Option<Severity> {
        Some(Severity::Error)
    }

    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        Some(Box::new(match self {
            PlannerError::Io(_) => "请确认工作区目录存在且当前进程有访问权限",
            PlannerError::Manifest(_) => "请修复 `legion.von` / `legions.von` 中的配置或 `VON` 语法",
            PlannerError::MissingWorkspace(_) => "请在工作区根目录放置 `legions.von`",
            PlannerError::MissingProjectManifest(_) => "请在项目目录放置 `legion.von`",
            PlannerError::MissingBuildTarget { .. } => "请在 `build` 段中声明对应 target",
            PlannerError::MissingDependency { .. } => "请确认依赖名称存在于 workspace 成员或项目依赖中",
            PlannerError::UnknownHostProviderContract { .. } => {
                "请确认 `[host_provider(...)]` 指向的 contract 标识与可见源码中的 `[host_contract]` 完全一致"
            }
            PlannerError::ConflictingHostProviders { .. } => {
                "请收窄有效依赖闭包，或移除重复的 provider，保证每个 `host_contract` 最多只有一个可见实现"
            }
        }))
    }

    fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
        match self {
            PlannerError::Manifest(error) => Some(error),
            _ => None,
        }
    }
}

impl From<std::io::Error> for PlannerError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<ManifestError> for PlannerError {
    fn from(value: ManifestError) -> Self {
        Self::Manifest(value)
    }
}

impl LegionWorkspace {
    pub fn discover(start: impl AsRef<Path>) -> Result<Self, PlannerError> {
        let start = start.as_ref();
        let workspace_root = find_workspace_root(start).ok_or_else(|| PlannerError::MissingWorkspace(start.to_path_buf()))?;
        let workspace_manifest_path = workspace_root.join("legions.von");
        let workspace_manifest = WorkspaceManifest::parse(&fs::read_to_string(&workspace_manifest_path)?)?;

        let mut projects = BTreeMap::new();
        let mut projects_by_name = BTreeMap::new();
        for member in &workspace_manifest.members {
            let member_dir = canonicalize_lossy(&workspace_root.join(member));
            let manifest_path = member_dir.join("legion.von");
            if !manifest_path.exists() {
                continue;
            }
            // 跳过无法解析的 manifest，避免单个测试项目阻塞整个工作区发现。
            // 这类项目通常是语法特性测试，其 manifest 字段不遵循标准格式。
            let manifest = match ProjectManifest::parse(&fs::read_to_string(&manifest_path)?) {
                Ok(manifest) => manifest,
                Err(error) => {
                    eprintln!("warning: 跳过无法解析的 manifest {}: {error}", manifest_path.display());
                    continue;
                }
            };
            projects_by_name.insert(manifest.name.clone(), member_dir.clone());
            // 同时按 workspace 成员路径的 basename 建立别名索引。
            // 例如 `projects/core` 的 manifest.name 可能是 `valkyrie-core`，
            // 但 auto_link/dependencies 中引用的是逻辑名 `core`（即目录名）。
            if let Some(basename) = Path::new(member).file_name().and_then(|s| s.to_str()) {
                projects_by_name.entry(basename.to_string()).or_insert_with(|| member_dir.clone());
            }
            projects.insert(member_dir, manifest);
        }

        Ok(Self { root_dir: workspace_root, workspace_manifest: Some(workspace_manifest), projects, projects_by_name })
    }

    pub fn project_manifest(&self, project_dir: &Path) -> Option<&ProjectManifest> {
        self.projects
            .get(project_dir)
            .or_else(|| self.projects.iter().find(|(candidate, _)| same_path(candidate, project_dir)).map(|(_, manifest)| manifest))
    }

    pub fn build_plan(&self, request: &BuildRequest) -> Result<BuildPlan, PlannerError> {
        let project_dir = canonicalize_lossy(&request.project_dir);
        let manifest = self.project_manifest(&project_dir).ok_or_else(|| PlannerError::MissingProjectManifest(project_dir.clone()))?;
        let build_target = select_build_target(manifest, &request.target)
            .ok_or_else(|| PlannerError::MissingBuildTarget { project: manifest.name.clone(), target: request.target })?;

        let dependencies = self.collect_dependencies(manifest, &request.target)?;
        let output_dir =
            request.output_dir.clone().unwrap_or_else(|| project_dir.join("dist").join(request.target.to_string().replace('-', "_")));

        // 收集源码闭包：包含项目自身及其所有依赖（含传递依赖）的源文件。
        // 这是模块系统的核心：让编译器能看到依赖项目定义的 struct/function/type。
        let mut visited = BTreeSet::new();
        let source_files = self.collect_source_closure(&project_dir, manifest, &request.target, &mut visited)?;
        let host_inventory = collect_host_inventory(&source_files)?;

        Ok(BuildPlan {
            workspace_root: self.root_dir.clone(),
            output_dir,
            project: PlannedProject {
                name: manifest.name.clone(),
                manifest_dir: project_dir.clone(),
                manifest_path: project_dir.join("legion.von"),
                source_files,
                host_contracts: host_inventory.contracts,
                host_provider_candidates: host_inventory.providers,
                selected_host_providers: host_inventory.selected_providers,
                build_target,
                dependencies,
            },
        })
    }

    pub fn build_plan_with_local_fallback(&self, request: &BuildRequest) -> Result<(BuildPlan, bool), PlannerError> {
        let project_dir = canonicalize_lossy(&request.project_dir);
        if let Some(manifest) = self.project_manifest(&project_dir) {
            return Ok((self.build_plan_for_manifest(project_dir, manifest, request)?, false));
        }

        let manifest_path = project_dir.join("legion.von");
        if !manifest_path.exists() {
            return Err(PlannerError::MissingProjectManifest(project_dir));
        }

        let manifest = ProjectManifest::parse(&fs::read_to_string(&manifest_path)?)?;
        Ok((self.build_plan_for_manifest(project_dir, &manifest, request)?, true))
    }

    /// 递归收集源码闭包：项目自身 + 所有依赖（含传递依赖）的源文件。
    ///
    /// 这是模块系统依赖管理的核心实现。通过递归遍历依赖图，
    /// 将所有相关项目的源文件收集到一起，供编译器做合并编译。
    ///
    /// 使用 `visited` 集合防止循环依赖导致的无限递归。
    fn collect_source_closure(
        &self,
        project_dir: &Path,
        manifest: &ProjectManifest,
        target: &CanonicalTarget,
        visited: &mut BTreeSet<PathBuf>,
    ) -> Result<Vec<PathBuf>, PlannerError> {
        let canonical_dir = canonicalize_lossy(project_dir);
        if visited.contains(&canonical_dir) {
            return Ok(Vec::new());
        }
        visited.insert(canonical_dir.clone());

        // 收集当前项目自身的源文件。
        let mut all_files = collect_source_files(&canonical_dir)?;

        // 收集当前项目的依赖名称（auto_link + 显式声明）。
        let mut dep_names = BTreeSet::new();
        if manifest.auto_link.core {
            dep_names.insert("core".to_string());
        }
        if manifest.auto_link.std {
            dep_names.insert("std".to_string());
        }
        for (dependency_name, dependency_spec) in &manifest.dependencies {
            if !matches!(dependency_spec, DependencySpec::Disabled) {
                dep_names.insert(dependency_name.clone());
            }
        }

        // 递归收集每个依赖的源文件。
        for dep_name in dep_names {
            if let Some(dep_dir) = self.projects_by_name.get(&dep_name) {
                if let Some(dep_manifest) = self.project_manifest(dep_dir) {
                    if !manifest_supports_target(dep_manifest, target) {
                        continue;
                    }
                    let dep_files = self.collect_source_closure(dep_dir, dep_manifest, target, visited)?;
                    all_files.extend(dep_files);
                }
            }
        }

        // 去重并排序，确保编译顺序稳定。
        all_files.sort();
        all_files.dedup();
        Ok(all_files)
    }

    fn build_plan_for_manifest(
        &self,
        project_dir: PathBuf,
        manifest: &ProjectManifest,
        request: &BuildRequest,
    ) -> Result<BuildPlan, PlannerError> {
        let build_target = select_build_target(manifest, &request.target)
            .ok_or_else(|| PlannerError::MissingBuildTarget { project: manifest.name.clone(), target: request.target.clone() })?;

        let dependencies = self.collect_dependencies(manifest, &request.target)?;
        let output_dir =
            request.output_dir.clone().unwrap_or_else(|| project_dir.join("dist").join(request.target.to_string().replace('-', "_")));

        let mut visited = BTreeSet::new();
        let source_files = self.collect_source_closure(&project_dir, manifest, &request.target, &mut visited)?;
        let host_inventory = collect_host_inventory(&source_files)?;

        Ok(BuildPlan {
            workspace_root: self.root_dir.clone(),
            output_dir,
            project: PlannedProject {
                name: manifest.name.clone(),
                manifest_dir: project_dir.clone(),
                manifest_path: project_dir.join("legion.von"),
                source_files,
                host_contracts: host_inventory.contracts,
                host_provider_candidates: host_inventory.providers,
                selected_host_providers: host_inventory.selected_providers,
                build_target,
                dependencies,
            },
        })
    }

    fn collect_dependencies(&self, manifest: &ProjectManifest, target: &CanonicalTarget) -> Result<Vec<PlannedDependency>, PlannerError> {
        let mut planned = Vec::new();
        let mut names = BTreeSet::new();

        if manifest.auto_link.core {
            names.insert("core".to_string());
        }
        if manifest.auto_link.std {
            names.insert("std".to_string());
        }
        for (dependency_name, dependency_spec) in &manifest.dependencies {
            if !matches!(dependency_spec, DependencySpec::Disabled) {
                names.insert(dependency_name.clone());
            }
        }

        for dependency_name in names {
            let Some(manifest_dir) = self.projects_by_name.get(&dependency_name).cloned()
            else {
                return Err(PlannerError::MissingDependency { project: manifest.name.clone(), dependency: dependency_name });
            };
            let Some(dependency_manifest) = self.project_manifest(&manifest_dir)
            else {
                return Err(PlannerError::MissingProjectManifest(manifest_dir));
            };
            if !manifest_supports_target(dependency_manifest, target) {
                continue;
            }
            planned.push(PlannedDependency { name: dependency_name, manifest_dir });
        }

        planned.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(planned)
    }
}

pub fn canonical_target(target: &str) -> Result<CanonicalTarget, valkyrie_compiler::CanonicalTargetParseError> {
    target.parse()
}

fn select_build_target(manifest: &ProjectManifest, target: &CanonicalTarget) -> Option<BuildTargetSpec> {
    manifest.build.iter().find(|item| item.target == *target).cloned()
}

fn collect_source_files(project_dir: &Path) -> Result<Vec<PathBuf>, PlannerError> {
    let mut files = Vec::new();
    let source_dir = project_dir.join("source");
    if source_dir.exists() {
        collect_v_files(&source_dir, &mut files)?;
    }
    files.sort();
    Ok(files)
}

fn collect_v_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), PlannerError> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_v_files(&path, files)?;
            continue;
        }
        if path.extension().is_some_and(|ext| ext == "v") {
            files.push(path);
        }
    }
    Ok(())
}

fn find_workspace_root(start: &Path) -> Option<PathBuf> {
    let start_dir = if start.is_dir() { start.to_path_buf() } else { start.parent()?.to_path_buf() };
    // 规范化为绝对路径，否则相对路径（如 `.`）的 parent() 会返回 None，导致无法向上遍历。
    let start_dir = canonicalize_lossy(&start_dir);

    let mut current = Some(start_dir.as_path());
    while let Some(dir) = current {
        if dir.join("legions.von").exists() {
            return Some(dir.to_path_buf());
        }
        current = dir.parent();
    }
    None
}

fn canonicalize_lossy(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn same_path(lhs: &Path, rhs: &Path) -> bool {
    normalize_path_for_lookup(lhs) == normalize_path_for_lookup(rhs)
}

fn normalize_path_for_lookup(path: &Path) -> String {
    let raw = path.to_string_lossy();
    let stripped = raw.strip_prefix(r"\\?\").unwrap_or(&raw);
    if cfg!(windows) {
        stripped.to_ascii_lowercase()
    }
    else {
        stripped.to_string()
    }
}

fn manifest_supports_target(manifest: &ProjectManifest, target: &CanonicalTarget) -> bool {
    let Some(sdk_vendor) = &manifest.sdk_vendor
    else {
        return true;
    };
    if sdk_vendor.targets.is_empty() {
        return true;
    }

    sdk_vendor
        .targets
        .iter()
        .any(|item| item == &target.to_string() || item.parse::<CanonicalTarget>().map(|parsed| parsed == *target).unwrap_or(false))
}

#[derive(Debug, Default)]
struct HostInventory {
    contracts: Vec<PlannedHostContract>,
    providers: Vec<PlannedHostProvider>,
    selected_providers: Vec<PlannedHostProvider>,
}

fn collect_host_inventory(source_files: &[PathBuf]) -> Result<HostInventory, PlannerError> {
    let mut inventory = HostInventory::default();
    for source_file in source_files {
        let source = fs::read_to_string(source_file)?;
        collect_host_attributes_from_file(source_file, &source, &mut inventory);
    }

    inventory
        .contracts
        .sort_by(|left, right| left.id.cmp(&right.id).then(left.source_file.cmp(&right.source_file)).then(left.line.cmp(&right.line)));
    inventory.providers.sort_by(|left, right| {
        left.contract
            .cmp(&right.contract)
            .then(left.symbol.cmp(&right.symbol))
            .then(left.source_file.cmp(&right.source_file))
            .then(left.line.cmp(&right.line))
    });
    inventory.selected_providers = select_host_providers(&inventory.contracts, &inventory.providers)?;
    Ok(inventory)
}

fn collect_host_attributes_from_file(source_file: &Path, source: &str, inventory: &mut HostInventory) {
    let mut namespace = String::new();
    let mut pending_host_contract: Option<usize> = None;
    let mut pending_host_providers: Vec<(String, usize)> = Vec::new();
    let mut owner_stack: Vec<(String, usize)> = Vec::new();
    let mut brace_depth: usize = 0;

    for (index, raw_line) in source.lines().enumerate() {
        let line_number = index + 1;
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("//") || line.starts_with('⍝') {
            continue;
        }

        let open_brace_count = line.chars().filter(|ch| *ch == '{').count();
        let close_brace_count = line.chars().filter(|ch| *ch == '}').count();
        let parsed_owner = parse_owner_name(line).map(str::to_string);

        if let Some(parsed_namespace) = parse_namespace(line) {
            namespace = parsed_namespace.to_string();
            continue;
        }

        if let Some(attributes) = parse_attribute_items(line) {
            for attribute in attributes {
                if attribute == "host_contract" {
                    pending_host_contract = Some(line_number);
                    continue;
                }

                if let Some(contract) = parse_host_provider_attribute(attribute) {
                    pending_host_providers.push((contract.to_string(), line_number));
                }
            }
            continue;
        }

        let Some(symbol_name) = parse_symbol_name(line)
        else {
            let next_brace_depth = brace_depth.saturating_add(open_brace_count).saturating_sub(close_brace_count);
            if let Some(owner_name) = parsed_owner {
                if next_brace_depth > brace_depth {
                    owner_stack.push((owner_name, next_brace_depth));
                }
            }
            brace_depth = next_brace_depth;
            while owner_stack.last().map(|(_, depth)| brace_depth < *depth).unwrap_or(false) {
                owner_stack.pop();
            }
            continue;
        };
        let qualified_symbol = qualify_symbol_name(&namespace, symbol_name, owner_stack.last().map(|(owner, _)| owner.as_str()));

        if let Some(contract_line) = pending_host_contract.take() {
            inventory.contracts.push(PlannedHostContract {
                id: qualified_symbol.clone(),
                source_file: source_file.to_path_buf(),
                line: contract_line,
            });
        }

        for (contract, provider_line) in pending_host_providers.drain(..) {
            inventory.providers.push(PlannedHostProvider {
                contract,
                symbol: qualified_symbol.clone(),
                source_file: source_file.to_path_buf(),
                line: provider_line,
            });
        }

        let next_brace_depth = brace_depth.saturating_add(open_brace_count).saturating_sub(close_brace_count);
        if let Some(owner_name) = parsed_owner {
            if next_brace_depth > brace_depth {
                owner_stack.push((owner_name, next_brace_depth));
            }
        }
        brace_depth = next_brace_depth;
        while owner_stack.last().map(|(_, depth)| brace_depth < *depth).unwrap_or(false) {
            owner_stack.pop();
        }
    }
}

fn select_host_providers(
    contracts: &[PlannedHostContract],
    providers: &[PlannedHostProvider],
) -> Result<Vec<PlannedHostProvider>, PlannerError> {
    let contract_ids: BTreeSet<String> = contracts.iter().map(|item| normalize_contract_reference(&item.id)).collect();
    for provider in providers {
        if !contract_ids.contains(&normalize_contract_reference(&provider.contract)) {
            return Err(PlannerError::UnknownHostProviderContract {
                provider: provider.symbol.clone(),
                contract: provider.contract.clone(),
                source_file: provider.source_file.clone(),
                line: provider.line,
            });
        }
    }

    let mut providers_by_contract: BTreeMap<String, Vec<&PlannedHostProvider>> = BTreeMap::new();
    for provider in providers {
        providers_by_contract.entry(normalize_contract_reference(&provider.contract)).or_default().push(provider);
    }

    let mut selected = Vec::new();
    for contract in contracts {
        let Some(candidates) = providers_by_contract.get(&normalize_contract_reference(&contract.id))
        else {
            continue;
        };

        if candidates.len() > 1 {
            return Err(PlannerError::ConflictingHostProviders {
                contract: contract.id.clone(),
                providers: candidates.iter().map(|item| format!("{} at {}:{}", item.symbol, item.source_file.display(), item.line)).collect(),
            });
        }

        selected.push((*candidates[0]).clone());
    }

    selected.sort_by(|left, right| {
        left.contract
            .cmp(&right.contract)
            .then(left.symbol.cmp(&right.symbol))
            .then(left.source_file.cmp(&right.source_file))
            .then(left.line.cmp(&right.line))
    });
    Ok(selected)
}

fn normalize_contract_reference(contract: &str) -> String {
    contract.replace("::", ".")
}

fn parse_namespace(line: &str) -> Option<&str> {
    line.strip_prefix("namespace ").or_else(|| line.strip_prefix("namespace! ")).and_then(|value| value.strip_suffix(';')).map(str::trim)
}

fn parse_attribute_items(line: &str) -> Option<Vec<&str>> {
    let inner = line.strip_prefix('[')?.strip_suffix(']')?.trim();
    if inner.is_empty() {
        return Some(Vec::new());
    }

    let mut items = Vec::new();
    let mut start = 0;
    let mut paren_depth = 0usize;
    let mut in_string = false;
    let mut previous_was_escape = false;

    for (index, ch) in inner.char_indices() {
        match ch {
            '"' if !previous_was_escape => {
                in_string = !in_string;
            }
            '(' if !in_string => {
                paren_depth += 1;
            }
            ')' if !in_string => {
                paren_depth = paren_depth.saturating_sub(1);
            }
            ',' if !in_string && paren_depth == 0 => {
                items.push(inner[start..index].trim());
                start = index + ch.len_utf8();
            }
            _ => {}
        }

        if ch == '\\' {
            previous_was_escape = !previous_was_escape;
        }
        else {
            previous_was_escape = false;
        }
    }

    items.push(inner[start..].trim());
    Some(items)
}

fn parse_host_provider_attribute(attribute: &str) -> Option<&str> {
    let inner = attribute.strip_prefix("host_provider(")?.strip_suffix(')')?.trim();
    if let Some(value) = inner.strip_prefix('"').and_then(|rest| rest.strip_suffix('"')) {
        return Some(value);
    }

    Some(inner)
}

fn parse_symbol_name(line: &str) -> Option<&str> {
    let micro_offset = line.find("micro ")?;
    let rest = &line[micro_offset + "micro ".len()..];
    let end = rest.find(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '.')).unwrap_or(rest.len());
    if end == 0 {
        return None;
    }
    Some(&rest[..end])
}

fn parse_owner_name(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("imply ").or_else(|| line.strip_prefix("class "))?;
    let end = rest.find(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '.')).unwrap_or(rest.len());
    if end == 0 {
        return None;
    }
    Some(&rest[..end])
}

fn qualify_symbol_name(namespace: &str, symbol_name: &str, owner_name: Option<&str>) -> String {
    if let Some(owner_name) = owner_name {
        let owner_basename = owner_name.rsplit('.').next().unwrap_or(owner_name);
        if namespace.is_empty() {
            return format!("{}::{}", owner_basename, symbol_name);
        }

        return format!("{}::{}::{}", namespace.replace('.', "::"), owner_basename, symbol_name);
    }

    if namespace.is_empty() {
        return symbol_name.to_string();
    }
    format!("{}.{}", namespace, symbol_name)
}
