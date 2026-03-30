use crate::config::types::{ComponentId, Selection};

/// Ordered list of component IDs representing the execution plan.
///
/// Components are ordered by their dependencies:
/// 1. Permissions (settings.json must exist first)
/// 2. Hooks (also settings.json)
/// 3. MCP Servers (~/.claude.json)
/// 4. Engram (may depend on MCP config)
/// 5. Orchestrator (CLAUDE.md injection)
/// 6. SubAgents (agent .md files)
/// 7. Skills (individual SKILL.md files)
pub struct ExecutionPlan {
    pub ordered_components: Vec<ComponentId>,
}

/// The canonical ordering of components for installation.
/// This defines the dependency-safe execution order.
const COMPONENT_ORDER: &[ComponentId] = &[
    ComponentId::Rtk,
    ComponentId::CodeReviewGraph,
    ComponentId::Permissions,
    ComponentId::McpServers,
    ComponentId::Orchestrator,
    ComponentId::SubAgents,
    ComponentId::Skills,
];

/// Build an execution plan from a user selection.
///
/// Takes the selected components and orders them according to the
/// dependency-safe canonical order. Components not in the selection
/// are excluded.
pub fn build_plan(selection: &Selection) -> ExecutionPlan {
    let ordered: Vec<ComponentId> = COMPONENT_ORDER
        .iter()
        .filter(|c| selection.components.contains(c))
        .copied()
        .collect();

    ExecutionPlan {
        ordered_components: ordered,
    }
}

/// Expand implicit dependencies.
///
/// Deduplicates while preserving order.
pub fn expand_dependencies(components: &[ComponentId]) -> Vec<ComponentId> {
    let mut expanded: Vec<ComponentId> = components.to_vec();

    // Deduplicate while preserving order
    let mut seen = std::collections::HashSet::new();
    expanded.retain(|c| seen.insert(*c));

    expanded
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::ConfigScope;

    fn selection_with(components: Vec<ComponentId>) -> Selection {
        Selection {
            components,
            skills: vec![],
            preset: crate::config::types::PresetId::Custom,
            model_preset: crate::config::types::ModelPreset::Balanced,
            custom_models: None,
            scope: ConfigScope::User,
            dry_run: false,
        }
    }

    #[test]
    fn test_plan_orders_components() {
        let sel = selection_with(vec![
            ComponentId::Skills,
            ComponentId::Permissions,
            ComponentId::Orchestrator,
        ]);
        let plan = build_plan(&sel);

        assert_eq!(
            plan.ordered_components,
            vec![
                ComponentId::Permissions,
                ComponentId::Orchestrator,
                ComponentId::Skills,
            ]
        );
    }

    #[test]
    fn test_plan_excludes_unselected() {
        let sel = selection_with(vec![ComponentId::Skills]);
        let plan = build_plan(&sel);
        assert_eq!(plan.ordered_components, vec![ComponentId::Skills]);
    }

    #[test]
    fn test_plan_full_preset_order() {
        // ALL (core) + OPTIMIZERS = everything
        let mut all: Vec<ComponentId> = ComponentId::ALL.to_vec();
        all.extend_from_slice(ComponentId::OPTIMIZERS);
        let sel = selection_with(all);
        let plan = build_plan(&sel);
        assert_eq!(plan.ordered_components, COMPONENT_ORDER.to_vec());
    }

    #[test]
    fn test_expand_deduplicates() {
        let components = vec![
            ComponentId::McpServers,
            ComponentId::McpServers,
            ComponentId::Skills,
        ];
        let expanded = expand_dependencies(&components);
        assert_eq!(expanded.len(), 2);
    }
}
