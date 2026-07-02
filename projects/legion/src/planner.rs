use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::{Display, Formatter},
    fs,
    path::{Path, PathBuf},
};

use miette::{Diagnostic, Severity};
use nyar_language::CanonicalTarget;
use nyar_package_manager::PackageManager as PackageLegion;
use serde::{Deserialize, Serialize};

use crate::manifest::{BuildTargetSpec, DependencySourcePreference, DependencySpec, ManifestError, ProjectManifest, WorkspaceManifest};

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

/// One isolated frontend compilation unit. Dependencies are represented by
/// their exported semantic contracts, never by concatenating their source
/// text into this group's parser input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannedSemanticSourceGroup {
    pub name: String,
    pub manifest_dir: PathBuf,
    pub source_files: Vec<PathBuf>,
    pub direct_dependencies: Vec<String>,
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
    /// Dependency-first semantic compilation groups. `source_files` remains
    /// the complete input closure for hashes, host inventory and run digests.
    pub semantic_source_groups: Vec<PlannedSemanticSourceGroup>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectResolutionMode {
    Workspace,
    Package,
    Script,
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
    ForcedWorkspaceDependencyMissing { project: String, dependency: String },
    RegistryDependencyMissingVersion { project: String, dependency: String },
    RegistryDependencyInstallFailed { project: String, dependency: String, version: String, registry: String, reason: String },
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
            Self::ForcedWorkspaceDependencyMissing { project, dependency } => {
                write!(f, "project '{}' requires workspace dependency '{}', but no workspace member was found", project, dependency)
            }
            Self::RegistryDependencyMissingVersion { project, dependency } => {
                write!(f, "project '{}' sets dependency '{}' source=registry but no version is provided", project, dependency)
            }
            Self::RegistryDependencyInstallFailed { project, dependency, version, registry, reason } => {
                write!(
                    f,
                    "project '{}' dependency '{}' failed to install from registry '{}' version '{}': {}",
                    project, dependency, registry, version, reason
                )
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
            PlannerError::ForcedWorkspaceDependencyMissing { .. } => "legion::planner::forced_workspace_dependency_missing",
            PlannerError::RegistryDependencyMissingVersion { .. } => "legion::planner::registry_dependency_missing_version",
            PlannerError::RegistryDependencyInstallFailed { .. } => "legion::planner::registry_dependency_install_failed",
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
            PlannerError::ForcedWorkspaceDependencyMissing { .. } => {
                "依赖已声明 `source: workspace`，请确认该依赖项目在当前 `legions.von`（含嵌套 members）中"
            }
            PlannerError::RegistryDependencyMissingVersion { .. } => "依赖已声明 `source: registry`，请同时提供 `version`",
            PlannerError::RegistryDependencyInstallFailed { .. } => {
                "请检查 registry 名称、网络连接、包名与版本是否可用，必要时先执行 `legion install` 验证"
            }
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
        register_workspace_members(&workspace_root, &workspace_manifest, &mut projects, &mut projects_by_name)?;

        Ok(Self { root_dir: workspace_root, workspace_manifest: Some(workspace_manifest), projects, projects_by_name })
    }

    pub fn discover_for_project(start: impl AsRef<Path>) -> Result<Self, PlannerError> {
        let start = start.as_ref();
        if let Some(workspace_root) = find_workspace_root(start) {
            return Self::discover(workspace_root);
        }

        let project_dir = resolve_project_root(start).ok_or_else(|| PlannerError::MissingProjectManifest(search_start_dir(start)))?;

        Ok(Self { root_dir: project_dir, workspace_manifest: None, projects: BTreeMap::new(), projects_by_name: BTreeMap::new() })
    }

    pub fn project_manifest(&self, project_dir: &Path) -> Option<&ProjectManifest> {
        self.projects
            .get(project_dir)
            .or_else(|| self.projects.iter().find(|(candidate, _)| same_path(candidate, project_dir)).map(|(_, manifest)| manifest))
    }

    /// 返回 workspace 中所有成员项目的 manifest 目录（有序）。
    pub fn member_manifest_dirs(&self) -> Vec<PathBuf> {
        self.projects.keys().cloned().collect()
    }

    /// 当前路径是否为仅有 `legions.von`、没有 `legion.von` 的 workspace 根目录。
    pub fn is_workspace_only_root(&self, project_dir: &Path) -> bool {
        self.workspace_manifest.is_some() && same_path(project_dir, &self.root_dir) && !self.root_dir.join("legion.von").exists()
    }

    pub fn build_plan(&self, request: &BuildRequest) -> Result<BuildPlan, PlannerError> {
        let project_dir = resolve_project_root(&request.project_dir).unwrap_or_else(|| search_start_dir(&request.project_dir));
        let manifest = self.project_manifest(&project_dir).ok_or_else(|| PlannerError::MissingProjectManifest(project_dir.clone()))?;
        let build_target = select_build_target(manifest, &request.target)
            .ok_or_else(|| PlannerError::MissingBuildTarget { project: manifest.name.clone(), target: request.target })?;

        let dependencies = self.collect_dependencies(manifest, &build_target)?;
        let output_dir = request.output_dir.clone().unwrap_or_else(|| default_dist_output_dir(&project_dir, &request.target));

        // 收集源码闭包：包含项目自身及其所有依赖（含传递依赖）的源文件。
        // 这是模块系统的核心：让编译器能看到依赖项目定义的 struct/function/type。
        let mut visited = BTreeSet::new();
        let source_files = self.collect_source_closure(&project_dir, manifest, &build_target, &mut visited)?;
        let host_inventory = collect_host_inventory(&source_files, &build_target.publish)?;
        let semantic_source_groups = self.collect_semantic_source_groups(&project_dir, manifest, &build_target)?;

        Ok(BuildPlan {
            workspace_root: self.root_dir.clone(),
            output_dir,
            project: PlannedProject {
                name: manifest.name.clone(),
                manifest_dir: project_dir.clone(),
                manifest_path: project_dir.join("legion.von"),
                source_files,
                semantic_source_groups,
                host_contracts: host_inventory.contracts,
                host_provider_candidates: host_inventory.providers,
                selected_host_providers: host_inventory.selected_providers,
                build_target,
                dependencies,
            },
        })
    }

    pub fn build_plan_with_local_fallback(&self, request: &BuildRequest) -> Result<(BuildPlan, ProjectResolutionMode), PlannerError> {
        let project_dir = resolve_project_root(&request.project_dir).unwrap_or_else(|| search_start_dir(&request.project_dir));
        if let Some(manifest) = self.project_manifest(&project_dir) {
            return Ok((self.build_plan_for_manifest(project_dir, manifest, request)?, ProjectResolutionMode::Workspace));
        }

        let manifest_path = project_dir.join("legion.von");
        if !manifest_path.exists() {
            return Err(PlannerError::MissingProjectManifest(project_dir));
        }

        let manifest = ProjectManifest::parse(&fs::read_to_string(&manifest_path)?)?;
        let mode = if self.workspace_manifest.is_some() { ProjectResolutionMode::Package } else { ProjectResolutionMode::Script };
        Ok((self.build_plan_for_manifest(project_dir, &manifest, request)?, mode))
    }

    /// 构造测试/基准编译计划：始终允许指定 target，并合并 `source/` + `test/` 源文件。
    pub fn build_test_plan(&self, request: &BuildRequest) -> Result<(BuildPlan, ProjectResolutionMode), PlannerError> {
        let project_dir = resolve_project_root(&request.project_dir).unwrap_or_else(|| search_start_dir(&request.project_dir));
        let (manifest, mode) = if let Some(manifest) = self.project_manifest(&project_dir) {
            (manifest.clone(), ProjectResolutionMode::Workspace)
        }
        else {
            let manifest_path = project_dir.join("legion.von");
            if !manifest_path.exists() {
                return Err(PlannerError::MissingProjectManifest(project_dir));
            }
            let manifest = ProjectManifest::parse(&fs::read_to_string(&manifest_path)?)?;
            let mode = if self.workspace_manifest.is_some() { ProjectResolutionMode::Package } else { ProjectResolutionMode::Script };
            (manifest, mode)
        };

        let mut build_target = select_build_target(&manifest, &request.target)
            .unwrap_or_else(|| BuildTargetSpec { target: request.target.clone(), ..BuildTargetSpec::default() });
        build_target.target = request.target.clone();

        let dependencies = self.collect_dependencies(&manifest, &build_target)?;
        let output_dir =
            request.output_dir.clone().unwrap_or_else(|| project_dir.join(".cache").join("test").join(request.target.as_canonical_str()));

        let mut visited = BTreeSet::new();
        let mut source_files = self.collect_source_closure(&project_dir, &manifest, &build_target, &mut visited)?;
        let mut test_files = collect_test_v_files(&project_dir)?;
        source_files.append(&mut test_files);
        source_files.sort();
        source_files.dedup();

        let host_inventory = collect_host_inventory(&source_files, &build_target.publish)?;
        let semantic_source_groups = self.collect_semantic_source_groups(&project_dir, &manifest, &build_target)?;

        Ok((
            BuildPlan {
                workspace_root: self.root_dir.clone(),
                output_dir,
                project: PlannedProject {
                    name: manifest.name.clone(),
                    manifest_dir: project_dir.clone(),
                    manifest_path: project_dir.join("legion.von"),
                    source_files,
                    semantic_source_groups,
                    host_contracts: host_inventory.contracts,
                    host_provider_candidates: host_inventory.providers,
                    selected_host_providers: host_inventory.selected_providers,
                    build_target,
                    dependencies,
                },
            },
            mode,
        ))
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
        build_target: &BuildTargetSpec,
        visited: &mut BTreeSet<PathBuf>,
    ) -> Result<Vec<PathBuf>, PlannerError> {
        let canonical_dir = canonicalize_lossy(project_dir);
        if visited.contains(&canonical_dir) {
            return Ok(Vec::new());
        }
        visited.insert(canonical_dir.clone());

        // 收集当前项目自身的源文件，并应用该项目针对当前 target 的 exclude_*。
        let mut all_files = collect_source_files(&canonical_dir)?;
        let local_build = select_build_target(manifest, &build_target.target).unwrap_or_else(|| build_target.clone());
        all_files = filter_excluded_sources(all_files, &canonical_dir, &local_build);

        // 收集当前项目的依赖名称（auto_link + 显式声明 + 平台隐式 SDK）。
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
        let implicit_names = implicit_sdk_dependencies(&build_target.publish).into_iter().map(|item| item.to_string()).collect::<BTreeSet<_>>();
        for implicit in &implicit_names {
            dep_names.insert(implicit.clone());
        }

        // 递归收集每个依赖的源文件。
        for dep_name in dep_names {
            let local_manifest_dir = self.projects_by_name.get(&dep_name).cloned();
            let dependency_spec = manifest.dependencies.get(&dep_name);
            let source_preference = dependency_spec.map(DependencySpec::source_preference).unwrap_or(DependencySourcePreference::Auto);
            let version_hint = dependency_spec.and_then(DependencySpec::version_hint);
            let registry_hint = dependency_spec.and_then(DependencySpec::registry_hint).unwrap_or("npm");
            let allow_missing_when_auto = implicit_names.contains(&dep_name);
            match resolve_dependency_source(
                source_preference,
                local_manifest_dir,
                version_hint,
                registry_hint,
                allow_missing_when_auto,
                &manifest.name,
                &dep_name,
            )? {
                ResolvedDependencySource::Workspace(dep_dir) => {
                    if let Some(dep_manifest) = self.project_manifest(&dep_dir) {
                        if !manifest_supports_build_context(dep_manifest, &build_target.target, &build_target.publish) {
                            continue;
                        }
                        let dep_files = self.collect_source_closure(&dep_dir, dep_manifest, build_target, visited)?;
                        all_files.extend(dep_files);
                    }
                }
                ResolvedDependencySource::Registry { version, registry } => {
                    let install_dir =
                        self.install_registry_dependency(self.root_dir.as_path(), &dep_name, &version, &registry, &manifest.name)?;
                    if let Some(dep_manifest) = self.load_project_manifest_from_dir(&install_dir) {
                        if manifest_supports_build_context(&dep_manifest, &build_target.target, &build_target.publish) {
                            let dep_files = self.collect_source_closure(&install_dir, &dep_manifest, build_target, visited)?;
                            all_files.extend(dep_files);
                        }
                    }
                    else {
                        all_files.extend(collect_source_files(&install_dir)?);
                    }
                }
                ResolvedDependencySource::MissingImplicit => {}
            }
        }

        // 去重并排序，确保编译顺序稳定。
        all_files.sort();
        all_files.dedup();
        Ok(all_files)
    }

    fn collect_semantic_source_groups(
        &self,
        project_dir: &Path,
        manifest: &ProjectManifest,
        build_target: &BuildTargetSpec,
    ) -> Result<Vec<PlannedSemanticSourceGroup>, PlannerError> {
        let mut groups = Vec::new();
        let mut visited = BTreeSet::new();
        self.collect_semantic_source_groups_inner(project_dir, manifest, build_target, &mut visited, &mut groups)?;
        Ok(groups)
    }

    fn collect_semantic_source_groups_inner(
        &self,
        project_dir: &Path,
        manifest: &ProjectManifest,
        build_target: &BuildTargetSpec,
        visited: &mut BTreeSet<PathBuf>,
        groups: &mut Vec<PlannedSemanticSourceGroup>,
    ) -> Result<(), PlannerError> {
        let manifest_dir = canonicalize_lossy(project_dir);
        if !visited.insert(manifest_dir.clone()) {
            return Ok(());
        }
        let dependencies = self.collect_dependencies(manifest, build_target)?;
        for dependency in &dependencies {
            let dependency_manifest = self
                .load_project_manifest_from_dir(&dependency.manifest_dir)
                .ok_or_else(|| PlannerError::MissingProjectManifest(dependency.manifest_dir.clone()))?;
            self.collect_semantic_source_groups_inner(&dependency.manifest_dir, &dependency_manifest, build_target, visited, groups)?;
        }
        let local_build = select_build_target(manifest, &build_target.target).unwrap_or_else(|| build_target.clone());
        let source_files = filter_excluded_sources(collect_source_files(&manifest_dir)?, &manifest_dir, &local_build);
        groups.push(PlannedSemanticSourceGroup {
            name: manifest.name.clone(),
            manifest_dir,
            source_files,
            direct_dependencies: dependencies.into_iter().map(|dependency| dependency.name).collect(),
        });
        Ok(())
    }

    fn build_plan_for_manifest(
        &self,
        project_dir: PathBuf,
        manifest: &ProjectManifest,
        request: &BuildRequest,
    ) -> Result<BuildPlan, PlannerError> {
        let build_target = select_build_target(manifest, &request.target)
            .ok_or_else(|| PlannerError::MissingBuildTarget { project: manifest.name.clone(), target: request.target.clone() })?;

        let dependencies = self.collect_dependencies(manifest, &build_target)?;
        let output_dir = request.output_dir.clone().unwrap_or_else(|| default_dist_output_dir(&project_dir, &request.target));

        let mut visited = BTreeSet::new();
        let source_files = self.collect_source_closure(&project_dir, manifest, &build_target, &mut visited)?;
        let host_inventory = collect_host_inventory(&source_files, &build_target.publish)?;
        let semantic_source_groups = self.collect_semantic_source_groups(&project_dir, manifest, &build_target)?;

        Ok(BuildPlan {
            workspace_root: self.root_dir.clone(),
            output_dir,
            project: PlannedProject {
                name: manifest.name.clone(),
                manifest_dir: project_dir.clone(),
                manifest_path: project_dir.join("legion.von"),
                source_files,
                semantic_source_groups,
                host_contracts: host_inventory.contracts,
                host_provider_candidates: host_inventory.providers,
                selected_host_providers: host_inventory.selected_providers,
                build_target,
                dependencies,
            },
        })
    }

    fn collect_dependencies(&self, manifest: &ProjectManifest, build_target: &BuildTargetSpec) -> Result<Vec<PlannedDependency>, PlannerError> {
        let mut planned = Vec::new();
        let mut names = BTreeSet::new();
        let mut declared_specs = BTreeMap::new();
        let implicit_names = implicit_sdk_dependencies(&build_target.publish).into_iter().map(|item| item.to_string()).collect::<BTreeSet<_>>();

        if manifest.auto_link.core {
            names.insert("core".to_string());
        }
        if manifest.auto_link.std {
            names.insert("std".to_string());
        }
        for (dependency_name, dependency_spec) in &manifest.dependencies {
            if !matches!(dependency_spec, DependencySpec::Disabled) {
                names.insert(dependency_name.clone());
                declared_specs.insert(dependency_name.clone(), dependency_spec.clone());
            }
        }
        for implicit in &implicit_names {
            names.insert(implicit.clone());
        }

        for dependency_name in names {
            let local_manifest_dir = self.projects_by_name.get(&dependency_name).cloned();
            let dependency_spec = declared_specs.get(&dependency_name);
            let source_preference = dependency_spec.map(DependencySpec::source_preference).unwrap_or(DependencySourcePreference::Auto);
            let version_hint = dependency_spec.and_then(DependencySpec::version_hint);
            let registry_hint = dependency_spec.and_then(DependencySpec::registry_hint).unwrap_or("npm");
            let allow_missing_when_auto = implicit_names.contains(&dependency_name);

            let manifest_dir = match resolve_dependency_source(
                source_preference,
                local_manifest_dir,
                version_hint,
                registry_hint,
                allow_missing_when_auto,
                &manifest.name,
                &dependency_name,
            )? {
                ResolvedDependencySource::Workspace(manifest_dir) => manifest_dir,
                ResolvedDependencySource::Registry { version, registry } => {
                    self.install_registry_dependency(self.root_dir.as_path(), &dependency_name, &version, &registry, &manifest.name)?
                }
                ResolvedDependencySource::MissingImplicit => continue,
            };
            if let Some(dependency_manifest) = self.project_manifest(&manifest_dir) {
                if !manifest_supports_build_context(dependency_manifest, &build_target.target, &build_target.publish) {
                    continue;
                }
            }
            planned.push(PlannedDependency { name: dependency_name, manifest_dir });
        }

        planned.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(planned)
    }

    fn registry_vendor_path(&self, dependency_name: &str, version: &str, registry: &str) -> PathBuf {
        self.root_dir.join("vendors").join(registry).join(format!("{dependency_name}@{version}"))
    }

    fn load_project_manifest_from_dir(&self, project_dir: &Path) -> Option<ProjectManifest> {
        self.project_manifest(project_dir).cloned().or_else(|| {
            let manifest_path = project_dir.join("legion.von");
            if !manifest_path.exists() {
                return None;
            }
            let source = fs::read_to_string(&manifest_path).ok()?;
            ProjectManifest::parse(&source).ok()
        })
    }

    fn install_registry_dependency(
        &self,
        project_dir: &Path,
        dependency_name: &str,
        version: &str,
        registry: &str,
        project_name: &str,
    ) -> Result<PathBuf, PlannerError> {
        let vendor_path = self.registry_vendor_path(dependency_name, version, registry);
        if vendor_path.join("legion.von").is_file() {
            return Ok(vendor_path);
        }

        let mut legion =
            PackageLegion::open(project_dir, crate::LEGION_PROJECT_LAYOUT).map_err(|error| PlannerError::RegistryDependencyInstallFailed {
                project: project_name.to_string(),
                dependency: dependency_name.to_string(),
                version: version.to_string(),
                registry: registry.to_string(),
                reason: error.to_string(),
            })?;
        let info =
            legion.install_one(dependency_name, version, registry, false).map_err(|error| PlannerError::RegistryDependencyInstallFailed {
                project: project_name.to_string(),
                dependency: dependency_name.to_string(),
                version: version.to_string(),
                registry: registry.to_string(),
                reason: error.to_string(),
            })?;
        info.install_path.map(PathBuf::from).ok_or_else(|| PlannerError::RegistryDependencyInstallFailed {
            project: project_name.to_string(),
            dependency: dependency_name.to_string(),
            version: version.to_string(),
            registry: registry.to_string(),
            reason: "missing install path".to_string(),
        })
    }
}

enum ResolvedDependencySource {
    Workspace(PathBuf),
    Registry { version: String, registry: String },
    MissingImplicit,
}

fn resolve_dependency_source(
    source_preference: DependencySourcePreference,
    local_manifest_dir: Option<PathBuf>,
    version_hint: Option<&str>,
    registry_hint: &str,
    allow_missing_when_auto: bool,
    project_name: &str,
    dependency_name: &str,
) -> Result<ResolvedDependencySource, PlannerError> {
    match source_preference {
        DependencySourcePreference::Workspace => local_manifest_dir.map(ResolvedDependencySource::Workspace).ok_or_else(|| {
            PlannerError::ForcedWorkspaceDependencyMissing { project: project_name.to_string(), dependency: dependency_name.to_string() }
        }),
        DependencySourcePreference::Registry => {
            let Some(version) = version_hint
            else {
                return Err(PlannerError::RegistryDependencyMissingVersion {
                    project: project_name.to_string(),
                    dependency: dependency_name.to_string(),
                });
            };
            Ok(ResolvedDependencySource::Registry { version: version.to_string(), registry: registry_hint.to_string() })
        }
        DependencySourcePreference::Auto => {
            if let Some(local_manifest_dir) = local_manifest_dir {
                return Ok(ResolvedDependencySource::Workspace(local_manifest_dir));
            }
            if let Some(version) = version_hint {
                return Ok(ResolvedDependencySource::Registry { version: version.to_string(), registry: registry_hint.to_string() });
            }
            if allow_missing_when_auto {
                return Ok(ResolvedDependencySource::MissingImplicit);
            }
            Err(PlannerError::MissingDependency { project: project_name.to_string(), dependency: dependency_name.to_string() })
        }
    }
}

pub fn canonical_target(target: &str) -> Result<CanonicalTarget, nyar_language::CanonicalTargetParseError> {
    target.parse()
}

fn default_dist_output_dir(project_dir: &Path, target: &CanonicalTarget) -> PathBuf {
    project_dir.join("dist").join(target.as_canonical_str())
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

/// Apply `build[].exclude_directories` / `exclude_files` relative to `project_dir`.
fn filter_excluded_sources(files: Vec<PathBuf>, project_dir: &Path, build_target: &BuildTargetSpec) -> Vec<PathBuf> {
    if build_target.exclude_directories.is_empty() && build_target.exclude_files.is_empty() {
        return files;
    }
    let project_dir = canonicalize_lossy(project_dir);
    files
        .into_iter()
        .filter(|path| {
            let Ok(relative) = path.strip_prefix(&project_dir)
            else {
                return true;
            };
            let relative = relative.to_string_lossy().replace('\\', "/");
            if build_target.exclude_files.iter().any(|item| {
                let item = item.replace('\\', "/");
                relative == item || relative.ends_with(item.trim_start_matches("./"))
            }) {
                return false;
            }
            if build_target.exclude_directories.iter().any(|item| {
                let item = item.replace('\\', "/").trim_end_matches('/').to_string();
                relative == item || relative.starts_with(&(item.clone() + "/"))
            }) {
                return false;
            }
            true
        })
        .collect()
}

/// 收集项目 `source/` + `test/` 下的全部 `.v` 文件（测试编译专用）。
pub fn collect_test_build_sources(project_dir: &Path) -> Result<Vec<PathBuf>, PlannerError> {
    let mut files = collect_source_files(project_dir)?;
    let test_dir = project_dir.join("test");
    if test_dir.exists() {
        collect_v_files(&test_dir, &mut files)?;
    }
    files.sort();
    files.dedup();
    Ok(files)
}

/// 仅收集项目 `test/` 下的 `.v` 文件。
pub fn collect_test_v_files(project_dir: &Path) -> Result<Vec<PathBuf>, PlannerError> {
    let mut files = Vec::new();
    let test_dir = project_dir.join("test");
    if test_dir.exists() {
        collect_v_files(&test_dir, &mut files)?;
    }
    files.sort();
    Ok(files)
}

/// 收集项目 `source/` 与 `test/` 中全部 `.v` 文件（覆盖率扫描，含 `compile_only/`）。
pub fn collect_project_v_files(project_dir: &Path) -> Result<Vec<PathBuf>, PlannerError> {
    let mut files = collect_source_files(project_dir)?;
    let test_dir = project_dir.join("test");
    if test_dir.exists() {
        collect_v_files_including_compile_only(&test_dir, &mut files)?;
    }
    files.sort();
    files.dedup();
    Ok(files)
}

fn collect_v_files_including_compile_only(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), PlannerError> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_v_files_including_compile_only(&path, files)?;
            continue;
        }
        if path.extension().is_some_and(|ext| ext == "v") {
            files.push(path);
        }
    }
    Ok(())
}

fn collect_v_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), PlannerError> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|name| name == "compile_only") {
                continue;
            }
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
    let start_dir = search_start_dir(start);

    // Prefer the outermost `legions.von`. Nested subspaces (e.g. `projects/legion._`)
    // only declare local members; shared deps like `core` / `nyar` / `std` live on the
    // parent super-workspace and are registered via nested member recursion.
    let mut outermost = None;
    let mut current = Some(start_dir.as_path());
    while let Some(dir) = current {
        if dir.join("legions.von").exists() {
            outermost = Some(dir.to_path_buf());
        }
        current = dir.parent();
    }
    outermost
}

fn canonicalize_lossy(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn search_start_dir(start: &Path) -> PathBuf {
    if start.is_dir() {
        return canonicalize_lossy(start);
    }

    start.parent().map(canonicalize_lossy).unwrap_or_else(|| start.to_path_buf())
}

fn resolve_project_root(start: &Path) -> Option<PathBuf> {
    let mut current = Some(search_start_dir(start));
    while let Some(dir) = current {
        if dir.join("legion.von").exists() {
            return Some(dir);
        }
        current = dir.parent().map(Path::to_path_buf);
    }
    None
}

fn register_workspace_members(
    workspace_root: &Path,
    workspace_manifest: &WorkspaceManifest,
    projects: &mut BTreeMap<PathBuf, ProjectManifest>,
    projects_by_name: &mut BTreeMap<String, PathBuf>,
) -> Result<(), PlannerError> {
    for member in &workspace_manifest.members {
        let member_dir = canonicalize_lossy(&workspace_root.join(member));
        let nested_workspace_manifest_path = member_dir.join("legions.von");
        if nested_workspace_manifest_path.exists() {
            let nested_workspace_manifest = WorkspaceManifest::parse(&fs::read_to_string(&nested_workspace_manifest_path)?)?;
            register_workspace_members(&member_dir, &nested_workspace_manifest, projects, projects_by_name)?;
        }

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

    Ok(())
}

fn same_path(lhs: &Path, rhs: &Path) -> bool {
    normalize_path_for_lookup(lhs) == normalize_path_for_lookup(rhs)
}

fn normalize_path_for_lookup(path: &Path) -> String {
    let raw = path.to_string_lossy();
    let stripped = raw.strip_prefix(r"\\?\").unwrap_or(&raw);
    if cfg!(windows) { stripped.to_ascii_lowercase() } else { stripped.to_string() }
}

fn manifest_supports_build_context(manifest: &ProjectManifest, target: &CanonicalTarget, requested_publish: &[String]) -> bool {
    let Some(sdk_vendor) = &manifest.sdk_vendor
    else {
        return true;
    };
    if !sdk_vendor.targets.is_empty() {
        let target_matches = sdk_vendor
            .targets
            .iter()
            .any(|item| item == &target.to_string() || item.parse::<CanonicalTarget>().map(|parsed| parsed == *target).unwrap_or(false));
        if !target_matches {
            return false;
        }
    }
    if !sdk_vendor.publish.is_empty() && !requested_publish.is_empty() {
        let publish_matches = sdk_vendor.publish.iter().any(|item| requested_publish.iter().any(|requested| requested == item));
        if !publish_matches {
            return false;
        }
    }
    true
}

fn implicit_sdk_dependencies(publish: &[String]) -> Vec<&'static str> {
    let mut deps = Vec::new();
    if publish.iter().any(|item| item == "mini-game") {
        deps.push("tencent.wechat.sdk");
    }
    if publish.iter().any(|item| item == "mini-program") {
        deps.push("tencent.wechat.miniprogram.sdk");
    }
    if publish.iter().any(|item| item == "unity-player") {
        deps.push("unity.engine.sdk");
    }
    deps
}

fn manifest_supports_target(manifest: &ProjectManifest, target: &CanonicalTarget) -> bool {
    manifest_supports_build_context(manifest, target, &[])
}

#[derive(Debug, Default)]
struct HostInventory {
    contracts: Vec<PlannedHostContract>,
    providers: Vec<PlannedHostProvider>,
    selected_providers: Vec<PlannedHostProvider>,
}

fn collect_host_inventory(source_files: &[PathBuf], publish: &[String]) -> Result<HostInventory, PlannerError> {
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
    inventory.selected_providers = select_host_providers(&inventory.contracts, &inventory.providers, publish)?;
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
    publish: &[String],
) -> Result<Vec<PlannedHostProvider>, PlannerError> {
    let contract_ids: BTreeSet<String> = contracts.iter().map(|item| normalize_contract_reference(&item.id)).collect();
    // Providers whose contracts were removed by `exclude_files` / arch gating are inert —
    // skip them instead of failing the whole plan (common on CLR std slices).
    let mut providers_by_contract: BTreeMap<String, Vec<&PlannedHostProvider>> = BTreeMap::new();
    for provider in providers {
        let contract = normalize_contract_reference(&provider.contract);
        if !contract_ids.contains(&contract) {
            continue;
        }
        providers_by_contract.entry(contract).or_default().push(provider);
    }

    let mut selected = Vec::new();
    for contract in contracts {
        let Some(candidates) = providers_by_contract.get(&normalize_contract_reference(&contract.id))
        else {
            continue;
        };

        let chosen = if candidates.len() == 1 {
            candidates[0]
        }
        else {
            candidates.iter().copied().min_by_key(|provider| host_provider_priority(&provider.source_file, publish)).ok_or_else(|| {
                PlannerError::ConflictingHostProviders {
                    contract: contract.id.clone(),
                    providers: candidates
                        .iter()
                        .map(|item| format!("{} at {}:{}", item.symbol, item.source_file.display(), item.line))
                        .collect(),
                }
            })?
        };

        selected.push(chosen.clone());
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

fn host_provider_priority(source_file: &Path, publish: &[String]) -> u8 {
    let path = source_file.to_string_lossy();
    if publish.iter().any(|item| item == "unity-player") && path.contains("unity.engine.sdk") {
        return 0;
    }
    if publish.iter().any(|item| item == "mini-game") && path.contains("tencent.wechat.sdk") {
        return 0;
    }
    if publish.iter().any(|item| item == "mini-program") && path.contains("tencent.wechat.miniprogram.sdk") {
        return 0;
    }
    if path.contains("std.adaptor.") {
        return 2;
    }
    1
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
