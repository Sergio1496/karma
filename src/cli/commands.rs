use clap::{Parser, Subcommand};

use crate::config::types::{ComponentId, ConfigScope, ModelPreset, PresetId};

#[derive(Parser)]
#[command(
    name = "karma",
    about = "Configure Claude Code with skills, orchestrators, sub-agents, MCP servers, and hooks",
    version,
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Install components and skills into Claude Code
    Install(InstallArgs),

    /// Sync managed assets (re-fetch remote, update managed sections)
    Sync(SyncArgs),

    /// Restore a previous configuration backup
    Restore(RestoreArgs),

    /// Analyze the current project and suggest components
    Analyze,

    /// Show current installation status
    Status,

    /// Check for updates
    Update,

    /// Show version information
    Version,

    /// Internal: evaluate PreToolUse hook for context isolation
    #[command(name = "hook-guard", hide = true)]
    HookGuard,
}

#[derive(Parser)]
pub struct InstallArgs {
    /// Preset to install
    #[arg(long, short, value_parser = parse_preset)]
    pub preset: Option<PresetId>,

    /// Individual components to install (repeatable)
    #[arg(long, short, value_parser = parse_component)]
    pub component: Vec<ComponentId>,

    /// Individual skills to install (repeatable)
    #[arg(long, short)]
    pub skill: Vec<String>,

    /// Model assignment preset: balanced, performance, or economy
    #[arg(long, short = 'm', value_parser = parse_model_preset, default_value = "balanced")]
    pub model_preset: ModelPreset,

    /// Configuration scope
    #[arg(long, value_parser = parse_scope, default_value = "user")]
    pub scope: ConfigScope,

    /// Preview changes without writing files
    #[arg(long)]
    pub dry_run: bool,

    /// Skip confirmation prompt
    #[arg(long, short)]
    pub yes: bool,
}

#[derive(Parser)]
pub struct SyncArgs {
    /// Force refresh from remote (ignore cache)
    #[arg(long)]
    pub refresh: bool,
}

#[derive(Parser)]
pub struct RestoreArgs {
    /// Restore the most recent backup
    #[arg(long)]
    pub latest: bool,

    /// Restore a specific backup by ID
    #[arg(long)]
    pub id: Option<String>,
}

fn parse_preset(s: &str) -> Result<PresetId, String> {
    match s.to_lowercase().as_str() {
        "minimal" => Ok(PresetId::Minimal),
        "recommended" => Ok(PresetId::Recommended),
        "full" => Ok(PresetId::Full),
        "custom" => Ok(PresetId::Custom),
        _ => Err(format!(
            "Unknown preset '{s}'. Valid: minimal, recommended, full, custom"
        )),
    }
}

fn parse_component(s: &str) -> Result<ComponentId, String> {
    match s.to_lowercase().as_str() {
        "skills" => Ok(ComponentId::Skills),
        "orchestrator" | "models" => Ok(ComponentId::Orchestrator),
        "mcp" | "mcp-servers" => Ok(ComponentId::McpServers),
        "permissions" => Ok(ComponentId::Permissions),
        "sub-agents" | "subagents" | "agents" => Ok(ComponentId::SubAgents),
        "context-isolation" | "isolation" | "ctx-iso" => Ok(ComponentId::ContextIsolation),
        "rtk" => Ok(ComponentId::Rtk),
        "code-review-graph" | "crg" | "graph" => Ok(ComponentId::CodeReviewGraph),
        _ => Err(format!(
            "Unknown component '{s}'. Valid: skills, orchestrator, mcp, permissions, agents, context-isolation, rtk, crg"
        )),
    }
}

fn parse_model_preset(s: &str) -> Result<ModelPreset, String> {
    match s.to_lowercase().as_str() {
        "balanced" => Ok(ModelPreset::Balanced),
        "performance" => Ok(ModelPreset::Performance),
        "economy" => Ok(ModelPreset::Economy),
        _ => Err(format!(
            "Unknown model preset '{s}'. Valid: balanced, performance, economy"
        )),
    }
}

fn parse_scope(s: &str) -> Result<ConfigScope, String> {
    match s.to_lowercase().as_str() {
        "user" => Ok(ConfigScope::User),
        "project" => Ok(ConfigScope::Project),
        _ => Err(format!("Unknown scope '{s}'. Valid: user, project")),
    }
}
