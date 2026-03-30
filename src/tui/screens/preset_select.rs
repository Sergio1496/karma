use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::Frame;

use crate::config::types::PresetId;
use crate::tui::app_state::AppState;
use crate::tui::widgets;

const PRESETS: &[(PresetId, &str, &str)] = &[
    (
        PresetId::Recommended,
        "Recomendado",
        "Routing + Agentes + Skills + Permisos + MCP",
    ),
    (PresetId::Full, "Completo", "Todos los componentes"),
    (PresetId::Minimal, "Minimo", "Solo model routing + agentes"),
    (
        PresetId::Custom,
        "Personalizado",
        "Elige componentes individuales",
    ),
];

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(6),
        Constraint::Length(1),
    ])
    .split(area);

    widgets::render_header(frame, chunks[0], "Elegir Preset");

    let items: Vec<(String, String, bool)> = PRESETS
        .iter()
        .map(|(id, name, desc)| (name.to_string(), desc.to_string(), *id == state.preset))
        .collect();

    widgets::render_selectable_list(frame, chunks[1], "Presets", &items, state.cursor, false);

    widgets::render_footer(
        frame,
        chunks[2],
        &[
            ("Flechas", "Navegar"),
            ("Enter", "Seleccionar"),
            ("Esc", "Atras"),
        ],
    );
}

pub fn preset_at(cursor: usize) -> PresetId {
    PRESETS
        .get(cursor)
        .map(|(id, _, _)| *id)
        .unwrap_or(PresetId::Recommended)
}
