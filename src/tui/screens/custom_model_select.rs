use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::config::types::{ModelAlias, AGENT_PHASES};
use crate::tui::app_state::AppState;
use crate::tui::styles;
use crate::tui::widgets;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Min(6),
        Constraint::Length(1),
    ])
    .split(area);

    widgets::render_header(frame, chunks[0], "Asignacion Custom de Modelos");

    let info = vec![
        Line::styled("", styles::normal()),
        Line::styled(
            "  Usa las flechas ← → para cambiar el modelo de cada agente:",
            styles::muted(),
        ),
    ];
    frame.render_widget(Paragraph::new(info), chunks[1]);

    let max_desc_len = AGENT_PHASES
        .iter()
        .map(|(_, desc)| desc.len())
        .max()
        .unwrap_or(30);

    let mut lines = Vec::new();
    for (i, ((phase_id, description), model)) in AGENT_PHASES
        .iter()
        .zip(state.custom_models.iter())
        .enumerate()
    {
        let is_selected = i == state.cursor;
        let cursor_char = if is_selected { ">" } else { " " };

        let model_style = match model {
            ModelAlias::Opus => Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
            ModelAlias::Sonnet => Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            ModelAlias::Haiku => Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        };

        let line_style = if is_selected {
            styles::highlight()
        } else {
            styles::normal()
        };

        // Separator between SDD and general-purpose agents
        if i == 8 && !lines.is_empty() {
            lines.push(Line::styled("  ── General Purpose ──", styles::muted()));
        }

        let model_label = format!(" ◀ {:^6} ▶ ", model.as_str());
        lines.push(Line::from(vec![
            Span::styled(format!("  {} ", cursor_char), line_style),
            Span::styled(format!("{:<16}", phase_id), line_style),
            Span::styled(
                format!("{:<width$}", description, width = max_desc_len + 2),
                styles::muted(),
            ),
            Span::styled(model_label, model_style),
        ]));
    }

    let list = Paragraph::new(lines).scroll((
        if state.cursor > chunks[2].height as usize - 2 {
            (state.cursor - (chunks[2].height as usize - 2)) as u16
        } else {
            0
        },
        0,
    ));
    frame.render_widget(list, chunks[2]);

    widgets::render_footer(
        frame,
        chunks[3],
        &[
            ("↑↓", "Navegar"),
            ("← →", "Cambiar modelo"),
            ("Enter", "Confirmar"),
            ("Esc", "Atras"),
        ],
    );
}
