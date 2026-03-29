use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::Frame;

use crate::config::types::ConfigScope;
use crate::tui::app_state::AppState;
use crate::tui::widgets;

const SCOPES: &[(ConfigScope, &str, &str)] = &[
    (ConfigScope::User, "Usuario (~/.claude/)", "Configuracion global para todos los proyectos"),
    (ConfigScope::Project, "Proyecto (.claude/)", "Configuracion solo para este proyecto"),
];

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(4),
        Constraint::Length(1),
    ])
    .split(area);

    widgets::render_header(frame, chunks[0], "Seleccionar Alcance");

    let items: Vec<(String, String, bool)> = SCOPES
        .iter()
        .map(|(id, name, desc)| {
            (name.to_string(), desc.to_string(), *id == state.scope)
        })
        .collect();

    widgets::render_selectable_list(frame, chunks[1], "Alcance", &items, state.cursor, false);

    widgets::render_footer(
        frame,
        chunks[2],
        &[("Flechas", "Navegar"), ("Enter", "Seleccionar"), ("Esc", "Atras")],
    );
}

pub fn scope_at(cursor: usize) -> ConfigScope {
    SCOPES.get(cursor).map(|(id, _, _)| *id).unwrap_or(ConfigScope::User)
}
