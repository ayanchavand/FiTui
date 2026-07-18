use ratatui::{
    prelude::*,
    widgets::{Clear, Paragraph, Padding},
};

use crate::{
    app::{App, FilterField},
    theme::Theme,
};

pub fn draw_filter_popup(f: &mut Frame, app: &App, theme: &Theme) {
    let area = centered_rect(60, 55, f.size());
    let filter = &app.filter;

    // Build the popup content
    let start_active = filter.active_field == FilterField::StartDate;
    let end_active = filter.active_field == FilterField::EndDate;
    let tag_active = filter.active_field == FilterField::Tag;

    // 1. Start Date Field line
    let start_label_style = if start_active {
        Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
    } else {
        theme.muted_text()
    };
    
    let start_val_style = if start_active {
        Style::default().fg(theme.foreground).bg(theme.surface).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.foreground)
    };

    let start_indicator = if start_active {
        Span::styled("▶ ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
    } else {
        Span::raw("  ")
    };

    let mut start_value_spans = Vec::new();
    if filter.start_date.is_empty() && !start_active {
        start_value_spans.push(Span::styled("YYYY-MM-DD (Start Date)", Style::default().fg(theme.subtle).add_modifier(Modifier::ITALIC)));
    } else {
        start_value_spans.push(Span::styled(filter.start_date.clone(), start_val_style));
        if start_active {
            start_value_spans.push(Span::styled("│", theme.cursor_style()));
        }
        if filter.start_date.len() < 10 {
            let mask = "YYYY-MM-DD";
            let remaining = &mask[filter.start_date.len()..];
            start_value_spans.push(Span::styled(remaining.to_string(), Style::default().fg(theme.subtle).add_modifier(Modifier::ITALIC)));
        }
    }

    let start_line_spans = {
        let mut spans = vec![
            start_indicator,
            Span::styled("Start Date", start_label_style),
            Span::styled(" │ ", Style::default().fg(theme.subtle)),
        ];
        spans.extend(start_value_spans);
        spans
    };
    let start_line = Line::from(start_line_spans);

    // 2. End Date Field line
    let end_label_style = if end_active {
        Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
    } else {
        theme.muted_text()
    };
    
    let end_val_style = if end_active {
        Style::default().fg(theme.foreground).bg(theme.surface).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.foreground)
    };

    let end_indicator = if end_active {
        Span::styled("▶ ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
    } else {
        Span::raw("  ")
    };

    let mut end_value_spans = Vec::new();
    if filter.end_date.is_empty() && !end_active {
        end_value_spans.push(Span::styled("YYYY-MM-DD (End Date)", Style::default().fg(theme.subtle).add_modifier(Modifier::ITALIC)));
    } else {
        end_value_spans.push(Span::styled(filter.end_date.clone(), end_val_style));
        if end_active {
            end_value_spans.push(Span::styled("│", theme.cursor_style()));
        }
        if filter.end_date.len() < 10 {
            let mask = "YYYY-MM-DD";
            let remaining = &mask[filter.end_date.len()..];
            end_value_spans.push(Span::styled(remaining.to_string(), Style::default().fg(theme.subtle).add_modifier(Modifier::ITALIC)));
        }
    }

    let end_line_spans = {
        let mut spans = vec![
            end_indicator,
            Span::styled("End Date  ", end_label_style),
            Span::styled(" │ ", Style::default().fg(theme.subtle)),
        ];
        spans.extend(end_value_spans);
        spans
    };
    let end_line = Line::from(end_line_spans);

    // 3. Tag Field line
    let tag_display = match filter.tag_index {
        None => "ALL".to_string(),
        Some(idx) => format!("#{}", app.tags[idx].as_str()),
    };

    let tag_label_style = if tag_active {
        Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
    } else {
        theme.muted_text()
    };

    let tag_val_style = if tag_active {
        Style::default().fg(theme.foreground).bg(theme.surface).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.accent_soft).add_modifier(Modifier::BOLD)
    };

    let tag_indicator = if tag_active {
        Span::styled("▶ ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
    } else {
        Span::raw("  ")
    };

    let tag_line = Line::from(vec![
        tag_indicator,
        Span::styled("Tag       ", tag_label_style),
        Span::styled(" │ ", Style::default().fg(theme.subtle)),
        Span::styled(tag_display, tag_val_style),
        Span::raw("  "),
        Span::styled(
            "← →",
            if tag_active {
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.muted)
            }
        ),
    ]);

    // Content builder
    let content = vec![
        Line::raw(""),
        Line::styled(" Filter Transactions", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Line::styled(" ───────────────────", Style::default().fg(theme.subtle)),
        Line::raw(""),
        start_line,
        Line::raw(""),
        end_line,
        Line::raw(""),
        tag_line,
        Line::raw(""),
        Line::raw(""),
        Line::styled(" ───────────────────", Style::default().fg(theme.subtle)),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("[", theme.muted_text()),
            Span::styled("Tab", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled("] Focus  ", theme.muted_text()),
            
            Span::styled("[", theme.muted_text()),
            Span::styled("Enter", theme.success()),
            Span::styled("] Apply  ", theme.muted_text()),
            
            Span::styled("[", theme.muted_text()),
            Span::styled("Esc", theme.danger()),
            Span::styled("] Cancel", theme.muted_text()),
        ]),
        Line::raw(""),
    ];

    let popup = Paragraph::new(content)
        .block(theme.popup(" Filter ").padding(Padding::new(2, 2, 0, 0)))
        .alignment(Alignment::Left);

    f.render_widget(Clear, area);
    f.render_widget(popup, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, rect: Rect) -> Rect {
    let vertical_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(rect);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical_layout[1])[1]
}
