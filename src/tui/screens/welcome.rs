use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::tui::styles;
use crate::tui::widgets;

const LOGO: &str = r#"
         /\___/\        _  __
        ( o   o )      | |/ /__ _ _ __ _ __ ___   __ _
        (  =^=  )      | ' // _` | '__| '_ ` _ \ / _` |
         )     (       | . \ (_| | |  | | | | | | (_| |
        (       )      |_|\_\__,_|_|  |_| |_| |_|\__,_|
       ( ||   || )
         ""   ""
"#;

pub fn render(frame: &mut Frame, area: Rect) {
    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(9),
        Constraint::Length(3),
        Constraint::Min(3),
        Constraint::Length(1),
    ])
    .split(area);

    widgets::render_header(frame, chunks[0], "Bienvenid@");

    let logo_lines: Vec<Line> = LOGO
        .lines()
        .map(|l| Line::styled(l.to_string(), styles::title()))
        .collect();
    frame.render_widget(Paragraph::new(logo_lines), chunks[1]);

    let version = Line::from(vec![
        Span::styled("  Configura Claude Code con agentes, model routing y optimizadores", styles::normal()),
    ]);
    frame.render_widget(Paragraph::new(vec![version]), chunks[2]);

    let info = vec![
        Line::styled("  Esta herramienta te ayuda a configurar:", styles::muted()),
        Line::styled("", styles::muted()),
        Line::styled("    - Model Routing (Opus/Sonnet/Haiku por tarea)", styles::normal()),
        Line::styled("    - Sub-Agentes (16 agentes especializados con modelo asignado)", styles::normal()),
        Line::styled("    - Skills (Librería de habilidades para Claude)", styles::normal()),
        Line::styled("    - MCP Servers (Context7 para docs de frameworks)", styles::normal()),
        Line::styled("    - Permisos (Seguridad por defecto)", styles::normal()),
        Line::styled("    - Optimizadores (Ahorra tokens con librerías como RTK o Code Review Graph)", styles::normal()),
        Line::styled("", styles::muted()),
        Line::styled("  Cualquier sugerencia es bienvenida <3", styles::muted()),
    ];
    frame.render_widget(Paragraph::new(info), chunks[3]);

    widgets::render_footer(frame, chunks[4], &[("Enter", "Empezar"), ("q", "Salir")]);
}
