use ratatui::{
    prelude::*,
    widgets::{Clear, Paragraph},
};

use crate::{
    app::{App, PopupKind},
    theme::Theme,
};

pub fn draw_popup(f: &mut Frame, app: &App, theme: &Theme) {
    if let Some(popup) = &app.popup {
        let area = centered_rect(super::POPUP_WIDTH_PERCENT, super::POPUP_HEIGHT_PERCENT, f.size());

        // Clear behind popup with slight shadow effect
        f.render_widget(Clear, area);

        let (title, lines, is_confirm) = match popup {
            PopupKind::Confirm { title, message, .. } => {
                (title.as_str(), message.clone(), true)
            }
            PopupKind::Info { title, message } => {
                (title.as_str(), message.clone(), false)
            }
        };

        // Enhanced styled button row with better visual separation
        let buttons = if is_confirm {
            Line::from(vec![
                Span::raw("   "),
                theme.bracket_open(),
                Span::styled("y", Style::default().fg(theme.credit).add_modifier(Modifier::BOLD)),
                theme.bracket_close(),
                Span::styled("Yes", theme.success()),
                Span::raw("      "),
                theme.bracket_open(),
                Span::styled("n", Style::default().fg(theme.debit).add_modifier(Modifier::BOLD)),
                theme.bracket_close(),
                Span::styled("No", theme.danger()),
            ])
        } else {
            Line::from(vec![
                theme.bracket_open(),
                Span::styled("Esc", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                theme.bracket_close(),
                Span::styled("Close", theme.muted_text()),
            ])
        };

        // Popup content with enhanced spacing and hierarchy
        let content = vec![
            Line::raw(""),
            Line::raw(""),
            Line::styled(
                lines,
                Style::default()
                    .fg(theme.foreground)
                    .add_modifier(Modifier::BOLD)
            ),
            Line::raw(""),
            Line::raw(""),
            Line::styled(
                "─────────────────────────────────────────────────",
                Style::default().fg(theme.subtle),
            ),
            Line::raw(""),
            buttons,
            Line::raw(""),
        ];

        let widget = Paragraph::new(content)
            .block(theme.popup(title))
            .alignment(Alignment::Center);

        f.render_widget(widget, area);
    }
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
