use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::Frame;

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

    widgets::render_header(frame, chunks[0], "Listo");

    let mut lines = vec![Line::styled("", styles::normal())];

    if let Some(ref result) = state.execution_result {
        if result.success() {
            lines.push(Line::styled(
                "  Instalacion completada con exito!",
                styles::success(),
            ));
            lines.push(Line::styled("", styles::normal()));

            let written = result.total_files_written();
            lines.push(Line::from(vec![
                Span::styled("  Archivos escritos: ", styles::muted()),
                Span::styled(format!("{}", written.len()), styles::normal()),
            ]));

            for path in &written {
                lines.push(Line::styled(
                    format!("    {}", path.display()),
                    styles::muted(),
                ));
            }
        } else {
            lines.push(Line::styled("  La instalacion fallo!", styles::error()));
            lines.push(Line::styled("", styles::normal()));

            for step in &result.apply.steps {
                if let Some(ref err) = step.error {
                    lines.push(Line::styled(
                        format!("  {} - {err}", step.component_id),
                        styles::error(),
                    ));
                }
            }

            if result.rollback.is_some() {
                lines.push(Line::styled("", styles::normal()));
                lines.push(Line::styled(
                    "  Los cambios se han revertido.",
                    styles::muted(),
                ));
            }
        }
    } else {
        lines.push(Line::styled(
            "  Sin resultados disponibles.",
            styles::muted(),
        ));
    }

    widgets::render_info_box(frame, chunks[1], "Resultado", lines);

    widgets::render_footer(frame, chunks[2], &[("q/Enter", "Salir")]);
}
