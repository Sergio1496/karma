pub mod agents;
pub mod behavior_profile;
pub mod code_review_graph;
pub mod context_isolation;
pub mod mcp;
pub mod orchestrator;
pub mod permissions;
pub mod rtk;
pub mod skills;

use std::path::PathBuf;

use anyhow::Result;

use crate::backup::manifest::Manifest;
use crate::config::paths::ClaudePaths;
use crate::config::types::{BehaviorProfileStatus, ComponentId, Selection};

/// Result of applying a component's configuration.
#[derive(Debug)]
pub struct ApplyResult {
    /// Whether any files were actually changed.
    pub changed: bool,
    /// Files that were written or modified.
    pub files_written: Vec<PathBuf>,
    /// Human-readable messages about what was done.
    pub messages: Vec<String>,
}

/// Context passed to each component during pipeline execution.
pub struct PipelineContext {
    pub paths: ClaudePaths,
    pub selection: Selection,
    pub dry_run: bool,
    pub backup_manifest: Option<Manifest>,
}

/// The core trait that every installable component must implement.
pub trait Component: Send + Sync {
    /// Unique identifier for this component.
    fn id(&self) -> &str;

    /// Human-readable display name.
    fn name(&self) -> &str;

    /// List of file paths this component will modify (used for backup targeting).
    fn affected_paths(&self, paths: &ClaudePaths) -> Vec<PathBuf>;

    /// Validate preconditions before applying.
    fn prepare(&self, ctx: &PipelineContext) -> Result<()>;

    /// Apply the configuration changes.
    fn apply(&self, ctx: &PipelineContext) -> Result<ApplyResult>;

    /// Undo changes (restore from backup).
    fn rollback(&self, ctx: &PipelineContext) -> Result<()>;
}

/// Create a component instance from its ID.
pub fn create_component(id: ComponentId) -> Box<dyn Component> {
    match id {
        ComponentId::Orchestrator => Box::new(orchestrator::OrchestratorComponent),
        ComponentId::Skills => Box::new(skills::SkillsComponent),
        ComponentId::McpServers => Box::new(mcp::McpComponent),
        ComponentId::Permissions => Box::new(permissions::PermissionsComponent),
        ComponentId::SubAgents => Box::new(agents::AgentsComponent),
        ComponentId::ContextIsolation => Box::new(context_isolation::ContextIsolationComponent),
        ComponentId::Rtk => Box::new(rtk::RtkComponent),
        ComponentId::CodeReviewGraph => Box::new(code_review_graph::CodeReviewGraphComponent),
        ComponentId::BehaviorProfile => {
            Box::new(behavior_profile::BehaviorProfileComponent)
        }
    }
}

/// Create component instances for an ordered list of IDs.
pub fn create_components(ids: &[ComponentId]) -> Vec<Box<dyn Component>> {
    ids.iter().map(|id| create_component(*id)).collect()
}

/// Status of an optimizer installation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizerStatus {
    /// Binary not found, not configured anywhere.
    NotInstalled,
    /// Binary available but not configured in this project.
    BinaryOnly,
    /// Configured globally (~/.claude/) but not in this project.
    GlobalOnly,
    /// Configured in this project (.claude/).
    ProjectInstalled,
}

impl OptimizerStatus {
    pub fn label(&self) -> &'static str {
        match self {
            OptimizerStatus::NotInstalled => "",
            OptimizerStatus::BinaryOnly => "[solo binario]",
            OptimizerStatus::GlobalOnly => "[solo global]",
            OptimizerStatus::ProjectInstalled => "[instalado en proyecto]",
        }
    }

    /// Whether this counts as "already working" for the current project.
    pub fn is_active(&self) -> bool {
        matches!(self, OptimizerStatus::ProjectInstalled)
    }
}

/// What the user wants to do with an optimizer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizerAction {
    /// Don't touch it.
    Skip,
    /// Install/configure in this project.
    InstallProject,
    /// Uninstall from this project (only if ProjectInstalled).
    UninstallProject,
}

impl OptimizerAction {
    pub fn label(&self) -> &'static str {
        match self {
            OptimizerAction::Skip => "No tocar",
            OptimizerAction::InstallProject => "Instalar en proyecto",
            OptimizerAction::UninstallProject => "Desinstalar del proyecto",
        }
    }

    /// Cycle to the next action (left/right arrows).
    /// If installed: No tocar ↔ Desinstalar
    /// If not installed: No tocar ↔ Instalar
    pub fn next(&self, status: OptimizerStatus) -> Self {
        if status.is_active() {
            // Installed in project: toggle between Skip and Uninstall
            match self {
                OptimizerAction::Skip => OptimizerAction::UninstallProject,
                OptimizerAction::UninstallProject => OptimizerAction::Skip,
                _ => OptimizerAction::Skip,
            }
        } else {
            // Not in project: toggle between Skip and Install
            match self {
                OptimizerAction::Skip => OptimizerAction::InstallProject,
                OptimizerAction::InstallProject => OptimizerAction::Skip,
                _ => OptimizerAction::Skip,
            }
        }
    }

    pub fn prev(&self, status: OptimizerStatus) -> Self {
        self.next(status) // Only 2 options, so prev == next
    }
}

/// Detect the installation status of an optimizer.
/// Checks project-level files (CLAUDE.md, .code-review-graph/, .mcp.json, settings.json).
pub fn detect_optimizer(id: ComponentId, paths: &ClaudePaths) -> OptimizerStatus {
    let cwd = std::env::current_dir().unwrap_or_default();

    match id {
        ComponentId::Rtk => {
            let has_binary = std::process::Command::new("rtk")
                .arg("--version")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

            if !has_binary {
                return OptimizerStatus::NotInstalled;
            }

            // Check project: settings.json hook OR CLAUDE.md with RTK reference OR RTK.md exists
            let project_settings = paths.project_settings_json(&cwd);
            let project_claude_md = paths.project_claude_md(&cwd);
            let project_rtk_md = cwd.join(".claude").join("RTK.md");

            if file_contains(&project_settings, "rtk hook claude")
                || file_contains(&project_claude_md, "RTK")
                || project_rtk_md.exists()
            {
                return OptimizerStatus::ProjectInstalled;
            }

            // Check global
            if file_contains(&paths.global_settings_json(), "rtk hook claude")
                || file_contains(&paths.global_claude_md(), "RTK")
                || paths.global_config_dir().join("RTK.md").exists()
            {
                return OptimizerStatus::GlobalOnly;
            }

            OptimizerStatus::BinaryOnly
        }
        ComponentId::ContextIsolation => {
            // No external binary needed — karma IS the binary.
            // Check if the hook is configured in settings.json or CLAUDE.md has the section.
            let project_settings = paths.project_settings_json(&cwd);
            let project_claude_md = paths.project_claude_md(&cwd);

            if file_contains(&project_settings, "karma hook-guard")
                || file_contains(&project_claude_md, "karma:context-isolation")
            {
                return OptimizerStatus::ProjectInstalled;
            }

            if file_contains(&paths.global_settings_json(), "karma hook-guard")
                || file_contains(&paths.global_claude_md(), "karma:context-isolation")
            {
                return OptimizerStatus::GlobalOnly;
            }

            // karma exists (we're running it), so binary is always available
            OptimizerStatus::BinaryOnly
        }
        ComponentId::CodeReviewGraph => {
            let has_tool = std::process::Command::new("uvx")
                .args(["code-review-graph", "--version"])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
                || std::process::Command::new("pip")
                    .args(["show", "code-review-graph"])
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false);

            if !has_tool {
                return OptimizerStatus::NotInstalled;
            }

            // Check project: .code-review-graph/ folder OR .mcp.json with server
            let graph_dir = cwd.join(".code-review-graph");
            let project_mcp = paths.project_mcp_json(&cwd);

            if graph_dir.exists() || file_contains(&project_mcp, "code-review-graph") {
                return OptimizerStatus::ProjectInstalled;
            }

            // Check global
            if file_contains(&paths.mcp_user_config(), "code-review-graph") {
                return OptimizerStatus::GlobalOnly;
            }

            OptimizerStatus::BinaryOnly
        }
        _ => OptimizerStatus::NotInstalled,
    }
}

/// Detect the installation status of a behavioral profile.
pub fn detect_behavior_profile(paths: &ClaudePaths) -> BehaviorProfileStatus {
    behavior_profile::detect(paths)
}

fn file_contains(path: &std::path::Path, needle: &str) -> bool {
    path.exists()
        && std::fs::read_to_string(path)
            .map(|c| c.contains(needle))
            .unwrap_or(false)
}
