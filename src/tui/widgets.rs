use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::tui::styles;

/// Render a header bar with the app name and current step.
pub fn render_header(frame: &mut Frame, area: Rect, step_name: &str) {
    let header = Line::from(vec![
        Span::styled(" karma ", styles::title()),
        Span::styled(" > ", styles::muted()),
        Span::styled(step_name, styles::subtitle()),
    ]);
    frame.render_widget(Paragraph::new(header), area);
}

/// Render a footer with key hints.
pub fn render_footer(frame: &mut Frame, area: Rect, hints: &[(&str, &str)]) {
    let spans: Vec<Span> = hints
        .iter()
        .enumerate()
        .flat_map(|(i, (key, desc))| {
            let mut s = vec![
                Span::styled(format!(" {key} "), styles::key_hint()),
                Span::styled(*desc, styles::muted()),
            ];
            if i < hints.len() - 1 {
                s.push(Span::styled("  ", styles::muted()));
            }
            s
        })
        .collect();

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

/// Render a list of items with a cursor indicator.
/// Each item is (label, description, selected).
pub fn render_selectable_list(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    items: &[(String, String, bool)],
    cursor: usize,
    multi_select: bool,
) {
    let lines: Vec<Line> = items
        .iter()
        .enumerate()
        .map(|(i, (label, desc, selected))| {
            let cursor_char = if i == cursor { ">" } else { " " };
            let check = if multi_select {
                if *selected {
                    "[x]"
                } else {
                    "[ ]"
                }
            } else if *selected {
                "(*)"
            } else {
                "( )"
            };

            let style = if i == cursor {
                styles::highlight()
            } else if *selected {
                styles::selected()
            } else {
                styles::normal()
            };

            Line::styled(format!(" {cursor_char} {check} {label}  {desc}"), style)
        })
        .collect();

    let block = Block::default()
        .title(Span::styled(format!(" {title} "), styles::title()))
        .borders(Borders::ALL)
        .border_style(styles::border_focused());

    frame.render_widget(Paragraph::new(lines).block(block), area);
}

/// Render a simple info box.
pub fn render_info_box(frame: &mut Frame, area: Rect, title: &str, lines: Vec<Line>) {
    let block = Block::default()
        .title(Span::styled(format!(" {title} "), styles::title()))
        .borders(Borders::ALL)
        .border_style(styles::border());

    frame.render_widget(Paragraph::new(lines).block(block), area);
}
