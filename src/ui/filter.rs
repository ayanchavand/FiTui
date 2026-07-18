use ratatui::{
    prelude::*,
    widgets::{Clear, Paragraph, Padding},
};

use crate::{
    app::{App, FilterField},
    theme::Theme,
};

pub fn draw_filter_popup(f: &mut Frame, app: &App, theme: &Theme) {
    let area = centered_rect(60, 45, f.size());
    let filter = &app.filter;

    // Build the popup content
    let date_active = filter.active_field == FilterField::MonthYear;
    let tag_active = filter.active_field == FilterField::Tag;

    // 1. Month/Year Field line
    let date_display = if filter.month_year.is_empty() && !date_active {
        "YYYY-MM (e.g. 2024-02)"
    } else {
        &filter.month_year
    };
    
    let date_label_style = if date_active {
        Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
    } else {
        theme.muted_text()
    };
    
    let date_val_style = if date_active {
        Style::default().fg(theme.foreground).bg(theme.surface).add_modifier(Modifier::BOLD)
    } else if filter.month_year.is_empty() {
        Style::default().fg(theme.subtle).add_modifier(Modifier::ITALIC)
    } else {
        Style::default().fg(theme.foreground)
    };

    let date_indicator = if date_active {
        Span::styled("▶ ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
    } else {
        Span::raw("  ")
    };

    let date_cursor = if date_active {
        Span::styled("│", theme.cursor_style())
    } else {
        Span::raw("")
    };

    let date_line = Line::from(vec![
        date_indicator,
        Span::styled("Month/Year", date_label_style),
        Span::styled(" │ ", Style::default().fg(theme.subtle)),
        Span::styled(date_display, date_val_style),
        date_cursor,
    ]);

    // 2. Tag Field line
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
        date_line,
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
