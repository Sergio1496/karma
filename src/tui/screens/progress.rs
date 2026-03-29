use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::Line;
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

    widgets::render_header(frame, chunks[0], "Instalando...");

    let mut lines: Vec<Line> = vec![Line::styled("", styles::normal())];

    for msg in &state.progress_messages {
        let style = if msg.contains("Error") || msg.contains("FAILED") {
            styles::error()
        } else if msg.contains("OK") || msg.contains("complete") {
            styles::success()
        } else {
            styles::normal()
        };
        lines.push(Line::styled(format!("  {msg}"), style));
    }

    if state.progress_messages.is_empty() {
        lines.push(Line::styled("  Iniciando pipeline...", styles::muted()));
    }

    widgets::render_info_box(frame, chunks[1], "Progreso", lines);

    widgets::render_footer(frame, chunks[2], &[("", "Espera...")]);
}
