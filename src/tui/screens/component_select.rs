use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::Frame;

use crate::tui::app_state::AppState;
use crate::tui::widgets;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(6),
        Constraint::Length(1),
    ])
    .split(area);

    widgets::render_header(frame, chunks[0], "Seleccionar Componentes");

    let items: Vec<(String, String, bool)> = state
        .components
        .iter()
        .map(|(id, selected)| {
            (id.display_name().to_string(), id.description().to_string(), *selected)
        })
        .collect();

    widgets::render_selectable_list(
        frame,
        chunks[1],
        "Componentes",
        &items,
        state.cursor,
        true,
    );

    widgets::render_footer(
        frame,
        chunks[2],
        &[
            ("Flechas", "Navegar"),
            ("Espacio", "Marcar"),
            ("Enter", "Continuar"),
            ("Esc", "Atras"),
        ],
    );
}
