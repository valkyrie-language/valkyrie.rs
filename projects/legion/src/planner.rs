use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::{Display, Formatter},
    fs,
    path::{Path, PathBuf},
};

use miette::{Diagnostic, Severity};
use nyar::CanonicalTarget;
use serde::{Deserialize, Serialize};

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
pub struct PlannedProject {
    pub name: String,
    pub manifest_dir: PathBuf,
    pub manifest_path: PathBuf,
    pub source_files: Vec<PathBuf>,
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
        self.projects.get(project_dir)
    }

    pub fn build_plan(&self, request: &BuildRequest) -> Result<BuildPlan, PlannerError> {
        let project_dir = canonicalize_lossy(&request.project_dir);
        let manifest = self.project_manifest(&project_dir).ok_or_else(|| PlannerError::MissingProjectManifest(project_dir.clone()))?;
        let build_target = select_build_target(manifest, &request.target)
            .ok_or_else(|| PlannerError::MissingBuildTarget { project: manifest.name.clone(), target: request.target })?;

        let dependencies = self.collect_dependencies(manifest)?;
        let output_dir =
            request.output_dir.clone().unwrap_or_else(|| project_dir.join("dist").join(request.target.to_string().replace('-', "_")));

        // 收集源码闭包：包含项目自身及其所有依赖（含传递依赖）的源文件。
        // 这是模块系统的核心：让编译器能看到依赖项目定义的 struct/function/type。
        let mut visited = BTreeSet::new();
        let source_files = self.collect_source_closure(&project_dir, manifest, &mut visited)?;

        Ok(BuildPlan {
            workspace_root: self.root_dir.clone(),
            output_dir,
            project: PlannedProject {
                name: manifest.name.clone(),
                manifest_dir: project_dir.clone(),
                manifest_path: project_dir.join("legion.von"),
                source_files,
                build_target,
                dependencies,
            },
        })
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
                    let dep_files = self.collect_source_closure(dep_dir, dep_manifest, visited)?;
                    all_files.extend(dep_files);
                }
            }
        }

        // 去重并排序，确保编译顺序稳定。
        all_files.sort();
        all_files.dedup();
        Ok(all_files)
    }

    fn collect_dependencies(&self, manifest: &ProjectManifest) -> Result<Vec<PlannedDependency>, PlannerError> {
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
            planned.push(PlannedDependency { name: dependency_name, manifest_dir });
        }

        planned.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(planned)
    }
}

pub fn canonical_target(target: &str) -> Result<CanonicalTarget, nyar::CanonicalTargetParseError> {
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
