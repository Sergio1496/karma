use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::config::types::{BehaviorProfileStatus, ProfileId};
use crate::tui::app_state::AppState;
use crate::tui::styles;
use crate::tui::widgets;

/// Map cursor position to a profile selection.
pub fn profile_at(cursor: usize) -> Option<ProfileId> {
    match cursor {
        0 => Some(ProfileId::Universal),
        1 => Some(ProfileId::Coding),
        2 => Some(ProfileId::Agents),
        3 => Some(ProfileId::Analysis),
        _ => None, // "Ninguno"
    }
}

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(4),
        Constraint::Min(4),
        Constraint::Length(1),
    ])
    .split(area);

    widgets::render_header(frame, chunks[0], "Perfil de Comportamiento");

    let mut info = vec![
        Line::styled("", styles::normal()),
        Line::styled(
            "  Reduce ~63% de tokens de output inyectando reglas en CLAUDE.md",
            styles::muted(),
        ),
    ];

    // Show warning if manual profile detected
    if let BehaviorProfileStatus::ManuallyDetected(p) = state.profile_status {
        info.push(Line::styled(
            format!(
                "  ! Perfil '{}' detectado manualmente en CLAUDE.md (no desinstalable auto)",
                p.display_name()
            ),
            styles::error(),
        ));
    } else {
        info.push(Line::styled("", styles::normal()));
    }

    frame.render_widget(Paragraph::new(info), chunks[1]);

    // Build profile list: 4 profiles + "Ninguno"
    let items: Vec<(Option<ProfileId>, &str, &str)> = ProfileId::ALL
        .iter()
        .map(|p| (Some(*p), p.display_name(), p.description()))
        .chain(std::iter::once((
            None,
            "Ninguno",
            "No aplicar perfil de comportamiento",
        )))
        .collect();

    let lines: Vec<Line> = items
        .iter()
        .enumerate()
        .map(|(i, (profile_opt, name, desc))| {
            let is_cursor = i == state.cursor;
            let is_selected = state.behavior_profile == *profile_opt;

            let radio = if is_selected { "(●)" } else { "( )" };
            let cursor_char = if is_cursor { ">" } else { " " };

            // Status tag based on profile_status
            let status_tag = if let Some(pid) = profile_opt {
                match state.profile_status {
                    BehaviorProfileStatus::Installed(installed) if installed == *pid => " [karma]",
                    BehaviorProfileStatus::ManuallyDetected(detected) if detected == *pid => {
                        " [manual]"
                    }
                    _ => "",
                }
            } else {
                ""
            };

            let style = if is_cursor {
                styles::highlight()
            } else {
                styles::normal()
            };

            if is_cursor {
                Line::styled(
                    format!(
                        " {cursor_char} {radio} {name}{status_tag}  {desc}"
                    ),
                    style,
                )
            } else {
                let radio_style = if is_selected {
                    styles::selected()
                } else {
                    styles::muted()
                };

                Line::from(vec![
                    Span::styled(format!(" {cursor_char} "), style),
                    Span::styled(format!("{radio} "), radio_style),
                    Span::styled(format!("{name}{status_tag}  "), style),
                    Span::styled(*desc, styles::muted()),
                ])
            }
        })
        .collect();

    let block = Block::default()
        .title(Span::styled(
            " Perfiles de Comportamiento ",
            styles::title(),
        ))
        .borders(Borders::ALL)
        .border_style(styles::border_focused());

    frame.render_widget(Paragraph::new(lines).block(block), chunks[2]);

    widgets::render_footer(
        frame,
        chunks[3],
        &[
            ("Flechas", "Navegar"),
            ("Enter", "Continuar"),
            ("Esc", "Atras"),
        ],
    );
}
