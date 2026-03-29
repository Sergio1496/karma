use std::fmt;

use serde::{Deserialize, Serialize};

/// Identifiers for installable components.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentId {
    Skills,
    Orchestrator,
    McpServers,
    Permissions,
    SubAgents,
    ContextIsolation,
    Rtk,
    CodeReviewGraph,
}

impl ComponentId {
    /// All available component IDs.
    /// Core components (shown in component select).
    pub const ALL: &[ComponentId] = &[
        ComponentId::Skills,
        ComponentId::Orchestrator,
        ComponentId::McpServers,
        ComponentId::Permissions,
        ComponentId::SubAgents,
    ];

    /// Optimizer tools (shown in optimizer select).
    pub const OPTIMIZERS: &[ComponentId] = &[
        ComponentId::ContextIsolation,
        ComponentId::Rtk,
        ComponentId::CodeReviewGraph,
    ];

    /// Whether this component is an optimizer (separate screen in TUI).
    pub fn is_optimizer(&self) -> bool {
        matches!(
            self,
            ComponentId::ContextIsolation | ComponentId::Rtk | ComponentId::CodeReviewGraph
        )
    }

    /// Human-readable name for display.
    pub fn display_name(&self) -> &'static str {
        match self {
            ComponentId::Skills => "Skills",
            ComponentId::Orchestrator => "Model Assignments",
            ComponentId::McpServers => "MCP Servers",
            ComponentId::Permissions => "Permisos",
            ComponentId::SubAgents => "Sub-Agentes",
            ComponentId::ContextIsolation => "Context Isolation",
            ComponentId::Rtk => "RTK (Token Killer)",
            ComponentId::CodeReviewGraph => "Code Review Graph",
        }
    }

    /// Short description of what this component does.
    pub fn description(&self) -> &'static str {
        match self {
            ComponentId::Skills => "Slash commands para workflows (branch-pr, issue-creation)",
            ComponentId::Orchestrator => "Tabla de routing Opus/Sonnet/Haiku por tarea en CLAUDE.md",
            ComponentId::McpServers => "Servidor MCP Context7 para docs de frameworks",
            ComponentId::Permissions => "Deny rules de seguridad (rm -rf, .env, credentials)",
            ComponentId::SubAgents => "16 sub-agentes con modelo asignado (SDD + proposito general)",
            ComponentId::ContextIsolation => "Delega busquedas pesadas a subagentes para no ensuciar el contexto",
            ComponentId::Rtk => "Proxy CLI que ahorra 60-90% de tokens en comandos",
            ComponentId::CodeReviewGraph => "Knowledge graph del codigo, ahorra ~8x tokens en review",
        }
    }
}

impl fmt::Display for ComponentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Skill identifier (dynamically populated from catalog).
pub type SkillId = String;

/// Installation presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PresetId {
    /// Orchestrator only.
    Minimal,
    /// Orchestrator + Skills + Permissions + MCP.
    Recommended,
    /// Everything.
    Full,
    /// User picks individual components.
    Custom,
}

impl PresetId {
    /// Returns the components included in this preset.
    pub fn components(&self) -> Vec<ComponentId> {
        match self {
            PresetId::Minimal => vec![ComponentId::Orchestrator],
            PresetId::Recommended => vec![
                ComponentId::Orchestrator,
                ComponentId::Skills,
                ComponentId::Permissions,
                ComponentId::McpServers,
            ],
            PresetId::Full => ComponentId::ALL.to_vec(),
            PresetId::Custom => vec![], // User selects manually
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            PresetId::Minimal => "Minimal",
            PresetId::Recommended => "Recommended",
            PresetId::Full => "Full",
            PresetId::Custom => "Custom",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            PresetId::Minimal => "SDD orchestrator only",
            PresetId::Recommended => "Orchestrator + Skills + Permissions + MCP servers",
            PresetId::Full => "All components enabled",
            PresetId::Custom => "Choose individual components",
        }
    }
}

// ── Model Assignment System ──

/// Claude model alias for phase assignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelAlias {
    Opus,
    Sonnet,
    Haiku,
}

impl ModelAlias {
    pub fn as_str(&self) -> &'static str {
        match self {
            ModelAlias::Opus => "opus",
            ModelAlias::Sonnet => "sonnet",
            ModelAlias::Haiku => "haiku",
        }
    }
}

impl fmt::Display for ModelAlias {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Model assignment preset — controls which model handles each SDD phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelPreset {
    /// Opus for architecture/orchestration, Sonnet for structural work, Haiku for archiving.
    Balanced,
    /// Same as Balanced but Opus also handles verification.
    Performance,
    /// Everything Sonnet except archiving (Haiku). Cheapest option.
    Economy,
}

impl ModelPreset {
    pub fn display_name(&self) -> &'static str {
        match self {
            ModelPreset::Balanced => "Balanced",
            ModelPreset::Performance => "Performance",
            ModelPreset::Economy => "Economy",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ModelPreset::Balanced => "Opus for planning & architecture, Sonnet for code, Haiku for archiving",
            ModelPreset::Performance => "Opus also handles verification (higher quality, higher cost)",
            ModelPreset::Economy => "Sonnet for everything, Haiku for archiving (lowest cost)",
        }
    }

    /// Get the model assignment for a specific agent/phase.
    pub fn model_for_phase(&self, phase: &str) -> ModelAlias {
        match self {
            ModelPreset::Balanced => match phase {
                // SDD phases
                "sdd-propose" | "sdd-design" => ModelAlias::Opus,
                "sdd-archive" => ModelAlias::Haiku,
                // General purpose
                "code-review" | "debugger" | "refactor" | "planner" => ModelAlias::Opus,
                "docs-writer" | "searcher" | "git-ops" => ModelAlias::Haiku,
                // Everything else: sonnet
                _ => ModelAlias::Sonnet,
            },
            ModelPreset::Performance => match phase {
                // SDD phases
                "sdd-propose" | "sdd-design" | "sdd-verify" => ModelAlias::Opus,
                "sdd-archive" => ModelAlias::Haiku,
                // General purpose
                "code-review" | "debugger" | "refactor" | "planner" | "test-writer" => ModelAlias::Opus,
                "docs-writer" | "searcher" | "git-ops" => ModelAlias::Haiku,
                // Everything else: sonnet
                _ => ModelAlias::Sonnet,
            },
            ModelPreset::Economy => match phase {
                "sdd-archive" | "docs-writer" | "searcher" | "git-ops" => ModelAlias::Haiku,
                _ => ModelAlias::Sonnet,
            },
        }
    }

    /// All agents/phases in order with their model assignments.
    pub fn all_assignments(&self) -> Vec<(&'static str, &'static str, ModelAlias)> {
        vec![
            // SDD workflow
            ("sdd-explore", "Codebase analysis", self.model_for_phase("sdd-explore")),
            ("sdd-propose", "Change proposals", self.model_for_phase("sdd-propose")),
            ("sdd-spec", "Specification writing", self.model_for_phase("sdd-spec")),
            ("sdd-design", "Architecture decisions", self.model_for_phase("sdd-design")),
            ("sdd-tasks", "Task breakdown", self.model_for_phase("sdd-tasks")),
            ("sdd-apply", "Implementation", self.model_for_phase("sdd-apply")),
            ("sdd-verify", "Verification", self.model_for_phase("sdd-verify")),
            ("sdd-archive", "Archiving", self.model_for_phase("sdd-archive")),
            // General purpose
            ("code-review", "Code review & PR analysis", self.model_for_phase("code-review")),
            ("debugger", "Bug diagnosis & fixing", self.model_for_phase("debugger")),
            ("test-writer", "Writing tests", self.model_for_phase("test-writer")),
            ("docs-writer", "Documentation", self.model_for_phase("docs-writer")),
            ("refactor", "Code restructuring", self.model_for_phase("refactor")),
            ("searcher", "Search & navigation", self.model_for_phase("searcher")),
            ("git-ops", "Git operations", self.model_for_phase("git-ops")),
            ("planner", "Implementation planning", self.model_for_phase("planner")),
            // Fallback
            ("default", "Any other delegation", self.model_for_phase("default")),
        ]
    }

    /// Render the assignments as a markdown table for injection into CLAUDE.md.
    pub fn render_markdown_table(&self) -> String {
        let mut table = String::from("| Phase | Purpose | Model |\n|-------|---------|-------|\n");
        for (phase, purpose, model) in self.all_assignments() {
            table.push_str(&format!("| `{}` | {} | **{}** |\n", phase, purpose, model));
        }
        table
    }
}

/// Where to install configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigScope {
    /// User-level: ~/.claude/
    User,
    /// Project-level: .claude/ in current directory
    Project,
}

/// The user's full selection for an installation run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Selection {
    pub components: Vec<ComponentId>,
    pub skills: Vec<SkillId>,
    pub preset: PresetId,
    pub model_preset: ModelPreset,
    pub scope: ConfigScope,
    pub dry_run: bool,
}

impl Default for Selection {
    fn default() -> Self {
        Self {
            components: PresetId::Recommended.components(),
            skills: vec![],
            preset: PresetId::Recommended,
            model_preset: ModelPreset::Balanced,
            scope: ConfigScope::User,
            dry_run: false,
        }
    }
}
