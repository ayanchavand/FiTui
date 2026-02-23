use ratatui::{
    prelude::*,
    widgets::{Clear, Paragraph},
};

use crate::{
    app::App,
    form::Field,
    models::{RecurringInterval, TransactionType, Tag},
    theme::Theme,
};

pub fn draw_transaction_form(f: &mut Frame, app: &App, theme: &Theme) {
    let area = centered_rect(65, 65, f.size());
    let form_content = build_form_content(app, theme);

    let title = if app.editing.is_some() {
        " ✏️  Edit Transaction "
    } else {
        " ➕ Add New Transaction "
    };

    let popup = Paragraph::new(form_content)
        .block(theme.popup(title))
        .alignment(Alignment::Left);

    f.render_widget(Clear, area);
    f.render_widget(popup, area);
}

fn build_form_content(app: &App, theme: &Theme) -> Vec<Line<'static>> {
    let form = &app.form;
    vec![
        Line::raw(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("📝 ", Style::default().fg(theme.accent)),
            Span::styled(
                "Fill in the transaction details below",
                Style::default().fg(theme.muted).add_modifier(Modifier::ITALIC)
            ),
        ]),
        Line::raw(""),
        Line::styled(
            "  ───────────────────────────────────────────────────",
            Style::default().fg(theme.subtle),
        ),
        Line::raw(""),
        create_form_field(
            "Source",
            &form.source,
            form.active,
            Field::Source,
            "e.g., Salary, Groceries, Rent",
            theme,
        ),
        Line::raw(""),
        create_form_field(
            "Amount",
            &form.amount,
            form.active,
            Field::Amount,
            "e.g., 1000.50",
            theme,
        ),
        Line::raw(""),
        create_form_field(
            "Date",
            &form.date,
            form.active,
            Field::Date,
            "YYYY-MM-DD",
            theme,
        ),
        Line::raw(""),
        Line::styled(
            "  ───────────────────────────────────────────────────",
            Style::default().fg(theme.subtle),
        ),
        Line::raw(""),
        create_type_selector(&form.kind, form.active == Field::Kind, theme),
        Line::raw(""),
        create_tag_selector(&app.tags, form.tag_index, form.active == Field::Tag, theme),
        Line::raw(""),
        create_recurring_selector(form.recurring, form.active == Field::Recurring, theme),
        Line::raw(""),
        create_recurring_interval_selector(&form.recurring_interval, form.active == Field::RecurringInterval, form.recurring, theme),
        Line::raw(""),
        Line::styled(
            "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
            Style::default().fg(theme.accent_soft),
        ),
        Line::raw(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("[", theme.muted_text()),
            Span::styled("Tab", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled("] Next  ", theme.muted_text()),
            
            Span::styled("[", theme.muted_text()),
            Span::styled("←→", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled("] Toggle  ", theme.muted_text()),
            
            Span::styled("[", theme.muted_text()),
            Span::styled("Enter", theme.success()),
            Span::styled("] Save  ", theme.muted_text()),
            
            Span::styled("[", theme.muted_text()),
            Span::styled("Esc", theme.danger()),
            Span::styled("] Cancel", theme.muted_text()),
        ]),
        Line::raw(""),
    ]
}

fn create_form_field(
    label: &str,
    value: &str,
    active_field: Field,
    field: Field,
    placeholder: &str,
    theme: &Theme,
) -> Line<'static> {
    let is_active = active_field == field;
    let display_value = if value.is_empty() && !is_active {
        placeholder.to_string()
    } else {
        value.to_string()
    };
    
    let label_style = if is_active {
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
    } else {
        theme.muted_text()
    };
    
    let value_style = if is_active {
        Style::default()
            .fg(theme.foreground)
            .bg(theme.surface)
            .add_modifier(Modifier::BOLD)
    } else if value.is_empty() {
        Style::default()
            .fg(theme.subtle)
            .add_modifier(Modifier::ITALIC)
    } else {
        Style::default().fg(theme.foreground)
    };
    
    let cursor = if is_active { 
        Span::styled("│", theme.cursor_style())
    } else { 
        Span::raw("")
    };
    
    let indicator = if is_active {
        Span::styled("▶ ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
    } else {
        Span::raw("  ")
    };
    
    Line::from(vec![
        indicator,
        Span::styled(format!("{:<9}", label), label_style),
        Span::styled("│ ", Style::default().fg(theme.subtle)),
        Span::styled(display_value, value_style),
        cursor,
    ])
}

fn create_type_selector(
    kind: &TransactionType,
    is_active: bool,
    theme: &Theme,
) -> Line<'static> {
    let (kind_icon, kind_label, kind_style) = match kind {
        TransactionType::Credit => ("↑", "Credit (Income)", theme.success()),
        TransactionType::Debit => ("↓", "Debit (Expense)", theme.danger()),
    };
    
    let label_style = if is_active {
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
    } else {
        theme.muted_text()
    };
    
    let indicator = if is_active {
        Span::styled("▶ ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
    } else {
        Span::raw("  ")
    };
    
    Line::from(vec![
        indicator,
        Span::styled("Type     ", label_style),
        Span::styled("│ ", Style::default().fg(theme.subtle)),
        Span::styled(kind_icon, kind_style.add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled(kind_label, kind_style),
        Span::raw("  "),
        Span::styled(
            "← →",
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
        ),
    ])
}

fn create_tag_selector(
    tags: &[Tag],
    index: usize,
    is_active: bool,
    theme: &Theme,
) -> Line<'static> {
    let tag = tags.get(index).map(|t| t.as_str()).unwrap_or("other");
    
    let label_style = if is_active {
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
    } else {
        theme.muted_text()
    };
    
    let indicator = if is_active {
        Span::styled("▶ ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
    } else {
        Span::raw("  ")
    };
    
    Line::from(vec![
        indicator,
        Span::styled("Tag      ", label_style),
        Span::styled("│ ", Style::default().fg(theme.subtle)),
        Span::styled(
            format!("#{}", tag),
            Style::default()
                .fg(theme.accent_soft)
                .add_modifier(Modifier::ITALIC | Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            "← →",
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
        ),
    ])
}

fn create_recurring_selector(recurring: bool, is_active: bool, theme: &Theme) -> Line<'static> {
    let (status_icon, status_text, status_style) = if recurring {
        ("🔄", "Yes", theme.success())
    } else {
        ("🚫", "No", Style::default().fg(theme.muted))
    };
    
    let label_style = if is_active {
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
    } else {
        theme.muted_text()
    };
    
    let indicator = if is_active {
        Span::styled("▶ ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
    } else {
        Span::raw("  ")
    };
    
    Line::from(vec![
        indicator,
        Span::styled("Recurring", label_style),
        Span::styled("│ ", Style::default().fg(theme.subtle)),
        Span::styled(status_icon, status_style),
        Span::raw(" "),
        Span::styled(status_text, status_style),
        Span::raw("  "),
        Span::styled(
            "← →",
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
        ),
    ])
}

fn create_recurring_interval_selector(interval: &RecurringInterval, is_active: bool, is_recurring: bool, theme: &Theme) -> Line<'static> {
    let label_style = if is_active {
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
    } else {
        theme.muted_text()
    };
    
    let indicator = if is_active {
        Span::styled("▶ ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
    } else {
        Span::raw("  ")
    };
    
    let interval_style = if is_recurring {
        Style::default().fg(theme.accent)
    } else {
        Style::default().fg(theme.muted)
    };
    
    let interval_text = interval.display().to_string();
    
    Line::from(vec![
        indicator,
        Span::styled("Interval", label_style),
        Span::styled("│ ", Style::default().fg(theme.subtle)),
        Span::styled(interval_text, interval_style),
        Span::raw("  "),
        Span::styled(
            "← →",
            if is_active {
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.muted)
            }
        ),
    ])
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
