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

    widgets::render_header(frame, chunks[0], "Seleccionar Skills");

    let items: Vec<(String, String, bool)> = state
        .skills
        .iter()
        .map(|(id, name, category, selected, installed)| {
            let status = if *installed { " [en proyecto]" } else { "" };
            let label = format!("{name} [{cat}]{status}", cat = category.display_name());
            (label, id.clone(), *selected)
        })
        .collect();

    widgets::render_selectable_list(
        frame,
        chunks[1],
        "Skills (marcado = instalar, desmarcado = no instalar / borrar)",
        &items,
        state.cursor,
        true,
    );

    widgets::render_footer(
        frame,
        chunks[2],
        &[
            ("Flechas", "Navegar"),
            ("Espacio", "Marcar/Desmarcar"),
            ("a", "Seleccionar todo"),
            ("Enter", "Continuar"),
            ("Esc", "Atras"),
        ],
    );
}
