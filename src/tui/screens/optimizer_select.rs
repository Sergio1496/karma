use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::components::{OptimizerAction, OptimizerStatus};
use crate::tui::app_state::AppState;
use crate::tui::styles;
use crate::tui::widgets;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(4),
        Constraint::Min(4),
        Constraint::Length(1),
    ])
    .split(area);

    widgets::render_header(frame, chunks[0], "Optimizadores");

    let info = vec![
        Line::styled("", styles::normal()),
        Line::styled(
            "  Usa las flechas izquierda/derecha para elegir la accion:",
            styles::muted(),
        ),
        Line::styled("", styles::normal()),
    ];
    frame.render_widget(Paragraph::new(info), chunks[1]);

    // Custom render for optimizer list
    let lines: Vec<Line> = state
        .optimizers
        .iter()
        .enumerate()
        .map(|(i, (id, action, status))| {
            let cursor_char = if i == state.cursor { ">" } else { " " };

            let status_label = match status {
                OptimizerStatus::ProjectInstalled => "[en proyecto]",
                OptimizerStatus::GlobalOnly => "[solo global]",
                OptimizerStatus::BinaryOnly => "[binario ok]",
                OptimizerStatus::NotInstalled => "[no instalado]",
            };

            let action_label = format!("< {} >", action.label());

            let style = if i == state.cursor {
                styles::highlight()
            } else {
                styles::normal()
            };

            let action_style = match action {
                OptimizerAction::InstallProject => styles::selected(),
                OptimizerAction::UninstallProject => styles::error(),
                OptimizerAction::Skip => styles::muted(),
            };

            if i == state.cursor {
                Line::styled(
                    format!(
                        " {cursor_char} {} {status_label}  {}  {}",
                        id.display_name(),
                        action_label,
                        id.description()
                    ),
                    style,
                )
            } else {
                Line::from(vec![
                    Span::styled(
                        format!(" {cursor_char} {} {status_label}  ", id.display_name()),
                        style,
                    ),
                    Span::styled(action_label, action_style),
                    Span::styled(format!("  {}", id.description()), styles::muted()),
                ])
            }
        })
        .collect();

    let block = Block::default()
        .title(Span::styled(" Optimizadores de Tokens ", styles::title()))
        .borders(Borders::ALL)
        .border_style(styles::border_focused());

    frame.render_widget(Paragraph::new(lines).block(block), chunks[2]);

    widgets::render_footer(
        frame,
        chunks[3],
        &[
            ("Flechas", "Navegar"),
            ("← →", "Cambiar accion"),
            ("Enter", "Continuar"),
            ("Esc", "Atras"),
        ],
    );
}
