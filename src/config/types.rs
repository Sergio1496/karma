use std::collections::BTreeMap;
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
    BehaviorProfile,
}

// ── Behavioral Profiles ──

/// Identifier for a behavioral profile that reduces output tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfileId {
    Universal,
    Coding,
    Agents,
    Analysis,
}

impl ProfileId {
    pub const ALL: &[ProfileId] = &[
        ProfileId::Universal,
        ProfileId::Coding,
        ProfileId::Agents,
        ProfileId::Analysis,
    ];

    pub fn display_name(&self) -> &'static str {
        match self {
            ProfileId::Universal => "Universal",
            ProfileId::Coding => "Coding",
            ProfileId::Agents => "Agents",
            ProfileId::Analysis => "Analysis",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ProfileId::Universal => "Reglas generales de conciseness para cualquier tarea",
            ProfileId::Coding => "Codigo primero, explicaciones minimas, sin boilerplate",
            ProfileId::Agents => "Output estructurado (JSON/tablas) para pipelines y bots",
            ProfileId::Analysis => "Datos primero, hallazgos claros, sin narrativa innecesaria",
        }
    }

    /// Asset path for the embedded template.
    pub fn template_asset(&self) -> &'static str {
        match self {
            ProfileId::Universal => "templates/behavior_profile_universal.md",
            ProfileId::Coding => "templates/behavior_profile_coding.md",
            ProfileId::Agents => "templates/behavior_profile_agents.md",
            ProfileId::Analysis => "templates/behavior_profile_analysis.md",
        }
    }

    /// Unique signature string embedded in each template for detection.
    pub fn signature(&self) -> &'static str {
        match self {
            ProfileId::Universal => "<!-- profile:universal -->",
            ProfileId::Coding => "<!-- profile:coding -->",
            ProfileId::Agents => "<!-- profile:agents -->",
            ProfileId::Analysis => "<!-- profile:analysis -->",
        }
    }
}

impl fmt::Display for ProfileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Detection status for a behavioral profile in CLAUDE.md.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BehaviorProfileStatus {
    /// No profile detected.
    NotInstalled,
    /// Installed via karma (has markers).
    Installed(ProfileId),
    /// Content detected manually (no karma markers).
    ManuallyDetected(ProfileId),
}

impl BehaviorProfileStatus {
    pub fn label(&self) -> &'static str {
        match self {
            BehaviorProfileStatus::NotInstalled => "",
            BehaviorProfileStatus::Installed(_) => "[karma]",
            BehaviorProfileStatus::ManuallyDetected(_) => "[manual]",
        }
    }

    pub fn installed_profile(&self) -> Option<ProfileId> {
        match self {
            BehaviorProfileStatus::NotInstalled => None,
            BehaviorProfileStatus::Installed(p) | BehaviorProfileStatus::ManuallyDetected(p) => {
                Some(*p)
            }
        }
    }

    pub fn is_karma_installed(&self) -> bool {
        matches!(self, BehaviorProfileStatus::Installed(_))
    }
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
            ComponentId::BehaviorProfile => "Perfil de Comportamiento",
        }
    }

    /// Short description of what this component does.
    pub fn description(&self) -> &'static str {
        match self {
            ComponentId::Skills => "Slash commands para workflows (branch-pr, issue-creation)",
            ComponentId::Orchestrator => {
                "Tabla de routing Opus/Sonnet/Haiku por tarea en CLAUDE.md"
            }
            ComponentId::McpServers => "Servidor MCP Context7 para docs de frameworks",
            ComponentId::Permissions => "Deny rules de seguridad (rm -rf, .env, credentials)",
            ComponentId::SubAgents => {
                "16 sub-agentes con modelo asignado (SDD + proposito general)"
            }
            ComponentId::ContextIsolation => {
                "Delega busquedas pesadas a subagentes para no ensuciar el contexto"
            }
            ComponentId::Rtk => "Proxy CLI que ahorra 60-90% de tokens en comandos",
            ComponentId::CodeReviewGraph => {
                "Knowledge graph del codigo, ahorra ~8x tokens en review"
            }
            ComponentId::BehaviorProfile => {
                "Reglas de conciseness que reducen ~63% de tokens de output"
            }
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

    /// Cycle to next model (Opus → Sonnet → Haiku → Opus).
    pub fn next(self) -> Self {
        match self {
            ModelAlias::Opus => ModelAlias::Sonnet,
            ModelAlias::Sonnet => ModelAlias::Haiku,
            ModelAlias::Haiku => ModelAlias::Opus,
        }
    }

    /// Cycle to previous model.
    pub fn prev(self) -> Self {
        match self {
            ModelAlias::Opus => ModelAlias::Haiku,
            ModelAlias::Sonnet => ModelAlias::Opus,
            ModelAlias::Haiku => ModelAlias::Sonnet,
        }
    }
}

impl fmt::Display for ModelAlias {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// All agent phases with their descriptions, in display order.
pub const AGENT_PHASES: &[(&str, &str)] = &[
    ("sdd-explore", "Analisis del codebase"),
    ("sdd-propose", "Propuestas de cambios"),
    ("sdd-spec", "Escritura de specs"),
    ("sdd-design", "Decisiones de arquitectura"),
    ("sdd-tasks", "Desglose de tareas"),
    ("sdd-apply", "Implementacion"),
    ("sdd-verify", "Verificacion"),
    ("sdd-archive", "Archivado"),
    ("code-review", "Code review y analisis de PRs"),
    ("debugger", "Diagnostico y correccion de bugs"),
    ("test-writer", "Escritura de tests"),
    ("docs-writer", "Documentacion"),
    ("refactor", "Reestructuracion de codigo"),
    ("searcher", "Busqueda y navegacion"),
    ("git-ops", "Operaciones Git"),
    ("planner", "Planificacion de implementacion"),
    ("default", "Cualquier otra delegacion"),
];

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
    /// User picks model for each agent individually.
    Custom,
}

impl ModelPreset {
    pub fn display_name(&self) -> &'static str {
        match self {
            ModelPreset::Balanced => "Balanced",
            ModelPreset::Performance => "Performance",
            ModelPreset::Economy => "Economy",
            ModelPreset::Custom => "Custom",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ModelPreset::Balanced => {
                "Opus for planning & architecture, Sonnet for code, Haiku for archiving"
            }
            ModelPreset::Performance => {
                "Opus also handles verification (higher quality, higher cost)"
            }
            ModelPreset::Economy => "Sonnet for everything, Haiku for archiving (lowest cost)",
            ModelPreset::Custom => "Choose the model for each agent individually",
        }
    }

    /// Get the model assignment for a specific agent/phase.
    /// For Custom preset, returns Sonnet as fallback — use Selection::model_for_phase instead.
    pub fn model_for_phase(&self, phase: &str) -> ModelAlias {
        match self {
            ModelPreset::Balanced => match phase {
                "sdd-propose" | "sdd-design" => ModelAlias::Opus,
                "sdd-archive" => ModelAlias::Haiku,
                "code-review" | "debugger" | "refactor" | "planner" => ModelAlias::Opus,
                "docs-writer" | "searcher" | "git-ops" => ModelAlias::Haiku,
                _ => ModelAlias::Sonnet,
            },
            ModelPreset::Performance => match phase {
                "sdd-propose" | "sdd-design" | "sdd-verify" => ModelAlias::Opus,
                "sdd-archive" => ModelAlias::Haiku,
                "code-review" | "debugger" | "refactor" | "planner" | "test-writer" => {
                    ModelAlias::Opus
                }
                "docs-writer" | "searcher" | "git-ops" => ModelAlias::Haiku,
                _ => ModelAlias::Sonnet,
            },
            ModelPreset::Economy => match phase {
                "sdd-archive" | "docs-writer" | "searcher" | "git-ops" => ModelAlias::Haiku,
                _ => ModelAlias::Sonnet,
            },
            // Custom uses Selection::model_for_phase with the custom map
            ModelPreset::Custom => ModelAlias::Sonnet,
        }
    }

    /// All agents/phases in order with their model assignments.
    pub fn all_assignments(&self) -> Vec<(&'static str, &'static str, ModelAlias)> {
        AGENT_PHASES
            .iter()
            .map(|(id, desc)| (*id, *desc, self.model_for_phase(id)))
            .collect()
    }

    /// Render the assignments as a markdown table for injection into CLAUDE.md.
    pub fn render_markdown_table(&self) -> String {
        let mut table = String::from("| Phase | Purpose | Model |\n|-------|---------|-------|\n");
        for (phase, purpose, model) in self.all_assignments() {
            table.push_str(&format!("| `{}` | {} | **{}** |\n", phase, purpose, model));
        }
        table
    }

    /// Generate default custom model assignments based on this preset.
    pub fn default_custom_models(&self) -> Vec<ModelAlias> {
        AGENT_PHASES
            .iter()
            .map(|(id, _)| self.model_for_phase(id))
            .collect()
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
    /// Per-phase model overrides (used when model_preset == Custom).
    pub custom_models: Option<BTreeMap<String, ModelAlias>>,
    pub scope: ConfigScope,
    pub dry_run: bool,
    /// Behavioral profile to inject into CLAUDE.md (None = skip).
    pub behavior_profile: Option<ProfileId>,
}

impl Selection {
    /// Get the model for a phase, using custom assignments when available.
    pub fn model_for_phase(&self, phase: &str) -> ModelAlias {
        if let Some(custom) = &self.custom_models {
            custom.get(phase).copied().unwrap_or(ModelAlias::Sonnet)
        } else {
            self.model_preset.model_for_phase(phase)
        }
    }

    /// All assignments resolved through custom or preset.
    pub fn all_assignments(&self) -> Vec<(&'static str, &'static str, ModelAlias)> {
        AGENT_PHASES
            .iter()
            .map(|(id, desc)| (*id, *desc, self.model_for_phase(id)))
            .collect()
    }

    /// Render assignments as markdown table.
    pub fn render_markdown_table(&self) -> String {
        let mut table = String::from("| Phase | Purpose | Model |\n|-------|---------|-------|\n");
        for (phase, purpose, model) in self.all_assignments() {
            table.push_str(&format!("| `{}` | {} | **{}** |\n", phase, purpose, model));
        }
        table
    }
}

impl Default for Selection {
    fn default() -> Self {
        Self {
            components: PresetId::Recommended.components(),
            skills: vec![],
            preset: PresetId::Recommended,
            model_preset: ModelPreset::Balanced,
            custom_models: None,
            scope: ConfigScope::User,
            dry_run: false,
            behavior_profile: None,
        }
    }
}
