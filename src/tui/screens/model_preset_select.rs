use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::Frame;

use crate::config::types::ModelPreset;
use crate::tui::app_state::AppState;
use crate::tui::styles;
use crate::tui::widgets;

const PRESETS: &[(ModelPreset, &str, &str)] = &[
    (
        ModelPreset::Balanced,
        "Balanced",
        "Opus para planificar y arquitectura, Sonnet para codigo, Haiku para archivar",
    ),
    (
        ModelPreset::Performance,
        "Performance",
        "Opus tambien verifica y escribe tests (mas calidad, mas coste)",
    ),
    (
        ModelPreset::Economy,
        "Economy",
        "Sonnet para todo, Haiku para archivar (menor coste)",
    ),
];

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(7),
        Constraint::Min(6),
        Constraint::Length(1),
    ])
    .split(area);

    widgets::render_header(frame, chunks[0], "Asignacion de Modelos");

    let info = vec![
        Line::styled("", styles::normal()),
        Line::styled(
            "  Elige como se asignan los modelos de IA a cada tarea:",
            styles::muted(),
        ),
        Line::styled("", styles::normal()),
        Line::from(vec![
            Span::styled("    Opus   ", styles::title()),
            Span::styled("-> Pensamiento profundo: arquitectura, debug, review, refactor", styles::normal()),
        ]),
        Line::from(vec![
            Span::styled("    Sonnet ", styles::subtitle()),
            Span::styled("-> Equilibrado: implementacion, specs, tareas, tests", styles::normal()),
        ]),
        Line::from(vec![
            Span::styled("    Haiku  ", styles::muted()),
            Span::styled("-> Rapido y barato: docs, git, busquedas, archivar", styles::normal()),
        ]),
    ];
    frame.render_widget(ratatui::widgets::Paragraph::new(info), chunks[1]);

    let items: Vec<(String, String, bool)> = PRESETS
        .iter()
        .map(|(id, name, desc)| {
            (name.to_string(), desc.to_string(), *id == state.model_preset)
        })
        .collect();

    widgets::render_selectable_list(
        frame,
        chunks[2],
        "Presets de Modelos",
        &items,
        state.cursor,
        false,
    );

    widgets::render_footer(
        frame,
        chunks[3],
        &[("Flechas", "Navegar"), ("Enter", "Seleccionar"), ("Esc", "Atras")],
    );
}

pub fn preset_at(cursor: usize) -> ModelPreset {
    PRESETS
        .get(cursor)
        .map(|(id, _, _)| *id)
        .unwrap_or(ModelPreset::Balanced)
}
