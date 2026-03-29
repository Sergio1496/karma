use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::Frame;

use crate::components::OptimizerAction;
use crate::tui::app_state::AppState;
use crate::tui::styles;
use crate::tui::widgets;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(6),
        Constraint::Length(1),
    ])
    .split(area);

    widgets::render_header(frame, chunks[0], "Confirmar Instalacion");

    let selection = state.build_selection();

    let mut lines = vec![
        Line::styled("", styles::normal()),
        Line::from(vec![
            Span::styled("  Preset:  ", styles::muted()),
            Span::styled(selection.preset.display_name(), styles::normal()),
        ]),
        Line::from(vec![
            Span::styled("  Modelos: ", styles::muted()),
            Span::styled(
                format!("{} ({})", selection.model_preset.display_name(), selection.model_preset.description()),
                styles::normal(),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Alcance: ", styles::muted()),
            Span::styled(format!("{:?}", selection.scope), styles::normal()),
        ]),
        Line::styled("", styles::normal()),
        Line::styled("  Componentes:", styles::subtitle()),
    ];

    for c in &selection.components {
        lines.push(Line::styled(
            format!("    + {}", c.display_name()),
            styles::selected(),
        ));
    }

    // Skills to install
    if !selection.skills.is_empty() {
        lines.push(Line::styled("", styles::normal()));
        lines.push(Line::styled("  Skills a instalar:", styles::subtitle()));
        for s in &selection.skills {
            lines.push(Line::styled(format!("    + {s}"), styles::selected()));
        }
    }

    // Skills to remove
    let skills_to_remove: Vec<_> = state
        .skills
        .iter()
        .filter(|(_, _, _, selected, installed)| !selected && *installed)
        .collect();
    if !skills_to_remove.is_empty() {
        lines.push(Line::styled("", styles::normal()));
        lines.push(Line::styled("  Skills a borrar:", styles::subtitle()));
        for (id, _, _, _, _) in &skills_to_remove {
            lines.push(Line::styled(format!("    - {id}"), styles::error()));
        }
    }

    // Show optimizer actions
    let has_optimizer_actions = state.optimizers.iter().any(|(_, a, _)| *a != OptimizerAction::Skip);
    if has_optimizer_actions {
        lines.push(Line::styled("", styles::normal()));
        lines.push(Line::styled("  Optimizadores:", styles::subtitle()));
        for (id, action, _) in &state.optimizers {
            match action {
                OptimizerAction::InstallProject => {
                    lines.push(Line::styled(
                        format!("    + Instalar: {}", id.display_name()),
                        styles::selected(),
                    ));
                }
                OptimizerAction::UninstallProject => {
                    lines.push(Line::styled(
                        format!("    - Desinstalar: {}", id.display_name()),
                        styles::error(),
                    ));
                }
                OptimizerAction::Skip => {}
            }
        }
    }

    lines.push(Line::styled("", styles::normal()));
    lines.push(Line::styled(
        "  Pulsa Enter para instalar, o Esc para volver.",
        styles::muted(),
    ));

    widgets::render_info_box(frame, chunks[1], "Revision", lines);

    widgets::render_footer(
        frame,
        chunks[2],
        &[("Enter", "Instalar"), ("Esc", "Atras"), ("q", "Salir")],
    );
}
